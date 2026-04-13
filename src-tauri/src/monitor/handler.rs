// Monitor 模块 - HTTP 请求/响应处理器
// 实现 hudsucker 的 HttpHandler trait
// 完整 MITM 拦截实现：域名过滤、请求/响应捕获、错误隔离

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hudsucker::{
    hyper::{Method, Request, Response, StatusCode, Uri},
    Body, HttpContext, HttpHandler, RequestOrResponse,
};
use http_body_util::{BodyExt, Full};
use tokio::sync::RwLock;

use crate::monitor::capture::{
    capture_non_stream_response, capture_request, extract_usage_from_response, match_domain,
    generate_request_id, RequestContext, ResponseContext,
};
use crate::monitor::config::ConfigManager;
use crate::monitor::parser::cost::calculate_cost;
use crate::monitor::parser::mcp::detect_mcp_call;
use crate::monitor::parser::sse::SseParser;
use crate::monitor::storage::MonitorStorage;
use crate::monitor::types::{LLMMetrics, LLMResponse, MCPCall, MonitorConfig, PricingConfig, Provider};
use crate::monitor::{
    MetricsEventPayload, RequestEventPayload, ResponseEventPayload, EVENT_METRICS,
    EVENT_NEW_REQUEST, EVENT_RESPONSE,
};

use tauri::Emitter;

// ============================================================================
// LLM API 路径检测
// ============================================================================

/// 已知的 LLM API 路径模式
/// 用于在域名不在配置列表时，通过路径模式 fallback 检测 LLM 请求
const LLM_API_PATHS: &[&str] = &[
    // OpenAI 兼容 API（大多数 Provider 使用）
    "/v1/chat/completions",
    "/v1/completions",
    "/v1/embeddings",
    // Anthropic
    "/v1/messages",
    // Google Gemini
    "/v1beta/models",
    "/v1/models:generateContent",
    "/v1/models:streamGenerateContent",
    // 其他常见路径
    "/chat/completions",
    "/completions",
    "/v2/coding",
];

/// 检查路径是否匹配 LLM API 路径模式
fn is_llm_api_path(path: &str) -> bool {
    LLM_API_PATHS.iter().any(|pattern| path.starts_with(pattern))
}

/// 已知的 LLM API 域名后缀模式
/// 用于 CONNECT 阶段 fallback 检测，即使域名不在配置列表中也进行 MITM
const LLM_DOMAIN_SUFFIXES: &[&str] = &[
    // OpenAI
    ".openai.com",
    // Anthropic
    ".anthropic.com",
    // Google Gemini
    "generativelanguage.googleapis.com",
    ".googleapis.com",
    // 智谱 AI (Zhipu/GLM)
    "open.bigmodel.cn",
    ".bigmodel.cn",
    // MiniMax
    ".minimaxi.com",
    "api.minimax.chat",
    // 百度千帆
    ".baidubce.com",
    // 字节豆包 (Volces/Doubao)
    ".volces.com",
    ".volcengineapi.com",
    // 无问芯穹 (Infini-AI)
    ".infini-ai.com",
    // 京东云
    ".jdcloud.com",
    // Moonshot/Kimi
    ".kimi.com",
    ".moonshot.cn",
    // DeepSeek
    ".deepseek.com",
    // Groq
    ".groq.com",
    // Mistral
    ".mistral.ai",
    // Cohere
    ".cohere.ai",
    ".cohere.com",
    // 通义千问 (Qwen/Alibaba)
    ".dashscope.aliyuncs.com",
    // 零一万物 (Yi)
    "api.lingyiwanwu.com",
    // 硅基流动 (SiliconFlow)
    ".siliconflow.cn",
];

