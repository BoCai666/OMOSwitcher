// MCP (Model Context Protocol) 调用检测器
// 移植自 packages/monitor/src/parsers/mcp-detector.ts

use std::collections::HashMap;

use crate::monitor::types::TransportType;

/// MCP 检测结果
#[derive(Debug, Clone)]
pub struct McpDetectionResult {
    /// 是否为 MCP 调用
    pub is_mcp_call: bool,
    /// 工具名称
    pub tool_name: Option<String>,
    /// 工具参数
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    /// 传输类型
    pub transport_type: Option<TransportType>,
    /// JSON-RPC 版本
    pub jsonrpc_version: Option<String>,
    /// RPC 调用 ID
    pub rpc_id: Option<String>,
}

impl Default for McpDetectionResult {
    fn default() -> Self {
        Self {
            is_mcp_call: false,
            tool_name: None,
            arguments: None,
            transport_type: None,
            jsonrpc_version: None,
            rpc_id: None,
        }
    }
}

/// MCP 结果提取结果
#[derive(Debug, Clone)]
pub struct McpExtractResult {
    /// 结果内容
    pub content: Option<serde_json::Value>,
    /// 是否为错误
    pub is_error: bool,
    /// 错误消息
    pub error_message: Option<String>,
}

impl Default for McpExtractResult {
    fn default() -> Self {
        Self {
            content: None,
            is_error: false,
            error_message: None,
        }
    }
}

/// 检测请求是否为 MCP 调用
///
/// 支持三种检测方式：
/// 1. JSON-RPC 2.0 格式 (标准 MCP)
///    - tools/call: body.params.name, body.params.arguments
///    - tools/list: tool_name = "tools/list"
/// 2. OpenAI Function Calling 格式 (body.tool_choice || body.tools)
/// 3. URL 路径模式 (/mcp, /tools/call)
pub fn detect_mcp_call(
    body: &serde_json::Value,
    url: &str,
    headers: &HashMap<String, String>,
) -> McpDetectionResult {
    // 检测1: JSON-RPC 2.0 格式 (标准 MCP)
    if body.get("jsonrpc").and_then(|v| v.as_str()) == Some("2.0") && body.get("method").is_some() {
        let method = body.get("method").and_then(|v| v.as_str()).unwrap_or("");

        // tools/call 请求
        if method == "tools/call" {
            if let Some(params) = body.get("params") {
                return McpDetectionResult {
                    is_mcp_call: true,
                    jsonrpc_version: Some("2.0".to_string()),
                    rpc_id: body.get("id").and_then(|v| match v {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        _ => None,
                    }),
                    tool_name: params
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    arguments: params.get("arguments").and_then(|v| {
                        v.as_object().map(|obj| {
                            obj.iter()
                                .map(|(k, val)| (k.clone(), val.clone()))
                                .collect()
                        })
                    }),
                    transport_type: Some(detect_transport_type(url, headers)),
                };
            }
        }

        // tools/list 请求
        if method == "tools/list" {
            return McpDetectionResult {
                is_mcp_call: true,
                jsonrpc_version: Some("2.0".to_string()),
                rpc_id: body.get("id").and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }),
                tool_name: Some("tools/list".to_string()),
                arguments: None,
                transport_type: Some(detect_transport_type(url, headers)),
            };
        }
    }

    // 检测2: OpenAI Function Calling 格式
    if body.get("tool_choice").is_some() || body.get("tools").is_some() {
        // 尝试从 tool_choice.function 或 tools[0].function 提取
        let func_call = body
            .get("tool_choice")
            .and_then(|v| v.get("function"))
            .or_else(|| {
                body.get("tools")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|t| t.get("function"))
            });

        if let Some(func) = func_call {
            let tool_name = func
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let arguments = func.get("arguments").and_then(|v| {
                // arguments 可能是字符串或对象
                match v {
                    serde_json::Value::String(s) => {
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(s).ok()
                    }
                    serde_json::Value::Object(obj) => Some(
                        obj.iter()
                            .map(|(k, val)| (k.clone(), val.clone()))
                            .collect(),
                    ),
                    _ => None,
                }
            });

            return McpDetectionResult {
                is_mcp_call: true,
                tool_name,
                arguments,
                transport_type: Some(TransportType::Http),
                jsonrpc_version: None,
                rpc_id: None,
            };
        }
    }

    // 检测3: 通过请求路径判断
    if url.contains("/mcp") || url.contains("/tools/call") {
        return McpDetectionResult {
            is_mcp_call: true,
            transport_type: Some(TransportType::Http),
            ..Default::default()
        };
    }

    McpDetectionResult::default()
}

/// 检测传输类型
fn detect_transport_type(url: &str, headers: &HashMap<String, String>) -> TransportType {
    let accept = headers
        .get("accept")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if accept.contains("text/event-stream") {
        return TransportType::Sse;
    }

    if url.contains("/mcp") || url.contains("/jsonrpc") {
        return TransportType::Http;
    }

    let content_type = headers
        .get("content-type")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if content_type.contains("application/json") {
        return TransportType::Http;
    }

    // 默认为 stdio
    TransportType::Stdio
}

