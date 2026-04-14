// Sync 模块 - 类型定义
// GitHub Gist 同步相关类型

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================================
// GitHub API 类型
// ============================================================================

/// GitHub 用户信息（/user API 返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
}

/// Device Flow 响应
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Device Flow Token 响应
#[derive(Debug, Deserialize)]
pub struct DeviceTokenResponse {
    pub access_token: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

// ============================================================================
// 认证状态
// ============================================================================

/// 同步认证状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthState {
    LoggedOut,
    LoggingIn {
        user_code: String,
        verification_uri: String,
    },
    OAuthLoggingIn,
    LoggedIn {
        user: GitHubUser,
    },
}

/// OAuth Web Flow 进行中的会话状态
/// 保存在 SyncCommandState 中，用于跨命令传递 PKCE 参数
#[derive(Debug)]
pub struct OAuthSession {
    /// PKCE code_verifier（43+ 字符的随机串）
    pub code_verifier: String,
    /// CSRF 防护随机串
    pub state: String,
    /// 实际使用的回调地址 http://127.0.0.1:PORT/callback
    #[allow(dead_code)]
    pub redirect_uri: String,
}

// ============================================================================
// Gist API 类型
// ============================================================================

/// Gist 文件
/// 列表 API 返回的 content 可能为 null 或缺失，只有详情 API 才有完整内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistFile {
    pub content: Option<String>,
    pub filename: Option<String>,
}

/// 创建/更新 Gist 请求
#[derive(Debug, Serialize)]
pub struct CreateGistRequest {
    pub description: String,
    pub public: bool,
    pub files: HashMap<String, GistFileContent>,
}

#[derive(Debug, Serialize)]
pub struct GistFileContent {
    pub content: String,
}

/// Gist 响应（GET /gists/{id}）
#[derive(Debug, Clone, Deserialize)]
pub struct GistResponse {
    pub id: String,
    pub description: Option<String>,
    pub updated_at: String,
    pub files: HashMap<String, GistFile>,
    #[allow(dead_code)]
    pub html_url: String,
}

// ============================================================================
// 同步元数据与结果
// ============================================================================

/// 同步元数据（本地存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub gist_id: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_sync_content_hash: Option<String>,
    pub github_user_id: Option<i64>,
    pub github_login: Option<String>,
}

impl Default for SyncMetadata {
    fn default() -> Self {
        Self {
            gist_id: None,
            last_sync_at: None,
            last_sync_content_hash: None,
            github_user_id: None,
            github_login: None,
        }
    }
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncResult {
    UpToDate,
    Uploaded {
        count: usize,
    },
    Downloaded {
        count: usize,
    },
    Conflict {
        local_updated_at: String,
        remote_updated_at: String,
        local_count: usize,
        remote_count: usize,
    },
    Error {
        message: String,
    },
}

/// 冲突解决选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
}
