// Tauri Commands 桥接层
// 提供前端与后端配置文件交互的命令

use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;

use crate::monitor::proxy::ProxyServer;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows 进程创建标志：创建新的独立控制台窗口
// CREATE_NEW_CONSOLE (0x00000010): 新进程拥有独立的控制台窗口
// 父进程关闭时子进程不受影响
#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

/// 清理 Windows 扩展长度路径前缀 (\\?\)
/// Tauri 的 path.resolve() 在 Windows 上会返回带 \\?\ 前缀的路径，
/// 但 Command::new() 和 CreateProcess API 不支持此前缀，会导致 ERROR_BAD_PATHNAME (267)。
/// 此函数在 Windows 上移除该前缀，其他平台直接返回原路径。
#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn normalize_path(path: PathBuf) -> PathBuf {
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
fn normalize_path(path: PathBuf) -> PathBuf {
    path
}

// 端口配置缓存（避免每次都读文件）
static MONITOR_PORTS_CACHE: OnceLock<(u16, u16)> = OnceLock::new();

/// 获取 OpenCode 配置目录路径（主配置文件所在目录）
/// 使用 ~/.config/opencode 目录（与 OhMyOpenCode 一致）
/// 跨平台支持: Windows/Linux/macOS
fn get_opencode_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config").join("opencode"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 OMOSwitcher 数据目录路径
/// 使用 ~/.config/omoswitcher 目录存储模型、预设、设置等
/// 跨平台支持: Windows/Linux/macOS
fn get_omoswitcher_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config").join("omoswitcher"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取主配置文件路径（位于 opencode 目录）
fn get_config_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("oh-my-opencode.json"))
}

/// 获取 OpenCode 配置文件路径（opencode.json）
fn get_opencode_config_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("opencode.json"))
}

/// 获取预设目录路径（位于 omoswitcher 目录）
fn get_presets_dir() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("presets"))
}

/// 获取模型列表文件路径（位于 omoswitcher 目录）
fn get_models_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("models.json"))
}

/// 获取应用设置文件路径（位于 omoswitcher 目录）
fn get_settings_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("settings.json"))
}

/// 获取 Monitor 配置文件路径
/// ~/.config/omoswitcher/monitor/config.jsonc
fn get_monitor_config_path() -> Result<PathBuf, String> {
    Ok(get_omoswitcher_dir()?.join("monitor").join("config.jsonc"))
}

