// Tauri Commands 桥接层
// 提供前端与后端配置文件交互的命令

use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{path::BaseDirectory, Manager};
use tokio::fs as async_fs;
use tokio::time::sleep;

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
#[tauri::command]
pub fn launch_opencode(
    working_path: String,
    proxy_enabled: bool,
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

        // 构建启动命令
        // --port 4096 让 OpenCode 启动 HTTP Server（热重载用）
        // --inspect 让 OpenCode 暴露调试端口（CDP 用）
        let ps_command = if proxy_enabled {
            // 启用代理模式，设置代理和证书路径
            format!(
                "$env:HTTP_PROXY='{}'; $env:HTTPS_PROXY='{}'; $env:NODE_EXTRA_CA_CERTS='{}'; cd '{}'; opencode --port 4096 --inspect",
                proxy_url, proxy_url, ca_cert_path, path
            )
        } else {
            // 直连模式，不设置代理
            format!("cd '{}'; opencode --port 4096 --inspect", path)
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

/// 清理 Monitor 服务所需的端口
fn cleanup_monitor_ports() -> Result<(), String> {
    let (web_port, proxy_port) = get_monitor_ports();

    for port in [web_port, proxy_port] {
        if is_port_in_use(port) {
            kill_port_process(port)?;
        }
    }

    Ok(())
}

// ============== Sidecar 监控服务管理 ==============

/// 全局状态存储 Monitor Sidecar 进程句柄
static MONITOR_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

/// 检查 Monitor Web API 是否已就绪
async fn is_monitor_api_ready(port: u16) -> bool {
    let url = format!("http://localhost:{}/api/health", port);

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// 清理已退出的 Monitor 进程句柄
fn cleanup_exited_monitor_process() -> Result<bool, String> {
    let mut process = MONITOR_PROCESS.lock().unwrap();

    let Some(child) = process.as_mut() else {
        return Ok(false);
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            println!("[Monitor] 检测到已退出的进程: {}", status);
            *process = None;
            Ok(false)
        }
        Ok(None) => Ok(true),
        Err(e) => {
            *process = None;
            Err(format!("检查 Monitor 进程状态失败: {}", e))
        }
    }
}

/// 等待 Monitor 服务真正就绪
async fn wait_for_monitor_ready(port: u16) -> Result<(), String> {
    const MAX_ATTEMPTS: usize = 40;
    const SLEEP_MS: u64 = 250;

    for _ in 0..MAX_ATTEMPTS {
        {
            let mut process = MONITOR_PROCESS.lock().unwrap();
            let Some(child) = process.as_mut() else {
                return Err("Monitor 进程句柄丢失".to_string());
            };

            match child.try_wait() {
                Ok(Some(status)) => {
                    *process = None;
                    return Err(format!("Monitor 进程启动后立即退出: {}", status));
                }
                Ok(None) => {}
                Err(e) => {
                    *process = None;
                    return Err(format!("检查 Monitor 进程状态失败: {}", e));
                }
            }
        }

        if is_monitor_api_ready(port).await {
            return Ok(());
        }

        sleep(Duration::from_millis(SLEEP_MS)).await;
    }

    Err(format!("等待 Monitor 服务就绪超时（端口 {}）", port))
}

/// Monitor 服务运行状态
#[derive(Serialize)]
pub struct MonitorStatus {
    /// 是否正在运行
    pub is_running: bool,
    /// 服务端口
    pub port: u16,
}

/// 启动 Monitor Sidecar 服务（使用内嵌的 Node.js 运行时）
#[tauri::command]
pub async fn start_monitor_service(
    app: tauri::AppHandle,
) -> Result<String, String> {
    // 检查是否已经在运行
    {
        let process_running = cleanup_exited_monitor_process()?;
        if process_running {
            return Ok("Monitor service already running".to_string());
        }
    }

    // 获取端口配置
    let (web_port, proxy_port) = get_monitor_ports();

    // ★ 启动前清理端口（解决残留进程问题）
    println!("[Monitor] 检查端口 {} 和 {}...", web_port, proxy_port);
    if let Err(e) = cleanup_monitor_ports() {
        println!("[Monitor] 警告: 端口清理失败: {}", e);
    }

    // ★ 使用 Tauri 2 推荐的 resolve 方法解析资源路径
    // 这在开发模式和生产模式下都能正确工作
    let node_exe_raw = app
        .path()
        .resolve("binaries/node/node.exe", BaseDirectory::Resource)
        .map_err(|e| format!("解析 Node.js 路径失败: {}", e))?;
    
    let monitor_dir_raw = app
        .path()
        .resolve("binaries/monitor-package", BaseDirectory::Resource)
        .map_err(|e| format!("解析 Monitor 目录路径失败: {}", e))?;

    // ★ 清理 Windows 扩展长度路径前缀 (\\?\)
    // Tauri 的 resolve() 在 Windows 上返回带 \\?\ 前缀的路径，
    // 但 Command::new() 和 current_dir() 不支持此前缀，会导致 ERROR_BAD_PATHNAME (267)
    let node_exe = normalize_path(node_exe_raw);
    let monitor_dir = normalize_path(monitor_dir_raw);

    println!("[Monitor] 使用内嵌 Node.js: {:?}", node_exe);
    println!("[Monitor] Monitor 目录: {:?}", monitor_dir);
    
    // 检查路径是否存在
    if !node_exe.exists() {
        return Err(format!("Node.js 可执行文件不存在: {:?}", node_exe));
    }
    if !monitor_dir.exists() {
        return Err(format!("Monitor 目录不存在: {:?}", monitor_dir));
    }

    let monitor_entry = monitor_dir.join("dist").join("index.js");
    if !monitor_entry.exists() {
        return Err(format!(
            "内嵌 Monitor 包不完整：缺少入口文件 {:?}。请先执行 monitor build/prepare 流程后重新打包。",
            monitor_entry
        ));
    }

    let monitor_node_modules = monitor_dir.join("node_modules");
    if !monitor_node_modules.exists() {
        return Err(format!(
            "内嵌 Monitor 包不完整：缺少依赖目录 {:?}。请先执行 monitor build/prepare 流程后重新打包。",
            monitor_node_modules
        ));
    }

    // 创建进程
    let mut cmd = Command::new(&node_exe);
    cmd.arg(&monitor_entry)
        .current_dir(&monitor_dir)
        .env("PORT", web_port.to_string())
        .env("PROXY_PORT", proxy_port.to_string())
        .env("HTTP_PROXY", "")
        .env("HTTPS_PROXY", "")
        .env("http_proxy", "")
        .env("https_proxy", "")
        .env("ALL_PROXY", "")
        .env("all_proxy", "")
        .env("NO_PROXY", "*");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NEW_CONSOLE);
    }

    // 启动进程
    let child = cmd
        .spawn()
        .map_err(|e| format!("启动 monitor 失败: {} (Node.js: {:?})", e, node_exe))?;

    let pid = child.id();
    println!("[Monitor] 已启动 (PID: {})", pid);

    // 存储进程句柄
    {
        let mut process = MONITOR_PROCESS.lock().unwrap();
        *process = Some(child);
    }

    println!("[Monitor] 等待 Web API 端口 {} 就绪...", web_port);
    if let Err(e) = wait_for_monitor_ready(web_port).await {
        let _ = stop_monitor_service();
        return Err(format!("启动 monitor 失败: {}", e));
    }

    println!("[Monitor] Web API 已就绪: http://localhost:{}", web_port);

    Ok(format!("Monitor service started (PID: {}, Port: {})", pid, web_port))
}

