// 启动管理、浏览器、热重载、端口管理命令

use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use super::get_monitor_ports;
use super::get_omoswitcher_dir;
use super::is_port_in_use;

/// 全局保存代理模式下启动的 opencode 子进程 PID
/// 窗口关闭时用于清理整个进程树
static OPENCODE_CHILD_PID: std::sync::LazyLock<Mutex<Option<u32>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

/// 清理代理模式启动的 opencode 子进程（整个进程树）
/// 在 Tauri 窗口关闭时调用
pub fn cleanup_opencode_child() {
    let pid = {
        let mut guard = OPENCODE_CHILD_PID.lock().unwrap_or_else(|e: std::sync::PoisonError<std::sync::MutexGuard<'_, Option<u32>>>| e.into_inner());
        guard.take()
    };

    if let Some(pid) = pid {
        tracing::info!("[启动] 清理 opencode 子进程树, PID={}", pid);
        #[cfg(target_os = "windows")]
        {
            // taskkill /T 终止整个进程树（包括 opencode 及其子进程）
            // /F 强制终止
            let pid_str = pid.to_string();
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid_str])
                .spawn();
        }
    }
}



/// 将字符串编码为 PowerShell -EncodedCommand 所需的 Base64（UTF-16LE + Base64）
/// 参见: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe
///   -EncodedCommand <Base64EncodedCommand>
///   Accepts a Base64-encoded string version of a command.
///   Use this parameter to submit commands to PowerShell that require complex quotation marks or curly braces.
#[cfg(target_os = "windows")]
fn encode_ps_command(command: &str) -> String {
    // PowerShell 要求 UTF-16LE 编码后再 Base64
    let utf16_bytes: Vec<u8> = command
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    base64_encode(&utf16_bytes)
}

/// 简易 Base64 编码（避免引入额外依赖）
#[cfg(target_os = "windows")]
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        result.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    result
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
        use std::os::windows::process::CommandExt;
        // CREATE_NEW_CONSOLE: 新进程拥有独立控制台窗口
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;

        let path = if working_path.is_empty() {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            working_path
        };

        // 构建 opencode 参数（热重载时带 --port）
        let mut opencode_args: Vec<String> = vec![];
        if hot_reload_enabled {
            tracing::info!("[启动] 热重载已启用，附加 --port {}", hot_reload_port);
            opencode_args.push("--port".to_string());
            opencode_args.push(hot_reload_port.to_string());
        }

        if proxy_enabled {
            // ====== 代理模式：子进程直接启动 opencode ======
            // 环境变量通过 .env() API 设置，不经过命令行字符串，零引号问题
            // 通过 cmd /C 解析 PATH 找到 opencode，CREATE_NEW_CONSOLE 创建独立控制台窗口
            tracing::info!(
                "[启动] 代理模式（子进程）: proxy={}, cert={}",
                proxy_url,
                if ca_cert_path.is_empty() { "(无)" } else { &ca_cert_path }
            );

            // 构建传给 cmd /C 的参数：opencode [--port N]
            let mut cmd_args = vec!["/C".to_string(), "opencode".to_string()];
            for arg in &opencode_args {
                cmd_args.push(arg.clone());
            }

            let mut cmd = Command::new("cmd");
            cmd.args(&cmd_args)
                .current_dir(&path)
                .env("HTTP_PROXY", &proxy_url)
                .env("HTTPS_PROXY", &proxy_url)
                .creation_flags(CREATE_NEW_CONSOLE);

            // 仅在证书路径有效时设置（避免传空值）
            if !ca_cert_path.is_empty() {
                cmd.env("NODE_EXTRA_CA_CERTS", &ca_cert_path);
            }

            cmd.spawn()
                .map(|child| {
                    let pid = child.id();
                    // 保存 PID，窗口关闭时清理
                    if let Ok(mut guard) = OPENCODE_CHILD_PID.lock() {
                        *guard = Some(pid);
                    }
                    tracing::info!("[启动] opencode 子进程启动成功（代理模式）PID={}", pid);
                })
                .map_err(|e| {
                    tracing::error!("[启动] opencode 子进程启动失败: {}", e);
                    format!("启动 opencode 失败: {}", e)
                })
        } else {
            // ====== 直连模式：PowerShell 窗口启动 ======
            // 用户可在独立终端中交互操作 opencode
            let ps_command = format!("cd '{}'; opencode {}", path, opencode_args.join(" "));
            let ps_command = ps_command.trim_end().to_string();
            tracing::info!("[启动] 直连模式（PowerShell）: {}", ps_command);

            let encoded = encode_ps_command(&ps_command);
            tracing::info!("[启动] EncodedCommand 长度: {} 字符", encoded.len());

            Command::new("powershell")
                .args(["-NoExit", "-EncodedCommand", &encoded])
                .creation_flags(CREATE_NEW_CONSOLE)
                .spawn()
                .map(|_| {
                    tracing::info!("[启动] opencode 进程启动成功（PowerShell 模式）");
                })
                .map_err(|e| {
                    tracing::error!("[启动] opencode 进程启动失败: {}", e);
                    format!("启动 opencode 失败: {}", e)
                })
        }
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

