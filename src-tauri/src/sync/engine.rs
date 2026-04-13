// Sync 引擎模块
// 实现同步核心逻辑：冲突检测、合并策略、上传/下载编排

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::gist;
use super::token;
use super::types::*;

// ============================================================================
// 同步动作
// ============================================================================

/// 同步动作判定结果
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    /// 双方无变更，无需同步
    NoSync,
    /// 本地有变更，需上传
    UploadNeeded,
    /// 远端有变更，需下载
    DownloadNeeded,
    /// 双方均有变更，存在冲突
    Conflict,
}

// ============================================================================
// 纯函数：哈希与冲突检测
// ============================================================================

/// 计算内容哈希（确定性指纹）
///
/// 使用 serde_json 规范化 JSON 后，通过 std::hash::DefaultHasher 生成确定性哈希。
/// 不引入额外 crate 依赖。
///
/// # 确定性保证
/// - JSON 内容：先解析再规范化序列化（key 排序），消除 key 排序差异
/// - 非 JSON 内容：直接对原始字符串哈希
pub fn compute_content_hash(content: &str) -> String {
    let canonical = match serde_json::from_str::<serde_json::Value>(content) {
        Ok(val) => canonicalize_json(&val),
        Err(_) => content.to_string(),
    };

    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 将 JSON Value 规范化为排序后的字符串表示
/// 递归排序 Object 的 key，确保相同内容产生相同输出
fn canonicalize_json(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let pairs: Vec<String> = keys
                .iter()
                .map(|k| format!("{}:{}", k, canonicalize_json(&map[*k])))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonicalize_json).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

/// 检测同步动作
///
/// 判定逻辑：
///   1. 对比 local_content_hash 与 last_sync_content_hash → 本地是否有变更
///   2. 对比 remote_updated_at 与 last_sync_at → 远端是否有变更
///   3. 双方都有变更 → Conflict
///   4. 仅本地变更 → UploadNeeded
///   5. 仅远端变更 → DownloadNeeded
///   6. 无变更 → NoSync
///
/// # 首次同步场景
/// - `last_sync_content_hash` 为 None：本地有内容即视为变更
/// - `last_sync_at` 为 None：远端有时间戳即视为变更
pub fn detect_sync_action(
    local_content_hash: &str,
    last_sync_content_hash: Option<&str>,
    remote_updated_at: &str,
    last_sync_at: Option<&str>,
) -> SyncAction {
    let local_changed = match last_sync_content_hash {
        Some(last_hash) => local_content_hash != last_hash,
        None => !local_content_hash.is_empty(),
    };

    let remote_changed = match last_sync_at {
        Some(last_at) => remote_updated_at != last_at,
        None => !remote_updated_at.is_empty(),
    };

    match (local_changed, remote_changed) {
        (true, true) => SyncAction::Conflict,
        (true, false) => SyncAction::UploadNeeded,
        (false, true) => SyncAction::DownloadNeeded,
        (false, false) => SyncAction::NoSync,
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 统计预设数量
///
/// 尝试将 JSON 解析为对象（Map）或数组来计数，失败时返回 0。
fn count_presets(json: &str) -> usize {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(serde_json::Value::Object(map)) => map.len(),
        Ok(serde_json::Value::Array(arr)) => arr.len(),
        _ => 0,
    }
}

/// 生成当前 UTC 时间戳（RFC 3339 格式）
fn current_timestamp() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| {
            // 降级：使用 Unix 时间戳
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("unix:{}", secs)
        })
}

// ============================================================================
// 异步同步操作
// ============================================================================

/// 上传预设到 Gist
///
/// 序列化 presets 为 JSON → 调 gist::upload_presets → 更新 sync-meta
pub async fn perform_upload(
    app: &tauri::AppHandle,
    token: &str,
    gist_id: Option<&str>,
    presets_json: &str,
    current_preset_name: &str,
) -> Result<SyncMetadata, String> {
    // 调用 gist 模块上传预设
    let response =
        gist::upload_presets(token, gist_id, presets_json, current_preset_name).await?;

    // 计算内容哈希
    let content_hash = compute_content_hash(presets_json);

    // 构建新的同步元数据
    let mut meta = token::get_sync_meta(app).await.unwrap_or_default();
    meta.gist_id = Some(response.id.clone());
    meta.last_sync_at = Some(response.updated_at.clone());
    meta.last_sync_content_hash = Some(content_hash);

    // 保存元数据
    token::save_sync_meta(app, &meta).await?;

    Ok(meta)
}

/// 从 Gist 下载预设
///
/// 调 gist::download_presets → 返回 (presets_json, current_preset_name)
pub async fn perform_download(
    token: &str,
    gist_id: &str,
) -> Result<(String, Option<String>), String> {
    gist::download_presets(token, gist_id).await
}

