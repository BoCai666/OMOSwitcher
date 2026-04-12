// Monitor 模块 - 请求/响应捕获管道
// 提供请求捕获、响应捕获、Provider 检测、域名匹配等功能
// 移植自 packages/monitor/src/proxy/request-capture.ts 和 response-capture.ts
// 注意：部分函数尚未被 handler 集成调用，保留供后续集成使用

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::monitor::parser::openai::parse_openai_request;
use crate::monitor::parser::sse::SseParser;
use crate::monitor::types::{
    DomainConfig, LLMRequest, LLMResponse, MatchType, ParsedResponseBody, Provider, Usage,
};

// ============================================================================
// ID 生成
// ============================================================================

/// 生成唯一请求 ID
/// 格式: req-{timestamp_base36}-{random_base36}
pub fn generate_request_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let random: u64 = rand::random();
    format!(
        "req-{}-{}",
        base36_encode(timestamp as u64),
        base36_encode(random & 0xFFFFFFFF)
    )
}

/// 生成唯一响应 ID
/// 格式: res-{timestamp_base36}-{random_base36}
pub fn generate_response_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let random: u64 = rand::random();
    format!(
        "res-{}-{}",
        base36_encode(timestamp as u64),
        base36_encode(random & 0xFFFFFFFF)
    )
}

/// 将数字转换为 base36 字符串
fn base36_encode(mut n: u64) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

// ============================================================================
// 域名匹配
// ============================================================================

/// 检查主机名是否匹配配置的域名列表
/// 支持 exact（精确匹配）、glob（通配符）、regex（正则）三种匹配类型
pub fn match_domain<'a>(host: &str, domains: &'a [DomainConfig]) -> Option<&'a DomainConfig> {
    for domain_config in domains {
        if !domain_config.enabled {
            continue;
        }

        let is_match = match &domain_config.match_type {
            MatchType::Exact => host == domain_config.domain,
            MatchType::Glob => glob_match(&domain_config.domain, host),
            MatchType::Regex => regex_match(&domain_config.domain, host),
        };

        if is_match {
            return Some(domain_config);
        }
    }
    None
}

/// 通配符匹配
/// 支持 * 匹配任意字符
fn glob_match(pattern: &str, text: &str) -> bool {
    // 简单实现：将 * 替换为 .* 进行匹配
    // 如果没有通配符，则精确匹配
    if !pattern.contains('*') {
        return text == pattern;
    }

    // 将 pattern 转换为正则表达式
    let regex_pattern = pattern.replace(".", r"\.").replace("*", ".*");

    regex_match(&regex_pattern, text)
}

/// 正则表达式匹配
fn regex_match(pattern: &str, text: &str) -> bool {
    // 使用正则表达式库进行匹配
    match regex::Regex::new(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => {
            tracing::warn!("无效的正则表达式: {}", pattern);
            false
        }
    }
}

// ============================================================================
// Provider 检测
// ============================================================================

/// 从 URL 检测 LLM Provider
/// 根据域名判断 Provider 类型
pub fn detect_provider(url: &str, domains: &[DomainConfig]) -> Provider {
    // 提取主机名
    let host = extract_host(url);

    // 查找匹配的域名配置
    if let Some(domain_config) = match_domain(&host, domains) {
        return provider_from_string(&domain_config.provider);
    }

    // 内置的 Provider 检测（作为后备）
    if host.contains("openai.com") || host.contains("api.openai.com") {
        Provider::OpenAI
    } else if host.contains("anthropic.com") || host.contains("api.anthropic.com") {
        Provider::Anthropic
    } else if host.contains("kimi.com") || host.contains("api.kimi.com") {
        Provider::Kimi
    } else {
        Provider::Unknown
    }
}

/// 从字符串解析 Provider
fn provider_from_string(s: &str) -> Provider {
    match s.to_lowercase().as_str() {
        "openai" => Provider::OpenAI,
        "anthropic" => Provider::Anthropic,
        "kimi" => Provider::Kimi,
        _ => Provider::Unknown,
    }
}

/// 从 URL 提取主机名
fn extract_host(url: &str) -> String {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .and_then(|host| host.split(':').next())
        .unwrap_or("")
        .to_string()
}

// ============================================================================
// 请求捕获
// ============================================================================

