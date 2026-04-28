// OpenCode Go 额度查询（网页抓取）
// 功能暂时禁用，完整实现见 git 历史

use super::{ProviderInfo, ProviderQuota};

/// 查询 OpenCode Go 额度（暂不可用）
pub(crate) async fn query_opencode_go(provider: &ProviderInfo) -> ProviderQuota {
    // TODO: 恢复时参考 git log src-tauri/src/quota/opencode_go.rs
    super::unsupported_quota(&provider.id, &provider.name)
}
