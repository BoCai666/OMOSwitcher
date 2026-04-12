// Monitor 模块 - Tauri Commands
// 提供前端与 Monitor 后端交互的命令桥接层
// 所有命令遵循 Result<T, String> 模式，错误消息使用中文

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex as AsyncMutex;

use crate::monitor::cert::CertManager;
use crate::monitor::config::ConfigManager;
use crate::monitor::proxy::ProxyServer;
use crate::monitor::storage::MonitorStorage;
use crate::monitor::tasks::{BackupInfo, DataExporter, DatabaseBackup, ExportFormat};
use crate::monitor::types::{
    DailyRecord, DeltaResult, DomainConfig, DomainStatsResult, LLMMetrics, LLMRequest,
    LLMResponse, MCPCall, ModelPricingConfig, MonitorConfig, PricingConfig,
    RequestListItem,
};

// ============================================================================
// Monitor 命令状态
// ============================================================================

/// Monitor 命令层共享状态
/// 用于 Tauri State 管理，整合代理服务、存储、配置、证书
#[derive(Clone)]
pub struct MonitorCommandState {
    /// 存储层
    pub storage: Arc<MonitorStorage>,
    /// 配置管理器
    pub config_manager: Arc<std::sync::Mutex<ConfigManager>>,
    /// 证书管理器
    pub cert_manager: Arc<std::sync::Mutex<CertManager>>,
    /// 数据目录
    pub data_dir: PathBuf,
    /// 数据库路径
    pub db_path: PathBuf,
    /// 代理服务器实例（惰性启动）
    pub proxy: Arc<AsyncMutex<Option<ProxyServer>>>,
}

impl MonitorCommandState {
    /// 创建新的命令状态（自动初始化存储、配置、证书）
    pub fn new() -> Result<Self, String> {
        // 确定路径
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let omoswitcher_dir = home.join(".config").join("omoswitcher");
        let data_dir = omoswitcher_dir.join("monitor");
        let db_path = omoswitcher_dir.join("monitor.db");
        let config_path = data_dir.join("config.jsonc");
        let certs_dir = data_dir.join("certs");

        // 初始化各组件
        let storage = MonitorStorage::new(&db_path)?;
        let config_manager = ConfigManager::new(&config_path)?;
        let cert_manager = CertManager::with_certs_dir(certs_dir)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            storage: Arc::new(storage),
            config_manager: Arc::new(std::sync::Mutex::new(config_manager)),
            cert_manager: Arc::new(std::sync::Mutex::new(cert_manager)),
            data_dir,
            db_path,
            proxy: Arc::new(AsyncMutex::new(None)),
        })
    }

    /// 停止代理服务器（用于窗口关闭时调用）
    pub async fn stop_proxy(&self) {
        let mut proxy = self.proxy.lock().await;
        if let Some(p) = proxy.take() {
            if p.is_running() {
                let _ = p.stop().await;
                tracing::info!("[Monitor] 代理服务器已停止");
            }
        }
    }
}

// ============================================================================
// 统计摘要类型
// ============================================================================

/// 统计摘要（用于首页仪表盘）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummary {
    /// 总请求数
    pub total_requests: i64,
    /// 总 token 数
    pub total_tokens: i64,
    /// 总费用（美元）
    pub total_cost: f64,
    /// 今日请求数
    pub today_requests: i64,
    /// 今日 token 数
    pub today_tokens: i64,
    /// 今日费用
    pub today_cost: f64,
}

// ============================================================================
// 数据查询命令
// ============================================================================

