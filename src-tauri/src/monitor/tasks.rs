// Monitor 模块 - 后台数据清理任务
// 提供数据清理、数据库备份和数据导出功能
// 注意：部分函数尚未被主流程集成调用，保留供后续集成使用

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::time::{interval, Duration};

use crate::monitor::storage::MonitorStorage;

// ============================================================================
// 辅助函数
// ============================================================================

/// 格式化时间戳字符串
/// 返回格式: YYYY-MM-DDTHH-MM-SS
fn format_timestamp(datetime: time::OffsetDateTime) -> String {
    // 使用 time crate 的格式化功能
    let year = datetime.year();
    let month = datetime.month() as u8;
    let day = datetime.day();
    let hour = datetime.hour();
    let minute = datetime.minute();
    let second = datetime.second();
    
    format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        year, month, day, hour, minute, second
    )
}

/// 格式化日期字符串
/// 返回格式: YYYY-MM-DD
fn format_date(datetime: time::OffsetDateTime) -> String {
    let year = datetime.year();
    let month = datetime.month() as u8;
    let day = datetime.day();
    
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// 格式化日期时间字符串
/// 返回格式: YYYY-MM-DD HH:MM:SS
fn format_datetime(datetime: time::OffsetDateTime) -> String {
    let year = datetime.year();
    let month = datetime.month() as u8;
    let day = datetime.day();
    let hour = datetime.hour();
    let minute = datetime.minute();
    let second = datetime.second();
    
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

// ============================================================================
// 配置类型
// ============================================================================

/// 数据清理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupConfig {
    /// 是否启用清理
    pub enabled: bool,
    /// 数据保留天数
    pub retention_days: u32,
    /// 是否在删除前归档
    pub archive_before_delete: bool,
    /// 清理执行时间（小时，0-23）
    pub cleanup_hour: u8,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
            archive_before_delete: true,
            cleanup_hour: 3, // 凌晨 3 点执行
        }
    }
}

/// 备份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfig {
    /// 是否启用自动备份
    pub enabled: bool,
    /// 最大备份数量
    pub max_backups: usize,
    /// 备份执行时间（小时，0-23）
    pub backup_hour: u8,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_backups: 7,
            backup_hour: 2, // 凌晨 2 点执行
        }
    }
}

// ============================================================================
// 清理结果类型
// ============================================================================

/// 数据清理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    /// 归档的记录数
    pub archived: u64,
    /// 删除的记录数
    pub deleted: u64,
    /// 归档文件路径（如果启用归档）
    pub archive_path: Option<String>,
}

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// 文件名
    pub filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 创建时间
    pub created_at: String,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// 导出文件路径
    pub path: String,
    /// 导出格式
    pub format: ExportFormat,
    /// 导出的记录数
    pub count: u64,
    /// 文件大小（字节）
    pub size: u64,
}

// ============================================================================
// 数据清理任务
// ============================================================================

/// 数据清理任务
/// 定期清理过期数据，支持归档和删除
pub struct DataCleanupTask {
    storage: Arc<MonitorStorage>,
    config: CleanupConfig,
    backup_dir: PathBuf,
    archive_dir: PathBuf,
}

impl DataCleanupTask {
    /// 创建新的清理任务
    pub fn new(storage: Arc<MonitorStorage>, config: CleanupConfig, data_dir: &Path) -> Self {
        let backup_dir = data_dir.join("backups");
        let archive_dir = data_dir.join("archives");

        Self {
            storage,
            config,
            backup_dir,
            archive_dir,
        }
    }

