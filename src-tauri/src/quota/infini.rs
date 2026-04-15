// 无问芯穹 (Infini) 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询无问芯穹 (Infini) Coding Plan 额度
/// 接口: GET https://cloud.infini-ai.com/maas/coding/usage
/// 响应: {"5_hour":{"quota":5000,"used":0,"remain":5000},"7_day":{...},"30_day":{...}}
pub(crate) async fn query_infini(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    let result = client
        .get("https://cloud.infini-ai.com/maas/coding/usage")
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    let resp = match result {
        Ok(r) => r,
        Err(e) => return error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    };

    if !resp.status().is_success() {
        return error_quota(&provider.id, &provider.name, &format!("HTTP {}", resp.status()));
    }

    match resp.json::<serde_json::Value>().await {
        Ok(data) => {
            // 解析 5 小时窗口
            let hour5 = data.get("5_hour");
            let quota = hour5.and_then(|h| h.get("quota")).and_then(|v| v.as_f64());
            let used = hour5.and_then(|h| h.get("used")).and_then(|v| v.as_f64());
            let remain = hour5.and_then(|h| h.get("remain")).and_then(|v| v.as_f64());

            let (quota_val, used_val, remain_val) = match (quota, used, remain) {
                (Some(q), Some(u), Some(r)) => (q, u, r),
                _ => return error_quota(&provider.id, &provider.name, "响应缺少 5_hour 配额数据"),
            };

            let percentage = if quota_val > 0.0 { (used_val / quota_val) * 100.0 } else { 0.0 };

            // 解析 7 天和 30 天数据
            let day7 = data.get("7_day");
            let weekly_used = day7.and_then(|d| d.get("used")).and_then(|v| v.as_f64());
            let weekly_quota = day7.and_then(|d| d.get("quota")).and_then(|v| v.as_f64());

            let day30 = data.get("30_day");
            let monthly_used = day30.and_then(|d| d.get("used")).and_then(|v| v.as_f64());
            let monthly_quota = day30.and_then(|d| d.get("quota")).and_then(|v| v.as_f64());

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
                quota_used: Some(used_val),
                quota_limit: Some(quota_val),
                reset_time: Some("5h-rolling".to_string()),
                daily_usage: None,
                weekly_usage: weekly_used,
                monthly_usage: monthly_used.or(monthly_quota),
                spending_limit: weekly_quota.or(monthly_quota),
                limit_remaining: Some(remain_val),
                limits: None,
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
    }
}
