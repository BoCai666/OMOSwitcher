// GitHub Gist API 客户端
// 实现完整的 Gist CRUD 操作

use std::collections::HashMap;

use reqwest::{header, Client, Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::sync::types::{CreateGistRequest, GistFileContent, GistResponse};

const GITHUB_API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "OMOSwitcher";

/// GitHub API 请求客户端构建器
fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

/// 构建通用请求头
fn build_headers(token: &str) -> reqwest::header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    headers.insert(
        header::USER_AGENT,
        USER_AGENT.parse().unwrap(),
    );
    headers.insert(
        header::ACCEPT,
        "application/vnd.github+json".parse().unwrap(),
    );
    headers
}

/// 处理 HTTP 响应错误
async fn handle_error(response: Response) -> String {
    let status = response.status();
    
    // 检查速率限制
    if let Some(remaining) = response.headers().get("X-RateLimit-Remaining") {
        if let Ok(remaining_str) = remaining.to_str() {
            if let Ok(remaining_num) = remaining_str.parse::<u32>() {
                if remaining_num < 5 {
                    tracing::warn!("GitHub API 速率限制即将耗尽，剩余请求次数: {}", remaining_num);
                }
            }
        }
    }
    
    match status {
        StatusCode::UNAUTHORIZED => "GitHub Token 已过期，请重新登录".to_string(),
        StatusCode::FORBIDDEN => {
            // 检查是否是速率限制
            if let Some(remaining) = response.headers().get("X-RateLimit-Remaining") {
                if let Ok("0") = remaining.to_str() {
                    return "GitHub API 速率限制已达上限，请稍后再试".to_string();
                }
            }
            "访问被拒绝，请检查 Token 权限".to_string()
        }
        StatusCode::NOT_FOUND => "Gist 未找到".to_string(),
        _ => {
            // 尝试解析错误消息
            if let Ok(error_body) = response.text().await {
                if !error_body.is_empty() {
                    return format!("请求失败 ({}): {}", status, error_body);
                }
            }
            format!("请求失败，HTTP 状态码: {}", status)
        }
    }
}

/// 检查速率限制并记录日志
fn check_rate_limit(headers: &header::HeaderMap) {
    if let Some(remaining) = headers.get("X-RateLimit-Remaining") {
        if let Ok(remaining_str) = remaining.to_str() {
            if let Ok(remaining_num) = remaining_str.parse::<u32>() {
                if remaining_num < 5 {
                tracing::warn!(
                        "GitHub API 速率限制即将耗尽，剩余请求次数: {}",
                        remaining_num
                    );
                }
            }
        }
    }
}

// ============================================================================
// Gist CRUD 操作
// ============================================================================