/// 会话状态信息（从 OpenCode Server 获取）
#[derive(Debug, serde::Deserialize)]
struct SessionStatusInfo {
    #[serde(rename = "type")]
    status_type: String,
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

/// 获取 OpenCode Server 上所有处于 busy 状态的会话 ID
/// 通过 GET /session/status 获取，返回 { sessionId: { type: "busy"|"idle" } }
#[tauri::command]
pub async fn get_active_sessions(port: u16) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("http://127.0.0.1:{}/session/status", port);
    tracing::info!("[热重载-恢复] GET {} 获取会话状态", url);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("[热重载-恢复] 获取会话状态失败: {}", e);
            format!("获取会话状态失败: {}", e)
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("[热重载-恢复] 获取会话状态返回 HTTP {}: {}", status,
            body.chars().take(200).collect::<String>());
        // 非致命：获取不到活跃会话不算错误，返回空列表
        return Ok(vec![]);
    }

    let status_map: std::collections::HashMap<String, SessionStatusInfo> = resp
        .json()
        .await
        .map_err(|e| {
            tracing::warn!("[热重载-恢复] 解析会话状态 JSON 失败: {}", e);
            format!("解析会话状态失败: {}", e)
        })?;

    // 筛选 busy 状态的 session
    let busy_sessions: Vec<String> = status_map
        .into_iter()
        .filter(|(_, info)| info.status_type == "busy")
        .map(|(id, _)| id)
        .collect();

    tracing::info!("[热重载-恢复] 发现 {} 个活跃会话", busy_sessions.len());
    Ok(busy_sessions)
}

/// 向指定会话发送恢复消息
/// 通过 POST /session/:id/message 异步发送恢复提示
#[tauri::command]
pub async fn resume_session(port: u16, session_id: String, message: String) -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 使用 prompt_async 端点异步发送，不等待 agent 完成回复
    let url = format!("http://127.0.0.1:{}/session/{}/prompt_async", port, session_id);
    tracing::info!("[热重载-恢复] POST {} 发送恢复消息", url);

    let body = serde_json::json!({
        "parts": [{ "type": "text", "text": message }]
    });

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("[热重载-恢复] 发送恢复消息失败 (session={}): {}", session_id, e);
            format!("发送恢复消息失败: {}", e)
        })?;

    let status = resp.status();
    if status.is_success() || status.as_u16() == 204 {
        tracing::info!("[热重载-恢复] 会话 {} 恢复消息发送成功", session_id);
        Ok(true)
    } else {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("[热重载-恢复] 会话 {} 恢复消息返回 HTTP {}: {}",
            session_id, status, body.chars().take(200).collect::<String>());
        Ok(false)
    }
}

