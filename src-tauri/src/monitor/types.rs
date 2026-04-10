// Monitor 模块 - 类型定义
// 精确匹配 TypeScript 类型定义和数据库 schema

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================================
// Provider 枚举 - 小写序列化与 TS 保持一致
// ============================================================================

/// LLM 服务提供商枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Kimi,
    Unknown,
}

// ============================================================================
// 请求/响应类型
// ============================================================================

/// LLM API 请求记录
/// 对应数据库 requests 表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMRequest {
    pub id: String,
    pub timestamp: i64,
    pub provider: Provider,
    pub model: String,
    pub method: String,
    pub url: String,
    /// 域名（迁移 002 添加）
    pub domain: Option<String>,
    pub headers: HashMap<String, String>,
    /// 完整的请求体 JSON
    pub body: serde_json::Value,
    /// 解析后的请求体
    pub parsed_body: Option<ParsedRequestBody>,
    /// 更新时间戳（迁移 002 添加）
    pub updated_at: Option<i64>,
}

/// 解析后的请求体
/// 使用 flatten 捕获额外字段 [key: string]: any
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedRequestBody {
    pub messages: Option<Vec<serde_json::Value>>,
    pub prompt: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    pub thinking: Option<serde_json::Value>,
    pub thinking_budget: Option<i64>,
    pub reasoning_effort: Option<String>,
    pub reasoning: Option<serde_json::Value>,
    pub extended_thinking: Option<serde_json::Value>,
    /// 捕获额外字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// LLM API 响应记录
/// 对应数据库 responses 表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMResponse {
    pub id: String,
    pub request_id: String,
    pub timestamp: i64,
    pub status_code: i32,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub parsed_body: Option<ParsedResponseBody>,
    /// 请求耗时(ms)
    pub duration: i64,
}

/// 解析后的响应体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedResponseBody {
    pub content: Option<String>,
    /// 思考内容（DeepSeek R1, Anthropic 等）
    pub thinking: Option<String>,
    pub choices: Option<Vec<serde_json::Value>>,
    pub usage: Option<Usage>,
    /// 捕获额外字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Token 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

// ============================================================================
// 指标类型
// ============================================================================

/// LLM 请求指标
/// 对应数据库 metrics 表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMMetrics {
    pub id: String,
    pub request_id: String,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    /// 估算费用 (USD)
    pub estimated_cost: f64,
    pub duration: i64,
    pub timestamp: i64,
}

/// 指标统计聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsStats {
    pub count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub model_stats: HashMap<String, ModelStatEntry>,
}

/// 单个模型的统计条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatEntry {
    pub count: i64,
    pub tokens: i64,
    pub cost: f64,
}

// ============================================================================
// MCP 调用类型
// ============================================================================

/// MCP (Model Context Protocol) 调用记录
/// 对应数据库 mcp_calls 表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPCall {
    pub id: String,
    pub request_id: String,
    pub jsonrpc_version: Option<String>,
    pub rpc_id: Option<String>,
    pub tool_name: String,
    pub tool_title: Option<String>,
    pub tool_description: Option<String>,
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    pub result_content: Option<serde_json::Value>,
    pub result_is_error: bool,
    pub error_message: Option<String>,
    pub execution_duration: Option<i64>,
    pub transport_type: Option<TransportType>,
    pub server_name: Option<String>,
    pub trace_id: Option<String>,
    pub timestamp: i64,
}

/// MCP 传输类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    Http,
}

// ============================================================================
// 域名统计类型
// ============================================================================

/// 域名统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStats {
    pub domain: String,
    pub count: i64,
    pub tokens: i64,
    pub cost: f64,
    pub avg_latency: f64,
    pub models: HashMap<String, ModelStatEntry>,
}

/// 域名统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStatsResult {
    pub domains: Vec<DomainStats>,
}

// ============================================================================
// 列表视图类型（精简版，不含大型字段）
// ============================================================================

/// 请求列表项（精简版，用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestListItem {
    pub id: String,
    pub timestamp: i64,
    pub provider: String,
    pub model: String,
    pub method: String,
    pub url: String,
    pub domain: Option<String>,
    pub tokens: Option<i64>,
    pub cost: Option<f64>,
    pub duration: Option<i64>,
    pub status_code: Option<i32>,
}

/// 增量查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaResult {
    pub new_requests: Vec<RequestListItem>,
    pub updated_requests: Vec<RequestListItem>,
}

// ============================================================================
// 每日记录类型
// ============================================================================

/// 每日记录
/// 对应数据库 daily_records 视图
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyRecord {
    /// 日期 (YYYY-MM-DD 格式)
    pub date: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub models: Vec<String>,
    pub model_stats: HashMap<String, ModelStatEntry>,
}

// ============================================================================
// 配置类型 - 与 config.jsonc 实际格式对齐
// ============================================================================

/// 域名配置
/// 与 config.jsonc 的 domains[] 格式保持一致
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainConfig {
    /// 域名（主机名）
    pub domain: String,
    /// 服务商名称
    pub provider: String,
    /// 是否启用监控
    pub enabled: bool,
    /// 匹配类型（默认为 "exact"）
    #[serde(default = "default_match_type")]
    pub match_type: MatchType,
}

fn default_match_type() -> MatchType {
    MatchType::Exact
}

/// 域名匹配类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Exact,
    Glob,
    Regex,
}

/// 模型定价配置
/// 与 config.jsonc 的 pricing.models[] 格式保持一致
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricingConfig {
    /// 模型名称（前缀匹配）
    pub model: String,
    /// 输入定价（美元/1M tokens）
    pub input: f64,
    /// 输出定价（美元/1M tokens）
    pub output: f64,
}