/// 请求捕获上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 请求 ID
    pub id: String,
    /// 请求时间戳
    pub timestamp: i64,
    /// 请求开始时间（用于计算耗时）
    pub start_time: std::time::Instant,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            id: generate_request_id(),
            timestamp,
            start_time: std::time::Instant::now(),
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 捕获请求管道
/// 解析请求体 -> 检测 Provider -> 构建 LLMRequest
pub fn capture_request(
    ctx: &RequestContext,
    method: &str,
    url: &str,
    headers: HashMap<String, String>,
    body: &[u8],
    domains: &[DomainConfig],
) -> LLMRequest {
    // 检测 Provider
    let provider = detect_provider(url, domains);

    // 提取域名
    let domain = extract_host(url);

    // 解析请求体
    let (body_json, parsed_body, model) = parse_request_body(body);

    LLMRequest {
        id: ctx.id.clone(),
        timestamp: ctx.timestamp,
        provider,
        model,
        method: method.to_string(),
        url: url.to_string(),
        domain: Some(domain),
        headers: sanitize_headers(headers),
        body: body_json,
        parsed_body,
        updated_at: None,
    }
}

/// 解析请求体
/// 返回 (JSON Value, ParsedRequestBody, model)
fn parse_request_body(
    body: &[u8],
) -> (
    serde_json::Value,
    Option<crate::monitor::types::ParsedRequestBody>,
    String,
) {
    if body.is_empty() {
        return (serde_json::Value::Null, None, "unknown".to_string());
    }

    // 尝试解析 JSON
    match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(json) => {
            // 使用 OpenAI 解析器解析
            let parsed = parse_openai_request(&json);

            // 提取 model
            let model = json
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            (json, Some(parsed), model)
        }
        Err(_) => {
            // 非 JSON，存储原始字符串
            let body_str = String::from_utf8_lossy(body).to_string();
            (
                serde_json::Value::String(body_str),
                None,
                "unknown".to_string(),
            )
        }
    }
}

/// 清理敏感 Header
/// 将 authorization、cookie 等敏感信息替换为 [REDACTED]
fn sanitize_headers(headers: HashMap<String, String>) -> HashMap<String, String> {
    let mut sanitized = HashMap::new();

    for (key, value) in headers {
        let lower_key = key.to_lowercase();

        let sanitized_value = if lower_key == "authorization"
            || lower_key == "proxy-authorization"
            || lower_key == "cookie"
        {
            "[REDACTED]".to_string()
        } else {
            value
        };

        sanitized.insert(key, sanitized_value);
    }

    sanitized
}

// ============================================================================
// 响应捕获
// ============================================================================

/// 响应捕获上下文
#[derive(Debug, Clone)]
pub struct ResponseContext {
    /// 响应 ID
    pub id: String,
    /// 关联的请求 ID
    pub request_id: String,
    /// 响应时间戳
    pub timestamp: i64,
    /// 请求开始时间（用于计算耗时）
    pub start_time: std::time::Instant,
}

impl ResponseContext {
    /// 创建新的响应上下文
    pub fn new(request_id: &str, start_time: std::time::Instant) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            id: generate_response_id(),
            request_id: request_id.to_string(),
            timestamp,
            start_time,
        }
    }
}

/// 捕获非流式响应
pub fn capture_non_stream_response(
    ctx: &ResponseContext,
    status_code: i32,
    headers: HashMap<String, String>,
    body: &[u8],
) -> LLMResponse {
    let duration = ctx.start_time.elapsed().as_millis() as i64;

    // 解析响应体
    let (body_json, parsed_body) = parse_response_body(body, false);

    LLMResponse {
        id: ctx.id.clone(),
        request_id: ctx.request_id.clone(),
        timestamp: ctx.timestamp,
        status_code,
        headers: convert_headers(headers),
        body: body_json,
        parsed_body,
        duration,
    }
}

/// 解析响应体
/// 返回 (JSON Value, ParsedResponseBody)
fn parse_response_body(
    body: &[u8],
    is_streaming: bool,
) -> (serde_json::Value, Option<ParsedResponseBody>) {
    if body.is_empty() {
        return (serde_json::Value::Null, None);
    }

    // 尝试解析 JSON
    match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(json) => {
            // 使用 OpenAI 解析器解析
            let parsed = crate::monitor::parser::openai::parse_openai_response(&json);
            (json, Some(parsed))
        }
        Err(_) => {
            // 如果是流式响应，尝试 SSE 解析
            if is_streaming {
                let body_str = String::from_utf8_lossy(body);
                let sse_parsed = parse_sse_response(&body_str);
                (
                    serde_json::Value::String(body_str.to_string()),
                    Some(sse_parsed),
                )
            } else {
                // 非 JSON，存储原始字符串
                let body_str = String::from_utf8_lossy(body).to_string();
                (serde_json::Value::String(body_str), None)
            }
        }
    }
}

