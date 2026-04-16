// Sync 模块 - Tauri Commands
// 提供 GitHub Gist 同步的前端交互命令
// 所有命令返回 JSON 字符串，遵循 Result<T, String> 模式

use std::sync::Arc;

use serde_json;
use tauri::{AppHandle, State};
use tokio::sync::Mutex as AsyncMutex;

use super::auth;
use super::engine;
use super::token;
use super::types::{AuthState, ConflictResolution, OAuthSession, SyncResult};

// ============================================================================
// Sync 命令状态
// ============================================================================

/// Sync 命令层共享状态
/// 用于管理认证过程中的临时状态
#[derive(Clone, Default)]
pub struct SyncCommandState {
    /// 正在进行的 Device Flow 的 device_code
    /// 用于在 start_device_login 和 complete_device_login 之间传递状态
    pub pending_device_code: Arc<AsyncMutex<Option<String>>>,
    /// Device Flow 响应信息（用于取消时清理）
    pub pending_user_code: Arc<AsyncMutex<Option<String>>>,
    /// OAuth Web Flow 进行中的会话（PKCE 参数）
    pub pending_oauth: Arc<AsyncMutex<Option<OAuthSession>>>,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取预设目录路径
fn get_presets_dir() -> Result<std::path::PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config").join("omoswitcher").join("presets"))
        .ok_or_else(|| "无法获取用户主目录".to_string())
}

/// 读取所有预设文件并合并为 JSON 字符串
/// 返回格式：{ "preset_name": { ...preset_content... }, ... }
async fn read_all_presets_as_map() -> Result<String, String> {
    let presets_dir = get_presets_dir()?;
    if !presets_dir.exists() {
        return Ok("{}".to_string());
    }

    let mut entries = tokio::fs::read_dir(&presets_dir)
        .await
        .map_err(|e| format!("读取预设目录失败: {}", e))?;

    let mut presets_map = serde_json::Map::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取预设条目失败: {}", e))?
    {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.ends_with(".json") {
            let preset_name = name.trim_end_matches(".json").to_string();
            let path = entry.path();

            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    presets_map.insert(preset_name, data);
                }
            }
        }
    }

    Ok(serde_json::to_string(&serde_json::Value::Object(presets_map))
        .unwrap_or_else(|_| "{}".to_string()))
}

/// 将预设 JSON 写入本地预设文件
/// 输入格式：{ "preset_name": { ...preset_content... }, ... }
async fn write_presets_from_map(presets_json: &str) -> Result<usize, String> {
    let presets_dir = get_presets_dir()?;
    
    // 确保预设目录存在
    tokio::fs::create_dir_all(&presets_dir)
        .await
        .map_err(|e| format!("创建预设目录失败: {}", e))?;

    let presets: serde_json::Value = serde_json::from_str(presets_json)
        .map_err(|e| format!("解析预设 JSON 失败: {}", e))?;

    let presets_map = presets
        .as_object()
        .ok_or_else(|| "预设数据格式错误：期望对象格式".to_string())?;

    let mut count = 0;
    for (preset_name, preset_data) in presets_map {
        let path = presets_dir.join(format!("{}.json", preset_name));
        let content = serde_json::to_string_pretty(preset_data)
            .map_err(|e| format!("序列化预设失败: {}", e))?;
        
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| format!("写入预设文件失败: {}", e))?;
        
        count += 1;
    }

    Ok(count)
}