/// 创建 Secret Gist
///
/// POST /gists
/// 成功返回 201 Created
pub async fn create_gist(
    token: &str,
    description: &str,
    files: HashMap<String, GistFileContent>,
) -> Result<GistResponse, String> {
    let client = build_client()?;
    let url = format!("{}/gists", GITHUB_API_BASE);
    
    let request_body = CreateGistRequest {
        description: description.to_string(),
        public: false, // OMOSwitcher 始终创建私有 Gist
        files,
    };
    
    let response = client
        .post(&url)
        .headers(build_headers(token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("创建 Gist 请求失败: {}", e))?;
    
    check_rate_limit(response.headers());
    
    let status = response.status();
    println!("[Gist] create_gist 响应状态: {}", status);
    
    if status == StatusCode::CREATED {
        response
            .json::<GistResponse>()
            .await
            .map_err(|e| format!("解析 Gist 响应失败: {}", e))
    } else {
        let body = response.text().await.unwrap_or_default();
        println!("[Gist] create_gist 失败响应体: {}", body);
        let error_msg = match status {
            StatusCode::UNAUTHORIZED => "GitHub Token 已过期，请重新登录".to_string(),
            StatusCode::FORBIDDEN => format!("访问被拒绝: {}", body),
            StatusCode::NOT_FOUND => format!("创建 Gist 返回 404 (可能是 Token 缺少 gist 权限): {}", body),
            _ => format!("创建 Gist 失败 ({}): {}", status, body),
        };
        Err(error_msg)
    }
}

/// 读取 Gist
///
/// GET /gists/{gist_id}
/// 成功返回 200 OK
pub async fn read_gist(token: &str, gist_id: &str) -> Result<GistResponse, String> {
    let client = build_client()?;
    let url = format!("{}/gists/{}", GITHUB_API_BASE, gist_id);
    
    let response = client
        .get(&url)
        .headers(build_headers(token))
        .send()
        .await
        .map_err(|e| format!("读取 Gist 请求失败: {}", e))?;
    
    check_rate_limit(response.headers());
    
    if response.status() == StatusCode::OK {
        response
            .json::<GistResponse>()
            .await
            .map_err(|e| format!("解析 Gist 响应失败: {}", e))
    } else {
        Err(handle_error(response).await)
    }
}

/// 更新 Gist
///
/// PATCH /gists/{gist_id}
/// 成功返回 200 OK
pub async fn update_gist(
    token: &str,
    gist_id: &str,
    files: HashMap<String, GistFileContent>,
) -> Result<GistResponse, String> {
    let client = build_client()?;
    let url = format!("{}/gists/{}", GITHUB_API_BASE, gist_id);
    
    // 更新请求只需要 files 字段
    #[derive(Serialize)]
    struct UpdateGistRequest {
        files: HashMap<String, GistFileContent>,
    }
    
    let request_body = UpdateGistRequest { files };
    
    let response = client
        .patch(&url)
        .headers(build_headers(token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("更新 Gist 请求失败: {}", e))?;
    
    check_rate_limit(response.headers());
    println!("[Gist] update_gist 响应状态: {}", response.status());
    
    if response.status() == StatusCode::OK {
        response
            .json::<GistResponse>()
            .await
            .map_err(|e| format!("解析 Gist 响应失败: {}", e))
    } else {
        Err(handle_error(response).await)
    }
}

/// 删除 Gist
///
/// DELETE /gists/{gist_id}
/// 成功返回 204 No Content
#[allow(dead_code)]
pub async fn delete_gist(token: &str, gist_id: &str) -> Result<(), String> {
    let client = build_client()?;
    let url = format!("{}/gists/{}", GITHUB_API_BASE, gist_id);
    
    let response = client
        .delete(&url)
        .headers(build_headers(token))
        .send()
        .await
        .map_err(|e| format!("删除 Gist 请求失败: {}", e))?;
    
    check_rate_limit(response.headers());
    
    if response.status() == StatusCode::NO_CONTENT {
        Ok(())
    } else {
        Err(handle_error(response).await)
    }
}

/// 查找 OMOSwitcher 专用 Gist
///
/// GET /gists?per_page=100
/// 搜索 description 包含 "OMOSwitcher" 的 gist
/// 返回第一个匹配的，或 None
pub async fn find_omoswitcher_gist(token: &str) -> Result<Option<GistResponse>, String> {
    let client = build_client()?;
    let url = format!("{}/gists?per_page=100", GITHUB_API_BASE);
    
    let response = client
        .get(&url)
        .headers(build_headers(token))
        .send()
        .await
        .map_err(|e| format!("获取 Gist 列表请求失败: {}", e))?;
    
    check_rate_limit(response.headers());
    
    if response.status() != StatusCode::OK {
        return Err(handle_error(response).await);
    }
    
    let gists: Vec<GistResponse> = response
        .json()
        .await
        .map_err(|e| format!("解析 Gist 列表响应失败: {}", e))?;
    
    // 搜索 description 包含 "OMOSwitcher" 的 gist
    for gist in &gists {
        if let Some(ref desc) = gist.description {
            if desc.contains("OMOSwitcher") {
                return Ok(Some(gist.clone()));
            }
        }
    }
    
    Ok(None)
}

// ============================================================================
// 预设同步操作
// ============================================================================

/// 预设元数据（存储在 metadata.json 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetMetadata {
    pub current_preset_name: String,
    pub updated_at: String,
    pub version: String,
}

/// 上传预设到 Gist
///
/// 如果 gist_id 存在 → update_gist
/// 如果不存在 → 先 find → 存在则 update，不存在则 create
pub async fn upload_presets(
    token: &str,
    gist_id: Option<&str>,
    presets_json: &str,
    current_preset_name: &str,
) -> Result<GistResponse, String> {
    // 构造 metadata
    let metadata = PresetMetadata {
        current_preset_name: current_preset_name.to_string(),
        updated_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("序列化元数据失败: {}", e))?;
    
    // 构造文件内容
    let mut files = HashMap::new();
    files.insert(
        "presets.json".to_string(),
        GistFileContent {
            content: presets_json.to_string(),
        },
    );
    files.insert(
        "metadata.json".to_string(),
        GistFileContent {
            content: metadata_json,
        },
    );
    
    match gist_id {
        Some(id) if !id.is_empty() => {
            // 已有 gist_id，直接更新
            println!("[Gist] upload_presets: 更新已有 Gist id={}", id);
            update_gist(token, id, files).await
        }
        _ => {
            // 没有 gist_id，先查找是否存在
            println!("[Gist] upload_presets: 无 gist_id，查找已有 Gist...");
            match find_omoswitcher_gist(token).await? {
                Some(existing_gist) => {
                    // 找到现有的，更新它
                    println!("[Gist] upload_presets: 找到已有 Gist id={}", existing_gist.id);
                    update_gist(token, &existing_gist.id, files).await
                }
                None => {
                    // 没有现有的，创建新的
                    println!("[Gist] upload_presets: 未找到已有 Gist，创建新的");
                    create_gist(token, "OMOSwitcher 预设配置", files).await
                }
            }
        }
    }
}

/// 从 Gist 下载预设
///
/// 返回 (presets_json, current_preset_name)
pub async fn download_presets(
    token: &str,
    gist_id: &str,
) -> Result<(String, Option<String>), String> {
    let gist = read_gist(token, gist_id).await?;
    
    // 提取 presets.json 内容
    let presets_json = gist
        .files
        .get("presets.json")
        .ok_or("Gist 中未找到 presets.json 文件")?
        .content
        .clone()
        .ok_or("Gist 中 presets.json 内容为空")?;
    
    // 尝试提取 metadata.json 获取 current_preset_name
    let current_preset_name = if let Some(metadata_file) = gist.files.get("metadata.json") {
        match &metadata_file.content {
            Some(content) => match serde_json::from_str::<PresetMetadata>(content) {
                Ok(metadata) => Some(metadata.current_preset_name),
                Err(e) => {
                    tracing::warn!("解析 metadata.json 失败: {}", e);
                    None
                }
            },
            None => None,
        }
    } else {
        None
    };
    
    Ok((presets_json, current_preset_name))
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preset_metadata_serialization() {
        let metadata = PresetMetadata {
            current_preset_name: "default".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("default"));
        assert!(json.contains("current_preset_name"));
    }
    
    #[test]
    fn test_build_headers() {
        let headers = build_headers("test_token");
        assert!(headers.contains_key(header::AUTHORIZATION));
        assert!(headers.contains_key(header::USER_AGENT));
        assert!(headers.contains_key(header::ACCEPT));
    }
}
