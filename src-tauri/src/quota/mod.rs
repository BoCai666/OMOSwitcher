// 供应商额度查询模块
// 并发查询各供应商 API 获取账户余额/配额信息

mod openrouter;
mod deepseek;
mod siliconflow;
pub mod zhipu;
mod infini;
mod minimax;
mod moonshot;
mod kimi_code;
mod opencode_go;

use serde::Serialize;

// 统一额度返回格式
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderQuota {
    pub(crate) provider_id: String,
    pub(crate) provider_name: String,
    pub(crate) quota_type: String,         // "balance" | "token_limit" | "unsupported"
    pub(crate) status: String,             // "success" | "error"
    pub(crate) error_message: Option<String>,

    // 余额型字段
    pub(crate) total_balance: Option<f64>,
    pub(crate) available_balance: Option<f64>,
    pub(crate) used_balance: Option<f64>,
    pub(crate) currency: Option<String>,

    // 配额型字段
    pub(crate) quota_percentage: Option<f64>,
    pub(crate) quota_used: Option<f64>,
    pub(crate) quota_limit: Option<f64>,
    pub(crate) reset_time: Option<String>,

    // OpenRouter 专用
    pub(crate) daily_usage: Option<f64>,
    pub(crate) weekly_usage: Option<f64>,
    pub(crate) monthly_usage: Option<f64>,
    pub(crate) spending_limit: Option<f64>,
    pub(crate) limit_remaining: Option<f64>,

    // 智谱 (Zhipu/GLM) 专用 - 完整 limits 数组，供详情弹窗使用
    pub(crate) limits: Option<serde_json::Value>,
}

// 从 opencode.json 中提取的供应商信息
#[derive(Clone)]
pub(crate) struct ProviderInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) api_key: String,
    pub(crate) base_url: Option<String>,
}

/// 获取 opencode.json 路径
pub(crate) fn get_opencode_config_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".config").join("opencode").join("opencode.json"))
}

/// 获取 auth.json 路径
/// ~/.local/share/opencode/auth.json
pub(crate) fn get_auth_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".local").join("share").join("opencode").join("auth.json"))
}