/// 向 OpenCode TUI 发送 toast 提示
/// title: 标题（可选），message: 内容，variant: "info" | "success" | "warning" | "error"
async fn tui_toast(port: u16, title: &str, message: &str, variant: &str) {
    let url = format!("http://127.0.0.1:{}/tui/show-toast", port);
    let body = serde_json::json!({
        "title": title,
        "message": message,
        "variant": variant,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("[TUI提示] 创建客户端失败: {}", e);
            return;
        }
    };

    match client.post(&url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!("[TUI提示] toast 发送成功: {} - {}", title, message);
        }
        Ok(resp) => {
            tracing::warn!("[TUI提示] toast 返回 HTTP {}", resp.status());
        }
        Err(e) => {
            tracing::warn!("[TUI提示] toast 发送失败: {}", e);
        }
    }
}

/// 轮询等待所有会话变为 idle 状态
/// poll_interval: 轮询间隔（毫秒）
/// max_wait: 最大等待时间（毫秒），超时后返回 false
async fn wait_for_idle(port: u16, poll_interval_ms: u64, max_wait_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let max_duration = std::time::Duration::from_millis(max_wait_ms);
    let poll_interval = std::time::Duration::from_millis(poll_interval_ms);

    loop {
        let busy = get_active_sessions(port).await.unwrap_or_default();
        if busy.is_empty() {
            tracing::info!("[热重载-等待] 所有会话已空闲，可以安全热重载");
            return true;
        }

        let elapsed = start.elapsed();
        if elapsed >= max_duration {
            tracing::warn!(
                "[热重载-等待] 等待超时（{}ms），仍有 {} 个活跃会话: {:?}",
                max_wait_ms, busy.len(), busy
            );
            return false;
        }

        tracing::info!(
            "[热重载-等待] 等待会话空闲... 已等 {}ms / {}ms，活跃会话: {:?}",
            elapsed.as_millis(), max_wait_ms, busy
        );
        tokio::time::sleep(poll_interval).await;
    }
}

/// 智能热重载：检测活跃会话 → 等待空闲 → dispose + rebuild
/// 如果有活跃会话，轮询等待其完成（最长 60 秒），空闲后再执行热重载
/// 超时仍有活跃会话则跳过热重载，避免中断
#[tauri::command]
pub async fn dispose_and_resume(port: u16) -> Result<usize, String> {
    // ====== 阶段 1：检测活跃会话 ======
    let active_sessions = get_active_sessions(port).await.unwrap_or_default();

    if !active_sessions.is_empty() {
        tracing::info!(
            "[热重载-等待] 检测到 {} 个活跃会话: {:?}，等待空闲...",
            active_sessions.len(), active_sessions
        );

        // 在 TUI 中提示用户正在等待
        tui_toast(port, "OMOSwitcher", "配置已保存，等待会话空闲后热重载...", "info").await;

        // 等待所有会话变为 idle，每 3 秒轮询一次，最长等 60 秒
        let all_idle = wait_for_idle(port, 3000, 600000).await;

        if !all_idle {
            tracing::warn!("[热重载-等待] 等待超时，仍有活跃会话，跳过本次热重载");
            tui_toast(port, "OMOSwitcher", "热重载跳过：会话仍在工作中", "warning").await;
            return Err("OpenCode 仍有活跃会话，等待超时后跳过热重载".to_string());
        }
    } else {
        tracing::info!("[热重载-等待] 无活跃会话，立即执行热重载");
    }

    // ====== 阶段 2：执行 dispose + rebuild ======
    // 所有会话已空闲，安全执行 dispose
    tui_toast(port, "OMOSwitcher", "正在热重载配置...", "info").await;
    dispose_instance(port).await?;
    tracing::info!("[热重载-完成] 配置热重载成功");

    Ok(0)
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