/// 解析 SSE 流式响应
fn parse_sse_response(body: &str) -> ParsedResponseBody {
    let mut parser = SseParser::new();
    parser.feed_chunk(body);
    parser.get_result()
}

/// 转换 Header 格式
fn convert_headers(headers: HashMap<String, String>) -> HashMap<String, String> {
    // 当前 headers 格式已经是 HashMap<String, String>，直接返回
    headers
}

// ============================================================================
// 流式响应累积器
// ============================================================================

/// 流式响应累积器
/// 用于增量累积流式响应的 content、thinking 和 usage
pub struct StreamingResponseAccumulator {
    /// SSE 解析器
    parser: SseParser,
    /// 响应上下文
    ctx: ResponseContext,
    /// 状态码
    status_code: i32,
    /// 响应头
    headers: HashMap<String, String>,
    /// 原始 body 累积
    raw_body: Vec<u8>,
}

impl StreamingResponseAccumulator {
    /// 创建新的流式响应累积器
    pub fn new(
        request_id: &str,
        start_time: std::time::Instant,
        status_code: i32,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            parser: SseParser::new(),
            ctx: ResponseContext::new(request_id, start_time),
            status_code,
            headers,
            raw_body: Vec::new(),
        }
    }

    /// 处理流式数据块
    pub fn feed_chunk(&mut self, chunk: &[u8]) {
        self.raw_body.extend_from_slice(chunk);

        // 尝试解析为 UTF-8 并喂给 SSE 解析器
        if let Ok(text) = std::str::from_utf8(chunk) {
            self.parser.feed_chunk(text);
        }
    }

    /// 获取当前累积的解析结果
    pub fn get_parsed_body(&self) -> ParsedResponseBody {
        self.parser.get_result()
    }

    /// 检查是否收到 [DONE] 信号
    pub fn is_done(&self) -> bool {
        self.parser.is_done()
    }

    /// 获取累积的原始 body
    pub fn get_raw_body(&self) -> &[u8] {
        &self.raw_body
    }

    /// 构建最终的 LLMResponse
    pub fn build_response(self) -> LLMResponse {
        let duration = self.ctx.start_time.elapsed().as_millis() as i64;
        let parsed_body = self.parser.get_result();

        // 尝试解析原始 body 为 JSON
        let body_json = if self.raw_body.is_empty() {
            serde_json::Value::Null
        } else {
            match serde_json::from_slice::<serde_json::Value>(&self.raw_body) {
                Ok(json) => json,
                Err(_) => {
                    // 非JSON，存储原始字符串
                    let body_str = String::from_utf8_lossy(&self.raw_body).to_string();
                    serde_json::Value::String(body_str)
                }
            }
        };

        LLMResponse {
            id: self.ctx.id,
            request_id: self.ctx.request_id,
            timestamp: self.ctx.timestamp,
            status_code: self.status_code,
            headers: self.headers,
            body: body_json,
            parsed_body: Some(parsed_body),
            duration,
        }
    }

    /// 获取请求 ID
    pub fn request_id(&self) -> &str {
        &self.ctx.request_id
    }
}

// ============================================================================
// Usage 提取
// ============================================================================

