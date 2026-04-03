/**
 * OpenCode Server 软重载模块
 *
 * 保存 oh-my-opencode 配置后，通过 Rust 后端请求运行中的 OpenCode Server：
 * - 空闲时销毁当前实例，让下一个新会话按最新配置重建
 * - 忙碌时进入等待状态，待当前对话结束后自动应用
 *
 * 为什么 HTTP 调用放在 Rust 端：
 * Tauri 2 的 webview 中 fetch 外部地址受安全策略限制，
 * 需要额外的 capability 配置。通过 Rust 端的 reqwest 发请求
 * 可以绕过这个限制，且与项目现有的 monitorApi 模式一致。
 */

import { invoke } from '@tauri-apps/api/core'

// 软重载状态（与 Rust 端 SoftReloadResult.state 对应）
export type SoftReloadState =
  | 'deferred_offline'
  | 'pending_idle'
  | 'applied_next_session'
  | 'failed'

// 软重载结果（与 Rust 端 SoftReloadResult 对应）
export interface SoftReloadResult {
  /** 是否成功进入预期状态 */
  success: boolean
  /** 可读的状态描述 */
  message: string
  /** 是否因为 OpenCode 未运行而延后到下次启动 */
  skipped: boolean
  /** 当前软重载所处状态 */
  state: SoftReloadState
}

/**
 * 首次请求软重载。
 *
 * 返回值只描述“当前状态”：
 * - OpenCode 未运行 → 下次启动生效
 * - 有活跃会话 → 等待空闲后自动应用
 * - 无活跃会话 → 已应用到下一个新会话
 */
export async function requestSoftReload(): Promise<SoftReloadResult> {
  return invoke<SoftReloadResult>('request_soft_reload_opencode_config')
}

/**
 * 检查并推进待处理的软重载。
 *
 * 仅当此前返回 pending_idle 时才需要轮询调用。
 */
export async function checkPendingSoftReload(): Promise<SoftReloadResult> {
  return invoke<SoftReloadResult>('check_pending_soft_reload')
}
