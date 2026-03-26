-- 数据库初始化脚本
-- 创建 requests 表
CREATE TABLE IF NOT EXISTS requests (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  provider TEXT,
  model TEXT,
  method TEXT,
  url TEXT,
  headers TEXT, -- JSON
  body TEXT, -- JSON
  parsed_body TEXT -- JSON
);

-- 创建 responses 表
CREATE TABLE IF NOT EXISTS responses (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  status_code INTEGER,
  headers TEXT, -- JSON
  body TEXT, -- JSON
  parsed_body TEXT, -- JSON
  duration INTEGER,
  FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);

-- 创建 metrics 表
CREATE TABLE IF NOT EXISTS metrics (
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
);

-- 创建 mcp_calls 表
CREATE TABLE IF NOT EXISTS mcp_calls (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  jsonrpc_version TEXT DEFAULT '2.0',
  rpc_id TEXT,
  tool_name TEXT NOT NULL,
  tool_title TEXT,
  tool_description TEXT,
  arguments TEXT, -- JSON
  result_content TEXT, -- JSON
  result_is_error BOOLEAN DEFAULT FALSE,
  error_message TEXT,
  execution_duration INTEGER,
  transport_type TEXT,
  server_name TEXT,
  trace_id TEXT,
  timestamp INTEGER NOT NULL,
  FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_requests_timestamp ON requests(timestamp);
CREATE INDEX IF NOT EXISTS idx_requests_time_model ON requests(timestamp, model);
CREATE INDEX IF NOT EXISTS idx_responses_request_id ON responses(request_id);
CREATE INDEX IF NOT EXISTS idx_metrics_request_id ON metrics(request_id);
CREATE INDEX IF NOT EXISTS idx_metrics_time_model ON metrics(timestamp, model);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_request_id ON mcp_calls(request_id);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_tool_name ON mcp_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_mcp_calls_timestamp ON mcp_calls(timestamp);

-- 创建 daily_records 视图
CREATE VIEW IF NOT EXISTS daily_records AS
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
  GROUP BY date(timestamp/0.0000029, 'unixepoch', 'localtime'), model
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
LEFT JOIN daily_models dm ON ds.date = dm.date;

-- 为视图查询创建索引优化（基于时间戳范围查询）
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp_model ON metrics(timestamp, model);
