// Monitor 模块 - 数据库迁移框架
// 管理数据库 schema 版本，确保迁移顺序执行且幂等

use rusqlite::Connection;

pub mod v001;
pub mod v002;

/// 迁移特征 - 所有数据库迁移必须实现此特征
pub trait Migration: Send {
    /// 返回迁移版本号
    fn version(&self) -> u32;

    /// 返回迁移描述
    fn description(&self) -> &str;

    /// 执行迁移升级
    fn up(&self, conn: &Connection) -> Result<(), String>;
}

/// 迁移管理器
/// 负责追踪已应用的迁移并按顺序执行未应用的迁移
pub struct MigrationManager;

impl MigrationManager {
    /// 确保迁移表存在
    pub fn ensure_migration_table(conn: &Connection) -> Result<(), String> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("创建迁移表失败: {}", e))?;

        Ok(())
    }

    /// 获取当前已应用的最高版本号
    pub fn current_version(conn: &Connection) -> Result<u32, String> {
        let version: Option<u32> = conn
            .query_row("SELECT MAX(version) FROM _migrations", [], |row| row.get(0))
            .map_err(|e| format!("查询当前版本失败: {}", e))?;

        Ok(version.unwrap_or(0))
    }

    /// 执行所有未应用的迁移
    /// 按版本号顺序执行，确保依赖关系正确
    pub fn run_pending(
        conn: &Connection,
        migrations: Vec<Box<dyn Migration>>,
    ) -> Result<(), String> {
        Self::ensure_migration_table(conn)?;

        // 按版本号排序
        let mut sorted: Vec<_> = migrations.into_iter().collect();
        sorted.sort_by_key(|m| m.version());

        let mut current = Self::current_version(conn)?;

        for migration in sorted {
            if migration.version() <= current {
                continue;
            }

            if migration.version() != current + 1 {
                return Err(format!(
                    "迁移版本跳跃: 当前 {}，期望 {}",
                    current + 1,
                    migration.version()
                ));
            }

            migration.up(conn)?;
            Self::record_migration(conn, migration.version(), migration.description())?;
            current = migration.version();
        }

        Ok(())
    }

    /// 记录已执行的迁移
    fn record_migration(conn: &Connection, version: u32, description: &str) -> Result<(), String> {
        let applied_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("获取时间戳失败: {}", e))?
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO _migrations (version, description, applied_at) VALUES (?, ?, ?)",
            rusqlite::params![version, description, applied_at],
        )
        .map_err(|e| format!("记录迁移失败: {}", e))?;

        Ok(())
    }
}
