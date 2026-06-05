use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// 从设置中读取悬浮球保存的位置
fn read_bubble_position(_app: &AppHandle) -> Result<(f64, f64), String> {
    let settings_path = crate::commands::get_settings_path()?;

    let content = if settings_path.exists() {
        std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("读取设置文件失败: {}", e))?
    } else {
        "{}".to_string()
    };

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析设置文件失败: {}", e))?;

    let pos_x = json
        .get("bubble")
        .and_then(|b| b.get("position_x"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1100.0);

    let pos_y = json
        .get("bubble")
        .and_then(|b| b.get("position_y"))
        .and_then(|v| v.as_f64())
        .unwrap_or(650.0);

    Ok((pos_x, pos_y))
}

/// 创建悬浮球窗口
#[tauri::command]
pub async fn create_bubble(app: AppHandle) -> Result<(), String> {
    // 防止重复创建
    if app.get_webview_window("bubble").is_some() {
        tracing::info!("[Bubble] 悬浮球窗口已存在，跳过创建");
        return Ok(());
    }

    // 读取保存的位置（默认右下角）
    let (pos_x, pos_y) = read_bubble_position(&app).unwrap_or((1100.0, 650.0));

    let _window =
        WebviewWindowBuilder::new(&app, "bubble", WebviewUrl::App("bubble.html".into()))
            .title("OMOSwitcher - 悬浮球")
            .inner_size(80.0, 80.0)
            .position(pos_x, pos_y)
            .always_on_top(true)
            .skip_taskbar(true)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .visible(true)
            .build()
            .map_err(|e| format!("创建悬浮球窗口失败: {}", e))?;

    tracing::info!(
        "[Bubble] 悬浮球窗口创建成功, position=({}, {})",
        pos_x,
        pos_y
    );

    // 通知前端
    let _ = app.emit("bubble-created", ());

    Ok(())
}

/// 销毁悬浮球窗口
#[tauri::command]
pub async fn destroy_bubble(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("bubble") {
        window.close().map_err(|e| format!("关闭悬浮球窗口失败: {}", e))?;
        tracing::info!("[Bubble] 悬浮球窗口已关闭");
        let _ = app.emit("bubble-destroyed", ());
    } else {
        tracing::info!("[Bubble] 悬浮球窗口不存在，无需关闭");
    }
    Ok(())
}

/// 切换悬浮球显示/隐藏，返回切换后的状态（true=显示）
#[tauri::command]
pub async fn toggle_bubble(app: AppHandle) -> Result<bool, String> {
    if app.get_webview_window("bubble").is_some() {
        // 窗口存在 → 销毁
        update_bubble_enabled_setting(&app, false)?;
        destroy_bubble(app).await?;
        tracing::info!("[Bubble] 悬浮球已隐藏");
        Ok(false)
    } else {
        // 窗口不存在 → 创建
        let app_clone = app.clone();
        create_bubble(app.clone()).await?;
        update_bubble_enabled_setting(&app_clone, true)?;
        tracing::info!("[Bubble] 悬浮球已显示");
        Ok(true)
    }
}

/// 获取悬浮球设置
#[tauri::command]
pub async fn get_bubble_settings() -> Result<String, String> {
    let settings_path = crate::commands::get_settings_path()?;

    if !settings_path.exists() {
        let default = serde_json::json!({
            "enabled": false,
            "position_x": 1100.0,
            "position_y": 650.0
        });
        return Ok(default.to_string());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("读取设置文件失败: {}", e))?;

    let settings: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

    let bubble = settings
        .get("bubble")
        .cloned()
        .unwrap_or(serde_json::json!({
            "enabled": false,
            "position_x": 1100.0,
            "position_y": 650.0
        }));

    Ok(bubble.to_string())
}

/// 保存悬浮球窗口位置
#[tauri::command]
pub async fn save_bubble_position(x: f64, y: f64) -> Result<(), String> {
    let settings_path = crate::commands::get_settings_path()?;

    // 读取现有设置
    let mut current: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("读取设置文件失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 确保 bubble 对象存在
    if current.get("bubble").is_none() {
        current["bubble"] = serde_json::json!({});
    }

    // 更新位置
    current["bubble"]["position_x"] = serde_json::json!(x);
    current["bubble"]["position_y"] = serde_json::json!(y);

    // 写回文件
    let parent = settings_path.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;

    std::fs::write(
        &settings_path,
        serde_json::to_string_pretty(&current).map_err(|e| format!("序列化设置失败: {}", e))?,
    )
    .map_err(|e| format!("写入设置文件失败: {}", e))?;

    tracing::info!("[Bubble] 位置已保存: ({}, {})", x, y);
    Ok(())
}

/// 更新设置文件中的 bubble.enabled
fn update_bubble_enabled_setting(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let _ = app;
    let settings_path = crate::commands::get_settings_path()?;

    // 读取现有设置
    let mut current: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("读取设置文件失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 更新 bubble.enabled
    if current.get("bubble").is_none() {
        current["bubble"] = serde_json::json!({
            "enabled": enabled,
            "position_x": 1100.0,
            "position_y": 650.0
        });
    } else {
        current["bubble"]["enabled"] = serde_json::json!(enabled);
    }

    // 写回文件
    std::fs::write(
        &settings_path,
        serde_json::to_string_pretty(&current).map_err(|e| format!("序列化设置失败: {}", e))?,
    )
    .map_err(|e| format!("写入设置文件失败: {}", e))?;

    Ok(())
}

/// 检查悬浮球窗口是否实际存在
#[tauri::command]
pub async fn is_bubble_visible(app: AppHandle) -> Result<bool, String> {
    Ok(app.get_webview_window("bubble").is_some())
}
