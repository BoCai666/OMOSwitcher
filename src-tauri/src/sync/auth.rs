// 认证模块
// GitHub OAuth Web Flow (PKCE) + Device Flow + PAT 三通道认证
// Web Flow 为首选方案，用户体验最好（浏览器自动打开，无需手动输入）

use crate::sync::types::*;
use crate::sync::token;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

/// GitHub OAuth App Client Secret
/// GitHub OAuth Apps 的 token 交换端点强制要求 client_secret（即使使用 PKCE）
/// 通过编译时环境变量注入，避免明文硬编码在源码中
/// 
/// 必须设置 OAUTH_CLIENT_SECRET 环境变量，否则编译失败
/// 构建命令: OAUTH_CLIENT_SECRET=xxx cargo build
const OAUTH_CLIENT_SECRET: &str = {
    match option_env!("OAUTH_CLIENT_SECRET") {
        Some(s) if !s.is_empty() => s,
        _ => {
            panic!("编译失败: 必须设置 OAUTH_CLIENT_SECRET 环境变量");
        }
    }
};

/// OAuth 回调固定端口
/// 避免 GitHub 不支持随机端口的 loopback 重定向
const OAUTH_CALLBACK_PORT: u16 = 12345;
const GITHUB_CLIENT_ID: &str = "Ov23liH5k1PnrueaLiYb";

const GITHUB_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_USER_URL: &str = "https://api.github.com/user";

/// OAuth 回调成功后返回给浏览器的 HTML 页面
const CALLBACK_SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>OMOSwitcher</title></head>
<body style="display:flex;justify-content:center;align-items:center;height:100vh;margin:0;font-family:system-ui,sans-serif;background:#1a1a2e;color:#eee">
<div style="text-align:center">
<h2 style="color:#00d4ff">✓ 授权成功</h2>
<p>正在返回 OMOSwitcher，可关闭此页面</p>
</div>
</body>
</html>"#;

/// OAuth 回调失败时返回给浏览器的 HTML 页面
const CALLBACK_ERROR_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>OMOSwitcher</title></head>
<body style="display:flex;justify-content:center;align-items:center;height:100vh;margin:0;font-family:system-ui,sans-serif;background:#1a1a2e;color:#eee">
<div style="text-align:center">
<h2 style="color:#ff4757">✗ 授权失败</h2>
<p id="msg"></p>
<script>const p=new URLSearchParams(location.search);document.getElementById('msg').textContent=p.get('error_description')||p.get('error')||'未知错误';</script>
</div>
</body>
</html>"#;

/// 构建带统一 header 的 reqwest Client
fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("OMOSwitcher")
        .build()
        .unwrap_or_default()
}

// ============================================================================
// PKCE 工具函数
// ============================================================================

/// 生成 PKCE code_verifier 和 code_challenge (S256)
///
/// code_verifier: 43 个随机字母数字字符
/// code_challenge: BASE64URL(SHA256(code_verifier))
fn generate_pkce_pair() -> (String, String) {
    let verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(43)
        .map(char::from)
        .collect();

    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    (verifier, challenge)
}

/// 生成指定长度的随机字符串（用于 CSRF state 参数）
fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

// ============================================================================
// OAuth Web Flow (PKCE)
// ============================================================================

/// 准备 OAuth Web Flow 参数
///
/// 生成 PKCE code_verifier + code_challenge 和 CSRF state，
/// 启动本地回调 HTTP 服务器（固定端口），返回 (auth_url, OAuthSession, listener)
pub async fn prepare_oauth_flow() -> Result<(String, OAuthSession, tokio::net::TcpListener), String> {
    let (code_verifier, code_challenge) = generate_pkce_pair();
    let state = generate_random_string(32);

    // 启动本地回调服务器（固定端口，与 GitHub OAuth App 注册的 callback URL 匹配）
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", OAUTH_CALLBACK_PORT))
        .await
        .map_err(|e| format!("启动回调服务器失败（端口 {} 被占用？）: {}", OAUTH_CALLBACK_PORT, e))?;
    let port = listener.local_addr().unwrap().port();
    let redirect_uri = format!("http://127.0.0.1:{}/", port);

    // 构建 GitHub 授权 URL
    // 注意：scope 参数必须正确编码，确保 GitHub 能识别
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        GITHUB_AUTHORIZE_URL,
        GITHUB_CLIENT_ID,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode("gist"),
        state,
        code_challenge,
    );

    // 调试：输出生成的授权 URL 以便排查 scope 问题
    tracing::info!("[OAuth] 授权 URL: {}", auth_url);
    tracing::info!("[OAuth] scope 参数: gist");

    Ok((
        auth_url,
        OAuthSession {
            code_verifier,
            state,
            redirect_uri,
        },
        listener,
    ))
}