/// 读取 opencode.json 中配置了 apiKey 的供应商列表
pub(crate) fn read_providers(config_path: &std::path::Path) -> Result<Vec<ProviderInfo>, String> {
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

    let provider_obj = json.get("provider")
        .and_then(|p| p.as_object())
        .ok_or("opencode.json 中未找到 provider 字段")?;

    let mut providers = Vec::new();
    for (id, config) in provider_obj {
        // 检查 apiKey 或 options.apiKey
        let api_key = config.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                config.get("options")
                    .and_then(|o| o.get("apiKey"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        let api_key = match api_key {
            Some(k) if !k.is_empty() => k,
            _ => continue,
        };

        let name = config.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();

        let base_url = config.get("options")
            .and_then(|o| o.get("baseURL"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                config.get("baseURL")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        providers.push(ProviderInfo {
            id: id.clone(),
            name,
            api_key,
            base_url,
        });
    }

    Ok(providers)
}

/// 创建 HTTP 客户端（10秒超时）
pub(crate) fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

/// 构建错误状态的 ProviderQuota
pub(crate) fn error_quota(id: &str, name: &str, msg: &str) -> ProviderQuota {
    ProviderQuota {
        provider_id: id.to_string(),
        provider_name: name.to_string(),
        quota_type: "unsupported".to_string(),
        status: "error".to_string(),
        error_message: Some(msg.to_string()),
        total_balance: None,
        available_balance: None,
        used_balance: None,
        currency: None,
        quota_percentage: None,
        quota_used: None,
        quota_limit: None,
        reset_time: None,
        daily_usage: None,
        weekly_usage: None,
        monthly_usage: None,
        spending_limit: None,
        limit_remaining: None,
        limits: None,
    }
}

/// 构建不支持状态的 ProviderQuota
pub(crate) fn unsupported_quota(id: &str, name: &str) -> ProviderQuota {
    ProviderQuota {
        provider_id: id.to_string(),
        provider_name: name.to_string(),
        quota_type: "unsupported".to_string(),
        status: "success".to_string(),
        error_message: None,
        total_balance: None,
        available_balance: None,
        used_balance: None,
        currency: None,
        quota_percentage: None,
        quota_used: None,
        quota_limit: None,
        reset_time: None,
        daily_usage: None,
        weekly_usage: None,
        monthly_usage: None,
        spending_limit: None,
        limit_remaining: None,
        limits: None,
    }
}

/// 根据 provider_id 判断供应商类型并查询额度
async fn query_provider(provider: &ProviderInfo) -> ProviderQuota {
    let id_raw = provider.id.to_lowercase();
    let id_lower = id_raw.replace('-', "").replace('_', "");

    // 同时检查 id 和 baseURL 来判断供应商类型
    let base_lower = provider.base_url
        .as_ref()
        .map(|u| u.to_lowercase())
        .unwrap_or_default();

    tracing::info!("[额度调度] 匹配 provider: id={}, baseURL={}",
        provider.id,
        provider.base_url.as_deref().unwrap_or("无"));

    // 优先按 id 匹配 - 注意：先匹配更具体的模式
    // Kimi Code (kimi-for-coding) 必须在 moonshot (kimi) 之前匹配
    if id_raw == "kimi-for-coding" || id_raw.contains("kimi-code") || id_lower.contains("kimicode") {
        return kimi_code::query_kimi_code(provider).await;
    }
    if id_lower == "openrouter" || id_lower.contains("openrouter") {
        return openrouter::query_openrouter(provider).await;
    }
    if id_lower == "deepseek" || id_lower.contains("deepseek") {
        tracing::info!("[额度调度] {} → 匹配到 DeepSeek", provider.id);
        return deepseek::query_deepseek(provider).await;
    }
    if id_lower.contains("opencode") && id_lower.contains("go") {
        tracing::info!("[额度调度] {} → 匹配到 OpenCode Go", provider.id);
        return opencode_go::query_opencode_go(provider).await;
    }
    if id_lower.contains("silicon") {
        return siliconflow::query_siliconflow(provider).await;
    }
    if id_lower.contains("zhipu") || id_lower.contains("glm") {
        return zhipu::query_zhipu(provider).await;
    }
    if id_lower.contains("moonshot") || id_lower == "kimi" || id_lower.contains("moonshotai") {
        return moonshot::query_moonshot(provider).await;
    }
    if id_lower.contains("minimax") {
        return minimax::query_minimax(provider).await;
    }
    if id_lower.contains("infini") || id_lower.contains("wuwen") {
        return infini::query_infini(provider).await;
    }

    // id 未匹配时，按 baseURL 匹配（自定义供应商可能 id 不含供应商名）
    if base_lower.contains("kimi.com/coding") || base_lower.contains("api.kimi.com") {
        return kimi_code::query_kimi_code(provider).await;
    }
    if base_lower.contains("infini-ai.com") {
        return infini::query_infini(provider).await;
    }
    if base_lower.contains("minimax") {
        return minimax::query_minimax(provider).await;
    }
    if base_lower.contains("z.ai") || base_lower.contains("bigmodel") {
        return zhipu::query_zhipu(provider).await;
    }

    unsupported_quota(&provider.id, &provider.name)
}

/// 从 auth.json 读取供应商信息
/// auth.json 结构为 Record<string, { type: "api"|"oauth"|"wellknown", key?: string, ... }>
/// 仅提取 type 为 "api" 或 "wellknown" 且包含 key 字段的条目
pub(crate) fn read_auth_providers() -> Vec<ProviderInfo> {
    let auth_path = match get_auth_path() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    if !auth_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&auth_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // auth.json 是 Record<string, AuthInfo> 对象格式
    let Some(obj) = json.as_object() else {
        return Vec::new();
    };

    let mut providers = Vec::new();
    for (id, info) in obj {
        // 只提取 type 为 "api" 或 "wellknown" 且有 key 字段的条目
        let auth_type = info.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if auth_type != "api" && auth_type != "wellknown" {
            continue;
        }

        let api_key = match info.get("key").and_then(|v| v.as_str()) {
            Some(k) if !k.is_empty() => k.to_string(),
            _ => continue,
        };

        // auth.json 中无 name 和 baseURL 信息，使用 provider id 作为名称
        providers.push(ProviderInfo {
            id: id.clone(),
            name: id.clone(),
            api_key,
            base_url: None,
        });
    }

    providers
}

/// 并发查询所有供应商额度
/// 由前端传入完整的可用供应商 ID 列表（包含 antigravity/auth 来源），
/// Rust 端从 opencode.json 和 auth.json 查找 apiKey 进行额度查询，
/// 找不到 apiKey 的供应商标记为 unsupported。
#[tauri::command]
pub async fn fetch_all_provider_quotas(provider_ids: Vec<String>) -> Result<String, String> {
    if provider_ids.is_empty() {
        return Ok("[]".to_string());
    }

    let config_path = get_opencode_config_path()?;

    // 从 opencode.json 读取有 apiKey 的供应商信息，构建 HashMap 方便查找
    let mut known_providers = if config_path.exists() {
        match read_providers(&config_path) {
            Ok(providers) => {
                let mut map = std::collections::HashMap::new();
                for p in providers {
                    map.insert(p.id.clone(), p);
                }
                map
            }
            Err(_) => std::collections::HashMap::new(),
        }
    } else {
        std::collections::HashMap::new()
    };

    // 从 models.json 读取内置供应商的 baseURL
    // models.json 结构: Record<providerId, { id, name?, api?, models }>
    let mut registry_base_urls = std::collections::HashMap::new();
    if let Some(home) = dirs::home_dir() {
        let models_path = home.join(".cache").join("opencode").join("models.json");
        if models_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&models_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = json.as_object() {
                        for (id, provider) in obj {
                            if let Some(api) = provider.get("api").and_then(|v| v.as_str()) {
                                if !api.is_empty() {
                                    registry_base_urls.insert(id.clone(), api.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 从 opencode.json 的 provider 字段提取自定义供应商的 baseURL
    // 自定义供应商的 baseURL 在 options.baseURL 或 baseURL 字段中
    let mut opencode_base_urls = std::collections::HashMap::new();
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider_obj) = json.get("provider").and_then(|p| p.as_object()) {
                    for (id, config) in provider_obj {
                        let base_url = config.get("options")
                            .and_then(|o| o.get("baseURL"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                config.get("baseURL")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            });
                        if let Some(url) = base_url {
                            opencode_base_urls.insert(id.clone(), url);
                        }
                    }
                }
            }
        }
    }

    // 从 auth.json 补充供应商信息
    for p in read_auth_providers() {
        known_providers.entry(p.id.clone())
            .and_modify(|existing| {
                // 已存在（来自 opencode.json）：用 auth.json 的 apiKey 覆盖（更可靠）
                existing.api_key = p.api_key.clone();
            })
            .or_insert_with(|| {
                // 新 provider（仅 auth.json 有）：
                // 优先用 opencode.json 自定义的 baseURL，其次用 models.json 内置的 baseURL
                let base_url = opencode_base_urls.get(&p.id).cloned()
                    .or_else(|| registry_base_urls.get(&p.id).cloned());
                ProviderInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    api_key: p.api_key,
                    base_url,
                }
            });
    }

    // 汇总有 apiKey 的供应商
    let known_ids: Vec<&str> = known_providers.keys().map(|s| s.as_str()).collect();
    tracing::info!("[额度查询] 已配置 apiKey 的供应商 ({} 个): {:?}", known_ids.len(), known_ids);

    tracing::info!("[额度查询] 前端传入 provider_ids ({} 个): {:?}", provider_ids.len(), provider_ids);

    // 对每个传入的 provider_id 决定查询策略
    let mut join_set = tokio::task::JoinSet::new();

    for id in &provider_ids {
        if let Some(provider) = known_providers.get(id) {
            // 找到 apiKey（来自配置文件），执行额度查询
            let key_len = provider.api_key.len();
            tracing::info!("[额度查询] {} → 有 apiKey (长度={}), 开始查询", id, key_len);
            let provider = provider.clone();
            join_set.spawn(async move {
                query_provider(&provider).await
            });
        } else {
            // 找不到 apiKey（OAuth/antigravity 来源或未配置），标记为不支持查询
            tracing::info!("[额度查询] {} → 无 apiKey，标记为 unsupported", id);
            let id_clone = id.clone();
            join_set.spawn(async move {
                unsupported_quota(&id_clone, &id_clone)
            });
        }
    }

    let mut quotas = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(quota) => {
                tracing::info!("[额度查询] 完成: provider={}, status={}, quotaType={}",
                    quota.provider_id, quota.status, quota.quota_type);
                quotas.push(quota)
            }
            Err(e) => {
                tracing::warn!("额度查询任务异常: {}", e);
            }
        }
    }

    tracing::info!("[额度查询] 全部完成，共 {} 条结果", quotas.len());

    // 按 provider_id 排序保持稳定顺序
    quotas.sort_by(|a, b| a.provider_id.cmp(&b.provider_id));

    serde_json::to_string(&quotas)
        .map_err(|e| format!("序列化额度数据失败: {}", e))
}