/// 读取当前激活的预设名称
/// 从设置文件中获取 currentPreset 字段
async fn get_current_preset_name() -> Option<String> {
    let settings_path = dirs::home_dir()
        .map(|p| p.join(".config").join("omoswitcher").join("settings.json"))?;
    
    if !settings_path.exists() {
        return None;
    }

    let content = tokio::fs::read_to_string(&settings_path).await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("currentPreset")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 获取认证状态
///
/// 检查 token 是否存在 → 验证有效性 → 返回 AuthState JSON
#[tauri::command]
pub async fn sync_get_auth_state(app: AppHandle) -> Result<String, String> {
    // 1. 检查 token 是否存在
    let token = token::get_token(&app)?;

    match token {
        None => {
            // 未登录
            let state = AuthState::LoggedOut;
            serde_json::to_string(&state)
                .map_err(|e| format!("序列化认证状态失败: {}", e))
        }
        Some(t) => {
            // 2. 验证 token 有效性
            match auth::validate_token(&t).await {
                Ok(user) => {
                    // Token 有效
                    let state = AuthState::LoggedIn { user };
                    serde_json::to_string(&state)
                        .map_err(|e| format!("序列化认证状态失败: {}", e))
                }
                Err(_) => {
                    // Token 无效或过期
                    let state = AuthState::LoggedOut;
                    serde_json::to_string(&state)
                        .map_err(|e| format!("序列化认证状态失败: {}", e))
                }
            }
        }
    }
}

/// 启动 Device Flow 登录
///
/// 向 GitHub 请求 device code → 保存到状态 → 返回 {user_code, verification_uri} JSON
#[tauri::command]
pub async fn sync_start_device_login(
    app: AppHandle,
    state: State<'_, SyncCommandState>,
) -> Result<String, String> {
    // 调用 auth 模块启动 Device Flow
    let response = auth::start_device_flow().await?;

    // 保存 device_code 到状态（供 complete_device_login 使用）
    {
        let mut pending = state.pending_device_code.lock().await;
        *pending = Some(response.device_code.clone());
    }
    {
        let mut user_code = state.pending_user_code.lock().await;
        *user_code = Some(response.user_code.clone());
    }

    // 构建返回给前端的 JSON
    let result = serde_json::json!({
        "user_code": response.user_code,
        "verification_uri": response.verification_uri,
        "expires_in": response.expires_in,
        "interval": response.interval,
    });

    // 更新认证状态为 LoggingIn
    let auth_state = AuthState::LoggingIn {
        user_code: response.user_code.clone(),
        verification_uri: response.verification_uri.clone(),
    };

    // 保存用户信息到元数据（可选）
    let meta = token::get_sync_meta(&app).await.unwrap_or_default();
    // 预留：可以在这里记录 login 开始时间

    let _ = (meta, auth_state); // 避免 unused warning

    Ok(serde_json::to_string(&result)
        .map_err(|e| format!("序列化响应失败: {}", e))?)
}

/// 完成 Device Flow 登录
///
/// 从状态取 device_code → 轮询 GitHub → 保存 token → 返回 GitHubUser JSON
#[tauri::command]
pub async fn sync_complete_device_login(
    app: AppHandle,
    state: State<'_, SyncCommandState>,
) -> Result<String, String> {
    // 从状态获取 device_code
    let device_code = {
        let pending = state.pending_device_code.lock().await;
        pending.clone().ok_or_else(|| {
            "没有正在进行的 Device Flow 登录，请先调用 sync_start_device_login".to_string()
        })?
    };

    // 轮询 GitHub 获取 token（默认 5 分钟超时，间隔 5 秒）
    let access_token = auth::poll_device_token(&device_code, 5, 300).await?;

    // 验证 token 并获取用户信息
    let user = auth::validate_token(&access_token).await?;

    // 保存 token 到 keyring
    token::save_token(&app, &access_token)?;

    // 更新同步元数据中的用户信息
    let mut meta = token::get_sync_meta(&app).await.unwrap_or_default();
    meta.github_user_id = Some(user.id);
    meta.github_login = Some(user.login.clone());
    token::save_sync_meta(&app, &meta).await?;

    // 清除 pending 状态
    {
        let mut pending = state.pending_device_code.lock().await;
        *pending = None;
    }
    {
        let mut user_code = state.pending_user_code.lock().await;
        *user_code = None;
    }

    // 返回用户信息 JSON
    serde_json::to_string(&user)
        .map_err(|e| format!("序列化用户信息失败: {}", e))
}

/// 使用 PAT 登录
///
/// 验证 PAT → 保存 token → 返回 GitHubUser JSON
#[tauri::command]
pub async fn sync_login_with_pat(app: AppHandle, pat: String) -> Result<String, String> {
    // 验证 PAT 并获取用户信息
    let (_token, user) = auth::authenticate_with_pat(&pat).await?;

    // 保存 token 到 keyring
    token::save_token(&app, &pat)?;

    // 更新同步元数据中的用户信息
    let mut meta = token::get_sync_meta(&app).await.unwrap_or_default();
    meta.github_user_id = Some(user.id);
    meta.github_login = Some(user.login.clone());
    token::save_sync_meta(&app, &meta).await?;

    // 返回用户信息 JSON
    serde_json::to_string(&user)
        .map_err(|e| format!("序列化用户信息失败: {}", e))
}

/// 登出
///
/// 清除本地 token 和同步元数据
#[tauri::command]
pub async fn sync_logout(app: AppHandle) -> Result<(), String> {
    auth::logout(&app).await
}

/// 获取同步状态
///
/// 返回 SyncMetadata JSON
#[tauri::command]
pub async fn sync_get_status(app: AppHandle) -> Result<String, String> {
    let meta = token::get_sync_meta(&app).await?;
    serde_json::to_string(&meta)
        .map_err(|e| format!("序列化同步元数据失败: {}", e))
}

/// 上传预设到 Gist
///
/// 读取本地预设 → 上传到 GitHub Gist → 更新同步元数据 → 返回 SyncResult JSON
#[tauri::command]
pub async fn sync_upload(app: AppHandle) -> Result<String, String> {
    // 获取 token
    let token = token::get_token(&app)?
        .ok_or_else(|| "未登录，请先进行 GitHub 认证".to_string())?;

    // 获取同步元数据
    let meta = token::get_sync_meta(&app).await?;

    // 读取所有本地预设
    let presets_json = read_all_presets_as_map().await?;

    // 获取当前激活的预设名称
    let current_preset = get_current_preset_name().await.unwrap_or_default();

    // 执行上传
    let _new_meta = engine::perform_upload(
        &app,
        &token,
        meta.gist_id.as_deref(),
        &presets_json,
        &current_preset,
    )
    .await?;

    // 构建返回结果
    let result = SyncResult::Uploaded {
        count: presets_json
            .parse::<serde_json::Value>()
            .map(|v| v.as_object().map(|m| m.len()).unwrap_or(0))
            .unwrap_or(0),
    };

    serde_json::to_string(&result)
        .map_err(|e| format!("序列化同步结果失败: {}", e))
}

/// 从 Gist 下载预设
///
/// 从 GitHub Gist 下载 → 写入本地预设文件 → 更新同步元数据 → 返回 SyncResult JSON
#[tauri::command]
pub async fn sync_download(app: AppHandle) -> Result<String, String> {
    // 获取 token
    let token = token::get_token(&app)?
        .ok_or_else(|| "未登录，请先进行 GitHub 认证".to_string())?;

    // 获取同步元数据
    let meta = token::get_sync_meta(&app).await?;

    // 获取 gist_id
    let gist_id = meta
        .gist_id
        .as_ref()
        .ok_or_else(|| "没有关联的 Gist，请先上传预设".to_string())?;

    // 执行下载
    let (presets_json, _, remote_updated_at) = engine::perform_download(&token, gist_id).await?;

    // 写入本地预设文件
    let count = write_presets_from_map(&presets_json).await?;

    // 更新同步元数据
    let mut new_meta = meta.clone();
    new_meta.last_sync_content_hash = Some(engine::compute_content_hash(&presets_json));
    new_meta.last_sync_at = Some(remote_updated_at);
    token::save_sync_meta(&app, &new_meta).await?;

    // 返回结果
    let result = SyncResult::Downloaded { count };

    serde_json::to_string(&result)
        .map_err(|e| format!("序列化同步结果失败: {}", e))
}

/// 执行完整同步
///
/// 检测同步方向 → 自动执行上传或下载 → 返回 SyncResult JSON
#[tauri::command]
pub async fn sync_perform(app: AppHandle) -> Result<String, String> {
    // 获取 token
    let token = token::get_token(&app)?
        .ok_or_else(|| "未登录，请先进行 GitHub 认证".to_string())?;

    // 读取所有本地预设
    let presets_json = read_all_presets_as_map().await?;

    // 获取当前激活的预设名称
    let current_preset = get_current_preset_name().await.unwrap_or_default();

    // 执行完整同步
    tracing::info!("[Sync:Command] 开始同步，本地预设: {} bytes", presets_json.len());
    let (result, downloaded_json) = match engine::full_sync(&app, &token, &presets_json, &current_preset).await {
        Ok((r, json)) => {
            tracing::info!("[Sync:Command] 同步完成: {:?}", r);
            (r, json)
        }
        Err(e) => {
            tracing::error!("[Sync:Command] 同步失败: {}", e);
            return Err(e);
        }
    };

    // 如果是下载，使用引擎返回的内容直接写入本地文件（无需重复 API 调用）
    if let Some(remote_json) = downloaded_json {
        write_presets_from_map(&remote_json).await?;
    }

    serde_json::to_string(&result)
        .map_err(|e| format!("序列化同步结果失败: {}", e))
}

/// 解决冲突
///
/// 根据 resolution 选择保留本地或远端版本
#[tauri::command]
pub async fn sync_resolve_conflict(app: AppHandle, resolution: String) -> Result<(), String> {
    // 解析 resolution（前端传入纯字符串，非 JSON 编码）
    let resolution = match resolution.as_str() {
        "KeepLocal" => ConflictResolution::KeepLocal,
        "KeepRemote" => ConflictResolution::KeepRemote,
        _ => return Err(format!("未知的冲突解决策略: {}", resolution)),
    };

    // 获取 token
    let token = token::get_token(&app)?
        .ok_or_else(|| "未登录，请先进行 GitHub 认证".to_string())?;

    // 获取同步元数据
    let meta = token::get_sync_meta(&app).await?;

    // 读取本地预设
    let presets_json = read_all_presets_as_map().await?;
    let current_preset = get_current_preset_name().await.unwrap_or_default();

    // 执行冲突解决
    let downloaded_json = engine::resolve_conflict(
        resolution.clone(),
        &app,
        &token,
        meta.gist_id.as_deref(),
        &presets_json,
        &current_preset,
    )
    .await?;

    // 如果是保留远端，使用引擎返回的内容直接写入本地文件
    if let Some(remote_json) = downloaded_json {
        write_presets_from_map(&remote_json).await?;
    }

    Ok(())
}

/// 取消正在进行的 Device Flow 登录
///
/// 清除 pending 状态
#[tauri::command]
pub async fn sync_cancel_device_login(state: State<'_, SyncCommandState>) -> Result<(), String> {
    // 清除 pending device_code
    {
        let mut pending = state.pending_device_code.lock().await;
        *pending = None;
    }
    // 清除 pending user_code
    {
        let mut user_code = state.pending_user_code.lock().await;
        *user_code = None;
    }

    Ok(())
}

// ============================================================================
// OAuth Web Flow 命令
// ============================================================================

/// 启动 OAuth Web Flow 登录
///
/// 1. 生成 PKCE 参数 + 启动本地回调服务器
/// 2. 自动打开浏览器到 GitHub 授权页面
/// 3. 等待用户授权后换取 token
/// 4. 返回用户信息 JSON
#[tauri::command]
pub async fn sync_start_oauth_login(
    app: AppHandle,
    state: State<'_, SyncCommandState>,
) -> Result<String, String> {
    // 1. 准备 OAuth 参数并启动回调服务器
    let (auth_url, session, listener) = auth::prepare_oauth_flow().await?;
    let expected_state = session.state.clone();
    let code_verifier = session.code_verifier.clone();
    tracing::info!("[OAuth:Command] prepare_oauth_flow 完成，等待浏览器回调");

    // 保存会话
    {
        let mut pending = state.pending_oauth.lock().await;
        *pending = Some(session);
    }

    // 自动打开浏览器到 GitHub 授权页面
    crate::commands::open_url_in_browser(auth_url)?;

    // 等待浏览器回调（最多 5 分钟）
    tracing::info!("[OAuth:Command] 正在等待浏览器回调...");
    let callback_data = auth::wait_for_callback(listener).await?;
    tracing::info!("[OAuth:Command] 收到回调数据: {}", callback_data.chars().take(300).collect::<String>());

    // 解析回调数据：格式 "code||state"
    let (code, received_state) = if let Some((c, s)) = callback_data.split_once("||") {
        (c.to_string(), Some(s.to_string()))
    } else {
        (callback_data, None)
    };
    tracing::info!("[OAuth:Command] 解析回调 → code 长度: {}, state: {}", code.len(), received_state.as_deref().unwrap_or("无"));

    // CSRF state 验证
    if let Some(rs) = &received_state {
        if rs != &expected_state {
            let mut pending = state.pending_oauth.lock().await;
            *pending = None;
            return Err("OAuth state 不匹配，可能遭受 CSRF 攻击，请重试".to_string());
        }
        tracing::info!("[OAuth:Command] CSRF state 验证通过");
    }

    // 用 code + code_verifier 换取 access_token
    tracing::info!("[OAuth:Command] 正在用 code 换取 access_token...");
    let access_token = match auth::exchange_code_for_token(&code, &code_verifier).await {
        Ok(t) => {
            tracing::info!("[OAuth:Command] access_token 获取成功，长度: {}", t.len());
            t
        }
        Err(e) => {
            tracing::error!("[OAuth:Command] access_token 获取失败: {}", e);
            return Err(e);
        }
    };

    // 验证 token 并获取用户信息
    tracing::info!("[OAuth:Command] 正在验证 token 获取用户信息...");
    let user = match auth::validate_token(&access_token).await {
        Ok(u) => {
            tracing::info!("[OAuth:Command] 用户验证成功: {} (id: {})", u.login, u.id);
            u
        }
        Err(e) => {
            tracing::warn!("[OAuth:Command] token 验证失败: {}", e);
            return Err(e);
        }
    };

    // 保存 token
    tracing::info!("[OAuth:Command] 正在保存 token...");
    if let Err(e) = token::save_token(&app, &access_token) {
        tracing::error!("[OAuth:Command] token 保存失败: {}", e);
        return Err(e);
    }
    tracing::info!("[OAuth:Command] token 保存成功");

    // 更新同步元数据中的用户信息
    let mut meta = token::get_sync_meta(&app).await.unwrap_or_default();
    meta.github_user_id = Some(user.id);
    meta.github_login = Some(user.login.clone());
    token::save_sync_meta(&app, &meta).await?;

    // 清除 pending 状态
    {
        let mut pending = state.pending_oauth.lock().await;
        *pending = None;
    }

    // 返回用户信息 JSON
    serde_json::to_string(&user).map_err(|e| format!("序列化用户信息失败: {}", e))
}

/// 取消 OAuth Web Flow 登录
#[tauri::command]
pub async fn sync_cancel_oauth_login(state: State<'_, SyncCommandState>) -> Result<(), String> {
    let mut pending = state.pending_oauth.lock().await;
    *pending = None;
    Ok(())
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_dir_path() {
        let path = get_presets_dir().unwrap();
        assert!(path.to_string_lossy().contains("omoswitcher"));
        assert!(path.to_string_lossy().contains("presets"));
    }
}
