// 模型管理命令

use super::async_fs;
use super::get_antigravity_accounts_path;
use super::get_models_path;
use super::get_opencode_auth_path;
use super::get_opencode_config_path;
use super::get_opencode_models_cache_path;

/// 读取模型配置文件（异步）
#[tauri::command]
pub async fn read_models() -> Result<String, String> {
    let path = get_models_path()?;
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取模型列表失败: {}", e))
}

/// 写入模型配置文件（异步）
#[tauri::command]
pub async fn write_models(content: String) -> Result<(), String> {
    let path = get_models_path()?;
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("写入模型列表失败: {}", e))
}

/// 默认模型列表
const DEFAULT_MODELS_JSON: &str = r#"[
  {"id": "wuwen/glm-5", "name": "GLM-5", "provider": "wuwen"},
  {"id": "wuwen/minimax-m2.5", "name": "MiniMax M2.5", "provider": "wuwen"},
  {"id": "wuwen/minimax-m2.7", "name": "MiniMax M2.7", "provider": "wuwen"},
  {"id": "wuwen/kimi-k2.5", "name": "Kimi K2.5", "provider": "wuwen"}
]"#;

/// 读取模型列表（合并降级逻辑，单次 IPC 调用）
/// 优先级：models.json > opencode.json provider > 默认值
#[tauri::command]
pub async fn read_models_with_fallback() -> Result<String, String> {
    let models_path = get_models_path()?;
    
    // 1. 尝试读取 models.json
    if models_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&models_path).await {
            // 检查是否为有效非空 JSON 数组
            if !content.trim().is_empty() && content.trim() != "[]" {
                return Ok(content);
            }
        }
    }
    
    // 2. 尝试从 opencode.json 的 provider 字段读取
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            // 解析 opencode.json 并提取模型
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider") {
                    if let Some(provider_obj) = provider.as_object() {
                        let models: Vec<serde_json::Value> = provider_obj
                            .iter()
                            .flat_map(|(provider_name, provider_config)| {
                                if let Some(models_obj) = provider_config.get("models").and_then(|m| m.as_object()) {
                                    models_obj.iter().map(move |(model_id, model_config)| {
                                        serde_json::json!({
                                            "id": format!("{}/{}", provider_name, model_id),
                                            "name": model_config.get("name").and_then(|n| n.as_str()).unwrap_or(model_id),
                                            "provider": provider_name
                                        })
                                    }).collect::<Vec<_>>()
                                } else {
                                    vec![]
                                }
                            })
                            .collect();
                        
                        if !models.is_empty() {
                            return Ok(serde_json::to_string(&models).unwrap_or(DEFAULT_MODELS_JSON.to_string()));
                        }
                    }
                }
            }
        }
    }
    
    // 3. 返回默认模型列表
    Ok(DEFAULT_MODELS_JSON.to_string())
}

/// 读取 OpenCode 模型注册表缓存
/// 从 ~/.cache/opencode/models.json 读取全量供应商和模型信息
#[tauri::command]
pub async fn read_opencode_models_cache() -> Result<String, String> {
    let path = get_opencode_models_cache_path()?;
    if !path.exists() {
        return Err("模型注册表缓存不存在。请先运行一次 OpenCode 以生成缓存。".to_string());
    }
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取模型注册表缓存失败: {}", e))
}

