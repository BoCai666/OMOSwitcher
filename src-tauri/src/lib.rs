// OMOSwitcher Tauri 应用库
// 包含应用初始化和命令定义

mod commands;
mod monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 Monitor 状态（存储、配置、证书）
    let monitor_state = monitor::command::MonitorCommandState::new()
        .expect("Monitor 初始化失败");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        // 注册 Monitor 状态管理
        .manage(monitor_state)
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
            // Monitor 代理服务命令
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
            // 窗口关闭时的处理
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                println!("[App] 窗口关闭...");
                // Monitor 服务会在应用退出时自动清理
                // 不再尝试在窗口关闭时执行异步操作，避免 runtime 已停止的问题
                let _ = window;
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}
