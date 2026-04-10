// OMOSwitcher Tauri 应用库
// 包含应用初始化和命令定义

mod commands;
mod monitor;

// 导入 Manager trait 用于 try_state 方法
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        // 注册 Monitor 状态管理
        .manage(commands::MonitorCommandState::new())
        .invoke_handler(tauri::generate_handler![
            // 配置管理命令
            commands::read_config,
            commands::write_config,
            commands::read_opencode_config,
            // 预设管理命令
            commands::list_presets,
            commands::read_preset,
            commands::save_preset,
            commands::delete_preset,
            commands::read_all_presets,
            // 模型管理命令
            commands::read_models,
            commands::write_models,
            commands::read_models_with_fallback,
            // 设置管理命令
            commands::read_settings,
            commands::write_settings,
            // 启动命令
            commands::launch_opencode,
            // 端口管理命令
            commands::kill_port_process,
            // Sidecar 监控服务命令（重构后使用 Rust 代理）
            commands::start_monitor_service,
            commands::stop_monitor_service,
            commands::get_monitor_status,
            // 证书相关命令
            commands::check_ca_cert_exists,
            // Monitor 端口配置命令
            commands::get_monitor_ports_config,
            // 模型注册表命令
            commands::read_opencode_models_cache,
            commands::get_available_provider_ids,
            commands::get_custom_provider_ids,
            commands::add_custom_provider,
            commands::delete_custom_provider,
            // ========== Monitor 数据查询命令 ==========
            monitor::command::monitor_get_requests,
            monitor::command::monitor_get_request,
            monitor::command::monitor_get_response,
            monitor::command::monitor_get_metrics,
            monitor::command::monitor_get_mcp_calls,
            monitor::command::monitor_get_stats_summary,
            monitor::command::monitor_get_daily_records,
            monitor::command::monitor_get_domain_stats,
            monitor::command::monitor_get_all_models,
            monitor::command::monitor_get_delta,
            // ========== Monitor 数据操作命令 ==========
            monitor::command::monitor_clear_data,
            monitor::command::monitor_export_data,
            monitor::command::monitor_backup,
            monitor::command::monitor_get_backups,
            // ========== Monitor 配置操作命令 ==========
            monitor::command::monitor_get_config,
            monitor::command::monitor_update_config,
            monitor::command::monitor_get_domains,
            monitor::command::monitor_add_domain,
            monitor::command::monitor_remove_domain,
            monitor::command::monitor_get_pricing,
            monitor::command::monitor_update_pricing,
            // ========== Monitor 证书操作命令 ==========
            monitor::command::monitor_cert_status,
            monitor::command::monitor_health,
        ])
        .on_window_event(|window, event| {
            // 窗口关闭时确保停止代理
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                println!("[App] 窗口关闭，停止 Monitor 服务...");
                // 异步停止代理服务
                if let Some(state) = window.try_state::<commands::MonitorCommandState>() {
                    // 使用 tokio 运行时异步停止代理
                    let state_clone = state.inner().clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            state_clone.stop_proxy().await;
                        });
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}