/// 停止 Monitor Sidecar 服务
#[tauri::command]
pub fn stop_monitor_service() -> Result<(), String> {
    let mut process = MONITOR_PROCESS.lock().unwrap();
    if let Some(mut child) = process.take() {
        child
            .kill()
            .map_err(|e| format!("停止 sidecar 失败: {}", e))?;
    }
    Ok(())
}

/// 获取 Monitor 服务运行状态
#[tauri::command]
pub fn get_monitor_status() -> Result<MonitorStatus, String> {
    // 获取端口（使用缓存，很快）
    let (web_port, _) = get_monitor_ports();
    let process_running = cleanup_exited_monitor_process()?;
    let is_running = process_running && is_port_in_use(web_port);
    
    Ok(MonitorStatus {
        is_running,
        port: web_port,
    })
}

/// 检查 CA 证书是否存在（通过调用 Monitor API）
#[tauri::command]
pub async fn check_ca_cert_exists() -> Result<bool, String> {
    let (web_port, _) = get_monitor_ports();
    let url = format!("http://localhost:{}/api/cert-status", web_port);
    
    // 使用 reqwest 调用 Monitor API
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求 Monitor API 失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Monitor API 返回错误: {}", response.status()));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    let exists = json["exists"].as_bool().unwrap_or(false);
    Ok(exists)
}