/// 获取请求列表
/// limit: 返回数量限制
/// start_date: 开始日期（可选，格式 YYYY-MM-DD）
/// end_date: 结束日期（可选，格式 YYYY-MM-DD）
#[tauri::command]
pub async fn monitor_get_requests(
    limit: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<RequestListItem>, String> {
    let storage = state.storage.clone();

    match (start_date, end_date) {
        (Some(start), Some(end)) => {
            storage
                .get_requests_by_date_range_with_metrics(&start, &end, Some(limit))
                .await
        }
        _ => storage.get_recent_requests_with_metrics(limit).await,
    }
}

/// 根据 ID 获取请求详情
#[tauri::command]
pub async fn monitor_get_request(
    id: String,
    state: State<'_, MonitorCommandState>,
) -> Result<Option<LLMRequest>, String> {
    state.storage.get_request_by_id(&id).await
}

/// 根据请求 ID 获取响应
#[tauri::command]
pub async fn monitor_get_response(
    request_id: String,
    state: State<'_, MonitorCommandState>,
) -> Result<Option<LLMResponse>, String> {
    state.storage.get_response_by_request_id(&request_id).await
}

/// 根据请求 ID 获取指标
#[tauri::command]
pub async fn monitor_get_metrics(
    request_id: String,
    state: State<'_, MonitorCommandState>,
) -> Result<Option<LLMMetrics>, String> {
    state.storage.get_metrics_by_request_id(&request_id).await
}

/// 根据请求 ID 获取 MCP 调用列表
#[tauri::command]
pub async fn monitor_get_mcp_calls(
    request_id: String,
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<MCPCall>, String> {
    state.storage.get_mcp_calls_by_request_id(&request_id).await
}

/// 获取统计摘要
#[tauri::command]
pub async fn monitor_get_stats_summary(
    state: State<'_, MonitorCommandState>,
) -> Result<StatsSummary, String> {
    let storage = state.storage.clone();

    // 获取当前时间戳范围
    let now = time::OffsetDateTime::now_utc();
    let today_start = {
        let date = time::Date::from_calendar_date(
            now.year(),
            now.month(),
            now.day(),
        ).map_err(|e| format!("日期计算失败: {}", e))?;
        let midday = time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT);
        midday.assume_utc().unix_timestamp() * 1000
    };
    let now_ts = now.unix_timestamp() * 1000;

    // 获取全部统计（过去一年）
    let one_year_ago = (now.unix_timestamp() - 365 * 24 * 60 * 60) * 1000;
    let total_stats = storage.get_metrics_stats(one_year_ago, now_ts).await?;

    // 获取今日统计
    let today_stats = storage.get_metrics_stats(today_start, now_ts).await?;

    // 统计请求总数
    let total_requests = storage.get_recent_requests_with_metrics(i64::MAX).await?;
    let today_requests_list = storage
        .get_requests_by_timestamp_range_with_metrics(today_start, now_ts, None)
        .await?;

    Ok(StatsSummary {
        total_requests: total_requests.len() as i64,
        total_tokens: total_stats.total_tokens,
        total_cost: total_stats.total_cost,
        today_requests: today_requests_list.len() as i64,
        today_tokens: today_stats.total_tokens,
        today_cost: today_stats.total_cost,
    })
}

/// 获取每日记录
#[tauri::command]
pub async fn monitor_get_daily_records(
    start_date: String,
    end_date: String,
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<DailyRecord>, String> {
    state.storage.get_daily_records(&start_date, &end_date).await
}

/// 获取域名统计
#[tauri::command]
pub async fn monitor_get_domain_stats(
    start_date: String,
    end_date: String,
    state: State<'_, MonitorCommandState>,
) -> Result<DomainStatsResult, String> {
    let storage = state.storage.clone();

    // 日期转时间戳
    let start_ts = date_to_timestamp(&start_date, true);
    let end_ts = date_to_timestamp(&end_date, false);

    storage.get_domain_stats(start_ts, end_ts).await
}

/// 获取所有模型列表
#[tauri::command]
pub async fn monitor_get_all_models(
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<String>, String> {
    state.storage.get_all_models().await
}

/// 获取增量更新数据
#[tauri::command]
pub async fn monitor_get_delta(
    since: i64,
    limit: i64,
    state: State<'_, MonitorCommandState>,
) -> Result<DeltaResult, String> {
    state.storage.get_delta(since, limit).await
}

// ============================================================================
// 数据操作命令
// ============================================================================

/// 清空所有数据
#[tauri::command]
pub async fn monitor_clear_data(
    state: State<'_, MonitorCommandState>,
) -> Result<(), String> {
    state.storage.clear().await
}

/// 导出数据
#[tauri::command]
pub async fn monitor_export_data(
    format: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
    state: State<'_, MonitorCommandState>,
) -> Result<String, String> {
    let export_format = match format.to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        "csv" => ExportFormat::Csv,
        _ => return Err(format!("不支持的导出格式: {}", format)),
    };

    let exporter = DataExporter::new(&state.data_dir);
    exporter.initialize().await?;

    let result = exporter
        .export_data(&state.storage, export_format, start_time, end_time)
        .await?;

    Ok(result.path)
}

/// 创建数据库备份
#[tauri::command]
pub async fn monitor_backup(
    state: State<'_, MonitorCommandState>,
) -> Result<String, String> {
    let backup_manager = DatabaseBackup::new(
        state.db_path.clone(),
        &state.data_dir,
        crate::monitor::tasks::BackupConfig::default(),
    );

    backup_manager.initialize().await?;
    let backup_path = backup_manager.create_backup().await?;

    Ok(backup_path.to_string_lossy().to_string())
}

/// 获取备份列表
#[tauri::command]
pub async fn monitor_get_backups(
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<BackupInfo>, String> {
    let backup_manager = DatabaseBackup::new(
        state.db_path.clone(),
        &state.data_dir,
        crate::monitor::tasks::BackupConfig::default(),
    );

    backup_manager.get_backup_list().await
}

// ============================================================================
// 配置操作命令
// ============================================================================

/// 获取配置
#[tauri::command]
pub async fn monitor_get_config(
    state: State<'_, MonitorCommandState>,
) -> Result<MonitorConfig, String> {
    let config_manager = state.config_manager.lock().map_err(|e| {
        format!("获取配置管理器锁失败: {}", e)
    })?;
    Ok(config_manager.get_config())
}

/// 更新配置
#[tauri::command]
pub async fn monitor_update_config(
    config: MonitorConfig,
    state: State<'_, MonitorCommandState>,
) -> Result<(), String> {
    // 更新配置文件
    let config_path = crate::monitor::config::default_config_path()?;
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    tokio::fs::write(&config_path, config_json)
        .await
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 重新加载配置
    let config_manager = state.config_manager.lock().map_err(|e| {
        format!("获取配置管理器锁失败: {}", e)
    })?;
    config_manager.reload()?;

    Ok(())
}

/// 获取域名配置列表
#[tauri::command]
pub async fn monitor_get_domains(
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<DomainConfig>, String> {
    let config_manager = state.config_manager.lock().map_err(|e| {
        format!("获取配置管理器锁失败: {}", e)
    })?;
    Ok(config_manager.get_config().domains)
}

/// 添加域名配置
#[tauri::command]
pub async fn monitor_add_domain(
    domain: DomainConfig,
    state: State<'_, MonitorCommandState>,
) -> Result<(), String> {
    // 获取配置（立即释放锁）
    let (config, config_path) = {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        
        let mut config = config_manager.get_config();
        config.domains.push(domain);
        
        let config_path = crate::monitor::config::default_config_path()?;
        (config, config_path)
    };

    // 写入文件（不持有锁）
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    tokio::fs::write(&config_path, config_json)
        .await
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 重新加载（重新获取锁）
    {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        config_manager.reload()?;
    }

    Ok(())
}

/// 移除域名配置
#[tauri::command]
pub async fn monitor_remove_domain(
    index: usize,
    state: State<'_, MonitorCommandState>,
) -> Result<(), String> {
    // 获取配置（立即释放锁）
    let (config, config_path) = {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;

        let mut config = config_manager.get_config();

        if index >= config.domains.len() {
            return Err(format!("域名索引越界: {} >= {}", index, config.domains.len()));
        }

        config.domains.remove(index);

        let config_path = crate::monitor::config::default_config_path()?;
        (config, config_path)
    };

    // 写入文件（不持有锁）
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    tokio::fs::write(&config_path, config_json)
        .await
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 重新加载（重新获取锁）
    {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        config_manager.reload()?;
    }

    Ok(())
}

/// 获取定价配置
#[tauri::command]
pub async fn monitor_get_pricing(
    state: State<'_, MonitorCommandState>,
) -> Result<Vec<ModelPricingConfig>, String> {
    let config_manager = state.config_manager.lock().map_err(|e| {
        format!("获取配置管理器锁失败: {}", e)
    })?;
    Ok(config_manager.get_config().pricing.models)
}

/// 更新定价配置
#[tauri::command]
pub async fn monitor_update_pricing(
    pricing: PricingConfig,
    state: State<'_, MonitorCommandState>,
) -> Result<(), String> {
    // 获取配置（立即释放锁）
    let (config, config_path) = {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;

        let mut config = config_manager.get_config();
        config.pricing = pricing;

        let config_path = crate::monitor::config::default_config_path()?;
        (config, config_path)
    };

    // 写入文件（不持有锁）
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    tokio::fs::write(&config_path, config_json)
        .await
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 重新加载（重新获取锁）
    {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        config_manager.reload()?;
    }

    Ok(())
}

// ============================================================================
// 证书操作命令
// ============================================================================

/// 证书状态（用于命令返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertStatusResponse {
    /// CA 证书文件是否存在
    pub ca_cert_exists: bool,
    /// CA 私钥文件是否存在
    pub ca_key_exists: bool,
    /// 证书目录是否存在
    pub cert_dir_exists: bool,
    /// CA 证书路径
    pub ca_cert_path: String,
}

/// 获取证书状态
#[tauri::command]
pub async fn monitor_cert_status(
    state: State<'_, MonitorCommandState>,
) -> Result<CertStatusResponse, String> {
    let cert_manager = state.cert_manager.lock().map_err(|e| {
        format!("获取证书管理器锁失败: {}", e)
    })?;

    let status = cert_manager.cert_status();
    let ca_cert_path = cert_manager.get_ca_cert_path();

    Ok(CertStatusResponse {
        ca_cert_exists: status.ca_cert_exists,
        ca_key_exists: status.ca_key_exists,
        cert_dir_exists: status.cert_dir_exists,
        ca_cert_path: ca_cert_path.to_string_lossy().to_string(),
    })
}

/// 健康检查
#[tauri::command]
pub async fn monitor_health(
    state: State<'_, MonitorCommandState>,
) -> Result<bool, String> {
    // 检查存储是否可用
    let _has_data = state.storage.has_data().await?;

    // 检查配置是否可读
    let config_ok = {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        let _config = config_manager.get_config();
        true
    };

    // 检查证书是否可用
    let cert_ok = {
        let cert_manager = state.cert_manager.lock().map_err(|e| {
            format!("获取证书管理器锁失败: {}", e)
        })?;
        let status = cert_manager.cert_status();
        status.ca_cert_exists && status.ca_key_exists
    };

    Ok(config_ok && cert_ok)
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

    // 使用 time crate 计算时间戳
    let month_u8 = month as u8;
    let day_u8 = day.min(28) as u8;
    let date = time::Date::from_calendar_date(
        year,
        time::Month::try_from(month_u8).unwrap_or(time::Month::January),
        day_u8,
    );
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

// ============================================================================
// 命令注册辅助
// ============================================================================

/// 获取所有 Monitor 命令的 invoke_handler 宏参数
/// 用于在 lib.rs 中注册
#[macro_export]
macro_rules! register_monitor_commands {
    () => {
        tauri::generate_handler![
            // 数据查询命令
            crate::monitor::command::monitor_get_requests,
            crate::monitor::command::monitor_get_request,
            crate::monitor::command::monitor_get_response,
            crate::monitor::command::monitor_get_metrics,
            crate::monitor::command::monitor_get_mcp_calls,
            crate::monitor::command::monitor_get_stats_summary,
            crate::monitor::command::monitor_get_daily_records,
            crate::monitor::command::monitor_get_domain_stats,
            crate::monitor::command::monitor_get_all_models,
            crate::monitor::command::monitor_get_delta,
            // 数据操作命令
            crate::monitor::command::monitor_clear_data,
            crate::monitor::command::monitor_export_data,
            crate::monitor::command::monitor_backup,
            crate::monitor::command::monitor_get_backups,
            // 配置操作命令
            crate::monitor::command::monitor_get_config,
            crate::monitor::command::monitor_update_config,
            crate::monitor::command::monitor_get_domains,
            crate::monitor::command::monitor_add_domain,
            crate::monitor::command::monitor_remove_domain,
            crate::monitor::command::monitor_get_pricing,
            crate::monitor::command::monitor_update_pricing,
            // 证书操作命令
            crate::monitor::command::monitor_cert_status,
            crate::monitor::command::monitor_health,
        ]
    };
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_to_timestamp() {
        // 测试日期转换
        let ts = date_to_timestamp("2024-01-01", true);
        assert!(ts > 0);

        let ts_end = date_to_timestamp("2024-01-01", false);
        assert!(ts_end > ts);
        assert_eq!(ts_end - ts, 86_399_999); // 一天的毫秒数减 1
    }

    #[test]
    fn test_stats_summary_serialization() {
        let summary = StatsSummary {
            total_requests: 100,
            total_tokens: 10000,
            total_cost: 5.0,
            today_requests: 10,
            today_tokens: 1000,
            today_cost: 0.5,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalRequests\":100"));
        assert!(json.contains("\"todayCost\":0.5"));
    }

    #[test]
    fn test_cert_status_response_serialization() {
        let status = CertStatusResponse {
            ca_cert_exists: true,
            ca_key_exists: true,
            cert_dir_exists: true,
            ca_cert_path: "/path/to/ca.crt".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"caCertExists\":true"));
        assert!(json.contains("\"caCertPath\""));
    }
}
