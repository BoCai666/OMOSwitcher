// 认证模块
// GitHub Device Flow 和 PAT 双通道认证

use crate::sync::types::*;
use crate::sync::token;

/// GitHub OAuth App Client ID
/// 用户需注册 GitHub OAuth App 获取自己的 client_id
const GITHUB_CLIENT_ID: &str = "";

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_USER_URL: &str = "https://api.github.com/user";

/// 构建带统一 header 的 reqwest Client
fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("OMOSwitcher")
        .build()
        .unwrap_or_default()
}

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
/// - `device_code`: Device Flow 返回的 device_code
/// - `interval`: 轮询间隔（秒）
/// - `timeout_secs`: 总超时（秒）
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
        // 检查总超时
        if start.elapsed() >= timeout {
            return Err("Device Code 已过期，请重新发起认证".to_string());
        }

        // 等待轮询间隔
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

        // 处理响应
        if let Some(access_token) = token_resp.access_token {
            return Ok(access_token);
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => {
                // 用户尚未完成授权，继续等待
                continue;
            }
            Some("slow_down") => {
                // 需要降低轮询频率
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
/// 成功则返回 (token, user_info)
pub async fn authenticate_with_pat(pat: &str) -> Result<(String, GitHubUser), String> {
    let user = validate_token(pat).await?;
    Ok((pat.to_string(), user))
}

/// 登出
///
/// 清除本地存储的 token 和同步元数据
pub async fn logout(app: &tauri::AppHandle) -> Result<(), String> {
    // 清除 keyring 中的 token
    token::delete_token(app)?;

    // 清除同步元数据（重置为默认空值）
    let empty_meta = SyncMetadata::default();
    token::save_sync_meta(app, &empty_meta).await?;

    Ok(())
}
