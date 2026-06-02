// MiniMax 额度查询
//
// 日志约定：
//   - info  级别：debug 模式可见（项目 lib.rs init_logging 配置 debug_assertions 下默认 info 级）
//   - debug 级别：仅 RUST_LOG=debug 时可见（大段 body、过程解析）
//   - warn/error 级别：异常路径，始终保留

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 MiniMax Coding Plan 额度
/// 接口: GET /v1/api/openplatform/coding_plan/remains
/// 国际版: https://api.minimax.io  国内版: https://api.minimaxi.com
/// 响应中 current_interval_usage_count 实际是**剩余量**，不是已用量
pub(crate) async fn query_minimax(provider: &ProviderInfo) -> ProviderQuota {
    let key_preview: String = provider.api_key.chars().take(8).collect();
    tracing::info!(
        "[MiniMax额度] 查询开始 provider_id={}, apiKey={}…",
        provider.id, key_preview
    );

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("[MiniMax额度] 创建 HTTP 客户端失败: {}", e);
            return error_quota(&provider.id, &provider.name, &e);
        }
    };

    // 根据 base_url 判断国际版/国内版
    let (primary_url, fallback_url) = match &provider.base_url {
        Some(b) if b.contains("minimaxi.com") || b.contains("minimaxi") => {
            let primary = "https://api.minimaxi.com/v1/api/openplatform/coding_plan/remains".to_string();
            let fallback = "https://api.minimax.io/v1/api/openplatform/coding_plan/remains".to_string();
            (primary, Some(fallback))
        }
        _ => {
            let primary = "https://api.minimax.io/v1/api/openplatform/coding_plan/remains".to_string();
            let fallback = "https://api.minimaxi.com/v1/api/openplatform/coding_plan/remains".to_string();
            (primary, Some(fallback))
        }
    };
    tracing::info!("[MiniMax额度] 主 URL = {}", primary_url);

    // 辅助：读取 body、记录响应、尝试解析 JSON
    async fn read_and_log(provider: &ProviderInfo, stage: &str, resp: reqwest::Response) -> Result<serde_json::Value, ProviderQuota> {
        let status = resp.status();
        let body = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("[MiniMax额度] {} 读取响应体失败: {}", stage, e);
                return Err(error_quota(&provider.id, &provider.name, &format!("读取响应失败: {}", e)));
            }
        };
        let preview: String = body.chars().take(2000).collect();
        if status.is_success() {
            // 完整 body 仅 debug 级（避免生产刷屏）
            tracing::debug!("[MiniMax额度] {} HTTP {} body(2000): {}", stage, status, preview);
        } else {
            tracing::warn!("[MiniMax额度] {} HTTP {} body(300): {}", stage, status, preview.chars().take(300).collect::<String>());
            return Err(error_quota(&provider.id, &provider.name, &format!("HTTP {} body={}", status, preview.chars().take(300).collect::<String>())));
        }
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(v) => Ok(v),
            Err(e) => {
                tracing::error!("[MiniMax额度] {} JSON 解析失败: {}, body={}", stage, e, preview);
                Err(error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)))
            }
        }
    }

    // 主 URL 请求
    let primary_resp = client
        .get(&primary_url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .header("Content-Type", "application/json")
        .send()
        .await;

    let primary_value = match primary_resp {
        Ok(r) => match read_and_log(provider, "主URL", r).await {
            Ok(v) => Some(v),
            Err(quota) => return quota,
        },
        Err(e) => {
            tracing::warn!("[MiniMax额度] 主 URL 网络错误: {}", e);
            None
        }
    };

    // 必要时走 fallback
    let json_value = match primary_value {
        Some(v) => v,
        None => {
            let fb_url = match &fallback_url {
                Some(u) => u.clone(),
                None => return error_quota(&provider.id, &provider.name, "请求失败且无 fallback"),
            };
            tracing::info!("[MiniMax额度] 主 URL 不可用，走 fallback: {}", fb_url);
            match client.get(&fb_url)
                .header("Authorization", format!("Bearer {}", provider.api_key))
                .header("Content-Type", "application/json")
                .send()
                .await
            {
                Ok(r) => match read_and_log(provider, "fallback", r).await {
                    Ok(v) => v,
                    Err(quota) => return quota,
                },
                Err(e) => {
                    tracing::error!("[MiniMax额度] fallback 网络错误: {}", e);
                    return error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e));
                }
            }
        }
    };

    parse_minimax_response(provider, json_value)
}