/// 从响应中提取 Usage 信息
pub fn extract_usage_from_response(response: &LLMResponse) -> Option<Usage> {
    response.parsed_body.as_ref()?.usage.clone()
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_request_id() {
        let id = generate_request_id();
        assert!(id.starts_with("req-"));
        assert!(id.len() > 10);
    }

    #[test]
    fn test_generate_response_id() {
        let id = generate_response_id();
        assert!(id.starts_with("res-"));
        assert!(id.len() > 10);
    }

    #[test]
    fn test_base36_encode() {
        assert_eq!(base36_encode(0), "0");
        assert_eq!(base36_encode(1), "1");
        assert_eq!(base36_encode(10), "a");
        assert_eq!(base36_encode(35), "z");
        assert_eq!(base36_encode(36), "10");
        assert_eq!(base36_encode(1234), "ya");
    }

    #[test]
    fn test_match_domain_exact() {
        let domains = vec![DomainConfig {
            domain: "api.openai.com".to_string(),
            provider: "OpenAI".to_string(),
            enabled: true,
            match_type: MatchType::Exact,
        }];

        assert!(match_domain("api.openai.com", &domains).is_some());
        assert!(match_domain("api.anthropic.com", &domains).is_none());
        assert!(match_domain("api.openai.com:443", &domains).is_none()); // 精确匹配不含端口
    }

    #[test]
    fn test_match_domain_glob() {
        let domains = vec![DomainConfig {
            domain: "*.openai.com".to_string(),
            provider: "OpenAI".to_string(),
            enabled: true,
            match_type: MatchType::Glob,
        }];

        assert!(match_domain("api.openai.com", &domains).is_some());
        assert!(match_domain("chat.openai.com", &domains).is_some());
        assert!(match_domain("openai.com", &domains).is_none());
    }

    #[test]
    fn test_match_domain_disabled() {
        let domains = vec![DomainConfig {
            domain: "api.openai.com".to_string(),
            provider: "OpenAI".to_string(),
            enabled: false, // 已禁用
            match_type: MatchType::Exact,
        }];

        assert!(match_domain("api.openai.com", &domains).is_none());
    }

    #[test]
    fn test_detect_provider() {
        let domains = vec![
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
        ];

        assert_eq!(
            detect_provider("https://api.openai.com/v1/chat/completions", &domains),
            Provider::OpenAI
        );
        assert_eq!(
            detect_provider("https://api.anthropic.com/v1/messages", &domains),
            Provider::Anthropic
        );
        assert_eq!(
            detect_provider("https://unknown.com/api", &domains),
            Provider::Unknown
        );
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(
            extract_host("https://api.openai.com/v1/chat/completions"),
            "api.openai.com"
        );
        assert_eq!(
            extract_host("https://api.openai.com:443/v1/chat/completions"),
            "api.openai.com"
        );
        assert_eq!(extract_host("http://localhost:8080/api"), "localhost");
    }

    #[test]
    fn test_sanitize_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer secret".to_string());
        headers.insert("Cookie".to_string(), "session=123".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let sanitized = sanitize_headers(headers);

        assert_eq!(sanitized.get("Authorization").unwrap(), "[REDACTED]");
        assert_eq!(sanitized.get("Cookie").unwrap(), "[REDACTED]");
        assert_eq!(sanitized.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_capture_request() {
        let ctx = RequestContext::new();
        let domains = vec![DomainConfig {
            domain: "api.openai.com".to_string(),
            provider: "OpenAI".to_string(),
            enabled: true,
            match_type: MatchType::Exact,
        }];

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer test-key".to_string());

        let body = r#"{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}"#;

        let request = capture_request(
            &ctx,
            "POST",
            "https://api.openai.com/v1/chat/completions",
            headers,
            body.as_bytes(),
            &domains,
        );

        assert!(request.id.starts_with("req-"));
        assert_eq!(request.provider, Provider::OpenAI);
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(request.domain, Some("api.openai.com".to_string()));
        assert_eq!(request.headers.get("Authorization").unwrap(), "[REDACTED]");
        assert!(request.parsed_body.is_some());
    }

    #[test]
    fn test_capture_non_stream_response() {
        let ctx = RequestContext::new();
        let start_time = std::time::Instant::now();

        // 短暂等待以产生可测量的 duration
        std::thread::sleep(std::time::Duration::from_millis(10));

        let response_ctx = ResponseContext::new(&ctx.id, start_time);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = r#"{"id": "chatcmpl-123", "choices": [{"message": {"content": "Hello!"}}], "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}}"#;

        let response =
            capture_non_stream_response(&response_ctx, 200, headers.clone(), body.as_bytes());

        assert!(response.id.starts_with("res-"));
        assert_eq!(response.request_id, ctx.id);
        assert_eq!(response.status_code, 200);
        assert!(response.duration >= 10);
        assert!(response.parsed_body.is_some());

        let parsed = response.parsed_body.unwrap();
        assert_eq!(parsed.content, Some("Hello!".to_string()));
        assert!(parsed.usage.is_some());
        let usage = parsed.usage.unwrap();
        assert_eq!(usage.total_tokens, 15);
    }

    #[test]
    fn test_streaming_response_accumulator() {
        let start_time = std::time::Instant::now();
        let mut accumulator =
            StreamingResponseAccumulator::new("req-test", start_time, 200, HashMap::new());

        // 喂入 SSE 数据
        let chunk1 = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
        let chunk2 = "data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n";
        let chunk3 = "data: [DONE]\n\n";

        accumulator.feed_chunk(chunk1.as_bytes());
        accumulator.feed_chunk(chunk2.as_bytes());
        accumulator.feed_chunk(chunk3.as_bytes());

        assert!(accumulator.is_done());

        let parsed = accumulator.get_parsed_body();
        assert_eq!(parsed.content, Some("Hello world".to_string()));
    }

    #[test]
    fn test_request_context_default() {
        let ctx = RequestContext::default();
        assert!(ctx.id.starts_with("req-"));
        assert!(ctx.timestamp > 0);
    }
}
