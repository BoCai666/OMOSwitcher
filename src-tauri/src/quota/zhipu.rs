// 智谱 (Zhipu/GLM) 额度查询 + 用量详情

use serde::Serialize;
use super::{ProviderInfo, ProviderQuota, build_client, error_quota, get_opencode_config_path, read_providers, read_auth_providers};

/// 查询智谱 (GLM/Zhipu) 额度
/// 支持 z.ai 和 open.bigmodel.cn 两个平台，自动 fallback
pub(crate) async fn query_zhipu(provider: &ProviderInfo) -> ProviderQuota {
    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return error_quota(&provider.id, &provider.name, &e),
    };

    // 根据 base_url 判断优先使用哪个平台
    // z.ai 和 bigmodel.cn 使用相同的 /api/monitor/usage/quota/limit 接口
    let (primary_url, fallback_url) = match &provider.base_url {
        Some(base) if base.contains("bigmodel") => {
            // bigmodel 优先，fallback 到 z.ai
            let primary = "https://open.bigmodel.cn/api/monitor/usage/quota/limit".to_string();
            let fallback = "https://api.z.ai/api/monitor/usage/quota/limit".to_string();
            (primary, Some(fallback))
        }
        _ => {
            // z.ai 优先（含 base_url 包含 "z.ai" 或无 base_url 的情况），fallback 到 bigmodel
            let primary = "https://api.z.ai/api/monitor/usage/quota/limit".to_string();
            let fallback = "https://open.bigmodel.cn/api/monitor/usage/quota/limit".to_string();
            (primary, Some(fallback))
        }
    };

    let result = client
        .get(&primary_url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    match result {
        Ok(resp) => {
            if !resp.status().is_success() {
                // 主 URL 失败，尝试 fallback
                if let Some(ref fallback) = fallback_url {
                    return query_zhipu_url(&client, provider, fallback).await;
                }
                let status = resp.status();
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}", status));
            }
            parse_zhipu_response(provider, resp).await
        }
        Err(_) => {
            // 网络错误，尝试 fallback
            if let Some(ref fallback) = fallback_url {
                return query_zhipu_url(&client, provider, fallback).await;
            }
            error_quota(&provider.id, &provider.name, "请求失败")
        }
    }
}

/// 查询指定 URL 的智谱额度
pub(crate) async fn query_zhipu_url(client: &reqwest::Client, provider: &ProviderInfo, url: &str) -> ProviderQuota {
    let result = client
        .get(url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .send()
        .await;

    match result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                return error_quota(&provider.id, &provider.name, &format!("HTTP {}", status));
            }
            parse_zhipu_response(provider, resp).await
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("请求失败: {}", e)),
    }
}

/// 解析智谱 API 响应
/// z.ai 和 bigmodel.cn 使用统一的 /api/monitor/usage/quota/limit 接口
/// 响应格式: { success: true, data: { level: "lite|standard|pro", limits: [{ type, percentage, nextResetTime }] } }
pub(crate) async fn parse_zhipu_response(provider: &ProviderInfo, resp: reqwest::Response) -> ProviderQuota {
    match resp.json::<serde_json::Value>().await {
        Ok(data) => {
            let data_obj = match data.get("data") {
                Some(d) => d,
                None => return error_quota(&provider.id, &provider.name, "响应缺少 data 字段"),
            };

            // 提取套餐等级（暂保留用于后续前端展示）
            let _level = data_obj
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // 从 limits 数组中查找 TOKENS_LIMIT 类型的配额
            let mut quota_percentage: Option<f64> = None;
            let mut reset_time: Option<String> = None;
            let limits_value = data_obj.get("limits").cloned();

            if let Some(limits) = data_obj.get("limits").and_then(|v| v.as_array()) {
                for lim in limits {
                    let lim_type = lim.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if lim_type == "TOKENS_LIMIT" {
                        quota_percentage = lim.get("percentage").and_then(|v| v.as_f64());
                        // 解析 nextResetTime（可能是毫秒时间戳数字或 ISO 字符串）
                        if let Some(reset) = lim.get("nextResetTime") {
                            if let Some(s) = reset.as_str() {
                                reset_time = Some(s.to_string());
                            } else if let Some(ms) = reset.as_i64() {
                                // 毫秒时间戳转为 ISO 8601 字符串
                                let secs = ms / 1000;
                                if let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(secs) {
                                    reset_time = Some(format!(
                                        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                                        dt.year(), dt.month() as u8, dt.day(),
                                        dt.hour(), dt.minute(), dt.second()
                                    ));
                                }
                            }
                        }
                        break;
                    }
                }
            }

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
                quota_percentage,
                quota_used: None,
                quota_limit: None,
                reset_time,
                daily_usage: None,
                weekly_usage: None,
                monthly_usage: None,
                spending_limit: None,
                limit_remaining: quota_percentage.map(|p| 100.0 - p),
                limits: limits_value,
            }
        }
        Err(e) => error_quota(&provider.id, &provider.name, &format!("解析响应失败: {}", e)),
    }
}

// ==================== 智谱用量详情查询 ====================