/// 获取 Monitor 端口配置
#[tauri::command]
pub fn get_monitor_ports_config() -> Result<(u16, u16), String> {
    let (web, proxy) = get_monitor_ports();
    Ok((web, proxy))
}

// ============== OpenCode 热重载 ==============

/// 热重载结果
#[derive(Serialize, Clone)]
pub struct HotReloadResult {
    /// 是否成功推送到 OpenCode
    pub success: bool,
    /// 可读的状态描述
    pub message: String,
    /// 是否因为 OpenCode 未运行而跳过（不是错误）
    pub skipped: bool,
}

/// 探测 OpenCode Server 是否在指定端口运行
async fn probe_opencode_server(client: &reqwest::Client, port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/config", port);
    match client.get(&url).send().await {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

/// 自动发现 OpenCode Server 端口
/// 依次尝试 4096/4097/4098
async fn discover_opencode_port(client: &reqwest::Client) -> Option<u16> {
    for port in [4096u16, 4097, 4098] {
        if probe_opencode_server(client, port).await {
            return Some(port);
        }
    }
    None
}

/// 调用 OpenCode Server PATCH /config 推送 agent 模型变更
/// 仅在无活跃对话时安全使用
async fn patch_opencode_config(
    client: &reqwest::Client,
    port: u16,
    agent_config: &serde_json::Value,
) -> Result<bool, String> {
    let url = format!("http://127.0.0.1:{}/config", port);
    let body = serde_json::json!({ "agent": agent_config });
    match client
        .patch(&url)
        .json(&body)
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                Ok(true)
            } else {
                let text = res.text().await.unwrap_or_default();
                Err(format!("PATCH /config 返回 {}: {}", status, text))
            }
        }
        Err(e) => Err(format!("PATCH /config 请求失败: {}", e)),
    }
}

