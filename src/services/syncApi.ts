/**
 * 同步服务 API 封装
 * 通过 Tauri invoke 与 Rust 后端同步模块通信
 */

import { invoke } from '@tauri-apps/api/core'
import { withErrorHandling } from '@/utils/errorHandler'

// ============================================================================
// 类型定义（与 Rust sync/types.rs 对齐）
// ============================================================================

/**
 * GitHub 用户信息
 */
export interface GitHubUser {
  id: number
  login: string
  avatar_url: string
  name: string | null
}

/**
 * 认证状态
 */
export type AuthState =
  | { type: 'LoggedOut' }
  | { type: 'LoggingIn'; user_code: string; verification_uri: string }
  | { type: 'OAuthLoggingIn' }
  | { type: 'LoggedIn'; user: GitHubUser }

/**
 * 同步元数据
 */
export interface SyncMetadata {
  gist_id: string | null
  last_sync_at: string | null
  last_sync_content_hash: string | null
  github_user_id: number | null
  github_login: string | null
}

/**
 * 同步结果
 */
export type SyncResult =
  | { type: 'UpToDate' }
  | { type: 'Uploaded'; count: number }
  | { type: 'Downloaded'; count: number }
  | { type: 'Conflict'; local_updated_at: string; remote_updated_at: string; local_count: number; remote_count: number }
  | { type: 'Error'; message: string }

/**
 * 冲突解决策略
 */
export type ConflictResolution = 'KeepLocal' | 'KeepRemote'

// ============================================================================
// API 函数
// ============================================================================

/**
 * 获取认证状态
 * @returns 当前 GitHub 认证状态
 */
export async function getAuthState(): Promise<AuthState | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_get_auth_state')
    return JSON.parse(json) as AuthState
  }, '获取认证状态失败')
}

/**
 * 启动 Device Flow 登录
 * @returns 用户码和验证链接
 */
export async function startDeviceLogin(): Promise<{ user_code: string; verification_uri: string } | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_start_device_login')
    return JSON.parse(json) as { user_code: string; verification_uri: string }
  }, '启动登录失败')
}

/**
 * 完成 Device Flow 登录
 * @returns 登录的 GitHub 用户信息
 */
export async function completeDeviceLogin(): Promise<GitHubUser> {
  const json = await invoke<string>('sync_complete_device_login')
  return JSON.parse(json) as GitHubUser
}

/**
 * 使用 Personal Access Token 登录
 * @param pat GitHub Personal Access Token
 * @returns 登录的 GitHub 用户信息
 */
export async function loginWithPat(pat: string): Promise<GitHubUser> {
  const json = await invoke<string>('sync_login_with_pat', { pat })
  return JSON.parse(json) as GitHubUser
}

/**
 * 登出
 * 清除 GitHub 认证状态
 */
export async function logout(): Promise<void> {
  await withErrorHandling(async () => {
    await invoke<void>('sync_logout')
  }, '登出失败')
}

/**
 * 获取同步状态
 * @returns 当前同步元数据
 */
export async function getSyncStatus(): Promise<SyncMetadata | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_get_status')
    return JSON.parse(json) as SyncMetadata
  }, '获取同步状态失败')
}

/**
 * 上传预设到 GitHub Gist
 * @returns 同步结果
 */
export async function uploadSync(): Promise<SyncResult | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_upload')
    return JSON.parse(json) as SyncResult
  }, '上传同步失败')
}

/**
 * 从 GitHub Gist 下载预设
 * @returns 同步结果
 */
export async function downloadSync(): Promise<SyncResult | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_download')
    return JSON.parse(json) as SyncResult
  }, '下载同步失败')
}

/**
 * 执行自动同步（上传或下载，视情况而定）
 * @returns 同步结果
 */
export async function performSync(): Promise<SyncResult | null> {
  return withErrorHandling(async () => {
    const json = await invoke<string>('sync_perform')
    return JSON.parse(json) as SyncResult
  }, '执行同步失败')
}

/**
 * 解决同步冲突
 * @param resolution 冲突解决策略
 */
export async function resolveConflict(resolution: ConflictResolution): Promise<void> {
  await withErrorHandling(async () => {
    await invoke<void>('sync_resolve_conflict', { resolution })
  }, '解决冲突失败')
}

/**
 * 取消 Device Flow 登录
 */
export async function cancelDeviceLogin(): Promise<void> {
  await withErrorHandling(async () => {
    await invoke<void>('sync_cancel_device_login')
  }, '取消登录失败')
}

/**
 * 启动 OAuth Web Flow 登录
 * 自动打开浏览器，等待用户授权后返回用户信息
 * @returns 登录的 GitHub 用户信息
 */
export async function startOAuthLogin(): Promise<GitHubUser> {
  const json = await invoke<string>('sync_start_oauth_login')
  return JSON.parse(json) as GitHubUser
}

/**
 * 取消 OAuth Web Flow 登录
 */
export async function cancelOAuthLogin(): Promise<void> {
  await withErrorHandling(async () => {
    await invoke<void>('sync_cancel_oauth_login')
  }, '取消登录失败')
}
