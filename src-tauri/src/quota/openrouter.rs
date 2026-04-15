// OpenRouter 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 OpenRouter 额度
pub(crate) async fn query_openrouter(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    let result = client
        .get("https://openrouter.ai/api/v1/key")
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    match result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}", status));
            }
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let limit = data.get("limit").and_then(|v| v.as_f64());
                    let limit_remaining = data.get("limit_remaining").and_then(|v| v.as_f64());
                    let usage = data.get("usage").and_then(|v| v.as_f64());
                    let daily = data.get("usage_daily").and_then(|v| v.as_f64());
                    let weekly = data.get("usage_weekly").and_then(|v| v.as_f64());
                    let monthly = data.get("usage_monthly").and_then(|v| v.as_f64());

                    // 计算已用百分比
                    let quota_percentage = match (usage, limit) {
                        (Some(u), Some(l)) if l > 0.0 => Some((u / l) * 100.0),
                        _ => None,
                    };

                    ProviderQuota {
                        provider_id: provider.id.clone(),
                        provider_name: provider.name.clone(),
                        quota_type: "balance".to_string(),
                        status: "success".to_string(),
                        error_message: None,
                        total_balance: limit,
                        available_balance: limit_remaining,
                        used_balance: usage,
                        currency: Some("USD".to_string()),
                        quota_percentage,
                        quota_used: usage,
                        quota_limit: limit,
                        reset_time: None,
                        daily_usage: daily,
                        weekly_usage: weekly,
                        monthly_usage: monthly,
                        spending_limit: limit,
                        limit_remaining,
                        limits: None,
                    }
                }
                Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    }
}
