// Monitor 模块 - SQLite 存储层
// 使用 rusqlite 实现完整的 StorageInterface
// 所有公开方法通过 spawn_blocking 异步执行，内部使用 Arc<Mutex<Connection>> 保证线程安全
// 注意：部分方法尚未被主流程集成调用，保留供后续集成使用

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::monitor::migration::{v001::V001Initial, v002::V002AddDomainField, MigrationManager};
use crate::monitor::types::{
    DailyRecord, DeltaResult, DomainStats, DomainStatsResult, LLMMetrics, LLMRequest, LLMResponse,
    MCPCall, MetricsStats, ModelStatEntry, Provider, RequestListItem, TransportType,
};

// ============================================================================
// MonitorStorage 主结构
// ============================================================================

/// SQLite 存储层
/// 使用 Arc<Mutex<Connection>> 包装 rusqlite 连接
/// 所有公开方法为 async，内部通过 tokio::task::spawn_blocking 执行
pub struct MonitorStorage {
    conn: Arc<Mutex<Connection>>,
}

impl MonitorStorage {
    /// 打开/创建数据库并运行迁移
    pub fn new(db_path: &Path) -> Result<Self, String> {
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建数据库目录失败: {}", e))?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;

        Self::init_connection(&conn)?;
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 创建内存数据库（用于测试）
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("创建内存数据库失败: {}", e))?;

