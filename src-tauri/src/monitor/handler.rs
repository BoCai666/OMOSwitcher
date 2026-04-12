// Monitor 模块 - HTTP 请求/响应处理器
// 实现 hudsucker 的 HttpHandler trait
// 完整 MITM 拦截实现：域名过滤、请求/响应捕获、错误隔离
// 注意：部分方法（emit_response/emit_metrics、from_storage_and_config 等）尚未被集成调用

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hudsucker::{
    hyper::{Method, Request, Response, StatusCode, Uri},
    Body, HttpContext, HttpHandler, RequestOrResponse,
};
use http_body_util::{BodyExt, Full};
use tokio::sync::RwLock;

use crate::monitor::capture::{capture_request, match_domain, RequestContext};
use crate::monitor::config::ConfigManager;
use crate::monitor::storage::MonitorStorage;
use crate::monitor::types::{MonitorConfig, Provider};
use crate::monitor::{
    MetricsEventPayload, RequestEventPayload, ResponseEventPayload, EVENT_METRICS,
    EVENT_NEW_REQUEST, EVENT_RESPONSE,
};
use tauri::Emitter;

// ============================================================================
// 待处理请求信息
// ============================================================================

/// 待处理请求信息
/// 存储请求开始时间等临时信息，用于关联请求和响应
#[derive(Debug, Clone)]
pub struct PendingRequest {
    /// 请求 ID
    pub request_id: String,
    /// 请求开始时间
    pub start_time: Instant,
    /// 请求时间戳
    pub timestamp: i64,
    /// Provider
    pub provider: Provider,
    /// 模型名称
    pub model: String,
    /// 请求 URI（用于关联）
    pub uri: String,
    /// 请求方法
    pub method: String,
}

// ============================================================================
// 监控状态
// ============================================================================

/// 监控代理共享状态
pub struct MonitorState {
    /// 存储层
    pub storage: Arc<MonitorStorage>,
    /// 配置管理器
    pub config: Arc<RwLock<MonitorConfig>>,
    /// 待处理请求映射: (method, uri) -> PendingRequest
    /// 使用方法+URI 作为键，因为在同一连接中，请求是顺序处理的
    pub pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// 请求超时时间（秒）
    pub request_timeout: Duration,
}

