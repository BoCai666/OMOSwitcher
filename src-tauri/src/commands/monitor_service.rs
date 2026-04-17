// Monitor 代理服务管理命令

use std::net::SocketAddr;

use serde::Serialize;

use super::get_monitor_ports;
use super::get_omoswitcher_dir;
use super::is_port_in_use;
use super::kill_port_process;

use crate::monitor::proxy::ProxyServer;

/// Monitor 服务运行状态
#[derive(Serialize)]
pub struct MonitorStatus {
    /// 是否正在运行
    pub is_running: bool,
    /// 服务端口
    pub port: u16,
}

/// 启动 Monitor 服务（使用 Rust 代理服务器，完整监控模式）
/// 创建带状态的 MonitorHandler，捕获 LLM API 调用
#[tauri::command]
pub async fn start_monitor_service(
    state: tauri::State<'_, crate::monitor::command::MonitorCommandState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 检查是否已经在运行
    {
        let proxy = state.proxy.lock().await;
        if let Some(ref p) = *proxy {
            if p.is_running() {
                return Ok("Monitor service already running".to_string());
            }
        }
    }

    // 获取端口配置
    let (_, proxy_port) = get_monitor_ports();
    let addr: SocketAddr = format!("127.0.0.1:{}", proxy_port)
        .parse()
        .map_err(|e| format!("解析代理地址失败: {}", e))?;

    // 启动前清理端口（解决残留进程问题）
    tracing::info!("[Monitor] 检查端口 {}...", proxy_port);
    if is_port_in_use(proxy_port) {
        if let Err(e) = kill_port_process(proxy_port) {
            tracing::warn!("[Monitor] 端口清理失败: {}", e);
        }
    }

    // 获取 CA Authority（从 CertManager）
    let ca = {
        let cert_manager = state.cert_manager.lock().map_err(|e| {
            format!("获取证书管理器锁失败: {}", e)
        })?;
        cert_manager.create_rcgen_authority().map_err(|e| {
            format!("创建 CA Authority 失败: {}", e)
        })?
    };

    // 获取当前配置
    let config = {
        let config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        config_manager.get_config()
    };

    // 创建 MonitorState
    let monitor_state = crate::monitor::handler::MonitorState::new(
        state.storage.clone(),
        config,
    );

    // 创建 MonitorHandler（带 AppHandle 用于发射事件）
    let handler = crate::monitor::handler::MonitorHandler::new(
        std::sync::Arc::new(monitor_state),
        Some(app_handle),
    );

    // 创建并启动 Rust 代理服务器
    tracing::info!("[Monitor] 启动 Rust 代理服务器 (完整模式, 端口 {})...", proxy_port);
    let proxy_server = ProxyServer::with_addr(addr)
        .map_err(|e| format!("创建代理服务器失败: {}", e))?;
    
    proxy_server.start(handler, ca).await
        .map_err(|e| format!("启动代理服务器失败: {}", e))?;

    // 保存到 state
    {
        let mut proxy = state.proxy.lock().await;
        *proxy = Some(proxy_server);
    }

    // 启动定时清理任务
    {
        use crate::monitor::tasks::{DataCleanupTask, CleanupConfig, DatabaseBackup, BackupConfig};
        
        let cleanup_task = DataCleanupTask::new(
            state.storage.clone(),
            CleanupConfig::default(),
            &state.data_dir,
        );
        
        // 初始化并启动清理任务
        let cleanup_task = std::sync::Arc::new(cleanup_task);
        if let Err(e) = cleanup_task.initialize().await {
            tracing::error!("[Monitor] 初始化清理任务失败: {}", e);
        } else {
            tracing::info!("[Monitor] 启动定时清理任务");
            std::sync::Arc::clone(&cleanup_task).start_scheduled_cleanup();
        }
        
        // 启动定时备份任务
        let backup = DatabaseBackup::new(
            state.db_path.clone(),
            &state.data_dir,
            BackupConfig::default(),
        );
        
        let backup = std::sync::Arc::new(backup);
        if let Err(e) = backup.initialize().await {
            tracing::error!("[Monitor] 初始化备份任务失败: {}", e);
        } else {
            tracing::info!("[Monitor] 启动定时备份任务");
            std::sync::Arc::clone(&backup).start_scheduled_backup();
        }
    }

    // 启动配置文件监听
    {
        let mut config_manager = state.config_manager.lock().map_err(|e| {
            format!("获取配置管理器锁失败: {}", e)
        })?;
        
        if let Err(e) = config_manager.start_watching() {
            tracing::warn!("[Monitor] 启动配置文件监听失败: {}", e);
        } else {
            tracing::info!("[Monitor] 配置文件监听已启动");
        }
    }

    tracing::info!("[Monitor] Rust 代理服务器已启动（完整监控模式）: http://localhost:{}", proxy_port);

    Ok(format!("Monitor service started (Full Mode, Port: {})", proxy_port))
}

/// 停止 Monitor 服务
#[tauri::command]
pub async fn stop_monitor_service(
    state: tauri::State<'_, crate::monitor::command::MonitorCommandState>,
) -> Result<(), String> {
    state.stop_proxy().await;
    Ok(())
}

/// 获取 Monitor 服务运行状态
#[tauri::command]
pub async fn get_monitor_status(
    state: tauri::State<'_, crate::monitor::command::MonitorCommandState>,
) -> Result<MonitorStatus, String> {
    // 获取端口（使用缓存）
    let (_, proxy_port) = get_monitor_ports();
    
    let proxy = state.proxy.lock().await;
    let is_running = proxy.as_ref().map(|p| p.is_running()).unwrap_or(false);
    
    Ok(MonitorStatus {
        is_running,
        port: proxy_port,
    })
}

/// 检查 CA 证书是否存在（直接检查文件系统）
#[tauri::command]
pub async fn check_ca_cert_exists() -> Result<bool, String> {
    // 获取 CA 证书路径
    let ca_cert_path = get_omoswitcher_dir()
        .map(|p| p.join("monitor").join("certs").join("ca.crt"))?;
    
    // 直接检查文件是否存在
    Ok(ca_cert_path.exists())
}

/// 获取 Monitor 端口配置
#[tauri::command]
pub fn get_monitor_ports_config() -> Result<(u16, u16), String> {
    let (web, proxy) = get_monitor_ports();
    Ok((web, proxy))
}