/// 端口配置
/// 与 config.jsonc 的 ports 格式保持一致
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortConfig {
    /// Web API 端口
    pub web: u16,
    /// 代理服务端口
    pub proxy: u16,
}

/// 定价配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingConfig {
    /// 匹配策略：'prefix'（前缀匹配）或 'exact'（精确匹配）
    pub match_strategy: String,
    /// 模型定价列表
    pub models: Vec<ModelPricingConfig>,
}

/// Monitor 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorConfig {
    pub domains: Vec<DomainConfig>,
    pub pricing: PricingConfig,
    pub ports: PortConfig,
}

// ============================================================================
// 证书状态
// ============================================================================

/// 证书状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertStatus {
    pub exists: bool,
    pub cert_path: Option<String>,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let request = LLMRequest {
            id: "test-id".to_string(),
            timestamp: 1234567890,
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            method: "POST".to_string(),
            url: "https://api.openai.com/v1/chat/completions".to_string(),
            domain: Some("api.openai.com".to_string()),
            headers,
            body: serde_json::json!({"model": "gpt-4"}),
            parsed_body: None,
            updated_at: None,
        };

        // 序列化
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"provider\":\"openai\""));
        assert!(json.contains("\"model\":\"gpt-4\""));

        // 反序列化
        let deserialized: LLMRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.provider, deserialized.provider);
        assert_eq!(request.model, deserialized.model);
    }

    #[test]
    fn test_llm_response_roundtrip() {
        let response = LLMResponse {
            id: "resp-id".to_string(),
            request_id: "req-id".to_string(),
            timestamp: 1234567890,
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"choices": []}),
            parsed_body: Some(ParsedResponseBody {
                content: Some("Hello".to_string()),
                thinking: None,
                choices: None,
                usage: Some(Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                }),
                extra: HashMap::new(),
            }),
            duration: 100,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LLMResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.status_code, deserialized.status_code);
    }

    #[test]
    fn test_llm_metrics_roundtrip() {
        let metrics = LLMMetrics {
            id: "metrics-id".to_string(),
            request_id: "req-id".to_string(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            estimated_cost: 0.03,
            duration: 500,
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: LLMMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.id, deserialized.id);
        assert_eq!(metrics.total_tokens, deserialized.total_tokens);
    }

    #[test]
    fn test_mcp_call_roundtrip() {
        let mcp_call = MCPCall {
            id: "mcp-id".to_string(),
            request_id: "req-id".to_string(),
            jsonrpc_version: Some("2.0".to_string()),
            rpc_id: Some("1".to_string()),
            tool_name: "fetch".to_string(),
            tool_title: Some("Fetch Tool".to_string()),
            tool_description: None,
            arguments: Some(HashMap::new()),
            result_content: Some(serde_json::json!({"result": "ok"})),
            result_is_error: false,
            error_message: None,
            execution_duration: Some(50),
            transport_type: Some(TransportType::Stdio),
            server_name: Some("mcp-server".to_string()),
            trace_id: None,
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&mcp_call).unwrap();
        let deserialized: MCPCall = serde_json::from_str(&json).unwrap();
        assert_eq!(mcp_call.id, deserialized.id);
        assert_eq!(mcp_call.tool_name, deserialized.tool_name);
        assert_eq!(deserialized.transport_type, Some(TransportType::Stdio));
    }

    #[test]
    fn test_provider_lowercase_serialization() {
        let providers = vec![
            (Provider::OpenAI, "\"openai\""),
            (Provider::Anthropic, "\"anthropic\""),
            (Provider::Kimi, "\"kimi\""),
            (Provider::Unknown, "\"unknown\""),
        ];

        for (provider, expected) in providers {
            let json = serde_json::to_string(&provider).unwrap();
            assert_eq!(
                json, expected,
                "Provider {:?} should serialize to {}",
                provider, expected
            );
        }
    }

    #[test]
    fn test_port_config_roundtrip() {
        let port_config = PortConfig {
            web: 7100,
            proxy: 7101,
        };

        let json = serde_json::to_string(&port_config).unwrap();
        assert!(json.contains("\"web\":7100"));
        assert!(json.contains("\"proxy\":7101"));

        let deserialized: PortConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(port_config.web, deserialized.web);
        assert_eq!(port_config.proxy, deserialized.proxy);
    }

    #[test]
    fn test_domain_config_roundtrip() {
        let domain_config = DomainConfig {
            domain: "api.openai.com".to_string(),
            provider: "OpenAI".to_string(),
            enabled: true,
            match_type: MatchType::Exact,
        };

        let json = serde_json::to_string(&domain_config).unwrap();
        assert!(json.contains("\"domain\":\"api.openai.com\""));
        assert!(json.contains("\"provider\":\"OpenAI\""));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"matchType\":\"exact\""));

        let deserialized: DomainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(domain_config.domain, deserialized.domain);
        assert_eq!(domain_config.enabled, deserialized.enabled);
    }

    #[test]
    fn test_parsed_request_body_with_extra_fields() {
        let mut extra = HashMap::new();
        extra.insert(
            "custom_field".to_string(),
            serde_json::json!("custom_value"),
        );

        let parsed_body = ParsedRequestBody {
            messages: None,
            prompt: Some("Hello".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(100),
            thinking: None,
            thinking_budget: None,
            reasoning_effort: None,
            reasoning: None,
            extended_thinking: None,
            extra,
        };

        let json = serde_json::to_string(&parsed_body).unwrap();
        assert!(json.contains("\"prompt\":\"Hello\""));
        assert!(json.contains("\"custom_field\":\"custom_value\""));

        let deserialized: ParsedRequestBody = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt, Some("Hello".to_string()));
        assert_eq!(
            deserialized.extra.get("custom_field"),
            Some(&serde_json::json!("custom_value"))
        );
    }
}
