-- 数据库迁移脚本 002
-- 添加 domain 字段和 updated_at 字段到 requests表

-- SQLite 不支持 IF NOT EXISTS，使用兼容方式
-- 先尝试添加，如果失败则忽略错误

-- 添加 domain 字段（TEXT 类型，可为空）
-- 注意：如果列已存在，此语句会报错，需要在应用层处理
ALTER TABLE requests ADD COLUMN domain TEXT;

-- 添加 updated_at 字段（INTEGER 类型，时间戳）
ALTER TABLE requests ADD COLUMN updated_at INTEGER;

-- 为 domain 字段创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_requests_domain ON requests(domain);
CREATE INDEX IF NOT EXISTS idx_requests_updated_at ON requests(updated_at);