/// 检查域名是否为已知的 LLM API 域名
fn is_known_llm_domain(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    LLM_DOMAIN_SUFFIXES.iter().any(|suffix| {
        host_lower == suffix.trim_start_matches('.')
            || host_lower.ends_with(suffix)
    })
}

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
    /// 待处理请求队列（先进先出，用于响应关联）
    /// HTTP/1.1 管道化是顺序的，对同一个连接来说响应也是顺序到达的
    pub pending_queue: Arc<RwLock<VecDeque<PendingRequest>>>,
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
            pending_queue: Arc::new(RwLock::new(VecDeque::new())),
            request_timeout: Duration::from_secs(300), // 5 分钟超时
        }
    }

    /// 从 ConfigManager 创建监控状态
    #[allow(dead_code)]
    pub fn from_config_manager(
        storage: Arc<MonitorStorage>,
        config_manager: &ConfigManager,
    ) -> Self {
        Self {
            storage,
            config: Arc::new(RwLock::new(config_manager.get_config())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_queue: Arc::new(RwLock::new(VecDeque::new())),
            request_timeout: Duration::from_secs(300),
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> MonitorConfig {
        self.config.read().await.clone()
    }

    /// 获取定价配置
    pub async fn get_pricing(&self) -> PricingConfig {
        self.config.read().await.pricing.clone()
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

    /// 添加待处理请求（同时添加到 HashMap 和队列）
    pub async fn add_pending_request(&self, method: &Method, uri: &Uri, pending: PendingRequest) {
        let key = Self::make_request_key(method, uri);
        
        // 添加到队列
        {
            let mut queue = self.pending_queue.write().await;
            queue.push_back(pending.clone());
        }
        
        // 添加到 HashMap
        {
            let mut pending_requests = self.pending_requests.write().await;
            
            // 清理过期的请求
            self.cleanup_expired_requests(&mut pending_requests);
            
            pending_requests.insert(key, pending);
        }
    }

    /// 从队列中弹出待处理请求（用于响应关联）
    /// 优先使用队列，失败时使用 HashMap 模糊查找作为 fallback
    pub async fn pop_pending_request(&self) -> Option<PendingRequest> {
        // 优先从队列弹出
        {
            let mut queue = self.pending_queue.write().await;
            if let Some(pending) = queue.pop_front() {
                // 同时从 HashMap 中移除
                let key = format!("{}:{}", pending.method, pending.uri);
                let mut pending_requests = self.pending_requests.write().await;
                pending_requests.remove(&key);
                return Some(pending);
            }
        }
        
        // 队列为空时，尝试从 HashMap 中获取最早的一个
        let mut pending_requests = self.pending_requests.write().await;
        
        // 清理过期的请求
        self.cleanup_expired_requests(&mut pending_requests);
        
        // 查找最早的请求（按 timestamp）
        if let Some((key, pending)) = pending_requests
            .iter()
            .min_by_key(|(_, p)| p.timestamp)
            .map(|(k, p)| (k.clone(), p.clone()))
        {
            pending_requests.remove(&key);
            return Some(pending);
        }
        
        None
    }

    /// 移除并返回待处理请求（兼容旧 API）
    /// 使用模糊匹配：先尝试精确匹配，失败则查找最接近的
    #[allow(dead_code)]
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
    /// AppHandle，用于发射事件
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
    #[allow(dead_code)]
    pub fn new(state: Arc<MonitorState>, app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            state: Some(state),
            app_handle,
        }
    }

    /// 从存储和配置创建 MonitorHandler（带 AppHandle）
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn state(&self) -> Option<Arc<MonitorState>> {
        self.state.clone()
    }

    /// 检查是否启用捕获
    #[allow(dead_code)]
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
    /// 优先检查域名匹配，其次通过路径模式 fallback 检测
    async fn is_llm_request(&self, host: &str, uri: &Uri) -> bool {
        if let Some(ref state) = self.state {
            // 域名匹配
            if state.should_intercept_domain(host).await {
                return true;
            }
            // 路径模式 fallback：检查 URI 是否匹配常见 LLM API 路径
            let path = uri.path();
            return is_llm_api_path(path);
        }
        false
    }

    /// 从请求头提取关键信息
    #[allow(dead_code)]
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
    fn emit_new_request(&self, request: &crate::monitor::types::LLMRequest) {
        if let Some(ref app_handle) = self.app_handle {
            let payload = RequestEventPayload {
                id: request.id.clone(),
                provider: format!("{:?}", request.provider).to_lowercase(),
                model: request.model.clone(),
                method: request.method.clone(),
                url: request.url.clone(),
                domain: request.domain.clone(),
                timestamp: request.timestamp,
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
    /// 2. 检测 MCP 调用
    /// 3. 存储到数据库
    /// 4. 记录请求信息到 pending_requests 以便响应时关联
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        // CONNECT 是 HTTPS 隧道建立请求，不是实际的 API 调用，直接透传
        if req.method() == Method::CONNECT {
            tracing::debug!("跳过 CONNECT 请求（HTTPS 隧道）");
            return req.into();
        }

        tracing::debug!("处理 HTTP 请求: {} {}", req.method(), req.uri());

        // HTTPS MITM 解密后的请求，URI 只有路径（如 /v1/chat/completions），
        // 没有 host 信息。需要从 Host header 中获取域名。
        let uri_host = Self::extract_host(req.uri());
        let host = if uri_host.is_empty() || uri_host.starts_with('/') {
            // URI 中没有 host，从 Host header 提取
            req.headers()
                .get("host")
                .and_then(|v| v.to_str().ok())
                .map(|h| h.split(':').next().unwrap_or(h).to_string())
                .unwrap_or_default()
        } else {
            uri_host
        };

        // 检查是否为 LLM 请求（域名匹配 + 路径模式 fallback）
        let is_llm = self.is_llm_request(&host, req.uri()).await;
        tracing::debug!("LLM 请求检测: host={}, uri={}, is_llm={}", host, req.uri(), is_llm);
        
        if is_llm {
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
                
                // 构建完整 URL（HTTPS MITM 解密后 URI 只有路径，需要拼接 Host header）
                let full_url = {
                    let uri_str = parts.uri.to_string();
                    if uri_str.starts_with('/') || uri_str.starts_with("http") {
                        // 已经是完整 URL 或只有路径
                        if uri_str.starts_with("http") {
                            uri_str
                        } else {
                            // 拼接 host + path
                            let scheme = if parts.uri.scheme_str() == Some("https") { "https" } else { "https" };
                            format!("{}://{}{}", scheme, host, uri_str)
                        }
                    } else {
                        uri_str
                    }
                };

                // 捕获请求
                let llm_request = capture_request(
                    &request_ctx,
                    parts.method.as_str(),
                    &full_url,
                    headers.clone(),
                    &body_bytes,
                    &config.domains,
                );
                
                tracing::debug!(
                    "请求捕获完成: id={}, provider={:?}, model={}, url={}",
                    llm_request.id, llm_request.provider, llm_request.model, llm_request.url
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
                let request_id_for_log = request_id.clone();
                
                if let Err(e) = storage.save_request(&llm_request).await {
                    tracing::error!("保存请求失败: {}", e);
                } else {
                    tracing::debug!("请求已保存到数据库: id={}", request_id_for_log);
                    tracing::info!(
                        "捕获请求: {} {} (id: {}, model: {})",
                        parts.method,
                        parts.uri,
                        request_id,
                        model
                    );
                    // 发射新请求事件
                    self.emit_new_request(&llm_request);
                }

                // MCP 调用检测
                // 尝试解析请求体为 JSON 进行 MCP 检测
                if let Ok(body_json) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                    let mcp_result = detect_mcp_call(&body_json, &parts.uri.to_string(), &headers);
                    
                    if mcp_result.is_mcp_call {
                        let mcp_id = generate_request_id();
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as i64)
                            .unwrap_or(0);
                        
                        let mcp_call = MCPCall {
                            id: mcp_id,
                            request_id: request_id.clone(),
                            jsonrpc_version: mcp_result.jsonrpc_version,
                            rpc_id: mcp_result.rpc_id,
                            tool_name: mcp_result.tool_name.unwrap_or_else(|| "unknown".to_string()),
                            tool_title: None,
                            tool_description: None,
                            arguments: mcp_result.arguments,
                            result_content: None,
                            result_is_error: false,
                            error_message: None,
                            execution_duration: None,
                            transport_type: mcp_result.transport_type,
                            server_name: None,
                            trace_id: None,
                            timestamp,
                        };
                        
                        // 异步保存 MCP 调用记录
                        let storage = Arc::clone(&state.storage);
                        let mcp_call_clone = mcp_call.clone();
                        tokio::spawn(async move {
                            if let Err(e) = storage.save_mcp_call(&mcp_call_clone).await {
                                tracing::error!("保存 MCP 调用记录失败: {}", e);
                            }
                        });
                        
                        tracing::info!(
                            "检测到 MCP 调用: {} (request_id: {}, tool: {})",
                            mcp_call.id,
                            request_id,
                            mcp_call.tool_name
                        );
                    }
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
    /// 1. 捕获响应信息（流式/非流式）
    /// 2. 提取 usage 和成本
    /// 3. 存储到数据库
    /// 4. 发射事件
    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        res: Response<Body>,
    ) -> Response<Body> {
        // 尝试获取关联的请求信息
        let pending = if let Some(state) = self.state.as_ref() {
            state.pop_pending_request().await
        } else {
            None
        };

        let Some(pending) = pending else {
            tracing::trace!("HTTP 响应（无关联请求）: {}", res.status());
            return res;
        };

        tracing::debug!(
            "处理 HTTP 响应: 关联请求 id={}, model={:?}, uri={}",
            pending.request_id, pending.model, pending.uri
        );

        // 读取响应 body
        let (parts, body) = res.into_parts();
        let body_bytes = match BodyExt::collect(body).await {
            Ok(collected) => collected.to_bytes().to_vec(),
            Err(e) => {
                tracing::error!("读取响应体失败: {}", e);
                // 重新构建响应（空 body）
                return Response::from_parts(parts, Body::empty());
            }
        };

        // 提取响应头
        let mut headers = HashMap::new();
        for (name, value) in &parts.headers {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }

        // 判断是否为流式响应
        let content_type = headers.get("content-type").map(|s| s.to_lowercase()).unwrap_or_default();
        let is_streaming = content_type.contains("text/event-stream") 
            || String::from_utf8_lossy(&body_bytes).contains("data:");
        
        tracing::debug!("响应类型判断: content_type={}, is_streaming={}", content_type, is_streaming);

        // 获取定价配置
        let pricing = if let Some(state) = self.state.as_ref() {
            state.get_pricing().await
        } else {
            PricingConfig {
                match_strategy: "prefix".to_string(),
                models: vec![],
            }
        };

        // 创建响应上下文
        let response_ctx = ResponseContext::new(&pending.request_id, pending.start_time);
        let response_id = response_ctx.id.clone();

        // 捕获响应
        let llm_response = if is_streaming {
            // 流式响应：使用 SSE 解析器解析整个响应体
            let body_str = String::from_utf8_lossy(&body_bytes);
            let mut sse_parser = SseParser::new();
            sse_parser.feed_chunk(&body_str);
            let parsed_body = sse_parser.get_result();
            
            // 构建 LLMResponse
            let duration = pending.start_time.elapsed().as_millis() as i64;
            LLMResponse {
                id: response_id,
                request_id: pending.request_id.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0),
                status_code: parts.status.as_u16() as i32,
                headers: headers.clone(),
                body: serde_json::Value::String(body_str.to_string()),
                parsed_body: Some(parsed_body.clone()),
                duration,
            }
        } else {
            // 非流式响应
            capture_non_stream_response(
                &response_ctx,
                parts.status.as_u16() as i32,
                headers.clone(),
                &body_bytes,
            )
        };

        // 提取 usage
        let usage = extract_usage_from_response(&llm_response);
        let duration = llm_response.duration;
        
        tracing::debug!(
            "响应解析完成: id={}, status={}, duration={}ms, usage={:?}",
            llm_response.id, llm_response.status_code, duration, usage
        );

        // 计算费用
        let (prompt_tokens, completion_tokens, total_tokens, estimated_cost) = match &usage {
            Some(u) => {
                let cost = calculate_cost(&pending.model, u.prompt_tokens, u.completion_tokens, &pricing);
                (u.prompt_tokens, u.completion_tokens, u.total_tokens, cost)
            }
            None => (0, 0, 0, 0.0),
        };

        // 异步保存响应
        if let Some(state) = self.state.as_ref() {
            let storage = Arc::clone(&state.storage);
            let response = llm_response.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_response(&response).await {
                    tracing::error!("保存响应失败: {}", e);
                }
            });

            // 构建并保存指标
            let metrics_id = generate_request_id();
            let metrics = LLMMetrics {
                id: metrics_id,
                request_id: pending.request_id.clone(),
                model: pending.model.clone(),
                provider: format!("{:?}", pending.provider).to_lowercase(),
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost,
                duration,
                timestamp: llm_response.timestamp,
            };

            let storage = Arc::clone(&state.storage);
            let metrics_clone = metrics.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_metrics(&metrics_clone).await {
                    tracing::error!("保存指标失败: {}", e);
                }
            });

            tracing::info!(
                "捕获响应: {} (request_id: {}, model: {}, tokens: {}, cost: ${:.6})",
                llm_response.id,
                pending.request_id,
                pending.model,
                total_tokens,
                estimated_cost
            );
        }

        // 发射事件
        self.emit_response(&pending.request_id, parts.status.as_u16() as i32, duration);
        self.emit_metrics(&pending.request_id, total_tokens, estimated_cost);

        // 重新构建响应（使用读取的 body）
        Response::from_parts(parts, Body::from(Full::new(hyper::body::Bytes::from(body_bytes))))
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
    /// 同时拦截常见 LLM API 域名后缀作为 fallback
    async fn should_intercept(&mut self, _ctx: &HttpContext, req: &Request<Body>) -> bool {
        // 从 CONNECT 请求的 URI 中提取主机名
        let host = Self::extract_host(req.uri());
        
        tracing::debug!("HTTPS CONNECT 域名检测: host={}", host);

        // 检查是否应该拦截此域名（配置列表匹配）
        let should_intercept = if self.is_llm_request(&host, req.uri()).await {
            tracing::debug!("域名匹配 LLM 请求（配置列表）: {}", host);
            true
        } else {
            // fallback：常见 LLM API 域名后缀
            let is_known = is_known_llm_domain(&host);
            if is_known {
                tracing::debug!("域名匹配已知 LLM 域名后缀: {}", host);
            }
            is_known
        };

        if should_intercept {
            tracing::info!("MITM 拦截 HTTPS 请求: {}", host);
        } else {
            tracing::trace!("HTTPS 隧道透传: {}", host);
        }

        should_intercept
    }
}
