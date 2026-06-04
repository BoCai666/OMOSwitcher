// 设置管理命令

use super::async_fs;
use super::get_settings_path;

/// 读取应用设置（异步）
#[tauri::command]
pub async fn read_settings() -> Result<String, String> {
    let path = get_settings_path()?;
    if !path.exists() {
        // 返回空的 JSON 对象
        return Ok("{}".to_string());
    }
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取设置失败: {}", e))
}

/// 写入应用设置（异步）
#[tauri::command]
pub async fn write_settings(content: String) -> Result<(), String> {
    let path = get_settings_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("写入设置失败: {}", e))
}