/// 获取已配置（可用）的供应商 ID 列表
/// 交叉比对 opencode.json provider 字段、antigravity-accounts.json
#[tauri::command]
pub async fn get_available_provider_ids() -> Result<Vec<String>, String> {
    let mut available = Vec::new();
    
    // 1. 从 opencode.json 的 provider 字段提取有 apiKey 的供应商
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider").and_then(|p| p.as_object()) {
                    for (provider_id, config) in provider {
                        // 检查是否有 apiKey 或 options.apiKey
                        let has_api_key = config.get("apiKey").and_then(|v| v.as_str()).is_some()
                            || config.get("options")
                                .and_then(|o| o.get("apiKey"))
                                .and_then(|v| v.as_str()).is_some();
                        if has_api_key {
                            available.push(provider_id.clone());
                        }
                    }
                }
            }
        }
    }
    
    // 2. 从 antigravity-accounts.json 提取 Google/OAuth 连接的供应商
    let antigravity_path = get_antigravity_accounts_path()?;
    if antigravity_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&antigravity_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // 如果有 antigravity 账号，说明 google provider 是通过 OAuth 认证的
                if let Some(accounts) = json.get("accounts").and_then(|a| a.as_array()) {
                    if !accounts.is_empty() {
                        // antigravity 使用 google provider
                        if !available.contains(&"google".to_string()) {
                            available.push("google".to_string());
                        }
                        // antigravity 也可能启用其他 provider（通过 rateLimitResetTimes 字段判断）
                        if let Some(first_account) = accounts.first() {
                            if let Some(rate_limits) = first_account.get("rateLimitResetTimes") {
                                // rateLimitResetTimes 的 key 格式为 "providerId:modelId"
                                // 提取所有不重复的 providerId
                                if let Some(limits_obj) = rate_limits.as_object() {
                                    for key in limits_obj.keys() {
                                        if let Some(provider_id) = key.split(':').next() {
                                            let pid = provider_id.to_string();
                                            if !available.contains(&pid) {
                                                available.push(pid);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 3. 从 auth.json 提取（如果存在）
    let auth_path = get_opencode_auth_path()?;
    if auth_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&auth_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // auth.json 可能是数组或对象格式
                if let Some(obj) = json.as_object() {
                    for key in obj.keys() {
                        if !available.contains(key) {
                            available.push(key.clone());
                        }
                    }
                } else if let Some(arr) = json.as_array() {
                    for item in arr {
                        if let Some(pid) = item.get("provider").and_then(|v| v.as_str()) {
                            if !available.contains(&pid.to_string()) {
                                available.push(pid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    available.sort();
    Ok(available)
}

/// 获取 opencode.json 中手动配置的供应商 ID 列表
/// 仅检查 opencode.json 的 provider 字段，不包含 auth.json 或 antigravity-accounts.json
#[tauri::command]
pub async fn get_custom_provider_ids() -> Result<Vec<String>, String> {
    let mut custom = Vec::new();
    
    // 从 opencode.json 的 provider 字段提取有 apiKey 的供应商
    let opencode_path = get_opencode_config_path()?;
    if opencode_path.exists() {
        if let Ok(content) = async_fs::read_to_string(&opencode_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(provider) = json.get("provider").and_then(|p| p.as_object()) {
                    for (provider_id, config) in provider {
                        // 检查是否有 apiKey 或 options.apiKey
                        let has_api_key = config.get("apiKey").and_then(|v| v.as_str()).is_some()
                            || config.get("options")
                                .and_then(|o| o.get("apiKey"))
                                .and_then(|v| v.as_str()).is_some();
                        if has_api_key {
                            custom.push(provider_id.clone());
                        }
                    }
                }
            }
        }
    }
    
    custom.sort();
    Ok(custom)
}

/// 添加自定义 provider 到 opencode.json
/// 在 provider 字段中添加（或更新）指定 ID 的供应商配置，然后写回文件
/// config_json: 完整的 provider 配置 JSON 字符串，例如：
///   {"npm":"@ai-sdk/openai-compatible","name":"My Provider","options":{"apiKey":"sk-xxx","baseURL":"https://..."},"models":{...}}
#[tauri::command]
pub async fn add_custom_provider(provider_id: String, config_json: String) -> Result<(), String> {
    let provider_config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("Provider 配置 JSON 格式无效: {}", e))?;

    let path = get_opencode_config_path()?;

    // 如果 opencode.json 不存在，创建一个带 provider 字段的新文件
    if !path.exists() {
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        let mut json = serde_json::json!({});
        json["provider"] = serde_json::json!({});
        if let Some(obj) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
            obj.insert(provider_id, provider_config);
        }
        let output = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;
        async_fs::write(&path, output)
            .await
            .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;
        return Ok(());
    }

    // 读取现有配置
    let content = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

    let mut json = serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

    // 确保 provider 字段存在
    if json.get("provider").is_none() {
        json["provider"] = serde_json::json!({});
    }

    // 添加或更新指定 provider
    if let Some(provider) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
        provider.insert(provider_id, provider_config);
    }

    // 格式化写回（保持美观缩进）
    let output = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

    async_fs::write(&path, output)
        .await
        .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;

    Ok(())
}

/// 删除 opencode.json 中指定的自定义 provider
/// 从 provider 字段中移除指定 ID 的供应商配置，然后写回文件
#[tauri::command]
pub async fn delete_custom_provider(provider_id: String) -> Result<(), String> {
    let path = get_opencode_config_path()?;
    if !path.exists() {
        return Err("opencode.json 配置文件不存在".to_string());
    }

    let content = async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

    let mut json = serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

    // 从 provider 对象中删除指定的 key
    if let Some(provider) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
        if provider.remove(&provider_id).is_none() {
            return Err(format!("provider \"{}\" 不存在于 opencode.json 中", provider_id));
        }
    } else {
        return Err("opencode.json 中没有 provider 字段".to_string());
    }

    // 格式化写回（保持美观缩进）
    let output = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

    async_fs::write(&path, output)
        .await
        .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;

    Ok(())
}