    /// 初始化：创建必要的目录
    pub async fn initialize(&self) -> Result<(), String> {
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| format!("创建备份目录失败: {}", e))?;
        fs::create_dir_all(&self.archive_dir)
            .await
            .map_err(|e| format!("创建归档目录失败: {}", e))?;
        Ok(())
    }

    /// 执行数据清理
    pub async fn cleanup(&self) -> Result<CleanupResult, String> {
        if !self.config.enabled {
            return Ok(CleanupResult {
                archived: 0,
                deleted: 0,
                archive_path: None,
            });
        }

        // 计算截止时间戳（当前时间 - retention_days 天）
        let cutoff_timestamp = self.calculate_cutoff_timestamp();

        tracing::info!(
            "[Cleanup] 开始清理 {} 天前的数据，截止时间戳: {}",
            self.config.retention_days,
            cutoff_timestamp
        );

        // 查询将要删除的数据数量
        let expired_count = self.count_expired_requests(cutoff_timestamp).await?;

        if expired_count == 0 {
            tracing::info!("[Cleanup] 没有过期数据需要清理");
            return Ok(CleanupResult {
                archived: 0,
                deleted: 0,
                archive_path: None,
            });
        }

        tracing::info!("[Cleanup] 发现 {} 条过期记录", expired_count);

        // 归档数据
        let mut archive_path = None;
        let mut archived = 0;
        if self.config.archive_before_delete {
            let result = self.archive_data(cutoff_timestamp).await?;
            archive_path = Some(result.0);
            archived = result.1;
        }

        // 删除过期数据
        let deleted = self.delete_expired_data(cutoff_timestamp).await?;

        tracing::info!(
            "[Cleanup] 清理完成: 归档 {} 条，删除 {} 条",
            archived,
            deleted
        );

        Ok(CleanupResult {
            archived,
            deleted,
            archive_path,
        })
    }

    /// 计算截止时间戳（毫秒）
    fn calculate_cutoff_timestamp(&self) -> i64 {
        let now = time::OffsetDateTime::now_utc();
        let cutoff = now - time::Duration::days(self.config.retention_days as i64);
        cutoff.unix_timestamp() * 1000
    }

    /// 统计过期请求数量
    async fn count_expired_requests(&self, cutoff_timestamp: i64) -> Result<u64, String> {
        self.storage.count_expired_requests(cutoff_timestamp).await
    }

    /// 归档过期数据
    async fn archive_data(&self, cutoff_timestamp: i64) -> Result<(String, u64), String> {
        let date_str = format_date(time::OffsetDateTime::now_utc());

        let archive_filename = format!("archive-{}.json", date_str);
        let archive_path = self.archive_dir.join(&archive_filename);

        // 查询过期数据
        let requests = self
            .storage
            .get_requests_by_timestamp_range_with_metrics(0, cutoff_timestamp, None)
            .await?;

        let count = requests.len() as u64;

        if count == 0 {
            return Ok((archive_path.to_string_lossy().to_string(), 0));
        }

        // 序列化为 JSON
        let json_content = serde_json::to_string_pretty(&requests)
            .map_err(|e| format!("序列化归档数据失败: {}", e))?;

        // 写入文件
        fs::write(&archive_path, json_content)
            .await
            .map_err(|e| format!("写入归档文件失败: {}", e))?;

        tracing::info!("[Cleanup] 数据已归档到: {:?}", archive_path);

        Ok((archive_path.to_string_lossy().to_string(), count))
    }

    /// 删除过期数据
    async fn delete_expired_data(&self, cutoff_timestamp: i64) -> Result<u64, String> {
        // 由于外键约束，删除 requests 会级联删除 responses, metrics, mcp_calls
        self.storage.delete_expired_requests(cutoff_timestamp).await
    }

    /// 启动定时清理任务
    pub fn start_scheduled_cleanup(self: Arc<Self>) {
        tokio::spawn(async move {
            let cleanup_hour = self.config.cleanup_hour;
            let mut interval = interval(Duration::from_secs(60)); // 每分钟检查一次

            loop {
                interval.tick().await;

                let now = time::OffsetDateTime::now_utc();
                if now.hour() as u8 == cleanup_hour && now.minute() == 0 {
                    tracing::info!("[Cleanup] 执行定时清理任务");
                    if let Err(e) = self.cleanup().await {
                        tracing::error!("[Cleanup] 定时清理失败: {}", e);
                    }
                }
            }
        });
    }
}

// ============================================================================
// 数据库备份功能
// ============================================================================

/// 数据库备份管理器
pub struct DatabaseBackup {
    db_path: PathBuf,
    backup_dir: PathBuf,
    config: BackupConfig,
}

