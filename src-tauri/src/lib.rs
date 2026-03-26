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
            // 模型管理命令
            commands::read_models,
            commands::write_models,
            // 设置管理命令
            commands::read_settings,
            commands::write_settings,
            // 启动命令
            commands::launch_opencode,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}
