// Monitor 模块 - 请求/响应捕获管道
// 提供请求捕获、响应捕获、Provider 检测、域名匹配等功能
// 移植自 packages/monitor/src/proxy/request-capture.ts 和 response-capture.ts

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

    tracing::debug!("检测 Provider: url={}, host={}", url, host);

    // 查找匹配的域名配置
    if let Some(domain_config) = match_domain(&host, domains) {
        tracing::debug!(
            "域名配置匹配: host={}, provider={}",
            host,
            domain_config.provider
        );
        return provider_from_string(&domain_config.provider);
    }

    // 内置的 Provider 检测（作为后备）
    let provider = detect_provider_by_host(&host);
    tracing::debug!("内置 Provider 检测: host={}, provider={:?}", host, provider);
    provider
}

/// 根据主机名检测 Provider
fn detect_provider_by_host(host: &str) -> Provider {
    let host_lower = host.to_lowercase();

    if host_lower.contains("openai.com") {
        Provider::OpenAI
    } else if host_lower.contains("anthropic.com") {
        Provider::Anthropic
    } else if host_lower.contains("kimi.com") || host_lower.contains("moonshot.cn") {
        Provider::Kimi
    } else if host_lower.contains("bigmodel.cn") || host_lower.contains("zhipuai") {
        Provider::ZhipuAI
    } else if host_lower.contains("minimaxi.com") || host_lower.contains("minimax.chat") {
        Provider::MiniMax
    } else if host_lower.contains("baidubce.com") || host_lower.contains("qianfan") {
        Provider::Qianfan
    } else if host_lower.contains("volces.com") || host_lower.contains("volcengineapi.com") {
        Provider::Volces
    } else if host_lower.contains("infini-ai.com") {
        Provider::InfiniAI
    } else if host_lower.contains("jdcloud.com") {
        Provider::JDCloud
    } else if host_lower.contains("googleapis.com") || host_lower.contains("generativelanguage") {
        Provider::Google
    } else if host_lower.contains("deepseek.com") {
        Provider::DeepSeek
    } else if host_lower.contains("groq.com") {
        Provider::Groq
    } else if host_lower.contains("mistral.ai") {
        Provider::Mistral
    } else if host_lower.contains("dashscope") || host_lower.contains("aliyuncs.com") {
        Provider::Qwen
    } else if host_lower.contains("siliconflow") {
        Provider::SiliconFlow
    } else {
        Provider::Unknown
    }
}

/// 从字符串解析 Provider
fn provider_from_string(s: &str) -> Provider {
    match s.to_lowercase().as_str() {
        "openai" => Provider::OpenAI,
        "anthropic" => Provider::Anthropic,
        "kimi" | "moonshot" => Provider::Kimi,
        "zhipuai" | "zhipu" | "glm" => Provider::ZhipuAI,
        "minimax" | "minimaxi" => Provider::MiniMax,
        "qianfan" | "baidu" => Provider::Qianfan,
        "volces" | "doubao" | "volcengine" => Provider::Volces,
        "infini" | "infini-ai" | "wuwen" => Provider::InfiniAI,
        "jdcloud" | "jingdong" | "jd" => Provider::JDCloud,
        "google" | "gemini" => Provider::Google,
        "deepseek" => Provider::DeepSeek,
        "groq" => Provider::Groq,
        "mistral" => Provider::Mistral,
        "qwen" | "tongyi" | "dashscope" => Provider::Qwen,
        "siliconflow" | "sf" => Provider::SiliconFlow,
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
    tracing::debug!(
        "捕获请求: id={}, method={}, url={}, body_len={}",
        ctx.id,
        method,
        url,
        body.len()
    );

    // 检测 Provider
    let provider = detect_provider(url, domains);

    // 提取域名
    let domain = extract_host(url);

    // 解析请求体
    let (body_json, parsed_body, model) = parse_request_body(body);

    tracing::debug!(
        "请求解析完成: id={}, provider={:?}, model={}, domain={}",
        ctx.id,
        provider,
        model,
        domain
    );

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
        tracing::debug!("请求体为空，返回默认值");
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

            tracing::debug!(
                "请求体 JSON 解析成功: model={}, has_messages={}",
                model,
                parsed.messages.is_some()
            );

            // 详细追踪 messages 结构（用于排查前端报错）
            if let Some(ref msgs) = parsed.messages {
                tracing::debug!("messages 数组长度: {}", msgs.len());
                for (i, msg) in msgs.iter().enumerate().take(3) {
                    // 只打印前 3 条，避免日志过长
                    if let Some(content) = msg.get("content") {
                        let content_type = if content.is_string() {
                            "string"
                        } else if content.is_array() {
                            let arr = content.as_array().unwrap();
                            let types: Vec<&str> = arr
                                .iter()
                                .map(|block| {
                                    if block.is_null() {
                                        "null"
                                    } else if block.is_object() {
                                        block
                                            .get("type")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("unknown_object")
                                    } else {
                                        "other"
                                    }
                                })
                                .collect();
                            &format!("array[{}] = {:?}", arr.len(), types)
                        } else if content.is_object() {
                            "object"
                        } else if content.is_null() {
                            "null"
                        } else {
                            "other"
                        };
                        tracing::debug!("messages[{}].content 类型: {}", i, content_type);
                    }
                }
            }

            (json, Some(parsed), model)
        }
        Err(e) => {
            // 非 JSON，存储原始字符串
            tracing::debug!("请求体非 JSON 格式，存储原始字符串: error={}", e);
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
// Usage 提取
// ============================================================================

/// 从响应中提取 Usage 信息
pub fn extract_usage_from_response(response: &LLMResponse) -> Option<Usage> {
    response.parsed_body.as_ref()?.usage.clone()
}
