// Tauri Commands 桥接层
// 提供前端与后端配置文件交互的命令

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

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
fn get_monitor_ports() -> (u16, u16) {
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

/// 读取应用设置
#[tauri::command]
pub fn read_settings() -> Result<String, String> {
    let path = get_settings_path()?;
    if !path.exists() {
        // 返回空的 JSON 对象
        return Ok("{}".to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("读取设置失败: {}", e))
}

/// 写入应用设置
#[tauri::command]
pub fn write_settings(content: String) -> Result<(), String> {
    let path = get_settings_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("写入设置失败: {}", e))
}

/// 读取主配置文件
#[tauri::command]
pub fn read_config() -> Result<String, String> {
    let path = get_config_path()?;
    fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {}", e))
}

/// 读取 OpenCode 配置文件 (opencode.json)
/// 用于从 provider 字段提取默认模型列表
#[tauri::command]
pub fn read_opencode_config() -> Result<String, String> {
    let path = get_opencode_config_path()?;
    fs::read_to_string(&path).map_err(|e| format!("读取 OpenCode 配置失败: {}", e))
}

/// 写入主配置文件
#[tauri::command]
pub fn write_config(content: String) -> Result<(), String> {
    let path = get_config_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("写入配置失败: {}", e))
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

/// 列出所有预设文件名
#[tauri::command]
pub fn list_presets() -> Result<Vec<String>, String> {
    let path = get_presets_dir()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.replace(".json", "")))
                .collect()
        })
        .map_err(|e| format!("读取预设列表失败: {}", e))
}

/// 读取指定预设文件内容
#[tauri::command]
pub fn read_preset(name: String) -> Result<String, String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    fs::read_to_string(&path).map_err(|e| format!("读取预设失败: {}", e))
}

/// 保存预设到文件
#[tauri::command]
pub fn save_preset(name: String, content: String) -> Result<(), String> {
    let presets_dir = get_presets_dir()?;
    // 确保预设目录存在
    fs::create_dir_all(&presets_dir).map_err(|e| format!("创建预设目录失败: {}", e))?;
    let path = presets_dir.join(format!("{}.json", name));
    fs::write(&path, content).map_err(|e| format!("保存预设失败: {}", e))
}

/// 删除指定预设文件
#[tauri::command]
pub fn delete_preset(name: String) -> Result<(), String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    fs::remove_file(&path).map_err(|e| format!("删除预设失败: {}", e))
}

/// 读取模型配置文件
#[tauri::command]
pub fn read_models() -> Result<String, String> {
    let path = get_models_path()?;
    fs::read_to_string(&path).map_err(|e| format!("读取模型列表失败: {}", e))
}

/// 写入模型配置文件
#[tauri::command]
pub fn write_models(content: String) -> Result<(), String> {
    let path = get_models_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("写入模型列表失败: {}", e))
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

    // 创建 sidecar 命令
    let mut sidecar = app
        .shell()
        .sidecar("monitor")
        .map_err(|e| format!("创建 sidecar 失败: {}", e))?;

    // 设置端口环境变量
    sidecar = sidecar
        .env("PORT", web_port.to_string())
        .env("PROXY_PORT", proxy_port.to_string());

    // 如果配置了企业代理 CA 证书，设置环境变量
    if !enterprise_ca_cert_path.is_empty() {
        sidecar = sidecar.env("ENTERPRISE_CA_CERT_PATH", &enterprise_ca_cert_path);
    }

    // 启动 sidecar 进程
    let (_rx, child) = sidecar
        .spawn()
        .map_err(|e| format!("启动 sidecar 失败: {}", e))?;

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
