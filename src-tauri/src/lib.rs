// OMOSwitcher Tauri 应用库
// 包含应用初始化和命令定义

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
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
            // Sidecar 监控服务命令
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
        ])
        .on_window_event(|_window, event| {
            // 窗口关闭时确保停止 sidecar（异步执行，不阻塞关闭）
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                println!("[App] 窗口关闭，停止 Monitor 服务...");
                // 只停止服务，端口清理交给下次启动时处理
                let _ = commands::stop_monitor_service();
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}