/// 解析 MiniMax Coding Plan API 响应
/// 响应格式:
/// {
///   "base_resp": { "status_code": 0, "status_msg": "success" },
///   "data": {
///     "model_remains": [{
///       "model_name": "general",
///       "current_interval_total_count": 0,                 // 0 = 不限量
///       "current_interval_usage_count": 0,                 // 实际是**剩余量**（命名反直觉）
///       "current_interval_remaining_percent": 91,          // ★ 5h 窗口剩余百分比 (0-100)
///       "current_weekly_total_count": 0,
///       "current_weekly_usage_count": 0,                   // 实际是**剩余量**
///       "current_weekly_remaining_percent": 100,           // ★ 周剩余百分比 (0-100)
///       "end_time": 1780383600000, "weekly_end_time": 1780848000000,
///       ...
///     }, { "model_name": "video", "current_interval_total_count": 3, ... }]
///   }
/// }
pub(crate) fn parse_minimax_response(provider: &ProviderInfo, data: serde_json::Value) -> ProviderQuota {
    tracing::debug!("[MiniMax额度] 响应顶层字段: {:?}",
        data.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()));

    // 检查 base_resp
    if let Some(status_code) = data.get("base_resp")
        .and_then(|b| b.get("status_code"))
        .and_then(|v| v.as_i64())
    {
        if status_code != 0 {
            let msg = data.get("base_resp")
                .and_then(|b| b.get("status_msg"))
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            tracing::warn!("[MiniMax额度] base_resp 非 0 ({}): {}", status_code, msg);
            return error_quota(&provider.id, &provider.name, msg);
        }
    }

    // 提取 data.model_remains
    let data_obj = data.get("data").unwrap_or(&data);
    let model_remains = match data_obj.get("model_remains").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            tracing::warn!("[MiniMax额度] 未找到 model_remains 数组");
            return error_quota(&provider.id, &provider.name, "未找到 model_remains 数据");
        }
    };
    tracing::debug!("[MiniMax额度] model_remains 共 {} 个条目", model_remains.len());
    for (i, m) in model_remains.iter().enumerate() {
        let name = m.get("model_name").and_then(|v| v.as_str()).unwrap_or("?");
        let total = read_f64(m, &["current_interval_total_count", "currentIntervalTotalCount"]);
        let remaining = read_f64(m, &["current_interval_usage_count", "currentIntervalUsageCount"]);
        let remaining_pct = m.get("current_interval_remaining_percent").and_then(|v| v.as_f64());
        tracing::debug!("[MiniMax额度]   [{}] {} total={} remaining={} remaining_pct={:?}", i, name, total, remaining, remaining_pct);
    }

    // 选主模型（卡片展示用）
    let main_model = match select_minimax_main_model(model_remains) {
        Some(m) => m,
        None => {
            tracing::warn!("[MiniMax额度] 未找到有效的模型配额数据");
            return error_quota(&provider.id, &provider.name, "未找到有效的模型配额数据");
        }
    };
    let main_name = main_model.get("model_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    tracing::info!("[MiniMax额度] 主模型 = {}, 5h 剩余 = {:?}%", main_name,
        main_model.get("current_interval_remaining_percent").and_then(|v| v.as_f64()));

    // 主模型 5h 窗口字段
    let main_total = read_f64(main_model, &["current_interval_total_count", "currentIntervalTotalCount"]);
    let main_remaining = read_f64(main_model, &["current_interval_usage_count", "currentIntervalUsageCount"]);
    let main_remaining_pct = main_model.get("current_interval_remaining_percent").and_then(|v| v.as_f64());
    let main_used = (main_total - main_remaining).max(0.0);
    // 5h 使用率：优先用 API 的 remaining_percent 反推（不限量模型也支持），否则自算
    let main_percentage = match main_remaining_pct {
        Some(p) => (100.0 - p).max(0.0).min(100.0),
        None if main_total > 0.0 => (main_used / main_total) * 100.0,
        _ => 0.0,
    };
    let main_reset_time = parse_minimax_end_time(
        main_model.get("end_time").or_else(|| main_model.get("endTime"))
    );
    // 不限量模型（total=0）时 quota_limit 设为 None，避免前端展示 "0/0"
    let main_quota_limit = if main_total > 0.0 { Some(main_total) } else { None };

    // 主模型周字段（不限量时周数据无意义，全部置 None）
    let weekly_total = read_f64(main_model, &["current_weekly_total_count", "currentWeeklyTotalCount"]);
    let weekly_remaining = read_f64(main_model, &["current_weekly_usage_count", "currentWeeklyUsageCount"]);
    let weekly_used = (weekly_total - weekly_remaining).max(0.0);
    let (weekly_usage, spending_limit) = if weekly_total > 0.0 {
        (Some(weekly_used), Some(weekly_total))
    } else {
        (None, None)
    };

    tracing::debug!(
        "[MiniMax额度] 解析完成 → 5h: total={}, remaining={}, percentage={:.2}%, reset={:?} | 周: total={}, remaining={}, used={}",
        main_total, main_remaining, main_percentage, main_reset_time,
        weekly_total, weekly_remaining, weekly_used
    );

    // 构建 limits 数组（详情弹窗展示用）
    let limits = build_minimax_limits(model_remains);

    ProviderQuota {
        provider_id: provider.id.clone(),
        provider_name: provider.name.clone(),
        quota_type: "token_limit".to_string(),
        status: "success".to_string(),
        error_message: None,
        // balance 型字段（不填）
        total_balance: None,
        available_balance: None,
        used_balance: None,
        currency: None,
        // 5h 窗口主展示
        quota_percentage: Some(main_percentage),
        quota_used: Some(main_used),
        quota_limit: main_quota_limit,                 // None 表示不限量
        limit_remaining: Some(main_remaining),         // 5h 剩余
        reset_time: main_reset_time,
        // 周期用量（按 5h/周/月 维度命名归一化）
        // weeklyUsage = 周已用；spendingLimit = 周上限；monthlyUsage 不填（API 未返回月数据）
        weekly_usage,
        monthly_usage: None,
        spending_limit,
        // 多模型明细
        limits: Some(serde_json::Value::Array(limits)),
        daily_usage: None,
    }
}

