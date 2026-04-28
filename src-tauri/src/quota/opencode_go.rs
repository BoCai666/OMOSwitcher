// OpenCode Go 额度查询（网页抓取）
// OpenCode Go 无官方 API，通过抓取 Dashboard 页面获取月度用量
// apiKey 格式: workspaceId|authCookie（用 | 分隔）
// authCookie 来源: 浏览器 F12 → Application → Cookies → opencode.ai → auth

use super::{ProviderInfo, ProviderQuota, build_client, error_quota};
use regex::Regex;

/// 查询 OpenCode Go 额度
/// apiKey 格式: workspace_id|auth_cookie
pub(crate) async fn query_opencode_go(provider: &ProviderInfo) -> ProviderQuota {
    // 从配置文件中解析凭证
    let parts: Vec<&str> = provider.api_key.splitn(2, '|').collect();
    let (workspace_id, auth_cookie) = match parts.as_slice() {
        [wid, cookie] if !wid.is_empty() && !cookie.is_empty() => (*wid, *cookie),
        _ => {
            return error_quota(
                &provider.id, &provider.name,
                "apiKey 格式需为 workspaceId|authCookie",
            );
        }
    };

    tracing::info!(
        "[OpenCode Go] 查询 workspace={}, cookie前8位={}",
        workspace_id,
        auth_cookie.chars().take(8).collect::<String>()
    );

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("[OpenCode Go] 创建 HTTP 客户端失败: {}", e);
            return error_quota(&provider.id, &provider.name, &e);
        }
    };

    let url = format!("https://opencode.ai/workspace/{}/go", workspace_id);

    let result = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Cookie", format!("auth={}", auth_cookie))
        .send()
        .await;

    match result {
        Ok(resp) => {
            let status_code = resp.status();
            tracing::info!("[OpenCode Go] HTTP 响应状态: {}", status_code);

            if !resp.status().is_success() {
                let error_body = resp.text().await.unwrap_or_default();
                let hint = if status_code.as_u16() == 302 || status_code.as_u16() == 401 {
                    "，Cookie 可能已过期，请重新获取"
                } else if status_code.as_u16() == 404 {
                    "，workspaceId 可能不正确"
                } else {
                    ""
                };
                tracing::warn!(
                    "[OpenCode Go] HTTP {} 错误: {}",
                    status_code,
                    &error_body[..error_body.len().min(200)]
                );
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}{}", status_code, hint));
            }

            match resp.text().await {
                Ok(html) => {
                    // 正则匹配 SolidJS SSR 水合数据：
                    // rollingUsage:$R[N]={status:"ok",resetInSec:8229,usagePercent:6}
                    let re_rolling = Regex::new(
                        r#"rollingUsage:\$R\[\d+\]=\{status:"ok",resetInSec:(\d+),usagePercent:(\d+)\}"#
                    ).unwrap();
                    let re_weekly = Regex::new(
                        r#"weeklyUsage:\$R\[\d+\]=\{status:"ok",resetInSec:(\d+),usagePercent:(\d+)\}"#
                    ).unwrap();
                    let re_monthly = Regex::new(
                        r#"monthlyUsage:\$R\[\d+\]=\{status:"ok",resetInSec:(\d+),usagePercent:(\d+)\}"#
                    ).unwrap();

                    let rolling = re_rolling.captures(&html).map(|c| {
                        (c[2].parse::<f64>().unwrap_or(0.0), c[1].parse::<i64>().unwrap_or(0))
                    });
                    let weekly = re_weekly.captures(&html).map(|c| {
                        (c[2].parse::<f64>().unwrap_or(0.0), c[1].parse::<i64>().unwrap_or(0))
                    });
                    let monthly = re_monthly.captures(&html).map(|c| {
                        (c[2].parse::<f64>().unwrap_or(0.0), c[1].parse::<i64>().unwrap_or(0))
                    });

                    match rolling {
                        Some((usage_pct, reset_secs)) => {
                            tracing::info!(
                                "[OpenCode Go] 解析成功: rolling={}%/{}s, weekly={:?}, monthly={:?}",
                                usage_pct, reset_secs, weekly, monthly
                            );

                            let reset_time = format_reset_countdown(reset_secs);

                            // 三维度详情 (供前端详情弹窗)
                            let limits = serde_json::json!([
                                {
                                    "type": "rolling",
                                    "label": "5小时滚动额度",
                                    "usagePercent": rolling.map(|(p, _)| p).unwrap_or(0.0),
                                    "resetInSec": rolling.map(|(_, r)| r).unwrap_or(0),
                                    "resetTime": rolling.map(|(_, r)| format_reset_countdown(r)).flatten(),
                                },
                                {
                                    "type": "weekly",
                                    "label": "周额度",
                                    "usagePercent": weekly.map(|(p, _)| p).unwrap_or(0.0),
                                    "resetInSec": weekly.map(|(_, r)| r).unwrap_or(0),
                                    "resetTime": weekly.map(|(_, r)| format_reset_countdown(r)).flatten(),
                                },
                                {
                                    "type": "monthly",
                                    "label": "月额度",
                                    "usagePercent": monthly.map(|(p, _)| p).unwrap_or(0.0),
                                    "resetInSec": monthly.map(|(_, r)| r).unwrap_or(0),
                                    "resetTime": monthly.map(|(_, r)| format_reset_countdown(r)).flatten(),
                                },
                            ]);

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
                                quota_percentage: Some(usage_pct),
                                quota_used: None,
                                quota_limit: None,
                                reset_time,
                                daily_usage: None,
                                weekly_usage: None,
                                monthly_usage: None,
                                spending_limit: None,
                                limit_remaining: None,
                                limits: Some(limits),
                            }
                        }
                        None => {
                            tracing::warn!("[OpenCode Go] 未在 HTML 中找到 rollingUsage 数据");
                            error_quota(
                                &provider.id, &provider.name,
                                "Dashboard 页面中未找到用量数据",
                            )
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[OpenCode Go] 读取响应体失败: {}", e);
                    error_quota(&provider.id, &provider.name, &format!("读取页面失败: {}", e))
                }
            }
        }
        Err(e) => {
            tracing::error!("[OpenCode Go] 请求失败: {}", e);
            error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e))
        }
    }
}

/// 将重置秒数格式化为中文倒计时
fn format_reset_countdown(secs: i64) -> Option<String> {
    if secs <= 0 {
        return None;
    }

    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    if days > 0 {
        if hours > 0 {
            Some(format!("{}天{}小时后重置", days, hours))
        } else {
            Some(format!("{}天后重置", days))
        }
    } else if hours > 0 {
        if minutes > 0 {
            Some(format!("{}小时{}分钟后重置", hours, minutes))
        } else {
            Some(format!("{}小时后重置", hours))
        }
    } else if minutes > 0 {
        Some(format!("{}分钟后重置", minutes))
    } else {
        Some("即将重置".to_string())
    }
}
