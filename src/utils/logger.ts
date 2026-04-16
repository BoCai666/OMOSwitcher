/**
 * 日志工具
 *
 * 开发模式 (npm run dev / tauri:dev)：log/warn/error 全部输出到 console
 * 生产构建 (npm run tauri:build)：log 调用被 Vite 静态消除（tree-shaking），
 *   仅保留 warn 和 error
 *
 * 使用方式：
 *   import { log, warn, error } from '@/utils/logger'
 *   log(`[模块名] 消息: ${数据}`)     // 生产构建时整行消除
 *   warn(`[模块名] 警告`)             // 始终保留
 *   error(`[模块名] 错误:`, e)        // 始终保留
 */

const noop = () => {}

/**
 * 开发模式日志 — 生产构建时完全消除（dead code elimination）
 *
 * 原理：Vite 将 import.meta.env.DEV 静态替换为 true/false 字面量，
 * 条件永远为 false 的分支在 Rollup tree-shaking 阶段被移除，
 * 产物中不含任何 console.log 调用。
 */
export const log = import.meta.env.DEV
  ? (message: unknown, ...args: unknown[]) => console.log(message, ...args)
  : noop

/**
 * 调试日志 — 同 log，生产构建消除
 */
export const debug = import.meta.env.DEV
  ? (message: unknown, ...args: unknown[]) => console.debug(message, ...args)
  : noop

/**
 * 警告日志 — 始终保留，用于重要提示
 */
export const warn = (message: unknown, ...args: unknown[]) => console.warn(message, ...args)

/**
 * 错误日志 — 始终保留，用于错误追踪
 */
export const error = (message: unknown, ...args: unknown[]) => console.error(message, ...args)
