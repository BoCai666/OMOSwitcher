// 配置管理命令

use super::async_fs;
use super::get_config_path;
use super::get_opencode_config_path;

/// 读取主配置文件（异步）
#[tauri::command]
pub async fn read_config() -> Result<String, String> {
    let path = get_config_path()?;
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取配置失败: {}", e))
}

/// 读取 OpenCode 配置文件 (opencode.json)（异步）
/// 用于从 provider 字段提取默认模型列表
#[tauri::command]
pub async fn read_opencode_config() -> Result<String, String> {
    let path = get_opencode_config_path()?;
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取 OpenCode 配置失败: {}", e))
}

/// 写入主配置文件（异步）
#[tauri::command]
pub async fn write_config(content: String) -> Result<(), String> {
    let path = get_config_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("写入配置失败: {}", e))
}
