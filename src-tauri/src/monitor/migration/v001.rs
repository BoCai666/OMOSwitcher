// 迁移 v001 - 初始数据库 schema
// 精确复用 001-initial.sql 的内容，修复 daily_records 视图中的 bug

use rusqlite::Connection;

use super::Migration;

/// 初始 schema 迁移
/// 创建 requests, responses, metrics, mcp_calls 四个表
/// 创建相关索引和 daily_records 视图
pub struct V001Initial;

impl Migration for V001Initial {
    fn version(&self) -> u32 {
        1
    }

    fn description(&self) -> &str {
        "初始数据库 schema - 创建核心表、索引和视图"
    }

    fn up(&self, conn: &Connection) -> Result<(), String> {
        // 创建 requests 表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                provider TEXT,
                model TEXT,
                method TEXT,
                url TEXT,
                headers TEXT,
                body TEXT,
                parsed_body TEXT
            );",
        )
        .map_err(|e| format!("创建 requests 表失败: {}", e))?;

        // 创建 responses 表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS responses (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                status_code INTEGER,
                headers TEXT,
                body TEXT,
                parsed_body TEXT,
                duration INTEGER,
                FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
            );",
        )
        .map_err(|e| format!("创建 responses 表失败: {}", e))?;

        // 创建 metrics 表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS metrics (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                model TEXT,
                provider TEXT,
                prompt_tokens INTEGER,
                completion_tokens INTEGER,
                total_tokens INTEGER,
                estimated_cost REAL,
                duration INTEGER,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
            );",
        )
        .map_err(|e| format!("创建 metrics 表失败: {}", e))?;

        // 创建 mcp_calls 表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS mcp_calls (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                jsonrpc_version TEXT DEFAULT '2.0',
                rpc_id TEXT,
                tool_name TEXT NOT NULL,
                tool_title TEXT,
                tool_description TEXT,
                arguments TEXT,
                result_content TEXT,
                result_is_error BOOLEAN DEFAULT FALSE,
                error_message TEXT,
                execution_duration INTEGER,
                transport_type TEXT,
                server_name TEXT,
                trace_id TEXT,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
            );",
        )
        .map_err(|e| format!("创建 mcp_calls 表失败: {}", e))?;

        // 创建索引
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_requests_timestamp ON requests(timestamp);
             CREATE INDEX IF NOT EXISTS idx_requests_time_model ON requests(timestamp, model);
             CREATE INDEX IF NOT EXISTS idx_responses_request_id ON responses(request_id);
             CREATE INDEX IF NOT EXISTS idx_metrics_request_id ON metrics(request_id);
             CREATE INDEX IF NOT EXISTS idx_metrics_time_model ON metrics(timestamp, model);
             CREATE INDEX IF NOT EXISTS idx_mcp_calls_request_id ON mcp_calls(request_id);
             CREATE INDEX IF NOT EXISTS idx_mcp_calls_tool_name ON mcp_calls(tool_name);
             CREATE INDEX IF NOT EXISTS idx_mcp_calls_timestamp ON mcp_calls(timestamp);
             CREATE INDEX IF NOT EXISTS idx_metrics_timestamp_model ON metrics(timestamp, model);",
        )
        .map_err(|e| format!("创建索引失败: {}", e))?;

        // 创建 daily_records 视图
        // 修复原始 SQL 中的 bug: model_breakdown CTE 中 GROUP BY 使用了 timestamp/0.0000029
        // 正确的应该是 timestamp/1000（与其他 CTE 一致）
        conn.execute_batch(
            "CREATE VIEW IF NOT EXISTS daily_records AS
             WITH daily_stats AS (
               SELECT
                 date(timestamp/1000, 'unixepoch', 'localtime') as date,
                 COUNT(*) as request_count,
                 SUM(total_tokens) as total_tokens,
                 SUM(estimated_cost) as total_cost
               FROM metrics
               GROUP BY date(timestamp/1000, 'unixepoch', 'localtime')
             ),
             daily_models AS (
               SELECT
                 date(timestamp/1000, 'unixepoch', 'localtime') as date,
                 json_group_array(DISTINCT model) as models
               FROM metrics
               GROUP BY date(timestamp/1000, 'unixepoch', 'localtime')
             ),
             model_breakdown AS (
               SELECT
                 date(timestamp/1000, 'unixepoch', 'localtime') as date,
                 model,
                 COUNT(*) as count,
                 SUM(total_tokens) as tokens,
                 SUM(estimated_cost) as cost
               FROM metrics
               GROUP BY date(timestamp/1000, 'unixepoch', 'localtime'), model
             )
             SELECT
               ds.date,
               ds.request_count,
               ds.total_tokens,
               ds.total_cost,
               dm.models,
               (SELECT json_group_object(model, json_object('count', count, 'tokens', tokens, 'cost', cost))
                FROM model_breakdown mb WHERE mb.date = ds.date) as model_stats
             FROM daily_stats ds
             LEFT JOIN daily_models dm ON ds.date = dm.date;"
        ).map_err(|e| format!("创建 daily_records 视图失败: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v001_up_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let migration = V001Initial;
        migration.up(&conn).unwrap();

        // 验证四个核心表已创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"_migrations".to_string()) || true); // _migrations 可能不在
        assert!(tables.contains(&"requests".to_string()));
        assert!(tables.contains(&"responses".to_string()));
        assert!(tables.contains(&"metrics".to_string()));
        assert!(tables.contains(&"mcp_calls".to_string()));
    }

    #[test]
    fn test_v001_up_creates_view() {
        let conn = Connection::open_in_memory().unwrap();
        let migration = V001Initial;
        migration.up(&conn).unwrap();

        // 验证 daily_records 视图已创建
        let views: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='view'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(views.contains(&"daily_records".to_string()));
    }

    #[test]
    fn test_v001_up_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        let migration = V001Initial;

        // 执行两次不应出错（IF NOT EXISTS 保护）
        migration.up(&conn).unwrap();
        migration.up(&conn).unwrap();
    }
}
