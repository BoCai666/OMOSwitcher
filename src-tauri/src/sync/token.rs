// Token 安全存储模块
// 提供 GitHub Token 的安全存储（OS Keychain）和同步元数据的文件存储

use std::path::PathBuf;

use serde_json;
use tauri::AppHandle;
use tauri_plugin_keyring::KeyringExt;
use tokio::fs as async_fs;

use super::types::SyncMetadata;

// ============================================================================
// 常量定义
// ============================================================================

/// Keychain 服务标识
const KEYRING_SERVICE: &str = "com.omoswitcher";

/// Keychain 用户标识（用于存储 GitHub Token）
const KEYRING_USER: &str = "github-token";

/// 同步元数据文件名
const SYNC_META_FILENAME: &str = "sync-meta.json";

/// 配置子目录名
const CONFIG_SUBDIR: &str = "omoswitcher";

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取同步元数据文件路径
/// Windows: %APPDATA%/omoswitcher/sync-meta.json
/// Unix: ~/.config/omoswitcher/sync-meta.json
fn get_sync_meta_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取系统配置目录".to_string())?;
    Ok(config_dir.join(CONFIG_SUBDIR).join(SYNC_META_FILENAME))
}

/// 确保配置目录存在（同步函数）
fn ensure_config_dir_exists() -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取系统配置目录".to_string())?;
    let omoswitcher_dir = config_dir.join(CONFIG_SUBDIR);
    
    if !omoswitcher_dir.exists() {
        std::fs::create_dir_all(&omoswitcher_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(())
}

// ============================================================================
// Token 存储 API (OS Keychain)
// ============================================================================

/// 保存 GitHub Token 到 OS Keychain
/// 
/// # 参数
/// - `app`: Tauri AppHandle，用于访问 keyring 插件
/// - `token`: GitHub Personal Access Token
/// 
/// # 返回
/// - `Ok(())`: 保存成功
/// - `Err(String)`: 保存失败，返回错误信息
pub fn save_token(app: &AppHandle, token: &str) -> Result<(), String> {
    app.keyring()
        .set_password(KEYRING_SERVICE, KEYRING_USER, token)
        .map_err(|e| format!("保存 Token 到 Keychain 失败: {}", e))
}

/// 从 OS Keychain 获取 GitHub Token
/// 
/// # 参数
/// - `app`: Tauri AppHandle，用于访问 keyring 插件
/// 
/// # 返回
/// - `Ok(Some(String))`: Token 存在且获取成功
/// - `Ok(None)`: Token 不存在
/// - `Err(String)`: 获取失败（非"不存在"的错误）
pub fn get_token(app: &AppHandle) -> Result<Option<String>, String> {
    app.keyring()
        .get_password(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("从 Keychain 读取 Token 失败: {}", e))
}

/// 从 OS Keychain 删除 GitHub Token
/// 
/// # 参数
/// - `app`: Tauri AppHandle，用于访问 keyring 插件
/// 
/// # 返回
/// - `Ok(())`: 删除成功或 Token 不存在
/// - `Err(String)`: 删除失败
pub fn delete_token(app: &AppHandle) -> Result<(), String> {
    app.keyring()
        .delete_password(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("从 Keychain 删除 Token 失败: {}", e))
}

// ============================================================================
// 同步元数据存储 API (文件系统)
// ============================================================================

/// 从文件系统读取同步元数据
/// 
/// 存储路径: ~/.config/omoswitcher/sync-meta.json
/// 
/// # 参数
/// - `_app`: Tauri AppHandle（预留用于未来扩展，当前未使用）
/// 
/// # 返回
/// - `Ok(SyncMetadata)`: 读取成功（文件不存在时返回默认值）
/// - `Err(String)`: 读取失败
pub async fn get_sync_meta(_app: &AppHandle) -> Result<SyncMetadata, String> {
    let path = get_sync_meta_path()?;
    
    // 文件不存在时返回默认元数据
    if !path.exists() {
        return Ok(SyncMetadata::default());
    }
    
    let content = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取同步元数据失败: {}", e))?;
    
    // 空文件也返回默认值
    if content.trim().is_empty() {
        return Ok(SyncMetadata::default());
    }
    
    serde_json::from_str(&content)
        .map_err(|e| format!("解析同步元数据失败: {}", e))
}

/// 保存同步元数据到文件系统
/// 
/// 存储路径: ~/.config/omoswitcher/sync-meta.json
/// 
/// # 参数
/// - `_app`: Tauri AppHandle（预留用于未来扩展，当前未使用）
/// - `meta`: 同步元数据
/// 
/// # 返回
/// - `Ok(())`: 保存成功
/// - `Err(String)`: 保存失败
pub async fn save_sync_meta(_app: &AppHandle, meta: &SyncMetadata) -> Result<(), String> {
    // 确保配置目录存在
    ensure_config_dir_exists()?;
    
    let path = get_sync_meta_path()?;
    
    // 确保父目录存在（双重保险）
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
    }
    
    let content = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("序列化同步元数据失败: {}", e))?;
    
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("写入同步元数据失败: {}", e))
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_meta_path() {
        let path = get_sync_meta_path().unwrap();
        assert!(path.to_string_lossy().contains("omoswitcher"));
        assert!(path.to_string_lossy().contains("sync-meta.json"));
    }
}
