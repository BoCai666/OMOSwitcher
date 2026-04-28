// OpenCode Go 额度查询（网页抓取）
// OpenCode Go 无官方 API，通过抓取 Dashboard 页面获取用量
// 凭证保存在 ~/.config/omoswitcher/settings.json 的 openCodeGo 字段中
// 格式: { "openCodeGo": { "id": "workspace_id", "cookie": "auth_cookie_value" } }

use super::{ProviderInfo, ProviderQuota, error_quota};
use regex::Regex;

/// 查询 OpenCode Go 额度
/// 从 settings.json 读取 workspaceId 和 cookie
pub(crate) async fn query_opencode_go(provider: &ProviderInfo) -> ProviderQuota {
    // 从 OMOSwitcher 的 settings.json 读取凭证
    let (workspace_id, auth_cookie) = match read_opencode_go_settings() {
        Some((id, cookie)) if !id.is_empty() && !cookie.is_empty() => (id, cookie),
        _ => {
            return error_quota(
                &provider.id, &provider.name,
                "请点击卡片上的齿轮图标设置 Workspace ID 和 Cookie",
            );
        }
    };

    tracing::info!(
        "[OpenCode Go] 查询 workspace={}, cookie前8位={}",
        workspace_id,
        auth_cookie.chars().take(8).collect::<String>()
    );

    let client = match reqwest::Client::builder()
        .no_proxy()                         // 绕过系统代理（Tauri 环境避免代理干扰）
        .timeout(std::time::Duration::from_secs(30))  // 网页抓取允许更长超时
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("[OpenCode Go] 创建 HTTP 客户端失败: {}", e);
            return error_quota(&provider.id, &provider.name, &e.to_string());
        }
    };

    let url = format!("https://opencode.ai/workspace/{}/go", workspace_id);

    let result = client
        .get(&url)
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

/// 从 OMOSwitcher settings.json 读取 OpenCode Go 凭证
/// settings.json 路径: ~/.config/omoswitcher/settings.json
/// 字段: openCodeGo.id (workspaceId), openCodeGo.cookie (auth cookie)
fn read_opencode_go_settings() -> Option<(String, String)> {
    let home = dirs::home_dir()?;
    let settings_path = home.join(".config").join("omoswitcher").join("settings.json");
    if !settings_path.exists() {
        tracing::info!("[OpenCode Go] settings.json 不存在");
        return None;
    }
    let content = std::fs::read_to_string(&settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let ocg = json.get("openCodeGo")?;
    let id = ocg.get("id")?.as_str()?.to_string();
    let cookie = ocg.get("cookie")?.as_str()?.to_string();
    if id.is_empty() || cookie.is_empty() {
        return None;
    }
    tracing::info!("[OpenCode Go] 从 settings.json 读到 id={}, cookie长度={}", id, cookie.len());
    Some((id, cookie))
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
