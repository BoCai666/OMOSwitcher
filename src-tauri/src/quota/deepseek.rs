// DeepSeek 额度查询

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};

/// 查询 DeepSeek 额度
pub(crate) async fn query_deepseek(provider: &ProviderInfo) -> ProviderQuota {
    // 脱敏显示 apiKey 前 8 个字符
    let key_preview: String = provider.api_key.chars().take(8).chain(std::iter::repeat('*')).take(12).collect();
    tracing::info!("[DeepSeek额度] 开始查询 provider_id={}, name={}, apiKey={}", provider.id, provider.name, key_preview);

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("[DeepSeek额度] 创建 HTTP 客户端失败: {}", e);
            return error_quota(&provider.id, &provider.name, &e);
        }
    };

    let result = client
        .get("https://api.deepseek.com/user/balance")
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    match result {
        Ok(resp) => {
            let status_code = resp.status();
            tracing::info!("[DeepSeek额度] HTTP 响应状态: {}", status_code);

            if !resp.status().is_success() {
                // 读取错误响应体
                let error_body = resp.text().await.unwrap_or_default();
                tracing::warn!("[DeepSeek额度] HTTP {} 错误响应体: {}", status_code, error_body.chars().take(300).collect::<String>());
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}: {}", status_code, error_body.chars().take(100).collect::<String>()));
            }
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    // 打印完整响应（截断防过长）
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    tracing::info!("[DeepSeek额度] API 响应: {}", data_str.chars().take(500).collect::<String>());

                    // balance_infos 是数组，取第一个
                    let balance_info = data.get("balance_infos")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first());

                    match balance_info {
                        Some(info) => {
                            // DeepSeek API 返回的数值字段是 JSON 字符串（如 "8.26"），
                            // 而非 JSON number，需先用 as_str() 再 parse
                            let total = info.get("total_balance")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<f64>().ok())
                                .or_else(|| info.get("total_balance").and_then(|v| v.as_f64()));
                            let currency = info.get("currency").and_then(|v| v.as_str()).unwrap_or("CNY").to_string();

                            tracing::info!("[DeepSeek额度] 解析结果: total={:?}, currency={}", total, currency);

                            // DeepSeek 没有已用额度查询 API，因此：
                            // - available_balance 直接使用 total_balance（全部余额可用）
                            // - used_balance 无法计算，设为 None
                            ProviderQuota {
                                provider_id: provider.id.clone(),
                                provider_name: provider.name.clone(),
                                quota_type: "balance".to_string(),
                                status: "success".to_string(),
                                error_message: None,
                                total_balance: total,
                                available_balance: total,
                                used_balance: None,
                                currency: Some(currency),
                                quota_percentage: None,
                                quota_used: None,
                                quota_limit: total,
                                reset_time: None,
                                daily_usage: None,
                                weekly_usage: None,
                                monthly_usage: None,
                                spending_limit: None,
                                limit_remaining: total,
                                limits: None,
                            }
                        }
                        None => {
                            tracing::warn!("[DeepSeek额度] 响应中未找到 balance_infos 数组");
                            error_quota(&provider.id, &provider.name, "响应中未找到 balance_infos")
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[DeepSeek额度] JSON 解析失败: {}", e);
                    error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e))
                }
            }
        }
        Err(e) => {
            tracing::error!("[DeepSeek额度] 请求失败: {}", e);
            error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e))
        }
    }
}
