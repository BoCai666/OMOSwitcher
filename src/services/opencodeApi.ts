/**
 * OpenCode Server 热重载模块
 * 通过 Rust 后端调用 OpenCode 内置 Server 的 PATCH /config API，
 * 在保存配置后将 agent/model 变更推送到运行中的 OpenCode，
 * 实现无需重启的热重载。
 *
 * 为什么 HTTP 调用放在 Rust 端：
 * Tauri 2 的 webview 中 fetch 外部地址受安全策略限制，
 * 需要额外的 capability 配置。通过 Rust 端的 reqwest 发请求
 * 可以绕过这个限制，且与项目现有的 monitorApi 模式一致。
 */

import { invoke } from '@tauri-apps/api/core'
import type { OhMyOpenCodeConfig } from '@/types'

// 热重载结果（与 Rust 端 HotReloadResult 对应）
export interface HotReloadResult {
  /** 是否成功推送到 OpenCode */
  success: boolean
  /** 可读的状态描述 */
  message: string
  /** 是否因为 OpenCode 未运行而跳过（不是错误） */
  skipped: boolean
}

/**
 * 热重载入口：将 OMOSwitcher 的配置推送到正在运行的 OpenCode。
 *
 * 此函数是"尽力而为"的：
 * - OpenCode 未运行 → 跳过，不报错
 * - OpenCode 运行但推送失败 → 返回失败信息
 * - 推送成功 → 返回成功信息
 *
 * 不应阻塞主保存流程，由调用方决定如何处理结果。
 */
export async function hotReloadConfig(
  config: OhMyOpenCodeConfig
): Promise<HotReloadResult> {
  return invoke<HotReloadResult>('hot_reload_opencode_config', {
    configJson: JSON.stringify(config),
  })
}