/// 获取 OpenCode 模型注册表缓存路径
/// ~/.cache/opencode/models.json (Unix 风格，OpenCode 在所有平台都使用此路径)
fn get_opencode_models_cache_path() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".cache").join("opencode").join("models.json"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 OpenCode 认证数据路径
/// ~/.local/share/opencode/auth.json (Unix 风格，OpenCode 在所有平台都使用此路径)
fn get_opencode_auth_path() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".local").join("share").join("opencode").join("auth.json"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 获取 antigravity 认证数据路径
/// ~/.config/opencode/antigravity-accounts.json (may not exist)
fn get_antigravity_accounts_path() -> Result<PathBuf, String> {
    Ok(get_opencode_dir()?.join("antigravity-accounts.json"))
}

// Monitor 端口配置结构
#[derive(Debug, Deserialize, Default)]
struct MonitorPorts {
    web: u16,
    proxy: u16,
}

#[derive(Debug, Deserialize, Default)]
struct MonitorConfig {
    ports: Option<MonitorPorts>,
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
fn remove_jsonc_comments(input: &str) -> String {
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

/// 启动 opencode 命令行工具
/// working_path: 工作目录路径，为空则使用用户主目录
/// proxy_enabled: 是否启用监控代理
/// hot_reload_enabled: 是否启用模型热重载
/// hot_reload_port: 热重载端口（仅 hot_reload_enabled 为 true 时有效）
#[tauri::command]
pub fn launch_opencode(
    working_path: String,
    proxy_enabled: bool,
    hot_reload_enabled: bool,
    hot_reload_port: u16,
) -> Result<(), String> {
    // 从配置读取代理端口
    let (_, proxy_port) = get_monitor_ports();
    let proxy_url = format!("http://localhost:{}", proxy_port);
    
    // Monitor CA 证书固定路径
    let ca_cert_path = get_omoswitcher_dir()
        .map(|p| p.join("monitor").join("certs").join("ca.crt"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 CREATE_NEW_CONSOLE 标志启动独立的 PowerShell 窗口
        // 新进程拥有独立的控制台，父进程关闭时子进程不受影响
        let path = if working_path.is_empty() {
            // 默认使用用户主目录
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            working_path
        };

        // 构建 opencode 命令（热重载时带 --port 参数启动 server）
        let opencode_cmd = if hot_reload_enabled {
            format!("opencode --port {}", hot_reload_port)
        } else {
            "opencode".to_string()
        };

        // 构建启动命令
        let ps_command = if proxy_enabled {
            // 启用代理模式，设置代理和证书路径
            format!(
                "$env:HTTP_PROXY='{}'; $env:HTTPS_PROXY='{}'; $env:NODE_EXTRA_CA_CERTS='{}'; cd '{}'; {}",
                proxy_url, proxy_url, ca_cert_path, path, opencode_cmd
            )
        } else {
            // 直连模式，不设置代理
            format!("cd '{}'; {}", path, opencode_cmd)
        };

        // 使用 CREATE_NEW_CONSOLE 标志创建独立的控制台窗口
        Command::new("powershell")
            .args(["-NoExit", "-Command", &ps_command])
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动 opencode 失败: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 非Windows系统：使用终端启动
        let path = if working_path.is_empty() {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            working_path
        };

        // 构建启动命令
        let bash_command = if proxy_enabled {
            // 启用代理模式，设置代理和证书路径
            format!(
                "export HTTP_PROXY='{}' HTTPS_PROXY='{}' NODE_EXTRA_CA_CERTS='{}' && cd '{}' && opencode; exec bash",
                proxy_url, proxy_url, ca_cert_path, path
            )
        } else {
            // 直连模式，不设置代理
            format!("cd '{}' && opencode; exec bash", path)
        };

        // 尝试使用常见的终端模拟器
        let terminals = [
            (
                "gnome-terminal",
                vec!["--", "bash", "-c", &bash_command],
            ),
            (
                "konsole",
                vec!["-e", "bash", "-c", &bash_command],
            ),
            (
                "xterm",
                vec!["-e", "bash", "-c", &bash_command],
            ),
        ];

        for (terminal, args) in &terminals {
            if Command::new(terminal).args(args).spawn().is_ok() {
                return Ok(());
            }
        }

        Err("无法找到可用的终端模拟器".to_string())
    }
}

/// 列出所有预设文件名（异步）
#[tauri::command]
pub async fn list_presets() -> Result<Vec<String>, String> {
    let path = get_presets_dir()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut entries = async_fs::read_dir(&path)
        .await
        .map_err(|e| format!("读取预设列表失败: {}", e))?;
    
    let mut names = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取预设条目失败: {}", e))?
    {
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.replace(".json", ""));
        }
    }
    Ok(names)
}

/// 读取指定预设文件内容（异步）
#[tauri::command]
pub async fn read_preset(name: String) -> Result<String, String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取预设失败: {}", e))
}

/// 保存预设到文件（异步）
#[tauri::command]
pub async fn save_preset(name: String, content: String) -> Result<(), String> {
    let presets_dir = get_presets_dir()?;
    // 确保预设目录存在
    async_fs::create_dir_all(&presets_dir)
        .await
        .map_err(|e| format!("创建预设目录失败: {}", e))?;
    let path = presets_dir.join(format!("{}.json", name));
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("保存预设失败: {}", e))
}

/// 删除指定预设文件（异步）
#[tauri::command]
pub async fn delete_preset(name: String) -> Result<(), String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    async_fs::remove_file(&path)
        .await
        .map_err(|e| format!("删除预设失败: {}", e))
}

/// 读取模型配置文件（异步）
#[tauri::command]
pub async fn read_models() -> Result<String, String> {
    let path = get_models_path()?;
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取模型列表失败: {}", e))
}

/// 写入模型配置文件（异步）
#[tauri::command]
pub async fn write_models(content: String) -> Result<(), String> {
    let path = get_models_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("写入模型列表失败: {}", e))
}

/// 默认模型列表
const DEFAULT_MODELS_JSON: &str = r#"[
  {"id": "wuwen/glm-5", "name": "GLM-5", "provider": "wuwen"},
  {"id": "wuwen/minimax-m2.5", "name": "MiniMax M2.5", "provider": "wuwen"},
  {"id": "wuwen/minimax-m2.7", "name": "MiniMax M2.7", "provider": "wuwen"},
  {"id": "wuwen/kimi-k2.5", "name": "Kimi K2.5", "provider": "wuwen"}
]"#;