/// 解决冲突
///
/// 根据 ConflictResolution 选择保留本地或远端版本：
/// - KeepLocal：上传本地版本到远端，覆盖远端
/// - KeepRemote：从远端下载并更新本地元数据
pub async fn resolve_conflict(
    resolution: ConflictResolution,
    app: &tauri::AppHandle,
    token: &str,
    gist_id: Option<&str>,
    local_presets_json: &str,
    current_preset_name: &str,
) -> Result<(), String> {
    match resolution {
        ConflictResolution::KeepLocal => {
            // 保留本地：上传本地版本到远端
            perform_upload(app, token, gist_id, local_presets_json, current_preset_name).await?;
            Ok(())
        }
        ConflictResolution::KeepRemote => {
            // 保留远端：下载远端版本并更新本地元数据
            let gist_id_str = gist_id
                .ok_or_else(|| "缺少 Gist ID，无法下载远端版本".to_string())?;
            let (remote_json, _) = perform_download(token, gist_id_str).await?;

            // 更新本地同步元数据
            let mut meta = token::get_sync_meta(app).await.unwrap_or_default();
            meta.last_sync_content_hash = Some(compute_content_hash(&remote_json));
            meta.last_sync_at = Some(current_timestamp());
            token::save_sync_meta(app, &meta).await?;

            Ok(())
        }
    }
}

