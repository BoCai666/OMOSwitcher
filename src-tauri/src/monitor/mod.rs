// Monitor 模块 - Rust 版本的 LLM API 监控服务
// 提供类型定义、迁移框架、存储层、解析器和代理服务器

pub mod capture;
pub mod cert;
pub mod command;
pub mod config;
pub mod handler;
pub mod migration;
pub mod parser;
pub mod proxy;
pub mod storage;
pub mod tasks;
pub mod types;

// ============================================================================
// Tauri 事件常量
// ============================================================================

/// 新请求事件 - 请求捕获后发射
pub const EVENT_NEW_REQUEST: &str = "monitor:new-request";
/// 响应事件 - 响应捕获后发射
pub const EVENT_RESPONSE: &str = "monitor:response";
/// 指标事件 - 指标计算后发射
pub const EVENT_METRICS: &str = "monitor:metrics";

// ============================================================================
// 事件 Payload 结构
// ============================================================================

use serde::{Deserialize, Serialize};

/// 请求事件 Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEventPayload {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub method: String,
    pub url: String,
    pub domain: Option<String>,
    pub timestamp: i64,
}

/// 响应事件 Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEventPayload {
    pub request_id: String,
    pub status_code: i32,
    pub duration: i64,
}

/// 指标事件 Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsEventPayload {
    pub request_id: String,
    pub total_tokens: i64,
    pub estimated_cost: f64,
}
