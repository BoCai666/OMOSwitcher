// SiliconFlow 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 SiliconFlow 额度
pub(crate) async fn query_siliconflow(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    let result = client
        .get("https://api.siliconflow.cn/v1/user/info")
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
                    let total = data_obj.and_then(|d| d.get("totalBalance")).and_then(|v| v.as_f64());
                    let charge = data_obj.and_then(|d| d.get("chargeBalance")).and_then(|v| v.as_f64());
                    let available = data_obj.and_then(|d| d.get("balance")).and_then(|v| v.as_f64());
                    let used = match (total, charge) {
                        (Some(t), Some(c)) => Some(t - c),
                        _ => None,
                    };

                    ProviderQuota {
                        provider_id: provider.id.clone(),
                        provider_name: provider.name.clone(),
                        quota_type: "balance".to_string(),
                        status: "success".to_string(),
                        error_message: None,
                        total_balance: total,
                        available_balance: charge.or(available),
                        used_balance: used,
                        currency: Some("CNY".to_string()),
                        quota_percentage: None,
                        quota_used: used,
                        quota_limit: total,
                        reset_time: None,
                        daily_usage: None,
                        weekly_usage: None,
                        monthly_usage: None,
                        spending_limit: None,
                        limit_remaining: charge.or(available),
                        limits: None,
                    }
                }
                Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    }
}
