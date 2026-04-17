// 启动管理、浏览器、热重载、端口管理命令

use std::process::Command;
use std::time::Duration;

use super::get_monitor_ports;
use super::get_omoswitcher_dir;
use super::is_port_in_use;

// Windows 进程创建标志：创建新的独立控制台窗口
// CREATE_NEW_CONSOLE (0x00000010): 新进程拥有独立的控制台窗口
// 父进程关闭时子进程不受影响
#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

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
    tracing::info!(
        "[启动] launch_opencode 参数: working_path={}, proxy_enabled={}, hot_reload_enabled={}, hot_reload_port={}",
        if working_path.is_empty() { "(默认主目录)" } else { &working_path },
        proxy_enabled, hot_reload_enabled, hot_reload_port
    );
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

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

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
            tracing::info!("[启动] 热重载已启用，构建 server 命令: opencode --port {}", hot_reload_port);
            format!("opencode --port {}", hot_reload_port)
        } else {
            tracing::info!("[启动] 热重载未启用，构建普通命令: opencode");
            "opencode".to_string()
        };

        // 构建启动命令
        let ps_command = if proxy_enabled {
            // 启用代理模式，设置代理和证书路径
            tracing::info!("[启动] 代理模式: proxy={}", proxy_url);
            format!(
                "$env:HTTP_PROXY='{}'; $env:HTTPS_PROXY='{}'; $env:NODE_EXTRA_CA_CERTS='{}'; cd '{}'; {}",
                proxy_url, proxy_url, ca_cert_path, path, opencode_cmd
            )
        } else {
            // 直连模式，不设置代理
            tracing::info!("[启动] 直连模式（无代理）");
            format!("cd '{}'; {}", path, opencode_cmd)
        };

        tracing::info!("[启动] 执行 PowerShell 命令: {}", ps_command);
        // 使用 CREATE_NEW_CONSOLE 标志创建独立的控制台窗口
        Command::new("powershell")
            .args(["-NoExit", "-Command", &ps_command])
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map(|_| {
                tracing::info!("[启动] opencode 进程启动成功");
            })
            .map_err(|e| {
                tracing::error!("[启动] opencode 进程启动失败: {}", e);
                format!("启动 opencode 失败: {}", e)
            })
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
        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

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

/// 检测 OpenCode Server 是否在指定端口运行
/// 通过 TCP 连接检测，2 秒超时
#[tauri::command]
pub async fn detect_opencode_server(port: u16) -> Result<bool, String> {
    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("[热重载] 检测 OpenCode Server: 尝试连接 {}", addr);
    let timeout = std::time::Duration::from_secs(2);
    
    let result = tokio::time::timeout(
        timeout,
        tokio::net::TcpStream::connect(&addr)
    )
    .await;
    
    match &result {
        Ok(Ok(_)) => tracing::info!("[热重载] Server 检测成功: {} 可连接", addr),
        Ok(Err(e)) => tracing::warn!("[热重载] Server 检测失败: {} 连接被拒: {}", addr, e),
        Err(_) => tracing::warn!("[热重载] Server 检测超时: {} 2秒内无响应", addr),
    }
    
    result
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
        tracing::error!("[热重载] 配置验证失败: 缺少 \"agent\" 字段, 实际 keys: {:?}", 
            config.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        return Err("热重载配置必须包含 \"agent\" 字段".to_string());
    }

    let url = format!("http://127.0.0.1:{}/config/", port);
    let body_str = config.to_string();
    tracing::info!("[热重载] 发送 PATCH 请求: url={}, body={}", url, 
        body_str.chars().take(500).collect::<String>());
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| {
            tracing::error!("[热重载] 创建 HTTP 客户端失败: {}", e);
            format!("创建 HTTP 客户端失败: {}", e)
        })?;
    
    let response = client
        .patch(&url)
        .json(&config)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("[热重载] PATCH 请求发送失败: {}", e);
            format!("热重载请求失败: {}", e)
        })?;
    
    let status = response.status();
    tracing::info!("[热重载] 收到响应: HTTP {}", status);
    
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::error!("[热重载] 热重载失败: HTTP {}, body={}", status, 
            body.chars().take(500).collect::<String>());
        return Err(format!("热重载失败 (HTTP {}): {}", status, body));
    }
    
    tracing::info!("[热重载] 配置推送成功");
    Ok(())
}

/// 触发 OpenCode 实例重建（dispose + lazy rebuild）
/// 先 POST /instance/dispose 销毁实例，再 GET /config/ 强制重建
/// 重建时 OhMyOpenCode 插件会重新调用 loadPluginConfig() 读取 oh-my-opencode.json
#[tauri::command]
pub async fn dispose_instance(port: u16) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 第一步：POST /instance/dispose — 销毁当前实例，清除缓存
    let dispose_url = format!("http://127.0.0.1:{}/instance/dispose", port);
    tracing::info!("[热重载] POST {} 触发实例销毁", dispose_url);

    let resp = client
        .post(&dispose_url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("[热重载] POST /instance/dispose 失败: {}", e);
            format!("实例销毁请求失败: {}", e)
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::error!("[热重载] 实例销毁失败: HTTP {}, body={}", status,
            body.chars().take(500).collect::<String>());
        return Err(format!("实例销毁失败 (HTTP {}): {}", status, body));
    }
    tracing::info!("[热重载] 实例销毁成功");

    // 短暂等待，确保 dispose 完成清理
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 第二步：GET /config/ — 触发实例 lazy rebuild，插件重新加载配置
    // 这一步非致命：dispose 已完成，即使重建触发失败，下次用户操作时也会自动重建
    let config_url = format!("http://127.0.0.1:{}/config/", port);
    tracing::info!("[热重载] GET {} 触发实例重建", config_url);

    match client.get(&config_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!("[热重载] 实例重建触发成功，插件配置已重载");
        }
        Ok(resp) => {
            tracing::warn!("[热重载] GET /config/ 返回 HTTP {}（重建可能延迟）", resp.status());
        }
        Err(e) => {
            tracing::warn!("[热重载] GET /config/ 触发重建失败（非致命）: {}", e);
        }
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