/// 从响应中提取 MCP 结果
pub fn extract_mcp_result(body: &serde_json::Value) -> McpExtractResult {
    if body.is_null() {
        return McpExtractResult::default();
    }

    // JSON-RPC 2.0 响应格式
    if body.get("jsonrpc").and_then(|v| v.as_str()) == Some("2.0") {
        if let Some(error) = body.get("error") {
            return McpExtractResult {
                is_error: true,
                error_message: error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| Some(serde_json::to_string(error).unwrap_or_default())),
                content: Some(error.clone()),
            };
        }

        return McpExtractResult {
            is_error: false,
            content: body.get("result").cloned(),
            error_message: None,
        };
    }

    // OpenAI function calling 结果
    if let Some(func_call) = body
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("function_call"))
    {
        let arguments = func_call.get("arguments");
        let content = arguments.and_then(|v| {
            // arguments 可能是字符串或对象
            match v {
                serde_json::Value::String(s) => serde_json::from_str::<serde_json::Value>(s).ok(),
                _ => Some(v.clone()),
            }
        });

        return McpExtractResult {
            is_error: false,
            content,
            error_message: None,
        };
    }

    McpExtractResult {
        is_error: false,
        content: Some(body.clone()),
        error_message: None,
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_jsonrpc_tools_call() {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": 1,
            "params": {
                "name": "fetch",
                "arguments": {"url": "https://example.com"}
            }
        });

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let result = detect_mcp_call(&body, "http://localhost/mcp", &headers);

        assert!(result.is_mcp_call);
        assert_eq!(result.tool_name, Some("fetch".to_string()));
        assert_eq!(result.jsonrpc_version, Some("2.0".to_string()));
        assert_eq!(result.rpc_id, Some("1".to_string()));
        assert!(result.arguments.is_some());
        assert_eq!(
            result.arguments.unwrap().get("url"),
            Some(&json!("https://example.com"))
        );
    }

    #[test]
    fn test_detect_jsonrpc_tools_list() {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 2
        });

        let result = detect_mcp_call(&body, "http://localhost/mcp", &HashMap::new());

        assert!(result.is_mcp_call);
        assert_eq!(result.tool_name, Some("tools/list".to_string()));
        assert_eq!(result.jsonrpc_version, Some("2.0".to_string()));
        assert_eq!(result.rpc_id, Some("2".to_string()));
    }

    #[test]
    fn test_detect_openai_function_calling() {
        let body = json!({
            "model": "gpt-4",
            "tool_choice": {
                "function": {
                    "name": "get_weather",
                    "arguments": "{\"location\": \"NYC\"}"
                }
            }
        });

        let result = detect_mcp_call(
            &body,
            "http://api.openai.com/v1/chat/completions",
            &HashMap::new(),
        );

        assert!(result.is_mcp_call);
        assert_eq!(result.tool_name, Some("get_weather".to_string()));
        assert!(result.arguments.is_some());
        assert_eq!(result.transport_type, Some(TransportType::Http));
    }

    #[test]
    fn test_detect_url_pattern() {
        let body = json!({"query": "test"});

        let result = detect_mcp_call(&body, "http://localhost/mcp/tools", &HashMap::new());
        assert!(result.is_mcp_call);
        assert_eq!(result.transport_type, Some(TransportType::Http));

        let result = detect_mcp_call(&body, "http://localhost/tools/call", &HashMap::new());
        assert!(result.is_mcp_call);
    }

    #[test]
    fn test_detect_non_mcp() {
        let body = json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let result = detect_mcp_call(
            &body,
            "http://api.openai.com/v1/chat/completions",
            &HashMap::new(),
        );
        assert!(!result.is_mcp_call);
    }

    #[test]
    fn test_transport_type_sse() {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "text/event-stream".to_string());

        let transport = detect_transport_type("http://localhost/mcp", &headers);
        assert_eq!(transport, TransportType::Sse);
    }

    #[test]
    fn test_transport_type_http_json() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let transport = detect_transport_type("http://localhost/api", &headers);
        assert_eq!(transport, TransportType::Http);
    }

    #[test]
    fn test_transport_type_stdio_default() {
        let transport = detect_transport_type("http://localhost/unknown", &HashMap::new());
        assert_eq!(transport, TransportType::Stdio);
    }

    #[test]
    fn test_extract_mcp_result_jsonrpc_success() {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"content": [{"type": "text", "text": "Hello"}]}
        });

        let result = extract_mcp_result(&body);
        assert!(!result.is_error);
        assert!(result.content.is_some());
        assert_eq!(
            result.content.unwrap().get("content"),
            Some(&json!([{"type": "text", "text": "Hello"}]))
        );
    }

    #[test]
    fn test_extract_mcp_result_jsonrpc_error() {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        });

        let result = extract_mcp_result(&body);
        assert!(result.is_error);
        assert_eq!(result.error_message, Some("Invalid Request".to_string()));
    }

    #[test]
    fn test_extract_mcp_result_openai_function_call() {
        let body = json!({
            "choices": [{
                "message": {
                    "function_call": {
                        "name": "get_weather",
                        "arguments": "{\"location\": \"NYC\"}"
                    }
                }
            }]
        });

        let result = extract_mcp_result(&body);
        assert!(!result.is_error);
        assert!(result.content.is_some());
        // arguments 是字符串，应被解析为 JSON 对象
        let content = result.content.unwrap();
        assert_eq!(content.get("location"), Some(&json!("NYC")));
    }

    #[test]
    fn test_extract_mcp_result_null_body() {
        let result = extract_mcp_result(&serde_json::Value::Null);
        assert!(!result.is_error);
        assert!(result.content.is_none());
    }

    #[test]
    fn test_extract_mcp_result_fallback() {
        let body = json!({"some": "data"});

        let result = extract_mcp_result(&body);
        assert!(!result.is_error);
        assert_eq!(result.content, Some(body));
    }
}
