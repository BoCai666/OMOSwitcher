// 迁移 v002 - 添加 domain 和 updated_at 字段
// 复用 002-add-domain-field.sql 的内容
// 注意：ALTER TABLE ADD COLUMN 在列已存在时会报错，需在应用层检查

use rusqlite::Connection;

use super::Migration;

/// Domain 字段迁移
/// 为 requests 表添加 domain 和 updated_at 列
pub struct V002AddDomainField;

impl V002AddDomainField {
    /// 检查表中是否存在指定列
    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        let mut stmt = match conn.prepare(&format!("PRAGMA table_info({})", table)) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let rows = match stmt.query_map([], |row| {
            let col_name: String = row.get(1)?;
            Ok(col_name)
        }) {
            Ok(r) => r,
            Err(_) => return false,
        };

        for row in rows {
            if let Ok(name) = row {
                if name == column {
                    return true;
                }
            }
        }

        false
    }
}

impl Migration for V002AddDomainField {
    fn version(&self) -> u32 {
        2
    }

    fn description(&self) -> &str {
        "添加 domain 和 updated_at 字段到 requests 表"
    }

    fn up(&self, conn: &Connection) -> Result<(), String> {
        // 添加 domain 列（如果不存在）
        if !Self::column_exists(conn, "requests", "domain") {
            conn.execute_batch("ALTER TABLE requests ADD COLUMN domain TEXT")
                .map_err(|e| format!("添加 domain 列失败: {}", e))?;
        }

        // 添加 updated_at 列（如果不存在）
        if !Self::column_exists(conn, "requests", "updated_at") {
            conn.execute_batch("ALTER TABLE requests ADD COLUMN updated_at INTEGER")
                .map_err(|e| format!("添加 updated_at 列失败: {}", e))?;
        }

        // 创建索引
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_requests_domain ON requests(domain);
             CREATE INDEX IF NOT EXISTS idx_requests_updated_at ON requests(updated_at);",
        )
        .map_err(|e| format!("创建 domain 索引失败: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::migration::v001::V001Initial;

    #[test]
    fn test_v002_adds_columns() {
        let conn = Connection::open_in_memory().unwrap();

        // 先执行 v001
        V001Initial.up(&conn).unwrap();

        // 执行 v002
        let migration = V002AddDomainField;
        migration.up(&conn).unwrap();

        // 验证列已添加
        assert!(V002AddDomainField::column_exists(
            &conn, "requests", "domain"
        ));
        assert!(V002AddDomainField::column_exists(
            &conn,
            "requests",
            "updated_at"
        ));
    }

    #[test]
    fn test_v002_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        V001Initial.up(&conn).unwrap();

        let migration = V002AddDomainField;

        // 执行两次不应出错
        migration.up(&conn).unwrap();
        migration.up(&conn).unwrap();
    }

    #[test]
    fn test_column_exists_utility() {
        let conn = Connection::open_in_memory().unwrap();
        V001Initial.up(&conn).unwrap();

        // 验证已存在的列
        assert!(V002AddDomainField::column_exists(&conn, "requests", "id"));
        assert!(V002AddDomainField::column_exists(
            &conn,
            "requests",
            "timestamp"
        ));
        assert!(V002AddDomainField::column_exists(
            &conn, "requests", "model"
        ));

        // 验证不存在的列
        assert!(!V002AddDomainField::column_exists(
            &conn,
            "requests",
            "nonexistent_column"
        ));
    }
}