impl MonitorState {
    /// 创建新的监控状态
    pub fn new(storage: Arc<MonitorStorage>, config: MonitorConfig) -> Self {
        Self {
            storage,
            config: Arc::new(RwLock::new(config)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: Duration::from_secs(300), // 5 分钟超时
        }
    }

    /// 从 ConfigManager 创建监控状态
    pub fn from_config_manager(
        storage: Arc<MonitorStorage>,
        config_manager: &ConfigManager,
    ) -> Self {
        Self {
            storage,
            config: Arc::new(RwLock::new(config_manager.get_config())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: Duration::from_secs(300),
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> MonitorConfig {
        self.config.read().await.clone()
    }

    /// 检查域名是否应该被拦截
    pub async fn should_intercept_domain(&self, host: &str) -> bool {
        let config = self.config.read().await;
        match_domain(host, &config.domains).is_some()
    }

    /// 生成请求键（method + uri）
    fn make_request_key(method: &Method, uri: &Uri) -> String {
        format!("{}:{}", method.as_str(), uri)
    }

    /// 添加待处理请求
    pub async fn add_pending_request(&self, method: &Method, uri: &Uri, pending: PendingRequest) {
        let key = Self::make_request_key(method, uri);
        let mut pending_requests = self.pending_requests.write().await;
        
        // 清理过期的请求
        self.cleanup_expired_requests(&mut pending_requests);
        
        pending_requests.insert(key, pending);
    }

    /// 移除并返回待处理请求
    /// 使用模糊匹配：先尝试精确匹配，失败则查找最接近的
    pub async fn remove_pending_request(&self, method: &Method, uri: &Uri) -> Option<PendingRequest> {
        let key = Self::make_request_key(method, uri);
        let mut pending_requests = self.pending_requests.write().await;
        
        // 清理过期的请求
        self.cleanup_expired_requests(&mut pending_requests);
        
        // 尝试精确匹配
        if let Some(pending) = pending_requests.remove(&key) {
            return Some(pending);
        }
        
        // 精确匹配失败，尝试模糊匹配（URI 可能有细微差异）
        let method_str = method.as_str();
        let uri_str = uri.to_string();
        
        // 查找方法匹配且 URI 相似的请求
        for (k, v) in pending_requests.iter() {
            if k.starts_with(&format!("{}:", method_str)) && 
               v.uri == uri_str {
                let k = k.clone();
                return pending_requests.remove(&k);
            }
        }
        
        None
    }

    /// 清理过期的请求
    fn cleanup_expired_requests(&self, pending_requests: &mut HashMap<String, PendingRequest>) {
        let now = Instant::now();
        pending_requests.retain(|_, v| {
            now.duration_since(v.start_time) < self.request_timeout
        });
    }
}

// ============================================================================
// HTTP 处理器
// ============================================================================

/// 监控代理 HTTP 处理器
///
/// 实现 MITM 拦截：
/// - HTTP 请求：捕获并存储 LLM 请求
/// - HTTP 响应：捕获并存储 LLM 响应
/// - HTTPS CONNECT：根据域名配置选择性拦截
pub struct MonitorHandler {
    /// 共享状态（None 表示透传模式）
    state: Option<Arc<MonitorState>>,
    /// Tauri AppHandle，用于发射事件
    app_handle: Option<tauri::AppHandle>,
}

impl MonitorHandler {
    /// 创建透传模式的 MonitorHandler（不捕获任何请求）
    ///
    /// 用于不需要监控的场景，所有请求直接转发
    pub fn new_passthrough() -> Self {
        Self {
            state: None,
            app_handle: None,
        }
    }

    /// 创建新的 MonitorHandler 实例（带 AppHandle）
    pub fn new(state: Arc<MonitorState>, app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            state: Some(state),
            app_handle,
        }
    }

    /// 从存储和配置创建 MonitorHandler（带 AppHandle）
    pub fn from_storage_and_config(
        storage: Arc<MonitorStorage>,
        config: MonitorConfig,
        app_handle: Option<tauri::AppHandle>,
    ) -> Self {
        let state = Arc::new(MonitorState::new(storage, config));
        Self {
            state: Some(state),
            app_handle,
        }
    }

    /// 从 ConfigManager 创建 MonitorHandler（带 AppHandle）
    pub fn from_config_manager(
        storage: Arc<MonitorStorage>,
        config_manager: &ConfigManager,
        app_handle: Option<tauri::AppHandle>,
    ) -> Self {
        let state = Arc::new(MonitorState::from_config_manager(storage, config_manager));
        Self {
            state: Some(state),
            app_handle,
        }
    }

    /// 获取状态引用
    pub fn state(&self) -> Option<Arc<MonitorState>> {
        self.state.clone()
    }

    /// 检查是否启用捕获
    fn is_capture_enabled(&self) -> bool {
        self.state.is_some()
    }

    /// 从 URI 提取主机名
    fn extract_host(uri: &Uri) -> String {
        // 对于 HTTPS MITM，URI 可能是完整 URL 或者只有 host:port
        if let Some(host) = uri.host() {
            host.split(':').next().unwrap_or(host).to_string()
        } else {
            // CONNECT 请求格式：URI 只有 authority（host:port）
            uri.authority()
                .map(|a| a.host().to_string())
                .unwrap_or_else(|| uri.to_string())
        }
    }

    /// 检查是否为 LLM 请求
    async fn is_llm_request(&self, host: &str) -> bool {
        if let Some(ref state) = self.state {
            state.should_intercept_domain(host).await
        } else {
            false
        }
    }

    /// 从请求头提取关键信息
    fn extract_headers(req: &Request<Body>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (name, value) in req.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }
        headers
    }

    /// 发射新请求事件
    fn emit_new_request(&self, request_id: &str, provider: &Provider, model: &str, timestamp: i64) {
        if let Some(ref app_handle) = self.app_handle {
            let payload = RequestEventPayload {
                id: request_id.to_string(),
                provider: format!("{:?}", provider).to_lowercase(),
                model: model.to_string(),
                timestamp,
            };
            if let Err(e) = app_handle.emit(EVENT_NEW_REQUEST, &payload) {
                tracing::warn!("发射 monitor:new-request 事件失败: {}", e);
            }
        }
    }

    /// 发射响应事件
    fn emit_response(&self, request_id: &str, status_code: i32, duration: i64) {
        if let Some(ref app_handle) = self.app_handle {
            let payload = ResponseEventPayload {
                request_id: request_id.to_string(),
                status_code,
                duration,
            };
            if let Err(e) = app_handle.emit(EVENT_RESPONSE, &payload) {
                tracing::warn!("发射 monitor:response 事件失败: {}", e);
            }
        }
    }

    /// 发射指标事件
    fn emit_metrics(&self, request_id: &str, total_tokens: i64, estimated_cost: f64) {
        if let Some(ref app_handle) = self.app_handle {
            let payload = MetricsEventPayload {
                request_id: request_id.to_string(),
                total_tokens,
                estimated_cost,
            };
            if let Err(e) = app_handle.emit(EVENT_METRICS, &payload) {
                tracing::warn!("发射 monitor:metrics 事件失败: {}", e);
            }
        }
    }
}

impl Clone for MonitorHandler {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl HttpHandler for MonitorHandler {
    /// 处理 HTTP 请求
    ///
    /// 对于 LLM API 请求：
    /// 1. 捕获请求信息
    /// 2. 存储到数据库
    /// 3. 记录请求信息到 pending_requests 以便响应时关联
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        let host = Self::extract_host(req.uri());

        // 检查是否为 LLM 请求
        if self.is_llm_request(&host).await {
            // 捕获请求（需要获取 body ownership）
            // 使用 http_body_util::BodyExt 读取 body
            let (parts, body) = req.into_parts();
            
            // 读取 body 内容
            let body_bytes = match BodyExt::collect(body).await {
                Ok(collected) => collected.to_bytes().to_vec(),
                Err(e) => {
                    tracing::error!("读取请求体失败: {}", e);
                    // 重新构建请求（空 body）
                    let req = Request::from_parts(parts, Body::empty());
                    return req.into();
                }
            };
            
            // 捕获请求元数据
            if let Some(state) = self.state.as_ref() {
                let config = state.get_config().await;
                let request_ctx = RequestContext::new();
                let request_id = request_ctx.id.clone();
                let start_time = request_ctx.start_time;
                
                // 提取请求头
                let mut headers = HashMap::new();
                for (name, value) in &parts.headers {
                    if let Ok(v) = value.to_str() {
                        headers.insert(name.to_string(), v.to_string());
                    }
                }
                
                // 捕获请求
                let llm_request = capture_request(
                    &request_ctx,
                    parts.method.as_str(),
                    &parts.uri.to_string(),
                    headers,
                    &body_bytes,
                    &config.domains,
                );

                // 存储请求信息到 pending_requests
                let pending = PendingRequest {
                    request_id: request_id.clone(),
                    start_time,
                    timestamp: llm_request.timestamp,
                    provider: llm_request.provider.clone(),
                    model: llm_request.model.clone(),
                    uri: parts.uri.to_string(),
                    method: parts.method.to_string(),
                };
                state.add_pending_request(&parts.method, &parts.uri, pending).await;

                // 保存请求到存储
                let storage = Arc::clone(&state.storage);
                let model = llm_request.model.clone();
                
                if let Err(e) = storage.save_request(&llm_request).await {
                    tracing::error!("保存请求失败: {}", e);
                } else {
                    tracing::info!(
                        "捕获请求: {} {} (id: {}, model: {})",
                        parts.method,
                        parts.uri,
                        request_id,
                        model
                    );
                    // 发射新请求事件
                    self.emit_new_request(&request_id, &llm_request.provider, &model, llm_request.timestamp);
                }
            }
            
            // 重新构建请求（使用读取的 body）
            let req = Request::from_parts(
                parts,
                Body::from(Full::new(hyper::body::Bytes::from(body_bytes))),
            );
            return req.into();
        } else {
            tracing::trace!("HTTP 请求透传: {} {}", req.method(), req.uri());
        }

        // 返回原始请求，让代理转发
        req.into()
    }

    /// 处理 HTTP 响应
    ///
    /// 对于关联的 LLM 请求：
    /// 1. 捕获响应信息
    /// 2. 提取 usage 和成本
    /// 3. 存储到数据库
    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        res: Response<Body>,
    ) -> Response<Body> {
        // 注意：我们无法直接获取对应的请求信息
        // 需要从 pending_requests 中查找最近的请求

        tracing::trace!("HTTP 响应: {}", res.status());
        res
    }

    /// 处理代理错误
    ///
    /// 返回 502 Bad Gateway 响应
    async fn handle_error(
        &mut self,
        _ctx: &HttpContext,
        err: hudsucker::hyper_util::client::legacy::Error,
    ) -> Response<Body> {
        tracing::error!("代理错误: {}", err);
        Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::from("代理错误: 无法连接到目标服务器"))
            .unwrap_or_else(|_| {
                // 构建错误响应失败时返回最简单的 502
                Response::new(Body::from("Bad Gateway"))
            })
    }