/// 等待浏览器回调并提取授权码
///
/// 启动后台任务监听本地 HTTP 请求，解析回调 URL 中的 code 参数。
/// 超时 5 分钟自动关闭。
pub async fn wait_for_callback(listener: tokio::net::TcpListener) -> Result<String, String> {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        accept_callback(listener),
    )
    .await;

    match result {
        Ok(Ok(code)) => Ok(code),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("OAuth 授权超时（5 分钟），请重试".to_string()),
    }
}

/// 接受回调请求，解析 code 参数
async fn accept_callback(listener: tokio::net::TcpListener) -> Result<String, String> {
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("接受回调连接失败: {}", e))?;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut buf_reader = tokio::io::BufReader::new(reader);
        let mut request_line = String::new();

        tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut request_line)
            .await
            .map_err(|e| format!("读取回调请求失败: {}", e))?;

        // 解析 HTTP 请求行：GET /?code=xxx&state=yyy HTTP/1.1
        let url_part = request_line.split_whitespace().nth(1).unwrap_or("");

        // 检查是否是回调请求（根路径带 query 参数）
        if !url_part.starts_with("/?") && url_part != "/" {
            // 不是回调请求，跳过
            let _ = tokio::io::AsyncWriteExt::write(
                &mut writer,
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n",
            )
            .await;
            continue;
        }

        // 解析 query 参数
        let query = url_part.split('?').nth(1).unwrap_or("");

        // 检查是否有 error 参数
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "error" {
                    let body = CALLBACK_ERROR_HTML
                        .replace("{error}", value)
                        .replace("{error_description}", "");
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = tokio::io::AsyncWriteExt::write(&mut writer, response.as_bytes()).await;
                    return Err(format!("GitHub 授权被拒绝: {}", urlencoding::decode(value).unwrap_or_default()));
                }
            }
        }

        // 提取 code 参数
        let mut code = None;
        let mut callback_state = None;
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                match key {
                    "code" => code = Some(value.to_string()),
                    "state" => callback_state = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        let code = match code {
            Some(c) => c,
            None => {
                let _ = tokio::io::AsyncWriteExt::write(
                    &mut writer,
                    b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n",
                )
                .await;
                return Err("回调 URL 中缺少 code 参数".to_string());
            }
        };

        // 返回成功 HTML 给浏览器
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            CALLBACK_SUCCESS_HTML.len(),
            CALLBACK_SUCCESS_HTML
        );
        let _ = tokio::io::AsyncWriteExt::write(&mut writer, response.as_bytes()).await;

        // 注意：state 验证由调用方完成，这里只传递回去
        // 将 state 附加到返回值中，格式 "code||state"
        if let Some(s) = callback_state {
            return Ok(format!("{}||{}", code, s));
        }
        return Ok(code);
    }
}

/// 用授权码 + PKCE code_verifier 换取 Access Token
pub async fn exchange_code_for_token(
    code: &str,
    code_verifier: &str,
) -> Result<String, String> {
    let client = build_http_client();

    let params = [
        ("client_id", GITHUB_CLIENT_ID),
        ("client_secret", OAUTH_CLIENT_SECRET),
        ("code", code),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(GITHUB_ACCESS_TOKEN_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("换取 Access Token 失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("[OAuth] token 交换失败 HTTP {}: {}", status, body);
        return Err(format!(
            "换取 Access Token 失败 (HTTP {}): {}",
            status, body
        ));
    }

    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("读取 Token 响应失败: {}", e))?;
    tracing::info!("[OAuth] token 交换响应: {}", resp_text.chars().take(300).collect::<String>());

    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| format!("解析 Token 响应失败: {}", e))?;

    // 检查错误
    if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        let desc = json
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("GitHub Token 交换失败 ({}): {}", error, desc));
    }

    json.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Token 响应中缺少 access_token".to_string())
}

