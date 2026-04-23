// Kimi Code (Kimi for Coding Plan) 额度查询
// 调用 api.kimi.com/coding/v1/usages 端点获取用量信息

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 Kimi Code 额度
/// 端点: GET https://api.kimi.com/coding/v1/usages
pub(crate) async fn query_kimi_code(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    // 构建 usages 端点 URL
    let base_url = provider
        .base_url
        .as_ref()
        .map(|u| u.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "https://api.kimi.com/coding/v1".to_string());
    let usage_url = format!("{}/usages", base_url);

    let result = client
        .get(&usage_url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    match result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}: {}", status, body));
            }
            match resp.json::<serde_json::Value>().await {
                Ok(data) => parse_kimi_code_usage(&provider.id, &provider.name, &data),
                Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e))
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e))
    }
}

/// 解析 Kimi Code usages 响应
/// 响应格式:
/// {
///   "usage": {
///     "limit": 100000,
///     "used": 50000,
///     "remaining": 50000,
///     "name": "Weekly limit",
///     "reset_at": "2025-12-23T05:24:18.443553353Z"
///   },
///   "limits": [
///     {
///       "detail": { "limit": 5000, "used": 1000, "name": "RPM" },
///       "window": { "duration": 60, "timeUnit": "MINUTE" }
///     }
///   ]
/// }
/// 解析字符串或数字为 f64
fn parse_number(value: &serde_json::Value) -> Option<f64> {
    value.as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
}

fn parse_kimi_code_usage(id: &str, name: &str, data: &serde_json::Value) -> ProviderQuota {
    // 1. 解析周额度 (usage 字段)
    let weekly_usage = data.get("usage");
    let weekly_limit = weekly_usage.and_then(|u| u.get("limit")).and_then(parse_number);
    let weekly_used = weekly_usage.and_then(|u| u.get("used")).and_then(parse_number);
    let weekly_remaining = weekly_usage.and_then(|u| u.get("remaining")).and_then(parse_number);
    let weekly_reset = weekly_usage
        .and_then(|u| u.get("resetTime"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 2. 解析 5 小时滚动窗口额度 (limits 数组中 window.duration=300 的项)
    let mut five_hour_limit: Option<f64> = None;
    let mut five_hour_used: Option<f64> = None;
    let mut five_hour_remaining: Option<f64> = None;
    let mut five_hour_reset: Option<String> = None;

    if let Some(limits_arr) = data.get("limits").and_then(|v| v.as_array()) {
        for limit_item in limits_arr {
            // 检查是否为 5 小时窗口 (300 分钟)
            let is_five_hour = limit_item
                .get("window")
                .and_then(|w| w.get("duration"))
                .and_then(|d| d.as_u64())
                .map(|d| d == 300)
                .unwrap_or(false);
            
            if is_five_hour {
                if let Some(detail) = limit_item.get("detail") {
                    five_hour_limit = detail.get("limit").and_then(parse_number);
                    five_hour_used = detail.get("used").and_then(parse_number);
                    five_hour_remaining = detail.get("remaining").and_then(parse_number);
                    five_hour_reset = detail
                        .get("resetTime")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                break;
            }
        }
    }

    // 3. 解析总额度
    let total_quota = data.get("totalQuota");
    let total_limit = total_quota.and_then(|t| t.get("limit")).and_then(parse_number);
    let total_remaining = total_quota.and_then(|t| t.get("remaining")).and_then(parse_number);

    // 卡片显示使用 5 小时额度，5 小时 used 缺失时显示 0/limit，不回退周额度
    let display_limit = five_hour_limit.or(weekly_limit);
    let display_used = if five_hour_limit.is_some() {
        five_hour_used.or(Some(0.0))
    } else {
        Some(0.0)
    };
    let display_remaining = if five_hour_limit.is_some() {
        five_hour_remaining.or(display_limit)
    } else {
        display_limit
    };
    let display_reset = five_hour_reset.clone().or(weekly_reset.clone());

    // 计算百分比
    let percentage = if let (Some(l), Some(u)) = (display_limit, display_used) {
        if l > 0.0 {
            Some((u / l * 100.0).min(100.0))
        } else {
            Some(0.0)
        }
    } else {
        None
    };

    // 构建完整的额度详情对象供前端使用
    let mut kimi_code_usage = serde_json::Map::new();
    
    // 5 小时详情：limit 存在即显示，used/remaining 缺失时补 0/limit
    if five_hour_limit.is_some() {
        let mut five_hour = serde_json::Map::new();
        five_hour.insert("limit".to_string(), serde_json::json!(five_hour_limit.unwrap_or(0.0)));
        five_hour.insert("used".to_string(), serde_json::json!(five_hour_used.unwrap_or(0.0)));
        five_hour.insert("remaining".to_string(), serde_json::json!(five_hour_remaining.unwrap_or(five_hour_limit.unwrap_or(0.0))));
        if let Some(reset) = &five_hour_reset {
            five_hour.insert("resetTime".to_string(), serde_json::json!(reset));
        }
        kimi_code_usage.insert("fiveHour".to_string(), serde_json::Value::Object(five_hour));
    }

    if let (Some(l), Some(u), Some(r)) = (weekly_limit, weekly_used, weekly_remaining) {
        let mut weekly = serde_json::Map::new();
        weekly.insert("limit".to_string(), serde_json::json!(l));
        weekly.insert("used".to_string(), serde_json::json!(u));
        weekly.insert("remaining".to_string(), serde_json::json!(r));
        if let Some(reset) = &weekly_reset {
            weekly.insert("resetTime".to_string(), serde_json::json!(reset));
        }
        kimi_code_usage.insert("weekly".to_string(), serde_json::Value::Object(weekly));
    }

    if let (Some(l), Some(r)) = (total_limit, total_remaining) {
        let mut monthly = serde_json::Map::new();
        monthly.insert("limit".to_string(), serde_json::json!(l));
        monthly.insert("remaining".to_string(), serde_json::json!(r));
        kimi_code_usage.insert("monthly".to_string(), serde_json::Value::Object(monthly));
    }

    // 将 kimiCodeUsage 注入到 limits 中（前端会读取）
    let mut limits_map = if let Some(serde_json::Value::Object(m)) = data.get("limits").cloned() {
        m
    } else {
        serde_json::Map::new()
    };
    limits_map.insert("_kimiCodeUsage".to_string(), serde_json::Value::Object(kimi_code_usage));
    let limits = Some(serde_json::Value::Object(limits_map));

    ProviderQuota {
        provider_id: id.to_string(),
        provider_name: name.to_string(),
        quota_type: "token_limit".to_string(),
        status: "success".to_string(),
        error_message: None,
        total_balance: None,
        available_balance: display_remaining,
        used_balance: display_used,
        currency: None,
        quota_percentage: percentage,
        quota_used: display_used,
        quota_limit: display_limit,
        reset_time: display_reset,
        daily_usage: None,
        weekly_usage: None,
        monthly_usage: None,
        spending_limit: None,
        limit_remaining: display_remaining,
        limits,
    }
}
