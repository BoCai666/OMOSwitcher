// MiniMax 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 MiniMax Coding Plan 额度
/// 接口: GET /v1/api/openplatform/coding_plan/remains
/// 国际版: https://api.minimax.io  国内版: https://api.minimaxi.com
/// 响应中 current_interval_usage_count 实际是**剩余量**，不是已用量
pub(crate) async fn query_minimax(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    // 根据 base_url 判断国际版/国内版
    let (primary_url, fallback_url) = match &provider.base_url {
        Some(b) if b.contains("minimaxi.com") || b.contains("minimaxi") => {
            // 国内版优先
            let primary = "https://api.minimaxi.com/v1/api/openplatform/coding_plan/remains".to_string();
            let fallback = "https://api.minimax.io/v1/api/openplatform/coding_plan/remains".to_string();
            (primary, Some(fallback))
        }
        _ => {
            // 国际版优先（含 .io 域名或无 base_url）
            let primary = "https://api.minimax.io/v1/api/openplatform/coding_plan/remains".to_string();
            let fallback = "https://api.minimaxi.com/v1/api/openplatform/coding_plan/remains".to_string();
            (primary, Some(fallback))
        }
    };

    let result = client
        .get(&primary_url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .header("Content-Type", "application/json")
        .send()
        .await;

    let resp = match result {
        Ok(r) => r,
        Err(_) => {
            // 网络错误，尝试 fallback
            if let Some(ref fb) = fallback_url {
                match client.get(fb)
                    .header("Authorization", format!("Bearer {}", provider.api_key))
                    .header("Content-Type", "application/json")
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => return error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
                }
            } else {
                return error_quota(&provider.id, &provider.name, "请求失败");
            }
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        // 尝试 fallback
        if let Some(ref fb) = fallback_url {
            let fb_result = client.get(fb)
                .header("Authorization", format!("Bearer {}", provider.api_key))
                .header("Content-Type", "application/json")
                .send()
                .await;
            match fb_result {
                Ok(r) if r.status().is_success() => return parse_minimax_response(provider, r).await,
                _ => return error_quota(&provider.id, &provider.name, &format!("HTTP {}", status)),
            }
        }
        return error_quota(&provider.id, &provider.name, &format!("HTTP {}", status));
    }

    parse_minimax_response(provider, resp).await
}

/// 解析 MiniMax Coding Plan API 响应
/// 响应格式:
/// {
///   "base_resp": { "status_code": 0 },
///   "data": {
///     "model_remains": [{
///       "model_name": "MiniMax-M2.7",
///       "current_interval_total_count": 1500,
///       "current_interval_usage_count": 1444,  // 实际是**剩余量**
///       "current_weekly_total_count": 15000,
///       "current_weekly_usage_count": 14444,   // 实际是**剩余量**
///       "start_time": ..., "end_time": ..., "remains_time": ...
///     }]
///   }
/// }
pub(crate) async fn parse_minimax_response(provider: &ProviderInfo, resp: reqwest::Response) -> ProviderQuota {
    match resp.json::<serde_json::Value>().await {
        Ok(data) => {
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
                    return error_quota(&provider.id, &provider.name, msg);
                }
            }

            // 提取 data 字段
            let data_obj = data.get("data").unwrap_or(&data);

            // 从 model_remains 数组中找文本模型（MiniMax-M 开头且有 total_count > 0）
            let model_remains = match data_obj.get("model_remains").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => return error_quota(&provider.id, &provider.name, "未找到 model_remains 数据"),
            };

            // 找到第一个有效的文本模型
            let chat_model = model_remains.iter().find(|m| {
                let name = m.get("model_name").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let total = m.get("current_interval_total_count")
                    .or_else(|| m.get("currentIntervalTotalCount"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                name.starts_with("minimax-m") && total > 0.0
            }).or_else(|| model_remains.iter().find(|m| {
                m.get("current_interval_total_count")
                    .or_else(|| m.get("currentIntervalTotalCount"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) > 0.0
            }));

            let model = match chat_model {
                Some(m) => m,
                None => return error_quota(&provider.id, &provider.name, "未找到有效的模型配额数据"),
            };

            // 提取 5 小时窗口配额
            let total = model.get("current_interval_total_count")
                .or_else(|| model.get("currentIntervalTotalCount"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            // current_interval_usage_count 实际是剩余量（MiniMax 命名反直觉）
            let remaining_count = model.get("current_interval_usage_count")
                .or_else(|| model.get("currentIntervalUsageCount"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let used = (total - remaining_count).max(0.0);
            let percentage = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };

            // 解析重置时间
            // end_time 可能是秒级或毫秒级时间戳，remains_time 是剩余秒数
            let reset_time = {
                let from_end_time = model.get("end_time")
                    .or_else(|| model.get("endTime"))
                    .and_then(|v| v.as_i64())
                    .and_then(|ts| {
                        if ts <= 0 { return None; }
                        // 判断秒级还是毫秒级
                        let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
                        time::OffsetDateTime::from_unix_timestamp(secs).ok().map(|dt| {
                            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                                dt.year(), dt.month() as u8, dt.day(),
                                dt.hour(), dt.minute(), dt.second())
                        })
                    });
                from_end_time
            };

            // 提取周配额
            let weekly_total = model.get("current_weekly_total_count")
                .or_else(|| model.get("currentWeeklyTotalCount"))
                .and_then(|v| v.as_f64());
            let weekly_remaining = model.get("current_weekly_usage_count")
                .or_else(|| model.get("currentWeeklyUsageCount"))
                .and_then(|v| v.as_f64());

            ProviderQuota {
                provider_id: provider.id.clone(),
                provider_name: provider.name.clone(),
                quota_type: "token_limit".to_string(),
                status: "success".to_string(),
                error_message: None,
                total_balance: None,
                available_balance: None,
                used_balance: None,
                currency: None,
                quota_percentage: Some(percentage),
                quota_used: Some(used),
                quota_limit: Some(total),
                reset_time,
                daily_usage: None,
                weekly_usage: weekly_total.map(|t| t - weekly_remaining.unwrap_or(0.0)),
                monthly_usage: weekly_total,
                spending_limit: None,
                limit_remaining: Some(remaining_count),
                limits: None,
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
    }
}