/// 从 model_remains 选主模型（用于卡片主展示）
/// 卡片只需"通用/聊天主模型"的用量百分比，不展示 video/speech/image 等专项模型。
/// 优先级：
///   1. 聊天类 + 限量模型（total > 0）→ 第一个
///   2. 聊天类 + 不限量但有 remaining_percent → 第一个
///   3. 兜底：所有模型中限量优先；再退化：不限量但有 remaining_percent；最后数组首项
fn select_minimax_main_model(models: &[serde_json::Value]) -> Option<&serde_json::Value> {
    let chat_models: Vec<&serde_json::Value> = models
        .iter()
        .filter(|m| {
            let name = m.get("model_name").and_then(|v| v.as_str()).unwrap_or("");
            is_minimax_chat_model(name)
        })
        .collect();

    // 1. 聊天类 + 限量
    if let Some(m) = chat_models.iter().find(|m| {
        read_f64(m, &["current_interval_total_count", "currentIntervalTotalCount"]) > 0.0
    }) {
        return Some(*m);
    }
    // 2. 聊天类 + 不限量但有 remaining_percent
    if let Some(m) = chat_models.iter().find(|m| {
        m.get("current_interval_remaining_percent").and_then(|v| v.as_f64()).is_some()
    }) {
        return Some(*m);
    }
    // 3. 兜底：所有模型中限量
    if let Some(m) = models.iter().find(|m| {
        read_f64(m, &["current_interval_total_count", "currentIntervalTotalCount"]) > 0.0
    }) {
        return Some(m);
    }
    // 4. 兜底：所有模型中有 remaining_percent
    if let Some(m) = models.iter().find(|m| {
        m.get("current_interval_remaining_percent").and_then(|v| v.as_f64()).is_some()
    }) {
        return Some(m);
    }
    // 5. 数组首项
    models.first()
}