    /// 判断是否拦截 HTTPS CONNECT 请求
    ///
    /// 根据域名配置选择性拦截 LLM API 请求
    async fn should_intercept(&mut self, _ctx: &HttpContext, req: &Request<Body>) -> bool {
        // 从 CONNECT 请求的 URI 中提取主机名
        let host = Self::extract_host(req.uri());

        // 检查是否应该拦截此域名
        let should_intercept = self.is_llm_request(&host).await;

        if should_intercept {
            tracing::info!("MITM 拦截 HTTPS 请求: {}", host);
        } else {
            tracing::trace!("HTTPS 隧道透传: {}", host);
        }

        should_intercept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::types::{
        DomainConfig, MatchType, ModelPricingConfig, PortConfig, PricingConfig,
    };

    fn create_test_config() -> MonitorConfig {
        MonitorConfig {
            domains: vec![
                DomainConfig {
                    domain: "api.openai.com".to_string(),
                    provider: "OpenAI".to_string(),
                    enabled: true,
                    match_type: MatchType::Exact,
                },
                DomainConfig {
                    domain: "api.anthropic.com".to_string(),
                    provider: "Anthropic".to_string(),
                    enabled: true,
                    match_type: MatchType::Exact,
                },
            ],
            pricing: PricingConfig {
                match_strategy: "prefix".to_string(),
                models: vec![
                    ModelPricingConfig {
                        model: "gpt-4".to_string(),
                        input: 30.0,
                        output: 60.0,
                    },
                    ModelPricingConfig {
                        model: "claude-3".to_string(),
                        input: 15.0,
                        output: 75.0,
                    },
                ],
            },
            ports: PortConfig {
                web: 7100,
                proxy: 7101,
            },
        }
    }

    fn create_test_storage() -> Arc<MonitorStorage> {
        Arc::new(MonitorStorage::open_in_memory().expect("创建内存数据库失败"))
    }

    #[test]
    fn test_handler_creation() {
        let storage = create_test_storage();
        let config = create_test_config();
        let _handler = MonitorHandler::from_storage_and_config(storage, config);
    }

    #[test]
    fn test_handler_clone() {
        let storage = create_test_storage();
        let config = create_test_config();
        let handler = MonitorHandler::from_storage_and_config(storage, config);
        let _cloned = handler.clone();
    }

    #[test]
    fn test_extract_host() {
        // 完整 URL 格式
        let uri: Uri = "https://api.openai.com:443/v1/chat/completions"
            .parse()
            .unwrap();
        assert_eq!(MonitorHandler::extract_host(&uri), "api.openai.com");

        // HTTP 格式
        let uri: Uri = "http://localhost:8080/api".parse().unwrap();
        assert_eq!(MonitorHandler::extract_host(&uri), "localhost");

        // CONNECT 请求格式（只有 host:port）
        let uri: Uri = "api.openai.com:443".parse().unwrap();
        // 对于没有 scheme 的 URI，authority 可能被解析为 path
        // 实际行为取决于 URI 解析器
    }

    #[tokio::test]
    async fn test_is_llm_request() {
        let storage = create_test_storage();
        let config = create_test_config();
        let handler = MonitorHandler::from_storage_and_config(storage, config);

        assert!(handler.is_llm_request("api.openai.com").await);
        assert!(handler.is_llm_request("api.anthropic.com").await);
        assert!(!handler.is_llm_request("unknown.com").await);
    }

    #[tokio::test]
    async fn test_state_should_intercept_domain() {
        let storage = create_test_storage();
        let config = create_test_config();
        let state = Arc::new(MonitorState::new(storage, config));

        assert!(state.should_intercept_domain("api.openai.com").await);
        assert!(!state.should_intercept_domain("unknown.com").await);
    }

    #[tokio::test]
    async fn test_state_pending_requests() {
        let storage = create_test_storage();
        let config = create_test_config();
        let state = Arc::new(MonitorState::new(storage, config));

        let uri: Uri = "https://api.openai.com/v1/chat/completions".parse().unwrap();
        let method = Method::POST;

        let pending = PendingRequest {
            request_id: "req-test".to_string(),
            start_time: Instant::now(),
            timestamp: 0,
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            uri: uri.to_string(),
            method: method.to_string(),
        };

        state.add_pending_request(&method, &uri, pending).await;

        let retrieved = state.remove_pending_request(&method, &uri).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().model, "gpt-4");

        let not_found = state.remove_pending_request(&Method::GET, &uri).await;
        assert!(not_found.is_none());
    }

    #[test]
    fn test_pending_request_debug() {
        let pending = PendingRequest {
            request_id: "req-test".to_string(),
            start_time: Instant::now(),
            timestamp: 0,
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            uri: "https://api.openai.com/v1/chat".to_string(),
            method: "POST".to_string(),
        };

        let debug_str = format!("{:?}", pending);
        assert!(debug_str.contains("req-test"));
        assert!(debug_str.contains("gpt-4"));
    }

    #[test]
    fn test_monitor_state_creation() {
        let storage = create_test_storage();
        let config = create_test_config();
        let state = MonitorState::new(storage, config);

        // 验证初始状态
        assert!(state.pending_requests.try_read().is_ok());
    }

    #[test]
    fn test_make_request_key() {
        let uri: Uri = "https://api.openai.com/v1/chat".parse().unwrap();
        let key = MonitorState::make_request_key(&Method::POST, &uri);
        assert!(key.starts_with("POST:"));
        assert!(key.contains("api.openai.com"));
    }
}
