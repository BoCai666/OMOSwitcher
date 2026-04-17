// Tauri Commands 桥接层
// 提供前端与后端配置文件交互的命令
// 已拆分为多个子模块，本文件负责共享辅助函数和 re-export

use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;
use tokio::fs as async_fs;

// 子模块声明
pub mod config;
pub mod launch;
pub mod model;
pub mod monitor_service;
pub mod preset;
pub mod settings;

// Re-export 所有子模块的命令函数，保持 commands::xxx 路径不变
pub use config::*;
pub use launch::*;
pub use model::*;
pub use monitor_service::*;
pub use preset::*;
pub use settings::*;

// ============== 共享辅助函数 ==============

/// 清理 Windows 扩展长度路径前缀 (\\?\)
/// Tauri 的 path.resolve() 在 Windows 上会返回带 \\?\ 前缀的路径，
/// 但 Command::new() 和 CreateProcess API 不支持此前缀，会导致 ERROR_BAD_PATHNAME (267)。
/// 此函数在 Windows 上移除该前缀，其他平台直接返回原路径。
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn normalize_path(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy().to_string();
    if path_str.starts_with(r"\\?\") {
        // 移除 \\?\ 前缀
        PathBuf::from(&path_str[4..])
    } else {
        path
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn normalize_path(path: PathBuf) -> PathBuf {
    path
}

// 端口配置缓存（避免每次都读文件）
static MONITOR_PORTS_CACHE: OnceLock<(u16, u16)> = OnceLock::new();

/// 获取 OpenCode 配置目录路径（主配置文件所在目录）
/// 使用 ~/.config/opencode 目录（与 OhMyOpenCode 一致）
/// 跨平台支持: Windows/Linux/macOS
pub fn get_opencode_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config").join("opencode"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 OMOSwitcher 数据目录路径
/// 使用 ~/.config/omoswitcher 目录存储模型、预设、设置等
/// 跨平台支持: Windows/Linux/macOS
pub fn get_omoswitcher_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config").join("omoswitcher"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取主配置文件路径（位于 opencode 目录）
pub fn get_config_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("oh-my-opencode.json"))
}

/// 获取 OpenCode 配置文件路径（opencode.json）
pub fn get_opencode_config_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("opencode.json"))
}

/// 获取预设目录路径（位于 omoswitcher 目录）
pub fn get_presets_dir() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("presets"))
}

/// 获取模型列表文件路径（位于 omoswitcher 目录）
pub fn get_models_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("models.json"))
}

/// 获取应用设置文件路径（位于 omoswitcher 目录）
pub fn get_settings_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("settings.json"))
}

/// 获取 Monitor 配置文件路径
/// ~/.config/omoswitcher/monitor/config.jsonc
pub fn get_monitor_config_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("monitor").join("config.jsonc"))
}

/// 获取 OpenCode 模型注册表缓存路径
/// ~/.cache/opencode/models.json (Unix 风格，OpenCode 在所有平台都使用此路径)
pub fn get_opencode_models_cache_path() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".cache").join("opencode").join("models.json"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 OpenCode 认证数据路径
/// ~/.local/share/opencode/auth.json (Unix 风格，OpenCode 在所有平台都使用此路径)
pub fn get_opencode_auth_path() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".local").join("share").join("opencode").join("auth.json"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 antigravity 认证数据路径
/// ~/.config/opencode/antigravity-accounts.json (may not exist)
pub fn get_antigravity_accounts_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("antigravity-accounts.json"))
}

// Monitor 端口配置结构
#[derive(Debug, Deserialize, Default)]
pub struct MonitorPorts {
    pub web: u16,
    pub proxy: u16,
}

#[derive(Debug, Deserialize, Default)]
pub struct MonitorConfig {
    pub ports: Option<MonitorPorts>,
}

/// 读取 Monitor 端口配置（带缓存）
/// 返回 (web_port, proxy_port)
pub fn get_monitor_ports() -> (u16, u16) {
    // 使用缓存，避免每次都读文件
    *MONITOR_PORTS_CACHE.get_or_init(|| {
        let default_ports = (7100, 7101);
        
        // 尝试读取 monitor/config.jsonc
        if let Ok(path) = get_monitor_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    // 移除 JSONC 注释（简单实现：移除 // 和 /* */ 注释）
                    let json_content = remove_jsonc_comments(&content);
                    if let Ok(config) = serde_json::from_str::<MonitorConfig>(&json_content) {
                        if let Some(ports) = config.ports {
                            return (ports.web, ports.proxy);
                        }
                    }
                }
            }
        }
        
        default_ports
    })
}

/// 移除 JSONC 注释（简单实现）
pub fn remove_jsonc_comments(input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut in_string = false;
    
    while i < chars.len() {
        let c = chars[i];
        
        // 处理字符串内的内容
        if in_string {
            result.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                result.push(chars[i]);
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        
        // 检查是否进入字符串
        if c == '"' {
            in_string = true;
            result.push(c);
            i += 1;
            continue;
        }
        
        // 检查行注释 //
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            // 跳过直到行尾
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        
        // 检查块注释 /* */
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            // 跳过直到 */
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // 跳过 */
            continue;
        }
        
        result.push(c);
        i += 1;
    }
    
    result
}

/// 检测端口是否被占用
pub fn is_port_in_use(port: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}
