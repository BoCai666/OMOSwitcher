// DeepSeek 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 DeepSeek 额度
pub(crate) async fn query_deepseek(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    let result = client
        .get("https://api.deepseek.com/user/balance")
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
                    // balance_infos 是数组，取第一个
                    let balance_info = data.get("balance_infos")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first());

                    match balance_info {
                        Some(info) => {
                            let total = info.get("total_balance").and_then(|v| v.as_f64());
                            let currency = info.get("currency").and_then(|v| v.as_str()).unwrap_or("CNY").to_string();
                            let available = info.get("granted_balance").and_then(|v| v.as_f64())
                                .or_else(|| info.get("available_balance").and_then(|v| v.as_f64()));
                            let used = match (total, available) {
                                (Some(t), Some(a)) => Some(t - a),
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
                                used_balance: used,
                                currency: Some(currency),
                                quota_percentage: None,
                                quota_used: used,
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
                        None => error_quota(&provider.id, &provider.name, "响应中未找到 balance_infos"),
                    }
                }
                Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    }
}