impl DatabaseBackup {
    /// 创建新的备份管理器
    pub fn new(db_path: PathBuf, data_dir: &Path, config: BackupConfig) -> Self {
        let backup_dir = data_dir.join("backups");
        Self {
            db_path,
            backup_dir,
            config,
        }
    }

    /// 初始化：创建备份目录
    pub async fn initialize(&self) -> Result<(), String> {
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| format!("创建备份目录失败: {}", e))?;
        tracing::info!("[Backup] 备份目录: {:?}", self.backup_dir);
        Ok(())
    }

    /// 创建数据库备份
    pub async fn create_backup(&self) -> Result<PathBuf, String> {
        // 生成备份文件名
        let timestamp = format_timestamp(time::OffsetDateTime::now_utc());

        let backup_filename = format!("backup-{}.db", timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);

        tracing::info!("[Backup] 创建备份: {:?}", backup_path);

        // 复制数据库文件
        fs::copy(&self.db_path, &backup_path)
            .await
            .map_err(|e| format!("复制数据库文件失败: {}", e))?;

        // 获取备份文件大小
        let metadata = fs::metadata(&backup_path)
            .await
            .map_err(|e| format!("获取备份文件信息失败: {}", e))?;

        tracing::info!(
            "[Backup] 备份完成: {:?} ({} 字节)",
            backup_path,
            metadata.len()
        );

        // 清理旧备份
        self.cleanup_old_backups().await?;

        Ok(backup_path)
    }

    /// 清理旧备份（保留最近的 max_backups 个）
    async fn cleanup_old_backups(&self) -> Result<(), String> {
        let mut backups = Vec::new();

        // 读取备份目录
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| format!("读取备份目录失败: {}", e))?;

        // 收集所有备份文件
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("读取备份目录条目失败: {}", e))?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("backup-") && filename_str.ends_with(".db") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((path, modified));
                        }
                    }
                }
            }
        }

        // 按修改时间排序（最新的在前）
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // 删除多余的旧备份
        let to_delete = backups.len().saturating_sub(self.config.max_backups);
        for (path, _) in backups.into_iter().take(to_delete) {
            tracing::info!("[Backup] 删除旧备份: {:?}", path);
            fs::remove_file(&path)
                .await
                .map_err(|e| format!("删除旧备份失败: {}", e))?;
        }

        Ok(())
    }

    /// 获取备份列表
    pub async fn get_backup_list(&self) -> Result<Vec<BackupInfo>, String> {
        let mut backup_list = Vec::new();

        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| format!("读取备份目录失败: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("读取备份目录条目失败: {}", e))?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("backup-") && filename_str.ends_with(".db") {
                    if let Ok(metadata) = entry.metadata().await {
                        let size = metadata.len();
                        let created_at = metadata
                            .modified()
                            .ok()
                            .map(|t| {
                                let datetime: time::OffsetDateTime = t.into();
                                format_datetime(datetime)
                            })
                            .unwrap_or_else(|| "unknown".to_string());

                        backup_list.push(BackupInfo {
                            filename: filename_str.to_string(),
                            size,
                            created_at,
                        });
                    }
                }
            }
        }

        // 按创建时间倒序排列
        backup_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backup_list)
    }

    /// 启动定时备份任务
    pub fn start_scheduled_backup(self: Arc<Self>) {
        tokio::spawn(async move {
            let backup_hour = self.config.backup_hour;
            let mut interval = interval(Duration::from_secs(60)); // 每分钟检查一次

            loop {
                interval.tick().await;

                let now = time::OffsetDateTime::now_utc();
                if now.hour() as u8 == backup_hour && now.minute() == 0 {
                    tracing::info!("[Backup] 执行定时备份任务");
                    if let Err(e) = self.create_backup().await {
                        tracing::error!("[Backup] 定时备份失败: {}", e);
                    }
                }
            }
        });
    }
}

// ============================================================================
// 数据导出功能
// ============================================================================

/// 数据导出器
pub struct DataExporter {
    export_dir: PathBuf,
}

impl DataExporter {
    /// 创建新的导出器
    pub fn new(data_dir: &Path) -> Self {
        let export_dir = data_dir.join("exports");
        Self { export_dir }
    }