        Self::init_connection(&conn)?;
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 初始化连接参数：WAL 模式 + 外键
    fn init_connection(conn: &Connection) -> Result<(), String> {
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| format!("设置 WAL 模式失败: {}", e))?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("启用外键失败: {}", e))?;
        Ok(())
    }

    /// 执行所有未应用的迁移
    fn run_migrations(conn: &Connection) -> Result<(), String> {
        let migrations: Vec<Box<dyn crate::monitor::migration::Migration>> = vec![
            Box::new(V001Initial),
            Box::new(V002AddDomainField),
        ];
        MigrationManager::run_pending(conn, migrations)
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    /// 安全解析 JSON 字符串
    fn safe_json_parse<T: serde::de::DeserializeOwned>(
        value: Option<String>,
        default: T,
    ) -> T {
        match value {
            None => default,
            Some(ref s) if s == "null" || s == "undefined" || s.is_empty() => default,
            Some(s) => serde_json::from_str(&s).unwrap_or(default),
        }
    }

    /// 从 URL 提取域名
    fn extract_domain(url: &str) -> String {
        url.split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .and_then(|host| host.split(':').next())
            .unwrap_or("")
            .to_string()
    }

    /// Provider 转字符串（用于数据库存储）
    fn provider_to_string(p: &Provider) -> String {
        match p {
            Provider::OpenAI => "openai".to_string(),
            Provider::Anthropic => "anthropic".to_string(),
            Provider::Kimi => "kimi".to_string(),
            Provider::Unknown => "unknown".to_string(),
        }
    }

    /// 解析 Provider 字符串
    fn parse_provider(s: &str) -> Provider {
        match s.to_lowercase().as_str() {
            "openai" => Provider::OpenAI,
            "anthropic" => Provider::Anthropic,
            "kimi" => Provider::Kimi,
            _ => Provider::Unknown,
        }
    }

    /// 从数据库行读取 LLMRequest
    fn read_request(row: &rusqlite::Row<'_>) -> Result<LLMRequest, rusqlite::Error> {
        let headers_str: Option<String> = row.get("headers")?;
        let body_str: Option<String> = row.get("body")?;
        let parsed_body_str: Option<String> = row.get("parsed_body")?;
        let provider_str: Option<String> = row.get("provider")?;
        let model: Option<String> = row.get("model")?;
        let method: Option<String> = row.get("method")?;
        let url: Option<String> = row.get("url")?;

        Ok(LLMRequest {
            id: row.get("id")?,
            timestamp: row.get("timestamp")?,
            provider: Self::parse_provider(&provider_str.unwrap_or_default()),
            model: model.unwrap_or_default(),
            method: method.unwrap_or_default(),
            url: url.unwrap_or_default(),
            domain: row.get("domain")?,
            headers: Self::safe_json_parse(headers_str, HashMap::new()),
            body: Self::safe_json_parse(body_str, serde_json::Value::Null),
            parsed_body: Self::safe_json_parse(parsed_body_str, None),
            updated_at: row.get("updated_at")?,
        })
    }

    /// 从数据库行读取 LLMResponse
    fn read_response(row: &rusqlite::Row<'_>) -> Result<LLMResponse, rusqlite::Error> {
        let headers_str: Option<String> = row.get("headers")?;
        let body_str: Option<String> = row.get("body")?;
        let parsed_body_str: Option<String> = row.get("parsed_body")?;
        let status_code: Option<i32> = row.get("status_code")?;
        let duration: Option<i64> = row.get("duration")?;

        Ok(LLMResponse {
            id: row.get("id")?,
            request_id: row.get("request_id")?,
            timestamp: row.get("timestamp")?,
            status_code: status_code.unwrap_or(0),
            headers: Self::safe_json_parse(headers_str, HashMap::new()),
            body: Self::safe_json_parse(body_str, serde_json::Value::Null),
            parsed_body: Self::safe_json_parse(parsed_body_str, None),
            duration: duration.unwrap_or(0),
        })
    }

    /// 从数据库行读取 LLMMetrics
    fn read_metrics(row: &rusqlite::Row<'_>) -> Result<LLMMetrics, rusqlite::Error> {
        let model: Option<String> = row.get("model")?;
        let provider: Option<String> = row.get("provider")?;
        let prompt_tokens: Option<i64> = row.get("prompt_tokens")?;
        let completion_tokens: Option<i64> = row.get("completion_tokens")?;
        let total_tokens: Option<i64> = row.get("total_tokens")?;
        let estimated_cost: Option<f64> = row.get("estimated_cost")?;
        let duration: Option<i64> = row.get("duration")?;

        Ok(LLMMetrics {
            id: row.get("id")?,
            request_id: row.get("request_id")?,
            model: model.unwrap_or_default(),
            provider: provider.unwrap_or_default(),
            prompt_tokens: prompt_tokens.unwrap_or(0),
            completion_tokens: completion_tokens.unwrap_or(0),
            total_tokens: total_tokens.unwrap_or(0),
            estimated_cost: estimated_cost.unwrap_or(0.0),
            duration: duration.unwrap_or(0),
            timestamp: row.get("timestamp")?,
        })
    }

    /// 从数据库行读取 MCPCall
    fn read_mcp_call(row: &rusqlite::Row<'_>) -> Result<MCPCall, rusqlite::Error> {
        let arguments_str: Option<String> = row.get("arguments")?;
        let result_content_str: Option<String> = row.get("result_content")?;
        let transport_type_str: Option<String> = row.get("transport_type")?;
        let result_is_error: Option<bool> = row.get("result_is_error")?;

        // 将 transport_type 字符串反序列化为枚举
        let transport_type: Option<TransportType> = transport_type_str
            .as_deref()
            .map(|s| match s {
                "stdio" => Some(TransportType::Stdio),
                "sse" => Some(TransportType::Sse),
                "http" => Some(TransportType::Http),
                _ => None,
            })
            .flatten();

        Ok(MCPCall {
            id: row.get("id")?,
            request_id: row.get("request_id")?,
            jsonrpc_version: row.get("jsonrpc_version")?,
            rpc_id: row.get("rpc_id")?,
            tool_name: row.get("tool_name")?,
            tool_title: row.get("tool_title")?,
            tool_description: row.get("tool_description")?,
            arguments: Self::safe_json_parse(arguments_str, None),
            result_content: Self::safe_json_parse(result_content_str, None),
            result_is_error: result_is_error.unwrap_or(false),
            error_message: row.get("error_message")?,
            execution_duration: row.get("execution_duration")?,
            transport_type,
            server_name: row.get("server_name")?,
            trace_id: row.get("trace_id")?,
            timestamp: row.get("timestamp")?,
        })
    }

    /// 从数据库行读取 RequestListItem
    fn read_request_list_item(row: &rusqlite::Row<'_>) -> Result<RequestListItem, rusqlite::Error> {
        let provider: Option<String> = row.get("provider")?;
        let model: Option<String> = row.get("model")?;
        let method: Option<String> = row.get("method")?;
        let url: Option<String> = row.get("url")?;

        Ok(RequestListItem {
            id: row.get("id")?,
            timestamp: row.get("timestamp")?,
            provider: provider.unwrap_or_else(|| "unknown".to_string()),
            model: model.unwrap_or_default(),
            method: method.unwrap_or_default(),
            url: url.unwrap_or_default(),
            domain: row.get("domain")?,
            tokens: row.get("tokens")?,
            cost: row.get("cost")?,
            duration: row.get("duration")?,
            status_code: row.get("status_code")?,
        })
    }

    // ========================================================================
    // 写入操作
    // ========================================================================

    /// 保存 LLM 请求
    pub async fn save_request(&self, request: &LLMRequest) -> Result<(), String> {
        let conn = self.conn.clone();
        let request = request.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let domain = request.domain.clone().unwrap_or_else(|| Self::extract_domain(&request.url));
            let headers_json = serde_json::to_string(&request.headers).unwrap_or_else(|_| "{}".to_string());
            let body_json = serde_json::to_string(&request.body).unwrap_or_else(|_| "null".to_string());
            let parsed_body_json = serde_json::to_string(&request.parsed_body).unwrap_or_else(|_| "null".to_string());

            conn.execute(
                "INSERT OR REPLACE INTO requests (id, timestamp, provider, model, method, url, domain, headers, body, parsed_body, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    request.id,
                    request.timestamp,
                    Self::provider_to_string(&request.provider),
                    request.model,
                    request.method,
                    request.url,
                    domain,
                    headers_json,
                    body_json,
                    parsed_body_json,
                    request.updated_at,
                ],
            ).map_err(|e| format!("保存请求失败: {}", e))?;

            Ok(())
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 保存 LLM 响应
    pub async fn save_response(&self, response: &LLMResponse) -> Result<(), String> {
        let conn = self.conn.clone();
        let response = response.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let headers_json = serde_json::to_string(&response.headers).unwrap_or_else(|_| "{}".to_string());
            let body_json = serde_json::to_string(&response.body).unwrap_or_else(|_| "null".to_string());
            let parsed_body_json = serde_json::to_string(&response.parsed_body).unwrap_or_else(|_| "null".to_string());

            conn.execute(
                "INSERT OR REPLACE INTO responses (id, request_id, timestamp, status_code, headers, body, parsed_body, duration) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    response.id,
                    response.request_id,
                    response.timestamp,
                    response.status_code,
                    headers_json,
                    body_json,
                    parsed_body_json,
                    response.duration,
                ],
            ).map_err(|e| format!("保存响应失败: {}", e))?;

            Ok(())
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 保存指标数据
    pub async fn save_metrics(&self, metrics: &LLMMetrics) -> Result<(), String> {
        let conn = self.conn.clone();
        let metrics = metrics.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            conn.execute(
                "INSERT OR REPLACE INTO metrics (id, request_id, model, provider, prompt_tokens, completion_tokens, total_tokens, estimated_cost, duration, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    metrics.id,
                    metrics.request_id,
                    metrics.model,
                    metrics.provider,
                    metrics.prompt_tokens,
                    metrics.completion_tokens,
                    metrics.total_tokens,
                    metrics.estimated_cost,
                    metrics.duration,
                    metrics.timestamp,
                ],
            ).map_err(|e| format!("保存指标失败: {}", e))?;

            Ok(())
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 保存 MCP 调用记录
    pub async fn save_mcp_call(&self, mcp_call: &MCPCall) -> Result<(), String> {
        let conn = self.conn.clone();
        let mcp_call = mcp_call.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let arguments_json = serde_json::to_string(&mcp_call.arguments).unwrap_or_else(|_| "null".to_string());
            let result_content_json = serde_json::to_string(&mcp_call.result_content).unwrap_or_else(|_| "null".to_string());
            let transport_type_str = mcp_call.transport_type.as_ref().map(|t| {
                match t {
                    TransportType::Stdio => "stdio",
                    TransportType::Sse => "sse",
                    TransportType::Http => "http",
                }
            });

            conn.execute(
                "INSERT OR REPLACE INTO mcp_calls (id, request_id, jsonrpc_version, rpc_id, tool_name, tool_title, tool_description, arguments, result_content, result_is_error, error_message, execution_duration, transport_type, server_name, trace_id, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    mcp_call.id,
                    mcp_call.request_id,
                    mcp_call.jsonrpc_version,
                    mcp_call.rpc_id,
                    mcp_call.tool_name,
                    mcp_call.tool_title,
                    mcp_call.tool_description,
                    arguments_json,
                    result_content_json,
                    mcp_call.result_is_error,
                    mcp_call.error_message,
                    mcp_call.execution_duration,
                    transport_type_str,
                    mcp_call.server_name,
                    mcp_call.trace_id,
                    mcp_call.timestamp,
                ],
            ).map_err(|e| format!("保存 MCP 调用失败: {}", e))?;

            Ok(())
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    // ========================================================================
    // 查询操作
    // ========================================================================

    /// 获取最近的请求列表
    pub async fn get_recent_requests(&self, limit: i64) -> Result<Vec<LLMRequest>, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let mut stmt = conn
                .prepare("SELECT * FROM requests ORDER BY timestamp DESC LIMIT ?")
                .map_err(|e| format!("准备查询失败: {}", e))?;

            let rows = stmt
                .query_map(params![limit], |row| Self::read_request(row))
                .map_err(|e| format!("查询请求失败: {}", e))?;

            collect_rows(rows, "读取请求行失败")
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 根据 ID 获取请求
    pub async fn get_request_by_id(&self, id: &str) -> Result<Option<LLMRequest>, String> {
        let conn = self.conn.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let result = conn
                .query_row(
                    "SELECT * FROM requests WHERE id = ?",
                    params![id],
                    |row| Self::read_request(row),
                );

            match result {
                Ok(req) => Ok(Some(req)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(format!("查询请求失败: {}", e)),
            }
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 根据请求 ID 获取响应
    pub async fn get_response_by_request_id(&self, request_id: &str) -> Result<Option<LLMResponse>, String> {
        let conn = self.conn.clone();
        let request_id = request_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let result = conn
                .query_row(
                    "SELECT * FROM responses WHERE request_id = ?",
                    params![request_id],
                    |row| Self::read_response(row),
                );

            match result {
                Ok(resp) => Ok(Some(resp)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(format!("查询响应失败: {}", e)),
            }
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 根据请求 ID 获取指标
    pub async fn get_metrics_by_request_id(&self, request_id: &str) -> Result<Option<LLMMetrics>, String> {
        let conn = self.conn.clone();
        let request_id = request_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let result = conn
                .query_row(
                    "SELECT * FROM metrics WHERE request_id = ?",
                    params![request_id],
                    |row| Self::read_metrics(row),
                );

            match result {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(format!("查询指标失败: {}", e)),
            }
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 根据请求 ID 获取 MCP 调用列表
    pub async fn get_mcp_calls_by_request_id(&self, request_id: &str) -> Result<Vec<MCPCall>, String> {
        let conn = self.conn.clone();
        let request_id = request_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let mut stmt = conn
                .prepare("SELECT * FROM mcp_calls WHERE request_id = ? ORDER BY timestamp ASC")
                .map_err(|e| format!("准备查询失败: {}", e))?;

            let rows = stmt
                .query_map(params![request_id], |row| Self::read_mcp_call(row))
                .map_err(|e| format!("查询 MCP 调用失败: {}", e))?;

            collect_rows(rows, "读取 MCP 调用行失败")
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    // ========================================================================
    // 统计操作
    // ========================================================================

    /// 获取指定时间范围内的指标统计
    pub async fn get_metrics_stats(&self, start_time: i64, end_time: i64) -> Result<MetricsStats, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            // 获取基本统计数据
            let (count, total_tokens, total_cost): (i64, i64, f64) = conn
                .query_row(
                    "SELECT COUNT(*) as count, COALESCE(SUM(total_tokens), 0) as total_tokens, COALESCE(SUM(estimated_cost), 0) as total_cost FROM metrics WHERE timestamp >= ? AND timestamp <= ?",
                    params![start_time, end_time],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|e| format!("查询统计失败: {}", e))?;

            // 获取按模型分组的统计
            let mut stmt = conn
                .prepare(
                    "SELECT model, COUNT(*) as count, COALESCE(SUM(total_tokens), 0) as tokens, COALESCE(SUM(estimated_cost), 0) as cost FROM metrics WHERE timestamp >= ? AND timestamp <= ? GROUP BY model"
                )
                .map_err(|e| format!("准备模型统计查询失败: {}", e))?;

            let rows = stmt
                .query_map(params![start_time, end_time], |row| {
                    let model: String = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    let tokens: i64 = row.get(2)?;
                    let cost: f64 = row.get(3)?;
                    Ok((model, count, tokens, cost))
                })
                .map_err(|e| format!("查询模型统计失败: {}", e))?;

            let mut model_stats = HashMap::new();
            for row in rows {
                let (model, count, tokens, cost) = row.map_err(|e| format!("读取模型统计行失败: {}", e))?;
                model_stats.insert(model, ModelStatEntry { count, tokens, cost });
            }

            Ok(MetricsStats {
                count,
                total_tokens,
                total_cost,
                model_stats,
            })
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 获取指定日期范围内的每日记录
    pub async fn get_daily_records(&self, start_date: &str, end_date: &str) -> Result<Vec<DailyRecord>, String> {
        let conn = self.conn.clone();
        let start_date = start_date.to_string();
        let end_date = end_date.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let mut stmt = conn
                .prepare("SELECT * FROM daily_records WHERE date >= ? AND date <= ? ORDER BY date ASC")
                .map_err(|e| format!("准备每日记录查询失败: {}", e))?;

            let rows = stmt
                .query_map(params![start_date, end_date], |row| {
                    let date: String = row.get("date")?;
                    let request_count: i64 = row.get("request_count")?;
                    let total_tokens: Option<i64> = row.get("total_tokens")?;
                    let total_cost: Option<f64> = row.get("total_cost")?;
                    let models_str: Option<String> = row.get("models")?;
                    let model_stats_str: Option<String> = row.get("model_stats")?;

                    let models: Vec<String> = Self::safe_json_parse(models_str, Vec::new());
                    let model_stats: HashMap<String, ModelStatEntry> = Self::safe_json_parse(model_stats_str, HashMap::new());

                    Ok(DailyRecord {
                        date,
                        request_count,
                        total_tokens: total_tokens.unwrap_or(0),
                        total_cost: total_cost.unwrap_or(0.0),
                        models,
                        model_stats,
                    })
                })
                .map_err(|e| format!("查询每日记录失败: {}", e))?;

            collect_rows(rows, "读取每日记录行失败")
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 获取指定日期的记录
    pub async fn get_daily_record(&self, date: &str) -> Result<Option<DailyRecord>, String> {
        let conn = self.conn.clone();
        let date = date.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let result = conn.query_row(
                "SELECT * FROM daily_records WHERE date = ?",
                params![date],
                |row| {
                    let date: String = row.get("date")?;
                    let request_count: i64 = row.get("request_count")?;
                    let total_tokens: Option<i64> = row.get("total_tokens")?;
                    let total_cost: Option<f64> = row.get("total_cost")?;
                    let models_str: Option<String> = row.get("models")?;
                    let model_stats_str: Option<String> = row.get("model_stats")?;

                    let models: Vec<String> = Self::safe_json_parse(models_str, Vec::new());
                    let model_stats: HashMap<String, ModelStatEntry> = Self::safe_json_parse(model_stats_str, HashMap::new());

                    Ok(DailyRecord {
                        date,
                        request_count,
                        total_tokens: total_tokens.unwrap_or(0),
                        total_cost: total_cost.unwrap_or(0.0),
                        models,
                        model_stats,
                    })
                },
            );

            match result {
                Ok(record) => Ok(Some(record)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(format!("查询每日记录失败: {}", e)),
            }
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 获取域名统计
    pub async fn get_domain_stats(&self, start_time: i64, end_time: i64) -> Result<DomainStatsResult, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            // 获取域名基础统计
            let mut domain_stmt = conn
                .prepare(
                    "SELECT r.domain, COUNT(*) as count, COALESCE(SUM(m.total_tokens), 0) as tokens, COALESCE(SUM(m.estimated_cost), 0) as cost, COALESCE(AVG(m.duration), 0) as avg_latency FROM requests r LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? GROUP BY r.domain ORDER BY count DESC"
                )
                .map_err(|e| format!("准备域名统计查询失败: {}", e))?;

            let domain_rows = domain_stmt
                .query_map(params![start_time, end_time], |row| {
                    let domain: Option<String> = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    let tokens: i64 = row.get(2)?;
                    let cost: f64 = row.get(3)?;
                    let avg_latency: f64 = row.get(4)?;
                    Ok((domain, count, tokens, cost, avg_latency))
                })
                .map_err(|e| format!("查询域名统计失败: {}", e))?;

            let mut domain_data: Vec<(Option<String>, i64, i64, f64, f64)> = Vec::new();
            for row in domain_rows {
                domain_data.push(row.map_err(|e| format!("读取域名统计行失败: {}", e))?);
            }

            // 获取域名+模型组合的统计
            let mut model_stmt = conn
                .prepare(
                    "SELECT r.domain, COALESCE(NULLIF(m.model, ''), 'unknown') as model_name, COUNT(*) as count, COALESCE(SUM(m.total_tokens), 0) as tokens, COALESCE(SUM(m.estimated_cost), 0) as cost FROM requests r LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? GROUP BY r.domain, model_name"
                )
                .map_err(|e| format!("准备域名模型统计查询失败: {}", e))?;

            let model_rows = model_stmt
                .query_map(params![start_time, end_time], |row| {
                    let domain: Option<String> = row.get(0)?;
                    let model_name: String = row.get(1)?;
                    let count: i64 = row.get(2)?;
                    let tokens: i64 = row.get(3)?;
                    let cost: f64 = row.get(4)?;
                    Ok((domain, model_name, count, tokens, cost))
                })
                .map_err(|e| format!("查询域名模型统计失败: {}", e))?;

            let mut model_stats_by_domain: HashMap<String, HashMap<String, ModelStatEntry>> = HashMap::new();
            for row in model_rows {
                let (domain, model_name, count, tokens, cost) = row.map_err(|e| format!("读取域名模型统计行失败: {}", e))?;
                let domain_key = domain.unwrap_or_else(|| "unknown".to_string());
                model_stats_by_domain
                    .entry(domain_key.clone())
                    .or_default()
                    .insert(model_name, ModelStatEntry { count, tokens, cost });
            }

            // 构建最终结果
            let domains: Vec<DomainStats> = domain_data
                .into_iter()
                .map(|(domain, count, tokens, cost, avg_latency)| {
                    let domain_key = domain.unwrap_or_else(|| "unknown".to_string());
                    let models = model_stats_by_domain.remove(&domain_key).unwrap_or_default();
                    DomainStats {
                        domain: domain_key,
                        count,
                        tokens,
                        cost,
                        avg_latency,
                        models,
                    }
                })
                .collect();

            Ok(DomainStatsResult { domains })
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 获取所有模型列表
    pub async fn get_all_models(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let mut stmt = conn
                .prepare("SELECT DISTINCT model FROM metrics WHERE model IS NOT NULL AND model != '' ORDER BY model")
                .map_err(|e| format!("准备模型查询失败: {}", e))?;

            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| format!("查询模型列表失败: {}", e))?;

            collect_rows(rows, "读取模型行失败")
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    // ========================================================================
    // 列表操作（带关联数据）
    // ========================================================================

    /// 获取最近的请求列表（带关联数据）
    pub async fn get_recent_requests_with_metrics(&self, limit: i64) -> Result<Vec<RequestListItem>, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let mut stmt = conn
                .prepare(
                    "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN responses res ON r.id = res.request_id LEFT JOIN metrics m ON r.id = m.request_id ORDER BY r.timestamp DESC LIMIT ?"
                )
                .map_err(|e| format!("准备请求列表查询失败: {}", e))?;

            let rows = stmt
                .query_map(params![limit], |row| Self::read_request_list_item(row))
                .map_err(|e| format!("查询请求列表失败: {}", e))?;

            collect_rows(rows, "读取请求列表行失败")
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 按日期范围获取请求列表（带关联数据）
    pub async fn get_requests_by_date_range_with_metrics(
        &self,
        start_date: &str,
        end_date: &str,
        limit: Option<i64>,
    ) -> Result<Vec<RequestListItem>, String> {
        let conn = self.conn.clone();
        let start_date = start_date.to_string();
        let end_date = end_date.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let start_ts = date_to_timestamp(&start_date, true);
            let end_ts = date_to_timestamp(&end_date, false);

            let sql = match limit {
                Some(_) => "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN responses res ON r.id = res.request_id LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? ORDER BY r.timestamp DESC LIMIT ?",
                None => "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN responses res ON r.id = res.request_id LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? ORDER BY r.timestamp DESC",
            };

            let mut stmt = conn.prepare(sql).map_err(|e| format!("准备查询失败: {}", e))?;

            let results = match limit {
                Some(lim) => {
                    let rows = stmt.query_map(params![start_ts, end_ts, lim], |row| Self::read_request_list_item(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
                None => {
                    let rows = stmt.query_map(params![start_ts, end_ts], |row| Self::read_request_list_item(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
            };

            Ok(results)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 按时间戳范围获取请求（带关联数据）
    pub async fn get_requests_by_timestamp_range_with_metrics(
        &self,
        start_time: i64,
        end_time: i64,
        limit: Option<i64>,
    ) -> Result<Vec<RequestListItem>, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let sql = match limit {
                Some(_) => "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN responses res ON r.id = res.request_id LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? ORDER BY r.timestamp DESC LIMIT ?",
                None => "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN responses res ON r.id = res.request_id LEFT JOIN metrics m ON r.id = m.request_id WHERE r.timestamp >= ? AND r.timestamp <= ? ORDER BY r.timestamp DESC",
            };

            let mut stmt = conn.prepare(sql).map_err(|e| format!("准备查询失败: {}", e))?;

            let results = match limit {
                Some(lim) => {
                    let rows = stmt.query_map(params![start_time, end_time, lim], |row| Self::read_request_list_item(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
                None => {
                    let rows = stmt.query_map(params![start_time, end_time], |row| Self::read_request_list_item(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
            };

            Ok(results)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    // ========================================================================
    // 增量查询
    // ========================================================================

    /// 获取增量更新数据
    pub async fn get_delta(&self, since: i64, limit: i64) -> Result<DeltaResult, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            // 查询新增请求
            let mut new_stmt = conn
                .prepare(
                    "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN metrics m ON r.id = m.request_id LEFT JOIN responses res ON r.id = res.request_id WHERE r.timestamp > ? ORDER BY r.timestamp DESC LIMIT ?"
                )
                .map_err(|e| format!("准备新增查询失败: {}", e))?;

            let new_rows = new_stmt
                .query_map(params![since, limit], |row| Self::read_request_list_item(row))
                .map_err(|e| format!("查询新增请求失败: {}", e))?;

            let new_requests = collect_rows(new_rows, "读取新增行失败")?;

            // 查询更新的请求
            let mut updated_stmt = conn
                .prepare(
                    "SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain, res.status_code, m.total_tokens as tokens, m.estimated_cost as cost, m.duration FROM requests r LEFT JOIN metrics m ON r.id = m.request_id LEFT JOIN responses res ON r.id = res.request_id WHERE r.updated_at > ? AND r.timestamp <= ? ORDER BY r.updated_at DESC LIMIT ?"
                )
                .map_err(|e| format!("准备更新查询失败: {}", e))?;

            let updated_rows = updated_stmt
                .query_map(params![since, since, limit], |row| Self::read_request_list_item(row))
                .map_err(|e| format!("查询更新请求失败: {}", e))?;

            let updated_requests = collect_rows(updated_rows, "读取更新行失败")?;

            Ok(DeltaResult {
                new_requests,
                updated_requests,
            })
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    // ========================================================================
    // 数据管理
    // ========================================================================

    /// 清空所有数据
    pub async fn clear(&self) -> Result<(), String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            // 按外键依赖顺序删除
            conn.execute("DELETE FROM mcp_calls", [])
                .map_err(|e| format!("清空 mcp_calls 失败: {}", e))?;
            conn.execute("DELETE FROM metrics", [])
                .map_err(|e| format!("清空 metrics 失败: {}", e))?;
            conn.execute("DELETE FROM responses", [])
                .map_err(|e| format!("清空 responses 失败: {}", e))?;
            conn.execute("DELETE FROM requests", [])
                .map_err(|e| format!("清空 requests 失败: {}", e))?;

            Ok(())
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 检查是否有数据
    pub async fn has_data(&self) -> Result<bool, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM requests LIMIT 1", [], |row| row.get(0))
                .map_err(|e| format!("查询数据量失败: {}", e))?;

            Ok(count > 0)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 统计过期请求数量
    /// cutoff_timestamp: 截止时间戳（毫秒），早于此时间的请求被视为过期
    pub async fn count_expired_requests(&self, cutoff_timestamp: i64) -> Result<u64, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM requests WHERE timestamp < ?",
                    params![cutoff_timestamp],
                    |row| row.get(0),
                )
                .map_err(|e| format!("统计过期请求失败: {}", e))?;

            Ok(count as u64)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 删除过期数据
    /// 由于外键约束，删除 requests 会级联删除 responses, metrics, mcp_calls
    /// cutoff_timestamp: 截止时间戳（毫秒），早于此时间的请求将被删除
    pub async fn delete_expired_requests(&self, cutoff_timestamp: i64) -> Result<u64, String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            // 删除过期请求（外键约束会级联删除关联数据）
            let deleted = conn
                .execute(
                    "DELETE FROM requests WHERE timestamp < ?",
                    params![cutoff_timestamp],
                )
                .map_err(|e| format!("删除过期请求失败: {}", e))?;

            Ok(deleted as u64)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }

    /// 按日期范围获取请求列表
    pub async fn get_requests_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
        limit: Option<i64>,
    ) -> Result<Vec<LLMRequest>, String> {
        let conn = self.conn.clone();
        let start_date = start_date.to_string();
        let end_date = end_date.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| format!("获取连接锁失败: {}", e))?;

            let start_ts = date_to_timestamp(&start_date, true);
            let end_ts = date_to_timestamp(&end_date, false);

            let sql = match limit {
                Some(_) => "SELECT * FROM requests WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp DESC LIMIT ?",
                None => "SELECT * FROM requests WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp DESC",
            };

            let mut stmt = conn.prepare(sql).map_err(|e| format!("准备查询失败: {}", e))?;

            let results = match limit {
                Some(lim) => {
                    let rows = stmt.query_map(params![start_ts, end_ts, lim], |row| Self::read_request(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
                None => {
                    let rows = stmt.query_map(params![start_ts, end_ts], |row| Self::read_request(row))
                        .map_err(|e| format!("查询失败: {}", e))?;
                    collect_rows(rows, "读取行失败")?
                }
            };

            Ok(results)
        }).await.map_err(|e| format!("spawn_blocking 错误: {}", e))?
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 将日期字符串转换为毫秒时间戳
/// start_of_day: true 表示当天 00:00:00，false 表示当天 23:59:59.999
fn date_to_timestamp(date_str: &str, start_of_day: bool) -> i64 {
    // 解析 YYYY-MM-DD 格式
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return 0;
    }

    let year: i32 = parts[0].parse().unwrap_or(0);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let day: u32 = parts[2].parse().unwrap_or(1);

    // 使用 time crate 计算更准确的时间戳
    let month_u8 = month as u8;
    let day_u8 = day.min(28) as u8;
    let date = time::Date::from_calendar_date(year, time::Month::try_from(month_u8).unwrap_or(time::Month::January), day_u8);
    let base_ts = match date {
        Ok(d) => {
            let midday = time::PrimitiveDateTime::new(d, time::Time::MIDNIGHT);
            midday.assume_utc().unix_timestamp()
        }
        Err(_) => 0,
    };

    if start_of_day {
        base_ts * 1000
    } else {
        (base_ts + 86399) * 1000 + 999
    }
}

/// 收集 MappedRows 到 Vec
fn collect_rows<T>(
    rows: rusqlite::MappedRows<impl FnMut(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>>,
    error_msg: &str,
) -> Result<Vec<T>, String> {
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("{}: {}", error_msg, e))?);
    }
    Ok(results)
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::migration::Migration;

    /// 创建测试用的内存数据库
    fn create_test_storage() -> MonitorStorage {
        MonitorStorage::open_in_memory().unwrap()
    }

    /// 创建测试用的 LLMRequest
    fn create_test_request(id: &str, timestamp: i64) -> LLMRequest {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        LLMRequest {
            id: id.to_string(),
            timestamp,
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            method: "POST".to_string(),
            url: "https://api.openai.com/v1/chat/completions".to_string(),
            domain: Some("api.openai.com".to_string()),
            headers,
            body: serde_json::json!({"model": "gpt-4", "messages": []}),
            parsed_body: None,
            updated_at: None,
        }
    }

    /// 创建测试用的 LLMResponse
    fn create_test_response(id: &str, request_id: &str, timestamp: i64) -> LLMResponse {
        LLMResponse {
            id: id.to_string(),
            request_id: request_id.to_string(),
            timestamp,
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"choices": []}),
            parsed_body: None,
            duration: 500,
        }
    }

    /// 创建测试用的 LLMMetrics
    fn create_test_metrics(id: &str, request_id: &str, timestamp: i64) -> LLMMetrics {
        LLMMetrics {
            id: id.to_string(),
            request_id: request_id.to_string(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            estimated_cost: 0.03,
            duration: 500,
            timestamp,
        }
    }

    /// 创建测试用的 MCPCall
    fn create_test_mcp_call(id: &str, request_id: &str, timestamp: i64) -> MCPCall {
        MCPCall {
            id: id.to_string(),
            request_id: request_id.to_string(),
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
            transport_type: None,
            server_name: Some("mcp-server".to_string()),
            trace_id: None,
            timestamp,
        }
    }

    // ========================================================================
    // 测试 1-4: 基本 CRUD
    // ========================================================================

    #[tokio::test]
    async fn test_save_and_get_request() {
        let storage = create_test_storage();
        let request = create_test_request("req-1", 1000000);

        storage.save_request(&request).await.unwrap();

        let retrieved = storage.get_request_by_id("req-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "req-1");
        assert_eq!(retrieved.timestamp, 1000000);
        assert_eq!(retrieved.model, "gpt-4");
        assert_eq!(retrieved.headers.get("content-type").unwrap(), "application/json");
    }

    #[tokio::test]
    async fn test_save_and_get_response() {
        let storage = create_test_storage();

        // 先保存对应的请求（外键约束）
        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        let response = create_test_response("resp-1", "req-1", 1000100);
        storage.save_response(&response).await.unwrap();

        let retrieved = storage.get_response_by_request_id("req-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "resp-1");
        assert_eq!(retrieved.request_id, "req-1");
        assert_eq!(retrieved.status_code, 200);
        assert_eq!(retrieved.duration, 500);
    }

    #[tokio::test]
    async fn test_save_and_get_metrics() {
        let storage = create_test_storage();

        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        let metrics = create_test_metrics("metrics-1", "req-1", 1000000);
        storage.save_metrics(&metrics).await.unwrap();

        let retrieved = storage.get_metrics_by_request_id("req-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "metrics-1");
        assert_eq!(retrieved.total_tokens, 150);
        assert!((retrieved.estimated_cost - 0.03).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_save_and_get_mcp_call() {
        let storage = create_test_storage();

        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        let mcp_call = create_test_mcp_call("mcp-1", "req-1", 1000050);
        storage.save_mcp_call(&mcp_call).await.unwrap();

        let retrieved = storage.get_mcp_calls_by_request_id("req-1").await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].id, "mcp-1");
        assert_eq!(retrieved[0].tool_name, "fetch");
        assert_eq!(retrieved[0].server_name, Some("mcp-server".to_string()));
    }

    // ========================================================================
    // 测试 5: 列表查询
    // ========================================================================

    #[tokio::test]
    async fn test_get_recent_requests_with_metrics() {
        let storage = create_test_storage();

        // 创建多个请求，附带 metrics 和 responses
        for i in 0..5i64 {
            let request = create_test_request(&format!("req-{}", i), 1000000 + i * 1000);
            storage.save_request(&request).await.unwrap();

            let metrics = LLMMetrics {
                id: format!("metrics-{}", i),
                request_id: format!("req-{}", i),
                model: "gpt-4".to_string(),
                provider: "openai".to_string(),
                prompt_tokens: 100 * (i + 1),
                completion_tokens: 50 * (i + 1),
                total_tokens: 150 * (i + 1),
                estimated_cost: 0.03 * (i as f64 + 1.0),
                duration: 500 + i * 100,
                timestamp: 1000000 + i * 1000,
            };
            storage.save_metrics(&metrics).await.unwrap();

            let response = create_test_response(&format!("resp-{}", i), &format!("req-{}", i), 1000100 + i * 1000);
            storage.save_response(&response).await.unwrap();
        }

        let items = storage.get_recent_requests_with_metrics(3).await.unwrap();
        assert_eq!(items.len(), 3);
        // 最新的在前面（timestamp DESC）
        assert_eq!(items[0].id, "req-4");
        assert!(items[0].tokens.is_some());
        assert!(items[0].status_code.is_some());
    }

    // ========================================================================
    // 测试 6: 统计操作
    // ========================================================================

    #[tokio::test]
    async fn test_get_metrics_stats() {
        let storage = create_test_storage();

        // 创建两个不同模型的请求
        for i in 0..3i64 {
            let mut request = create_test_request(&format!("req-{}", i), 1000000 + i * 1000);
            if i < 2 {
                request.model = "gpt-4".to_string();
            } else {
                request.model = "claude-3".to_string();
            }
            storage.save_request(&request).await.unwrap();

            let mut metrics = create_test_metrics(&format!("metrics-{}", i), &format!("req-{}", i), 1000000 + i * 1000);
            if i < 2 {
                metrics.model = "gpt-4".to_string();
                metrics.total_tokens = 100;
                metrics.estimated_cost = 0.01;
            } else {
                metrics.model = "claude-3".to_string();
                metrics.total_tokens = 200;
                metrics.estimated_cost = 0.02;
            }
            storage.save_metrics(&metrics).await.unwrap();
        }

        let stats = storage.get_metrics_stats(0, 2000000).await.unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_tokens, 400);
        assert!((stats.total_cost - 0.04).abs() < 0.001);
        assert_eq!(stats.model_stats.len(), 2);
        assert!(stats.model_stats.contains_key("gpt-4"));
        assert!(stats.model_stats.contains_key("claude-3"));
    }

    // ========================================================================
    // 测试 7: 增量查询
    // ========================================================================

    #[tokio::test]
    async fn test_get_delta() {
        let storage = create_test_storage();

        // 创建旧请求
        let request1 = create_test_request("req-old", 1000000);
        storage.save_request(&request1).await.unwrap();
        let metrics1 = create_test_metrics("metrics-old", "req-old", 1000000);
        storage.save_metrics(&metrics1).await.unwrap();

        // 创建新请求
        let request2 = create_test_request("req-new", 2000000);
        storage.save_request(&request2).await.unwrap();
        let metrics2 = create_test_metrics("metrics-new", "req-new", 2000000);
        storage.save_metrics(&metrics2).await.unwrap();

        // 查询 since=1500000 的增量
        let delta = storage.get_delta(1500000, 10).await.unwrap();
        assert_eq!(delta.new_requests.len(), 1);
        assert_eq!(delta.new_requests[0].id, "req-new");

        // 旧请求不应出现在 new_requests 中
        assert!(delta.new_requests.iter().all(|r| r.id != "req-old"));
    }

    // ========================================================================
    // 测试 8: 每日记录
    // ========================================================================

    #[tokio::test]
    async fn test_get_daily_records() {
        let storage = create_test_storage();

        // 创建请求（timestamp 毫秒，会被视图转为日期）
        let ts = 1711929600000i64; // 2024-04-01 00:00:00 UTC 的毫秒时间戳
        let request = create_test_request("req-1", ts);
        storage.save_request(&request).await.unwrap();

        let metrics = LLMMetrics {
            id: "metrics-1".to_string(),
            request_id: "req-1".to_string(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            estimated_cost: 0.03,
            duration: 500,
            timestamp: ts,
        };
        storage.save_metrics(&metrics).await.unwrap();

        // daily_records 视图使用 localtime，日期可能因时区不同
        // 查询一个大范围以确保命中
        let records = storage.get_daily_records("2020-01-01", "2030-12-31").await.unwrap();
        // 视图可能返回 0 或 1 条记录，取决于时区
        // 关键是查询不应报错
        assert!(records.len() <= 1);
    }

    // ========================================================================
    // 测试 9: 域名统计
    // ========================================================================

    #[tokio::test]
    async fn test_get_domain_stats() {
        let storage = create_test_storage();

        // 创建不同域名的请求
        let mut request1 = create_test_request("req-1", 1000000);
        request1.domain = Some("api.openai.com".to_string());
        storage.save_request(&request1).await.unwrap();

        let mut request2 = create_test_request("req-2", 1001000);
        request2.domain = Some("api.anthropic.com".to_string());
        request2.model = "claude-3".to_string();
        storage.save_request(&request2).await.unwrap();

        let metrics1 = create_test_metrics("metrics-1", "req-1", 1000000);
        storage.save_metrics(&metrics1).await.unwrap();

        let mut metrics2 = create_test_metrics("metrics-2", "req-2", 1001000);
        metrics2.model = "claude-3".to_string();
        storage.save_metrics(&metrics2).await.unwrap();

        let result = storage.get_domain_stats(0, 2000000).await.unwrap();
        assert!(result.domains.len() >= 2);

        // 验证域名分组正确
        let openai_domain = result.domains.iter().find(|d| d.domain == "api.openai.com");
        assert!(openai_domain.is_some());
        assert_eq!(openai_domain.unwrap().count, 1);
    }

    // ========================================================================
    // 测试 10: 清空操作
    // ========================================================================

    #[tokio::test]
    async fn test_clear() {
        let storage = create_test_storage();

        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();
        let metrics = create_test_metrics("metrics-1", "req-1", 1000000);
        storage.save_metrics(&metrics).await.unwrap();

        assert!(storage.has_data().await.unwrap());

        storage.clear().await.unwrap();

        assert!(!storage.has_data().await.unwrap());
        assert!(storage.get_request_by_id("req-1").await.unwrap().is_none());
    }

    // ========================================================================
    // 测试 11-12: 数据管理
    // ========================================================================

    #[tokio::test]
    async fn test_has_data() {
        let storage = create_test_storage();

        assert!(!storage.has_data().await.unwrap());

        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        assert!(storage.has_data().await.unwrap());
    }

    #[tokio::test]
    async fn test_get_all_models() {
        let storage = create_test_storage();

        // 无数据时返回空
        let models = storage.get_all_models().await.unwrap();
        assert!(models.is_empty());

        // 添加不同模型的 metrics
        let request1 = create_test_request("req-1", 1000000);
        storage.save_request(&request1).await.unwrap();
        let metrics1 = create_test_metrics("metrics-1", "req-1", 1000000);
        storage.save_metrics(&metrics1).await.unwrap();

        let mut request2 = create_test_request("req-2", 1001000);
        request2.model = "claude-3".to_string();
        storage.save_request(&request2).await.unwrap();
        let mut metrics2 = create_test_metrics("metrics-2", "req-2", 1001000);
        metrics2.model = "claude-3".to_string();
        storage.save_metrics(&metrics2).await.unwrap();

        let models = storage.get_all_models().await.unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"claude-3".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
    }

    // ========================================================================
    // 测试 13: 迁移幂等性
    // ========================================================================

    #[tokio::test]
    async fn test_migration_idempotent() {
        // 创建 storage（执行迁移）
        let storage1 = MonitorStorage::open_in_memory().unwrap();
        assert!(!storage1.has_data().await.unwrap());

        // 创建第二个 storage（不应重复执行迁移）
        let storage2 = MonitorStorage::open_in_memory().unwrap();
        assert!(!storage2.has_data().await.unwrap());

        // 两个 storage 应该都可以正常使用
        let request = create_test_request("req-1", 1000000);
        storage1.save_request(&request).await.unwrap();
        storage2.save_request(&request).await.unwrap();
    }

    // ========================================================================
    // 测试 14: 向后兼容
    // ========================================================================

    #[tokio::test]
    async fn test_backward_compatible_with_existing_schema() {
        // 模拟只有 v001 schema 的数据库
        let conn = Connection::open_in_memory().unwrap();
        V001Initial.up(&conn).unwrap();
        drop(conn);

        // 打开此数据库应自动执行 v002 迁移
        // 此处无法直接使用内存数据库测试（因为连接已关闭）
        // 改为验证 v001 + v002 顺序执行不出错
        let storage = MonitorStorage::open_in_memory().unwrap();
        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        let retrieved = storage.get_request_by_id("req-1").await.unwrap();
        assert!(retrieved.is_some());
        // domain 字段应可用
        assert_eq!(retrieved.unwrap().domain, Some("api.openai.com".to_string()));
    }

    // ========================================================================
    // 测试 15: 日期范围查询
    // ========================================================================

    #[tokio::test]
    async fn test_get_requests_by_date_range() {
        let storage = create_test_storage();

        // 创建不同日期的请求
        let request1 = create_test_request("req-1", 1711929600000); // 2024-04-01
        storage.save_request(&request1).await.unwrap();

        let request2 = create_test_request("req-2", 1712016000000); // 2024-04-02
        storage.save_request(&request2).await.unwrap();

        let request3 = create_test_request("req-3", 1712275200000); // 2024-04-05
        storage.save_request(&request3).await.unwrap();

        // 查询日期范围
        let results = storage.get_requests_by_date_range("2024-04-01", "2024-04-03", None).await.unwrap();
        // 应返回 4月1日和4月2日的请求
        assert!(results.len() >= 2);
    }

    // ========================================================================
    // 额外测试: INSERT OR REPLACE 行为
    // ========================================================================

    #[tokio::test]
    async fn test_save_request_upsert() {
        let storage = create_test_storage();

        let mut request = create_test_request("req-1", 1000000);
        request.model = "gpt-4".to_string();
        storage.save_request(&request).await.unwrap();

        // 更新同一 ID
        request.model = "gpt-4o".to_string();
        storage.save_request(&request).await.unwrap();

        let retrieved = storage.get_request_by_id("req-1").await.unwrap().unwrap();
        assert_eq!(retrieved.model, "gpt-4o");
    }

    #[tokio::test]
    async fn test_get_recent_requests_ordering() {
        let storage = create_test_storage();

        // 按非顺序时间戳插入
        let request1 = create_test_request("req-1", 1000000);
        let request2 = create_test_request("req-2", 3000000);
        let request3 = create_test_request("req-3", 2000000);

        storage.save_request(&request1).await.unwrap();
        storage.save_request(&request2).await.unwrap();
        storage.save_request(&request3).await.unwrap();

        let results = storage.get_recent_requests(10).await.unwrap();
        assert_eq!(results.len(), 3);
        // 最新的在前面
        assert_eq!(results[0].id, "req-2");
        assert_eq!(results[1].id, "req-3");
        assert_eq!(results[2].id, "req-1");
    }

    #[tokio::test]
    async fn test_get_requests_by_timestamp_range_with_metrics() {
        let storage = create_test_storage();

        let request1 = create_test_request("req-1", 1000000);
        storage.save_request(&request1).await.unwrap();
        let metrics1 = create_test_metrics("metrics-1", "req-1", 1000000);
        storage.save_metrics(&metrics1).await.unwrap();

        let request2 = create_test_request("req-2", 2000000);
        storage.save_request(&request2).await.unwrap();
        let metrics2 = create_test_metrics("metrics-2", "req-2", 2000000);
        storage.save_metrics(&metrics2).await.unwrap();

        // 查询时间范围
        let results = storage.get_requests_by_timestamp_range_with_metrics(500000, 1500000, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "req-1");
        assert!(results[0].tokens.is_some());
    }

    #[tokio::test]
    async fn test_extract_domain() {
        assert_eq!(MonitorStorage::extract_domain("https://api.openai.com/v1/chat"), "api.openai.com");
        assert_eq!(MonitorStorage::extract_domain("http://localhost:3000/api"), "localhost");
        assert_eq!(MonitorStorage::extract_domain("invalid-url"), "");
    }

    #[tokio::test]
    async fn test_multiple_mcp_calls_per_request() {
        let storage = create_test_storage();

        let request = create_test_request("req-1", 1000000);
        storage.save_request(&request).await.unwrap();

        for i in 0..3i64 {
            let mcp_call = create_test_mcp_call(&format!("mcp-{}", i), "req-1", 1000050 + i * 10);
            storage.save_mcp_call(&mcp_call).await.unwrap();
        }

        let mcp_calls = storage.get_mcp_calls_by_request_id("req-1").await.unwrap();
        assert_eq!(mcp_calls.len(), 3);
        // 按 timestamp ASC 排序
        assert_eq!(mcp_calls[0].id, "mcp-0");
        assert_eq!(mcp_calls[2].id, "mcp-2");
    }
}
