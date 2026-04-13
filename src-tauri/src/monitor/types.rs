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
    ZhipuAI,
    MiniMax,
    Qianfan,
    Volces,
    InfiniAI,
    JDCloud,
    Google,
    DeepSeek,
    Groq,
    Mistral,
    Qwen,
    SiliconFlow,
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