/// 磀查是否有活跃的 session（正在生成中的）
async fn has_active_session(client: &reqwest::Client, port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/session/status", port);
    match client.get(&url).send().await {
        Ok(res) => {
            if !res.status().is_success() {
                return false;
            }
            // 解析响应，检查是否有活跃 session
            match res.json::<serde_json::Value>().await {
                Ok(status_map) => {
                    if let Some(obj) = status_map.as_object() {
                        // session status 中如果有 "active" 状态的 session，说明正在对话
                        return obj.values().any(|v| {
                            v.get("status")
                                .and_then(|s| s.as_str())
                                .map(|s| s == "running" || s == "generating" || s == "busy")
                                .unwrap_or(false)
                        });
                    }
                    false
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// 通过 CDP 执行 JavaScript 代码
async fn cdp_runtime_evaluate(
    client: &reqwest::Client,
    port: u16,
    expression: &str,
) -> Result<serde_json::Value, String> {
    // 1. 获取 CDP WebSocket URL
    let list_url = format!("http://127.0.0.1:{}/json/list", port);
    let list_response = client.get(&list_url).send().await
        .map_err(|e| format!("获取 CDP 目标列表失败: {}", e))?;
    
    let targets: serde_json::Value = list_response.json().await
        .map_err(|e| format!("解析 CDP 目标列表失败: {}", e))?;
    
    // 找到第一个有效的 WebSocket URL
    let ws_url = targets
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|t| t.get("webSocketDebuggerUrl"))
        .and_then(|url| url.as_str())
        .ok_or_else(|| "未找到 CDP WebSocket URL".to_string())?;
    
    // 2. 连接到 WebSocket
    let ws_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建 WebSocket 客户端失败: {}", e))?
    
    let ws_response = ws_client
        .get(ws_url)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Key-Protocol", "chat")
        .header("Sec-WebSocket-Version", "13")
        .send()
        .await
        .map_err(|e| format!("连接 CDP WebSocket 失败: {}", e))?;
    
    // 3. 发送 Runtime.evaluate 命令
    let request_id = "1";
    let cdp_message = serde_json::json!({
        "id": request_id,
        "method": "Runtime.evaluate",
        "params": {
            "expression": expression,
            "returnByValue": true
        }
    });
    
    // WebSocket 返回的是文本，需要手动解析
    let ws_response_text = ws_client
        .post(ws_url)
        .header("Content-Type", "application/json")
        .body(cdp_message.to_string())
        .send()
        .await
        .map_err(|e| format!("发送 CDP 命令失败: {}", e))?;
    
    let response_text = ws_response_text.text().await
        .map_err(|e| format!("读取 CDP 响应失败: {}", e))?;
    
    let response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析 CDP 响应失败: {}", e))?;
    
    // 检查是否有错误
    if let Some(error) = response.get("error") {
        return Err(format!("CDP 执行错误: {}", error));
    }
    
    Ok(response)
}

/// 在 OpenCode TUI 中显示 toast 通知
async fn show_opencode_toast(
    client: &reqwest::Client,
    port: u16,
    message: &str,
    variant: &str,
) -> bool {
    let url = format!("http://127.0.0.1:{}/tui/show-toast", port);
    let body = serde_json::json!({
        "message": message,
        "variant": variant,
        "duration": 5000
    });
    match client
        .post(&url)
        .json(&body)
        .send()
        .await
    {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

/// 热重载：将 oh-my-opencode.json 中的 agent/model 配置推送到运行中的 OpenCode
///
/// 策略：
/// - 无活跃对话 → PATCH /config 全局热重载（安全，不会崩溃）
/// - 有活跃对话 → 仅显示 toast 提示用户手动 /models 切换（避免崩溃）
///
/// 映射规则：
/// - oh-my-opencode.json 的 agents.{name}.model → opencode agent.{name}.model
/// - oh-my-opencode.json 的 categories.{name}.model → opencode agent.{name}.model
///
/// 尽力而为：OpenCode 未运行则跳过，不报错
#[tauri::command]
pub async fn hot_reload_opencode_config(config_json: String) -> Result<HotReloadResult, String> {
    // 解析前端传入的 OhMyOpenCodeConfig
    let config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("解析配置 JSON 失败: {}", e))?;

    // 构建 agent 映射
    let mut agent_map = serde_json::Map::new();

    // Agent 配置
    if let Some(agents) = config.get("agents").and_then(|v| v.as_object()) {
        for (name, val) in agents {
            if let Some(model) = val.get("model").and_then(|m| m.as_str()) {
                let mut agent_obj = serde_json::Map::new();
                agent_obj.insert("model".to_string(), serde_json::Value::String(model.to_string()));
                agent_map.insert(name.clone(), serde_json::Value::Object(agent_obj));
            }
        }
    }

    // Category 配置也映射为 agent 条目
    if let Some(categories) = config.get("categories").and_then(|v| v.as_object()) {
        for (name, val) in categories {
            if let Some(model) = val.get("model").and_then(|m| m.as_str()) {
                let mut agent_obj = serde_json::Map::new();
                agent_obj.insert("model".to_string(), serde_json::Value::String(model.to_string()));
                agent_map.insert(name.clone(), serde_json::Value::Object(agent_obj));
            }
        }
    }

    let agent_config = serde_json::Value::Object(agent_map);

    // 构建 HTTP 客户端（短超时，避免阻塞）
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 探测端口
    let port = match discover_opencode_port(&client).await {
        Some(p) => p,
        None => {
            return Ok(HotReloadResult {
                success: false,
                skipped: true,
                message: "OpenCode 未运行，配置将在下次启动时生效".to_string(),
            });
        }
    };

    // 检查是否有活跃对话
    let active = has_active_session(&client, port).await;
    if active {
        // 有活跃对话：不能用 PATCH /config（会导致崩溃），改为 toast 提示
        show_opencode_toast(
            &client,
            port,
            "OMOSwitcher 已更新模型配置，对话结束后将自动生效。或使用 /models 手动切换",
            "info",
        )
        .await;
        return Ok(HotReloadResult {
            success: true,
            skipped: false,
            message: "配置已保存，有活跃对话，将在对话结束后生效".to_string(),
        });
    }

    // 无活跃对话：安全地推送配置
    match patch_opencode_config(&client, port, &agent_config).await {
        Ok(true) => Ok(HotReloadResult {
            success: true,
            skipped: false,
            message: "配置已保存并热重载到 OpenCode".to_string(),
        }),
        Ok(false) => Ok(HotReloadResult {
            success: false,
            skipped: false,
            message: "热重载推送失败，配置将在下次启动时生效".to_string(),
        }),
        Err(e) => {
            println!("[HotReload] 热重载错误: {}", e);
            Ok(HotReloadResult {
                success: false,
                skipped: false,
                message: format!("热重载失败: {}", e),
            })
        }
    }
}
