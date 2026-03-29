// Tauri Commands 桥接层
// 提供前端与后端配置文件交互的命令

use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::fs as async_fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows 进程创建标志：创建新的独立控制台窗口
// CREATE_NEW_CONSOLE (0x00000010): 新进程拥有独立的控制台窗口
// 父进程关闭时子进程不受影响
#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

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

// Monitor 端口配置结构
#[derive(Debug, Deserialize, Default)]
struct MonitorPorts {
    web: u16,
    proxy: u16,
}

#[derive(Debug, Deserialize, Default)]
struct AppSettings {
    monitor_ports: Option<MonitorPorts>,
}

/// 读取 Monitor 端口配置
/// 返回 (web_port, proxy_port)
pub fn get_monitor_ports() -> (u16, u16) {
    // 默认端口
    let default_ports = (7100, 7101);
    
    // 尝试读取 settings.json
    if let Ok(path) = get_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    if let Some(ports) = settings.monitor_ports {
                        return (ports.web, ports.proxy);
                    }
                }
            }
        }
    }
    
    default_ports
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
/// proxy_ca_cert_path: 企业代理 CA 证书路径（可选）
#[tauri::command]
pub fn launch_opencode(
    working_path: String,
    proxy_enabled: bool,
    proxy_ca_cert_path: String,
) -> Result<(), String> {
    // 从配置读取代理端口
    let (_, proxy_port) = get_monitor_ports();
    let proxy_url = format!("http://localhost:{}", proxy_port);
    
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
        let ps_command = if proxy_enabled {
            // 启用代理模式
            if proxy_ca_cert_path.is_empty() {
                // 没有配置企业代理证书，只设置代理
                format!(
                    "$env:HTTP_PROXY='{}'; $env:HTTPS_PROXY='{}'; cd '{}'; opencode",
                    proxy_url, proxy_url, path
                )
            } else {
                // 配置了企业代理证书，设置代理和证书路径
                format!(
                    "$env:HTTP_PROXY='{}'; $env:HTTPS_PROXY='{}'; $env:NODE_EXTRA_CA_CERTS='{}'; cd '{}'; opencode",
                    proxy_url, proxy_url, proxy_ca_cert_path, path
                )
            }
        } else {
            // 直连模式，不设置代理
            format!("cd '{}'; opencode", path)
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
            // 启用代理模式
            if proxy_ca_cert_path.is_empty() {
                // 没有配置企业代理证书，只设置代理
                format!(
                    "export HTTP_PROXY='{}' HTTPS_PROXY='{}' && cd '{}' && opencode; exec bash",
                    proxy_url, proxy_url, path
                )
            } else {
                // 配置了企业代理证书，设置代理和证书路径
                format!(
                    "export HTTP_PROXY='{}' HTTPS_PROXY='{}' NODE_EXTRA_CA_CERTS='{}' && cd '{}' && opencode; exec bash",
                    proxy_url, proxy_url, proxy_ca_cert_path, path
                )
            }
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
        // Windows: 使用 netstat + taskkill
        // 1. 查找占用端口的 PID
        let output = Command::new("cmd")
            .args(["/C", &format!("netstat -ano | findstr :{}", port)])
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
static MONITOR_PROCESS: Mutex<Option<CommandChild>> = Mutex::new(None);

/// Monitor 服务运行状态
#[derive(Serialize)]
pub struct MonitorStatus {
    /// 是否正在运行
    pub is_running: bool,
    /// 服务端口
    pub port: u16,
}

/// 启动 Monitor Sidecar 服务
/// enterprise_ca_cert_path: 企业代理 CA 证书路径（可选）
#[tauri::command]
pub async fn start_monitor_service(
    app: tauri::AppHandle,
    enterprise_ca_cert_path: String,
) -> Result<String, String> {
    // 检查是否已经在运行
    {
        let process = MONITOR_PROCESS.lock().unwrap();
        if process.is_some() {
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

    // 创建 sidecar 命令
    let mut sidecar = app
        .shell()
        .sidecar("monitor")
        .map_err(|e| format!("创建 sidecar 失败: {}", e))?;

    // 设置端口环境变量
    sidecar = sidecar
        .env("PORT", web_port.to_string())
        .env("PROXY_PORT", proxy_port.to_string());

    // 清除代理环境变量，避免 http-proxy 尝试连接上游代理
    // Monitor 作为透明代理，不应链式代理
    sidecar = sidecar
        .env("HTTP_PROXY", "")
        .env("HTTPS_PROXY", "")
        .env("http_proxy", "")
        .env("https_proxy", "")
        .env("ALL_PROXY", "")
        .env("all_proxy", "")
        .env("NO_PROXY", "*");  // 禁用所有上游代理

    // 如果配置了企业代理 CA 证书，设置环境变量
    if !enterprise_ca_cert_path.is_empty() {
        sidecar = sidecar.env("ENTERPRISE_CA_CERT_PATH", &enterprise_ca_cert_path);
    }

    // 启动 sidecar 进程
    let (mut rx, child) = sidecar
        .spawn()
        .map_err(|e| format!("启动 sidecar 失败: {}", e))?;

    // 获取 app handle 用于在异步任务中发送事件
    let app_handle = app.clone();

    // 在后台异步读取 sidecar 输出并打印到终端
    tauri::async_runtime::spawn(async move {
        println!("[Monitor Sidecar] 开始监听输出...");
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let output = String::from_utf8_lossy(&line);
                    println!("[Monitor] {}", output.trim());
                    // 同时发送事件到前端（可选）
                    let _ = app_handle.emit("monitor:log", output.trim().to_string());
                }
                CommandEvent::Stderr(line) => {
                    let output = String::from_utf8_lossy(&line);
                    eprintln!("[Monitor ERR] {}", output.trim());
                    let _ = app_handle.emit("monitor:error", output.trim().to_string());
                }
                CommandEvent::Error(err) => {
                    eprintln!("[Monitor ERROR] {}", err);
                    let _ = app_handle.emit("monitor:error", err);
                }
                CommandEvent::Terminated(payload) => {
                    println!("[Monitor] 进程已终止: code={:?}, signal={:?}", payload.code, payload.signal);
                    let _ = app_handle.emit("monitor:terminated", payload);
                    break;
                }
                _ => {}
            }
        }
    });

    // 存储进程句柄
    {
        let mut process = MONITOR_PROCESS.lock().unwrap();
        *process = Some(child);
    }

    Ok("Monitor service started".to_string())
}

/// 停止 Monitor Sidecar 服务
#[tauri::command]
pub fn stop_monitor_service() -> Result<(), String> {
    let mut process = MONITOR_PROCESS.lock().unwrap();
    if let Some(child) = process.take() {
        child
            .kill()
            .map_err(|e| format!("停止 sidecar 失败: {}", e))?;
    }
    Ok(())
}

/// 获取 Monitor 服务运行状态
#[tauri::command]
pub fn get_monitor_status() -> Result<MonitorStatus, String> {
    let process = MONITOR_PROCESS.lock().unwrap();
    let (web_port, _) = get_monitor_ports();
    Ok(MonitorStatus {
        is_running: process.is_some(),
        port: web_port,
    })
}