/// 完整同步流程
///
/// 编排完整的同步过程：
/// 1. 加载本地同步元数据
/// 2. 计算本地内容哈希
/// 3. 获取远端信息（如有 Gist）
/// 4. 检测同步动作
/// 5. 执行对应操作
pub async fn full_sync(
    app: &tauri::AppHandle,
    token: &str,
    local_presets_json: &str,
    current_preset_name: &str,
) -> Result<SyncResult, String> {
    // 1. 加载同步元数据
    let meta = token::get_sync_meta(app).await?;

    // 2. 计算本地内容哈希（空内容用空字符串表示"无内容"）
    let local_hash = if local_presets_json.trim().is_empty() {
        String::new()
    } else {
        compute_content_hash(local_presets_json)
    };

    // 3 & 4. 检测同步动作
    let action = match &meta.gist_id {
        Some(gid) => {
            // 已有 Gist，获取远端信息
            let remote = gist::read_gist(token, gid).await?;
            detect_sync_action(
                &local_hash,
                meta.last_sync_content_hash.as_deref(),
                &remote.updated_at,
                meta.last_sync_at.as_deref(),
            )
        }
        None => {
            // 没有 Gist，根据本地内容决定
            if local_hash.is_empty() {
                SyncAction::NoSync
            } else {
                SyncAction::UploadNeeded
            }
        }
    };

    // 5. 执行同步动作
    match action {
        SyncAction::NoSync => Ok(SyncResult::UpToDate),
        SyncAction::UploadNeeded => {
            let _new_meta = perform_upload(
                app,
                token,
                meta.gist_id.as_deref(),
                local_presets_json,
                current_preset_name,
            )
            .await?;
            let count = count_presets(local_presets_json);
            Ok(SyncResult::Uploaded { count })
        }
        SyncAction::DownloadNeeded => {
            let gid = meta
                .gist_id
                .as_ref()
                .ok_or_else(|| "缺少 Gist ID，无法下载".to_string())?;
            let (remote_json, _) = perform_download(token, gid).await?;

            // 更新同步元数据
            let mut new_meta = meta.clone();
            new_meta.last_sync_content_hash = Some(compute_content_hash(&remote_json));
            new_meta.last_sync_at = Some(current_timestamp());
            token::save_sync_meta(app, &new_meta).await?;

            let count = count_presets(&remote_json);
            Ok(SyncResult::Downloaded { count })
        }
        SyncAction::Conflict => {
            let gid = meta
                .gist_id
                .as_ref()
                .ok_or_else(|| "缺少 Gist ID，无法报告冲突".to_string())?;
            let remote = gist::read_gist(token, gid).await?;
            let (remote_json, _) = perform_download(token, gid).await?;

            Ok(SyncResult::Conflict {
                local_updated_at: meta.last_sync_at.clone().unwrap_or_default(),
                remote_updated_at: remote.updated_at.clone(),
                local_count: count_presets(local_presets_json),
                remote_count: count_presets(&remote_json),
            })
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // 测试 1: 无同步需求 — 内容相同
    // ------------------------------------------------------------------------
    #[test]
    fn test_no_sync_needed() {
        let action = detect_sync_action(
            "abc123",             // local hash
            Some("abc123"),       // last sync hash（相同）
            "2024-01-01T00:00:00Z", // remote updated_at
            Some("2024-01-01T00:00:00Z"), // last sync at（相同）
        );
        assert_eq!(action, SyncAction::NoSync);
    }

    // ------------------------------------------------------------------------
    // 测试 2: 本地有变更 → UploadNeeded
    // ------------------------------------------------------------------------
    #[test]
    fn test_upload_needed() {
        let action = detect_sync_action(
            "new_hash",           // local hash（已变更）
            Some("old_hash"),     // last sync hash
            "2024-01-01T00:00:00Z", // remote updated_at
            Some("2024-01-01T00:00:00Z"), // last sync at（相同）
        );
        assert_eq!(action, SyncAction::UploadNeeded);
    }

    // ------------------------------------------------------------------------
    // 测试 3: 远端有变更 → DownloadNeeded
    // ------------------------------------------------------------------------
    #[test]
    fn test_download_needed() {
        let action = detect_sync_action(
            "abc123",             // local hash
            Some("abc123"),       // last sync hash（相同）
            "2024-01-02T00:00:00Z", // remote updated_at（已变更）
            Some("2024-01-01T00:00:00Z"), // last sync at
        );
        assert_eq!(action, SyncAction::DownloadNeeded);
    }

    // ------------------------------------------------------------------------
    // 测试 4: 双方都有变更 → Conflict
    // ------------------------------------------------------------------------
    #[test]
    fn test_conflict_detected() {
        let action = detect_sync_action(
            "new_hash",           // local hash（已变更）
            Some("old_hash"),     // last sync hash
            "2024-01-02T00:00:00Z", // remote updated_at（已变更）
            Some("2024-01-01T00:00:00Z"), // last sync at
        );
        assert_eq!(action, SyncAction::Conflict);
    }

    // ------------------------------------------------------------------------
    // 测试 5: 相同内容产生相同哈希（确定性）
    // ------------------------------------------------------------------------
    #[test]
    fn test_content_hash_deterministic() {
        let content = r#"{"preset_a": {"model": "gpt-4"}, "preset_b": {"model": "claude-3"}}"#;
        let hash1 = compute_content_hash(content);
        let hash2 = compute_content_hash(content);
        assert_eq!(hash1, hash2, "相同内容必须产生相同的哈希值");
        assert!(!hash1.is_empty(), "哈希值不应为空");
        assert_eq!(hash1.len(), 16, "哈希值应为 16 字符的十六进制字符串");
    }

    // ------------------------------------------------------------------------
    // 测试 6: 本地空，远端有数据 → DownloadNeeded
    // ------------------------------------------------------------------------
    #[test]
    fn test_empty_local_sync() {
        // 空内容用空字符串表示"无本地预设"
        let action = detect_sync_action(
            "",                   // local hash 为空（无本地内容）
            None,                 // 无上次同步记录
            "2024-01-01T00:00:00Z", // remote updated_at（有数据）
            None,                 // 无上次同步记录
        );
        assert_eq!(action, SyncAction::DownloadNeeded);
    }

    // ------------------------------------------------------------------------
    // 测试 7: 远端空，本地有数据 → UploadNeeded
    // ------------------------------------------------------------------------
    #[test]
    fn test_empty_remote_sync() {
        let local_hash = compute_content_hash(r#"{"preset_a": {}}"#);
        let action = detect_sync_action(
            &local_hash,          // local hash（有内容）
            None,                 // 无上次同步记录
            "",                   // remote updated_at（空）
            None,                 // 无上次同步记录
        );
        assert_eq!(action, SyncAction::UploadNeeded);
    }

    // ------------------------------------------------------------------------
    // 测试 8: 不同内容产生不同哈希
    // ------------------------------------------------------------------------
    #[test]
    fn test_content_hash_different_for_different_content() {
        let hash1 = compute_content_hash(r#"{"a": 1}"#);
        let hash2 = compute_content_hash(r#"{"a": 2}"#);
        assert_ne!(hash1, hash2, "不同内容必须产生不同的哈希值");
    }

    // ------------------------------------------------------------------------
    // 测试 9: JSON 规范化 — key 排序不影响哈希
    // ------------------------------------------------------------------------
    #[test]
    fn test_json_canonicalization() {
        let json1 = r#"{"b": 2, "a": 1}"#;
        let json2 = r#"{"a": 1, "b": 2}"#;
        let hash1 = compute_content_hash(json1);
        let hash2 = compute_content_hash(json2);
        assert_eq!(
            hash1, hash2,
            "不同 key 排序的相同 JSON 应该产生相同的哈希值"
        );
    }

    // ------------------------------------------------------------------------
    // 测试 10: count_presets 统计
    // ------------------------------------------------------------------------
    #[test]
    fn test_count_presets_object() {
        let json = r#"{"preset_a": {"model": "gpt-4"}, "preset_b": {"model": "claude-3"}}"#;
        assert_eq!(count_presets(json), 2);
    }

    #[test]
    fn test_count_presets_array() {
        let json = r#"[1, 2, 3]"#;
        assert_eq!(count_presets(json), 3);
    }

    #[test]
    fn test_count_presets_empty() {
        assert_eq!(count_presets(""), 0);
        assert_eq!(count_presets("invalid"), 0);
    }
}