/// 读取模型列表（合并降级逻辑，单次 IPC 调用）
/// 优先级：models.json > opencode.json provider > 默认值
#[tauri::command]
pub async fn read_models_with_fallback() -> Result<String, String> {
    let models_path = get_models_path()?;
    
    // 1. 尝试读取 models.json
    if models_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&models_path).await {
            // 检查是否为有效非空 JSON 数组
            if !content.trim().is_empty() && content.trim() != "[]" {
                return Ok(content);
            }
        }
    }
    
    // 2. 尝试从 opencode.json 的 provider 字段读取
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            // 解析 opencode.json 并提取模型
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider") {
                    if let Some(provider_obj) = provider.as_object() {
                        let models: Vec<serde_json::Value> = provider_obj
                            .iter()
                            .flat_map(|(provider_name, provider_config)| {
                                if let Some(models_obj) = provider_config.get("models").and_then(|m| m.as_object()) {
                                    models_obj.iter().map(move |(model_id, model_config)| {
                                        serde_json::json!({
                                            "id": format!("{}/{}", provider_name, model_id),
                                            "name": model_config.get("name").and_then(|n| n.as_str()).unwrap_or(model_id),
                                            "provider": provider_name
                                        })
                                    }).collect::<Vec<_>>()
                                } else {
                                    vec![]
                                }
                            })
                            .collect();
                        
                        if !models.is_empty() {
                            return Ok(serde_json::to_string(&models).unwrap_or(DEFAULT_MODELS_JSON.to_string()));
                        }
                    }
                }
            }
        }
    }
    
    // 3. 返回默认模型列表
    Ok(DEFAULT_MODELS_JSON.to_string())
}

/// 读取 OpenCode 模型注册表缓存
/// 从 ~/.cache/opencode/models.json 读取全量供应商和模型信息
#[tauri::command]
pub async fn read_opencode_models_cache() -> Result<String, String> {
    let path = get_opencode_models_cache_path()?;
    if !path.exists() {
        return Err("模型注册表缓存不存在。请先运行一次 OpenCode 以生成缓存。".to_string());
    }
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取模型注册表缓存失败: {}", e))
}