    /// 初始化：创建导出目录
    pub async fn initialize(&self) -> Result<(), String> {
        fs::create_dir_all(&self.export_dir)
            .await
            .map_err(|e| format!("创建导出目录失败: {}", e))?;
        Ok(())
    }

    /// 导出数据
    pub async fn export_data(
        &self,
        storage: &MonitorStorage,
        format: ExportFormat,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<ExportResult, String> {
        // 确定时间范围
        let start = start_time.unwrap_or(0);
        let end = end_time.unwrap_or_else(|| {
            time::OffsetDateTime::now_utc().unix_timestamp() * 1000
        });

        // 获取数据
        let requests = storage
            .get_requests_by_timestamp_range_with_metrics(start, end, None)
            .await?;

        let count = requests.len() as u64;

        // 生成文件名
        let timestamp = format_timestamp(time::OffsetDateTime::now_utc());

        let (filename, content) = match format {
            ExportFormat::Json => {
                let json_content = serde_json::to_string_pretty(&requests)
                    .map_err(|e| format!("序列化 JSON 失败: {}", e))?;
                (format!("export-{}.json", timestamp), json_content)
            }
            ExportFormat::Csv => {
                let csv_content = self.generate_csv(&requests)?;
                (format!("export-{}.csv", timestamp), csv_content)
            }
        };

        let export_path = self.export_dir.join(&filename);

        // 写入文件
        fs::write(&export_path, content)
            .await
            .map_err(|e| format!("写入导出文件失败: {}", e))?;

        // 获取文件大小
        let metadata = fs::metadata(&export_path)
            .await
            .map_err(|e| format!("获取导出文件信息失败: {}", e))?;

        tracing::info!(
            "[Export] 数据已导出到: {:?} ({} 条记录, {} 字节)",
            export_path,
            count,
            metadata.len()
        );

        Ok(ExportResult {
            path: export_path.to_string_lossy().to_string(),
            format,
            count,
            size: metadata.len(),
        })
    }

    /// 生成 CSV 内容
    fn generate_csv(&self, requests: &[crate::monitor::types::RequestListItem]) -> Result<String, String> {
        let mut csv = String::new();
        
        // CSV 表头
        csv.push_str("id,timestamp,provider,model,method,url,domain,tokens,cost,duration,status_code\n");

        // 数据行
        for req in requests {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                req.id,
                req.timestamp,
                req.provider,
                req.model,
                req.method,
                req.url,
                req.domain.as_deref().unwrap_or(""),
                req.tokens.unwrap_or(0),
                req.cost.unwrap_or(0.0),
                req.duration.unwrap_or(0),
                req.status_code.unwrap_or(0)
            ));
        }

        Ok(csv)
    }
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 创建数据库备份
/// 备份文件保存到 backups 目录
pub async fn backup_database(db_path: &Path, data_dir: &Path) -> Result<PathBuf, String> {
    let backup_manager = DatabaseBackup::new(
        db_path.to_path_buf(),
        data_dir,
        BackupConfig::default(),
    );

    backup_manager.initialize().await?;
    backup_manager.create_backup().await
}

