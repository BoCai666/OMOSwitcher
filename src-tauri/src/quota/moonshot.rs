// Moonshot (Kimi) 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 Moonshot (Kimi) 额度
pub(crate) async fn query_moonshot(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    let result = client
        .get("https://api.moonshot.cn/v1/users/me/balance")
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
                    let data_obj = data.get("data");
                    let available = data_obj.and_then(|d| d.get("available_balance")).and_then(|v| v.as_f64());
                    let voucher = data_obj.and_then(|d| d.get("voucher_balance")).and_then(|v| v.as_f64());
                    let cash = data_obj.and_then(|d| d.get("cash_balance")).and_then(|v| v.as_f64());
                    let total = match (voucher, cash) {
                        (Some(v), Some(c)) => Some(v + c),
                        (Some(v), None) => Some(v),
                        (None, Some(c)) => Some(c),
                        _ => None,
                    };

                    ProviderQuota {
                        provider_id: provider.id.clone(),
                        provider_name: provider.name.clone(),
                        quota_type: "balance".to_string(),
                        status: "success".to_string(),
                        error_message: None,
                        total_balance: total,
                        available_balance: available,
                        used_balance: None,
                        currency: Some("CNY".to_string()),
                        quota_percentage: None,
                        quota_used: None,
                        quota_limit: total,
                        reset_time: None,
                        daily_usage: None,
                        weekly_usage: None,
                        monthly_usage: None,
                        spending_limit: None,
                        limit_remaining: available,
                        limits: None,
                    }
                }
                Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    }
}
