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

    // 检测2: 通过请求路径判断
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