/// 判断是否为聊天/通用类模型（排除 video/speech/image/audio/tts/asr/music/embedding 等专项模型）
fn is_minimax_chat_model(name: &str) -> bool {
    let lower = name.to_lowercase();
    !lower.starts_with("video")
        && !lower.starts_with("speech")
        && !lower.starts_with("tts")
        && !lower.starts_with("asr")
        && !lower.starts_with("image")
        && !lower.starts_with("audio")
        && !lower.starts_with("music")
        && !lower.starts_with("embedding")
}

/// 把 model_remains 展开为前端展示用的 limits 数组
/// 每个模型生成 1-2 行：5h 维度（必填） + 周维度（如果 weekly_total > 0）
fn build_minimax_limits(models: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut limits = Vec::new();
    for m in models {
        let model_name = m.get("model_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if model_name.is_empty() {
            continue;
        }
        let total = read_f64(m, &["current_interval_total_count", "currentIntervalTotalCount"]);
        let remaining = read_f64(m, &["current_interval_usage_count", "currentIntervalUsageCount"]);
        let remaining_pct = m.get("current_interval_remaining_percent").and_then(|v| v.as_f64());
        let used = (total - remaining).max(0.0);
        let usage_percent = match remaining_pct {
            Some(p) => (100.0 - p).max(0.0).min(100.0),
            None if total > 0.0 => (used / total) * 100.0,
            _ => 0.0,
        };
        let reset_time = parse_minimax_end_time(
            m.get("end_time").or_else(|| m.get("endTime"))
        );
        limits.push(serde_json::json!({
            "type": "5h-rolling",
            "modelName": model_name,
            "usagePercent": usage_percent,
            "remainingPercent": remaining_pct.unwrap_or(0.0),
            "limit": total,
            "remaining": remaining,
            "used": used,
            "resetTime": reset_time,
        }));

        // 周维度（仅当 weekly_total > 0 时输出；API 对不限量模型的周配额也常为 0）
        let weekly_total = read_f64(m, &["current_weekly_total_count", "currentWeeklyTotalCount"]);
        if weekly_total > 0.0 {
            let weekly_remaining = read_f64(m, &["current_weekly_usage_count", "currentWeeklyUsageCount"]);
            let weekly_used = (weekly_total - weekly_remaining).max(0.0);
            let weekly_remaining_pct = m.get("current_weekly_remaining_percent").and_then(|v| v.as_f64());
            let weekly_usage_pct = match weekly_remaining_pct {
                Some(p) => (100.0 - p).max(0.0).min(100.0),
                None => (weekly_used / weekly_total) * 100.0,
            };
            limits.push(serde_json::json!({
                "type": "weekly",
                "modelName": model_name,
                "usagePercent": weekly_usage_pct,
                "remainingPercent": weekly_remaining_pct.unwrap_or(0.0),
                "limit": weekly_total,
                "remaining": weekly_remaining,
                "used": weekly_used,
                "resetTime": parse_minimax_end_time(
                    m.get("weekly_end_time").or_else(|| m.get("weeklyEndTime"))
                ),
            }));
        }
    }
    limits
}

/// 读取 f64 字段（兼容 snake_case 和 camelCase 两种命名）
fn read_f64(v: &serde_json::Value, keys: &[&str]) -> f64 {
    for k in keys {
        if let Some(n) = v.get(*k).and_then(|x| x.as_f64()) {
            return n;
        }
    }
    0.0
}

/// 解析 end_time 时间戳（秒或毫秒）→ ISO 8601 UTC 字符串
fn parse_minimax_end_time(v: Option<&serde_json::Value>) -> Option<String> {
    let ts = v?.as_i64()?;
    if ts <= 0 {
        return None;
    }
    let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
    time::OffsetDateTime::from_unix_timestamp(secs).ok().map(|dt| {
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            dt.year(), dt.month() as u8, dt.day(),
            dt.hour(), dt.minute(), dt.second())
    })
}