/// 获取已配置（可用）的供应商 ID 列表
/// 交叉比对 opencode.json provider 字段、antigravity-accounts.json
#[tauri::command]
pub async fn get_available_provider_ids() -> Result<Vec<String>, String> {
    let mut available = Vec::new();
    
    // 1. 从 opencode.json 的 provider 字段提取有 apiKey 的供应商
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider").and_then(|p| p.as_object()) {
                    for (provider_id, config) in provider {
                        // 检查是否有 apiKey 或 options.apiKey
                        let has_api_key = config.get("apiKey").and_then(|v| v.as_str()).is_some()
                            || config.get("options")
                                .and_then(|o| o.get("apiKey"))
                                .and_then(|v| v.as_str()).is_some();
                        if has_api_key {
                            available.push(provider_id.clone());
                        }
                    }
                }
            }
        }
    }
    
    // 2. 从 antigravity-accounts.json 提取 Google/OAuth 连接的供应商
    let antigravity_path = get_antigravity_accounts_path()?;
    if antigravity_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&antigravity_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // 如果有 antigravity 账号，说明 google provider 是通过 OAuth 认证的
                if let Some(accounts) = json.get("accounts").and_then(|a| a.as_array()) {
                    if !accounts.is_empty() {
                        // antigravity 使用 google provider
                        if !available.contains(&"google".to_string()) {
                            available.push("google".to_string());
                        }
                        // antigravity 也可能启用其他 provider（通过 rateLimitResetTimes 字段判断）
                        if let Some(first_account) = accounts.first() {
                            if let Some(rate_limits) = first_account.get("rateLimitResetTimes") {
                                // rateLimitResetTimes 的 key 格式为 "providerId:modelId"
                                // 提取所有不重复的 providerId
                                if let Some(limits_obj) = rate_limits.as_object() {
                                    for key in limits_obj.keys() {
                                        if let Some(provider_id) = key.split(':').next() {
                                            let pid = provider_id.to_string();
                                            if !available.contains(&pid) {
                                                available.push(pid);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 3. 从 auth.json 提取（如果存在）
    let auth_path = get_opencode_auth_path()?;
    if auth_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&auth_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // auth.json 可能是数组或对象格式
                if let Some(obj) = json.as_object() {
                    for key in obj.keys() {
                        if !available.contains(key) {
                            available.push(key.clone());
                        }
                    }
                } else if let Some(arr) = json.as_array() {
                    for item in arr {
                        if let Some(pid) = item.get("provider").and_then(|v| v.as_str()) {
                            if !available.contains(&pid.to_string()) {
                                available.push(pid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    available.sort();
    Ok(available)
}

/// 获取 opencode.json 中手动配置的供应商 ID 列表
/// 仅检查 opencode.json 的 provider 字段，不包含 auth.json 或 antigravity-accounts.json
#[tauri::command]
pub async fn get_custom_provider_ids() -> Result<Vec<String>, String> {
    let mut custom = Vec::new();
    
    // 从 opencode.json 的 provider 字段提取有 apiKey 的供应商
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider").and_then(|p| p.as_object()) {
                    for (provider_id, config) in provider {
                        // 检查是否有 apiKey 或 options.apiKey
                        let has_api_key = config.get("apiKey").and_then(|v| v.as_str()).is_some()
                            || config.get("options")
                                .and_then(|o| o.get("apiKey"))
                                .and_then(|v| v.as_str()).is_some();
                        if has_api_key {
                            custom.push(provider_id.clone());
                        }
                    }
                }
            }
        }
    }
    
    custom.sort();
    Ok(custom)
}

/// 读取所有预设（合并命令，避免 N+1 问题）
/// 一次返回所有预设的完整数据
#[tauri::command]
pub async fn read_all_presets() -> Result<String, String> {
    let presets_dir = get_presets_dir()?;
    if !presets_dir.exists() {
        return Ok("[]".to_string());
    }
    
    let mut entries = async_fs::read_dir(&presets_dir)
        .await
        .map_err(|e| format!("读取预设目录失败: {}", e))?;
    
    let mut presets: Vec<serde_json::Value> = Vec::new();
    
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取预设条目失败: {}", e))?
    {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        
        if name.ends_with(".json") {
            let preset_name = name.trim_end_matches(".json");
            let path = entry.path();
            
            if let Ok(content) = async_fs::read_to_string(&path).await {
                if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 添加预设名称
                    if let Some(obj) = data.as_object_mut() {
                        obj.insert("name".to_string(), serde_json::json!(preset_name));
                    }
                    presets.push(data);
                }
            }
        }
    }
    
    // 按更新时间倒序排列
    presets.sort_by(|a, b| {
        let a_time = a.get("updatedAt").and_then(|t| t.as_str()).unwrap_or("");
        let b_time = b.get("updatedAt").and_then(|t| t.as_str()).unwrap_or("");
        b_time.cmp(a_time)
    });
    
    Ok(serde_json::to_string(&presets).unwrap_or("[]".to_string()))
}

// ============== 端口管理工具 ==============

/// 检测端口是否被占用
pub fn is_port_in_use(port: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

/// 终止占用指定端口的进程（跨平台）
/// 返回: true 表示成功终止了进程，false 表示端口未被占用
#[tauri::command]
pub fn kill_port_process(port: u16) -> Result<bool, String> {
    if !is_port_in_use(port) {
        return Ok(false); // 端口未被占用
    }

    println!("[PortManager] 端口 {} 被占用，尝试清理...", port);

    #[cfg(target_os = "windows")]
    {
        // CREATE_NO_WINDOW: 不创建控制台窗口
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        // Windows: 使用 netstat + taskkill
        // 1. 查找占用端口的 PID
        let output = Command::new("cmd")
            .args(["/C", &format!("netstat -ano | findstr :{}", port)])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("执行 netstat 失败: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut killed = false;

        // 解析 PID（格式："... 127.0.0.1:7100 ... PID"）
        for line in stdout.lines() {
            // 只处理 LISTENING 状态的行
            if !line.contains("LISTENING") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&pid_str) = parts.last() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // 排除 PID 0 (系统)
                    if pid > 0 {
                        // 2. 终止进程
                        let kill_result = Command::new("taskkill")
                            .args(["/F", "/PID", &pid.to_string()])
                            .creation_flags(CREATE_NO_WINDOW)
                            .output();

                        if let Ok(result) = kill_result {
                            if result.status.success() {
                                println!("[PortManager] 已终止进程 {} (占用端口 {})", pid, port);
                                killed = true;
                            }
                        }
                    }
                }
            }
        }

        if killed {
            // 等待端口释放
            std::thread::sleep(Duration::from_millis(500));
            Ok(true)
        } else {
            Err(format!("无法终止占用端口 {} 的进程", port))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: 使用 lsof + kill
        let output = Command::new("lsof")
            .args(["-t", "-i", &format!(":{}", port)])
            .output()
            .map_err(|e| format!("执行 lsof 失败: {}", e))?;

        let pids = String::from_utf8_lossy(&output.stdout);
        let mut killed = false;

        for pid_str in pids.lines() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let _ = Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .status();
                println!("[PortManager] 已终止进程 {} (占用端口 {})", pid, port);
                killed = true;
            }
        }

        if killed {
            std::thread::sleep(Duration::from_millis(500));
        }
        Ok(killed)
    }
}

// ============== Monitor 代理服务管理 ==============

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

/// 添加自定义 provider 到 opencode.json
/// 在 provider 字段中添加（或更新）指定 ID 的供应商配置，然后写回文件
/// config_json: 完整的 provider 配置 JSON 字符串，例如：
///   {"npm":"@ai-sdk/openai-compatible","name":"My Provider","options":{"apiKey":"sk-xxx","baseURL":"https://..."},"models":{...}}
#[tauri::command]
pub async fn add_custom_provider(provider_id: String, config_json: String) -> Result<(), String> {
    let provider_config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("Provider 配置 JSON 格式无效: {}", e))?;

    let path = get_opencode_config_path()?;

    // 如果 opencode.json 不存在，创建一个带 provider 字段的新文件
    if !path.exists() {
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        let mut json = serde_json::json!({});
        json["provider"] = serde_json::json!({});
        if let Some(obj) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
            obj.insert(provider_id, provider_config);
        }
        let output = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;
        async_fs::write(&path, output)
            .await
            .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;
        return Ok(());
    }

    // 读取现有配置
    let content = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

    let mut json = serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

    // 确保 provider 字段存在
    if json.get("provider").is_none() {
        json["provider"] = serde_json::json!({});
    }

    // 添加或更新指定 provider
    if let Some(provider) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
        provider.insert(provider_id, provider_config);
    }

    // 格式化写回（保持美观缩进）
    let output = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

    async_fs::write(&path, output)
        .await
        .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;

    Ok(())
}

/// 删除 opencode.json 中指定的自定义 provider
/// 从 provider 字段中移除指定 ID 的供应商配置，然后写回文件
#[tauri::command]
pub async fn delete_custom_provider(provider_id: String) -> Result<(), String> {
    let path = get_opencode_config_path()?;
    if !path.exists() {
        return Err("opencode.json 配置文件不存在".to_string());
    }

    let content = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

    let mut json = serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

    // 从 provider 对象中删除指定的 key
    if let Some(provider) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
        if provider.remove(&provider_id).is_none() {
            return Err(format!("provider \"{}\" 不存在于 opencode.json 中", provider_id));
        }
    } else {
        return Err("opencode.json 中没有 provider 字段".to_string());
    }

    // 格式化写回（保持美观缩进）
    let output = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

    async_fs::write(&path, output)
        .await
        .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;

    Ok(())
}

/// 检测 OpenCode Server 是否在指定端口运行
/// 通过 TCP 连接检测，2 秒超时
#[tauri::command]
pub async fn detect_opencode_server(port: u16) -> Result<bool, String> {
    let addr = format!("127.0.0.1:{}", port);
    let timeout = std::time::Duration::from_secs(2);
    
    tokio::time::timeout(
        timeout,
        tokio::net::TcpStream::connect(&addr)
    )
    .await
    .map(|result| result.is_ok())
    .map_err(|e| format!("检测 OpenCode Server 超时: {}", e))
}

/// 向 OpenCode Server 发送 PATCH /config/ 请求以触发热重载
/// config: 仅包含 { "agent": { [agentName]: { "model": "..." } } } 的 JSON
#[tauri::command]
pub async fn hot_reload_config(
    port: u16,
    config: serde_json::Value,
) -> Result<(), String> {
    // 验证 config body 仅包含 "agent" 顶层 key
    if config.as_object().map_or(true, |o| !o.contains_key("agent")) {
        return Err("热重载配置必须包含 \"agent\" 字段".to_string());
    }

    let url = format!("http://127.0.0.1:{}/config/", port);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    let response = client
        .patch(&url)
        .json(&config)
        .send()
        .await
        .map_err(|e| format!("热重载请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("热重载失败 (HTTP {}): {}", status, body));
    }
    
    Ok(())
}

/// 在系统默认浏览器中打开 URL
#[tauri::command]
 pub fn open_url_in_browser(url: String) -> Result<(), String> {
     #[cfg(target_os = "windows")]
     {
        // Windows: 使用 rundll32 打开 URL
        // 避免 cmd /C start 把 URL 中的 & 解释为命令分隔符
         Command::new("rundll32")
             .args(["url.dll,FileProtocolHandler", &url])
             .spawn()
             .map_err(|e| format!("打开浏览器失败: {}", e))?;
     }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    Ok(())
}