/// 智谱用量详情返回格式
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ZhipuUsageDetails {
    provider_id: String,
    /// 模型用量列表
    model_usage: Vec<ModelUsageItem>,
    /// 工具用量列表
    tool_usage: Vec<ToolUsageItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelUsageItem {
    model_code: String,
    usage: f64,
    input_tokens: Option<f64>,
    output_tokens: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolUsageItem {
    tool_name: String,
    usage: f64,
}

/// 查询智谱供应商的模型用量和工具用量详情
#[tauri::command]
pub async fn fetch_zhipu_usage_details(provider_id: String) -> Result<String, String> {
    let client = build_client().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 从 opencode.json 和 auth.json 查找该供应商的 apiKey 和 baseURL
    let provider = find_provider_info(&provider_id)?;
    let (base, fallback_base) = match &provider.base_url {
        Some(b) if b.contains("bigmodel") => {
            ("https://open.bigmodel.cn".to_string(), Some("https://api.z.ai".to_string()))
        }
        _ => {
            ("https://api.z.ai".to_string(), Some("https://open.bigmodel.cn".to_string()))
        }
    };

    // 查询最近 7 天的用量
    let now = time::OffsetDateTime::now_utc();
    let start = now - time::Duration::days(7);
    // 手动格式化: "2026-04-08+00:00:00"
    let fmt_time = |dt: time::OffsetDateTime| -> String {
        format!(
            "{:04}-{:02}-{:02}+{:02}:{:02}:{:02}",
            dt.year(), dt.month() as u8, dt.day(),
            dt.hour(), dt.minute(), dt.second()
        )
    };
    let start_str = fmt_time(start);
    let end_str = fmt_time(now);

    let auth_header = format!("Bearer {}", provider.api_key);

    // 并发请求 model-usage 和 tool-usage
    let model_usage_url = format!(
        "{}/api/monitor/usage/model-usage?startTime={}&endTime={}",
        base, start_str, end_str
    );
    let tool_usage_url = format!(
        "{}/api/monitor/usage/tool-usage?startTime={}&endTime={}",
        base, start_str, end_str
    );

    let model_resp = client.get(&model_usage_url)
        .header("Authorization", &auth_header)
        .send()
        .await;

    // 如果主 URL 失败，尝试 fallback
    let model_data = match model_resp {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<serde_json::Value>().await.ok()
        }
        _ => {
            if let Some(ref fb) = fallback_base {
                let fb_url = format!(
                    "{}/api/monitor/usage/model-usage?startTime={}&endTime={}",
                    fb, start_str, end_str
                );
                let fb_resp = client.get(&fb_url)
                    .header("Authorization", &auth_header)
                    .send()
                    .await;
                match fb_resp {
                    Ok(r) if r.status().is_success() => r.json::<serde_json::Value>().await.ok(),
                    _ => None,
                }
            } else {
                None
            }
        }
    };

    let tool_resp = client.get(&tool_usage_url)
        .header("Authorization", &auth_header)
        .send()
        .await;

    let tool_data = match tool_resp {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<serde_json::Value>().await.ok()
        }
        _ => {
            if let Some(ref fb) = fallback_base {
                let fb_url = format!(
                    "{}/api/monitor/usage/tool-usage?startTime={}&endTime={}",
                    fb, start_str, end_str
                );
                let fb_resp = client.get(&fb_url)
                    .header("Authorization", &auth_header)
                    .send()
                    .await;
                match fb_resp {
                    Ok(r) if r.status().is_success() => r.json::<serde_json::Value>().await.ok(),
                    _ => None,
                }
            } else {
                None
            }
        }
    };

    // 解析模型用量和工具用量
    let model_usage = parse_model_usage(&model_data);
    let tool_usage = parse_tool_usage(&tool_data);

    let details = ZhipuUsageDetails {
        provider_id: provider_id.clone(),
        model_usage,
        tool_usage,
    };

    serde_json::to_string(&details)
        .map_err(|e| format!("序列化用量详情失败: {}", e))
}

/// 查找供应商信息（从 opencode.json + auth.json）
fn find_provider_info(provider_id: &str) -> Result<ProviderInfo, String> {
    let config_path = get_opencode_config_path()?;
    if config_path.exists() {
        if let Ok(providers) = read_providers(&config_path) {
            if let Some(p) = providers.into_iter().find(|p| p.id == provider_id) {
                return Ok(p);
            }
        }
    }

    let auth_providers = read_auth_providers();
    if let Some(p) = auth_providers.into_iter().find(|p| p.id == provider_id) {
        return Ok(p);
    }

    Err(format!("未找到供应商 {} 的认证信息", provider_id))
}

/// 解析模型用量响应
fn parse_model_usage(data: &Option<serde_json::Value>) -> Vec<ModelUsageItem> {
    let data = match data {
        Some(d) => d,
        None => return Vec::new(),
    };

    let items = match data.get("data").and_then(|d| d.get("modelUsage")).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    items.iter().filter_map(|item| {
        let model_code = item.get("modelCode").and_then(|v| v.as_str())?.to_string();
        let usage = item.get("usage").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let input_tokens = item.get("inputTokens").and_then(|v| v.as_f64());
        let output_tokens = item.get("outputTokens").and_then(|v| v.as_f64());
        Some(ModelUsageItem { model_code, usage, input_tokens, output_tokens })
    }).collect()
}

/// 解析工具用量响应
fn parse_tool_usage(data: &Option<serde_json::Value>) -> Vec<ToolUsageItem> {
    let data = match data {
        Some(d) => d,
        None => return Vec::new(),
    };

    let items = match data.get("data").and_then(|d| d.as_array()) {
        Some(arr) => arr,
        None => match data.get("data").and_then(|d| d.get("toolUsage")).and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Vec::new(),
        }
    };

    items.iter().filter_map(|item| {
        let tool_name = item.get("toolName")
            .or_else(|| item.get("name"))
            .and_then(|v| v.as_str())?.to_string();
        let usage = item.get("usage")
            .or_else(|| item.get("count"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        Some(ToolUsageItem { tool_name, usage })
    }).collect()
}