// ============================================================================
// Device Flow（保留作为备用）
// ============================================================================

/// 启动 Device Flow 认证
///
/// 向 GitHub 请求 device code，返回用户需要访问的验证 URL 和 user_code
pub async fn start_device_flow() -> Result<DeviceCodeResponse, String> {
    let client = build_http_client();

    let params = [("client_id", GITHUB_CLIENT_ID), ("scope", "gist")];

    let resp = client
        .post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("请求 GitHub Device Code 失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "请求 GitHub Device Code 失败 (HTTP {}): {}",
            status, body
        ));
    }

    resp.json::<DeviceCodeResponse>()
        .await
        .map_err(|e| format!("解析 Device Code 响应失败: {}", e))
}

/// 轮询 Device Token
///
/// 以指定间隔轮询 GitHub，直到用户完成授权或超时
pub async fn poll_device_token(
    device_code: &str,
    interval: u64,
    timeout_secs: u64,
) -> Result<String, String> {
    let client = build_http_client();
    let mut current_interval = interval;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() >= timeout {
            return Err("Device Code 已过期，请重新发起认证".to_string());
        }

        std::thread::sleep(std::time::Duration::from_secs(current_interval));

        let params = [
            ("client_id", GITHUB_CLIENT_ID),
            ("device_code", device_code),
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:device_code",
            ),
        ];

        let resp = client
            .post(GITHUB_ACCESS_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("轮询 GitHub Access Token 失败: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "轮询 GitHub Access Token 失败 (HTTP {}): {}",
                status, body
            ));
        }

        let token_resp: DeviceTokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("解析 Access Token 响应失败: {}", e))?;

        if let Some(access_token) = token_resp.access_token {
            return Ok(access_token);
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                current_interval += 5;
                continue;
            }
            Some("expired_token") => {
                return Err("Device Code 已过期，请重新发起认证".to_string());
            }
            Some("access_denied") => {
                return Err("用户拒绝了授权请求".to_string());
            }
            Some(other) => {
                let desc = token_resp
                    .error_description
                    .unwrap_or_else(|| "未知错误".to_string());
                return Err(format!("GitHub 认证失败 ({}): {}", other, desc));
            }
            None => {
                return Err("GitHub 认证响应异常：无 access_token 也无 error".to_string());
            }
        }
    }
}

// ============================================================================
// 通用认证操作
// ============================================================================

/// 验证 token 有效性
///
/// 调用 GitHub API 获取用户信息，验证 token 是否有效
pub async fn validate_token(token: &str) -> Result<GitHubUser, String> {
    let client = build_http_client();

    let resp = client
        .get(GITHUB_API_USER_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("验证 GitHub Token 失败: {}", e))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("GitHub Token 无效或已过期".to_string());
    }

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "验证 GitHub Token 失败 (HTTP {}): {}",
            status, body
        ));
    }

    resp.json::<GitHubUser>()
        .await
        .map_err(|e| format!("解析 GitHub 用户信息失败: {}", e))
}

/// PAT 认证
///
/// 直接用 Personal Access Token 验证有效性
pub async fn authenticate_with_pat(pat: &str) -> Result<(String, GitHubUser), String> {
    let user = validate_token(pat).await?;
    Ok((pat.to_string(), user))
}

/// 登出
///
/// 清除本地存储的 token 和同步元数据
pub async fn logout(app: &tauri::AppHandle) -> Result<(), String> {
    token::delete_token(app)?;

    let empty_meta = SyncMetadata::default();
    token::save_sync_meta(app, &empty_meta).await?;

    Ok(())
}