/// 导出数据到文件
/// 支持 JSON 和 CSV 格式
pub async fn export_data(
    storage: &MonitorStorage,
    data_dir: &Path,
    format: ExportFormat,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<ExportResult, String> {
    let exporter = DataExporter::new(data_dir);
    exporter.initialize().await?;
    exporter.export_data(storage, format, start_time, end_time).await
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// 创建临时测试目录
    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let temp_base = env::temp_dir().join("omoswitcher_tests").join(name);
        // 清理可能存在的旧目录
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();
        temp_base
    }

    #[tokio::test]
    async fn test_cleanup_config_default() {
        let config = CleanupConfig::default();
        assert!(config.enabled);
        assert_eq!(config.retention_days, 90);
        assert!(config.archive_before_delete);
        assert_eq!(config.cleanup_hour, 3);
    }

    #[tokio::test]
    async fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_backups, 7);
        assert_eq!(config.backup_hour, 2);
    }

    #[tokio::test]
    async fn test_database_backup_initialize() {
        let temp_dir = create_temp_dir("backup_init");
        let db_path = temp_dir.join("test.db");
        let data_dir = temp_dir.join("data");

        let backup = DatabaseBackup::new(
            db_path,
            &data_dir,
            BackupConfig::default(),
        );

        backup.initialize().await.unwrap();

        // 验证备份目录已创建
        assert!(data_dir.join("backups").exists());
    }

    #[tokio::test]
    async fn test_data_exporter_initialize() {
        let temp_dir = create_temp_dir("exporter_init");
        let data_dir = temp_dir.join("data");

        let exporter = DataExporter::new(&data_dir);
        exporter.initialize().await.unwrap();

        // 验证导出目录已创建
        assert!(data_dir.join("exports").exists());
    }

    #[tokio::test]
    async fn test_generate_csv() {
        let temp_dir = create_temp_dir("csv_gen");
        let exporter = DataExporter::new(&temp_dir);

        let requests = vec![
            crate::monitor::types::RequestListItem {
                id: "req-1".to_string(),
                timestamp: 1000000,
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                method: "POST".to_string(),
                url: "https://api.openai.com/v1/chat/completions".to_string(),
                domain: Some("api.openai.com".to_string()),
                tokens: Some(150),
                cost: Some(0.03),
                duration: Some(500),
                status_code: Some(200),
            },
        ];

        let csv = exporter.generate_csv(&requests).unwrap();
        
        assert!(csv.contains("id,timestamp,provider,model"));
        assert!(csv.contains("req-1"));
        assert!(csv.contains("gpt-4"));
        assert!(csv.contains("api.openai.com"));
    }

    #[tokio::test]
    async fn test_backup_info_serialization() {
        let info = BackupInfo {
            filename: "backup-2024-01-01T00-00-00.db".to_string(),
            size: 1024,
            created_at: "2024-01-01 00:00:00".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"filename\""));
        assert!(json.contains("\"size\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[tokio::test]
    async fn test_export_result_serialization() {
        let result = ExportResult {
            path: "/path/to/export.json".to_string(),
            format: ExportFormat::Json,
            count: 100,
            size: 2048,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"path\""));
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"count\""));
    }

    #[tokio::test]
    async fn test_cleanup_result_serialization() {
        let result = CleanupResult {
            archived: 10,
            deleted: 20,
            archive_path: Some("/path/to/archive.json".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"archived\":10"));
        assert!(json.contains("\"deleted\":20"));
        assert!(json.contains("\"archivePath\""));
    }

    #[test]
    fn test_calculate_cutoff_timestamp() {
        let config = CleanupConfig {
            retention_days: 30,
            ..Default::default()
        };
        
        let temp_dir = create_temp_dir("cutoff_test");
        let storage = Arc::new(
            MonitorStorage::open_in_memory().unwrap()
        );
        
        let task = DataCleanupTask::new(
            storage,
            config,
            &temp_dir,
        );

        let cutoff = task.calculate_cutoff_timestamp();
        
        // 验证截止时间戳是大约 30 天前
        let now = time::OffsetDateTime::now_utc().unix_timestamp() * 1000;
        let expected_cutoff = now - (30 * 24 * 60 * 60 * 1000);
        
        // 允许 1 秒的误差
        assert!((cutoff - expected_cutoff).abs() < 1000);
    }

    #[test]
    fn test_format_timestamp() {
        let dt = time::OffsetDateTime::from_unix_timestamp(1704067200).unwrap(); // 2024-01-01 00:00:00 UTC
        let formatted = format_timestamp(dt);
        assert!(formatted.starts_with("2024-01-01"));
    }

    #[test]
    fn test_format_date() {
        let dt = time::OffsetDateTime::from_unix_timestamp(1704067200).unwrap(); // 2024-01-01 00:00:00 UTC
        let formatted = format_date(dt);
        assert_eq!(formatted, "2024-01-01");
    }

    #[test]
    fn test_format_datetime() {
        let dt = time::OffsetDateTime::from_unix_timestamp(1704067200).unwrap(); // 2024-01-01 00:00:00 UTC
        let formatted = format_datetime(dt);
        assert!(formatted.starts_with("2024-01-01"));
        assert!(formatted.contains(":"));
    }
}
