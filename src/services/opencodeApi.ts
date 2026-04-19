/**
 * OpenCode 热重载 API 服务
 * 通过 Tauri 命令与 OpenCode Server 交互
 */
import { invoke } from '@tauri-apps/api/core'
import type { OhMyOpenCodeConfig } from '@/types'
import { log, warn } from '@/utils/logger'

/**
 * 检测 OpenCode Server 是否在指定端口运行
 */
export async function detectOpenCodeServer(port: number): Promise<boolean> {
  try {
    log(`[opencodeApi] detectOpenCodeServer 端口=${port}`)
    const result = await invoke<boolean>('detect_opencode_server', { port })
    log(`[opencodeApi] detectOpenCodeServer 结果=${result}`)
    return result
  } catch (e) {
    warn(`[opencodeApi] detectOpenCodeServer 异常:`, e)
    return false
  }
}

/**
 * 将 OhMyOpenCode 配置转换为 OpenCode PATCH body
 * 仅提取 agent 模型，跳过 category
 */
export function buildOpenCodeAgentConfig(config: OhMyOpenCodeConfig): Record<string, unknown> {
  const agentConfig: Record<string, { model: string }> = {}
  
  if (config.agents) {
    for (const [name, value] of Object.entries(config.agents)) {
      if (value?.model) {
        agentConfig[name] = { model: value.model }
      }
    }
  }
  
  const body = { agent: agentConfig }
  log(`[opencodeApi] buildOpenCodeAgentConfig: agentCount=${Object.keys(agentConfig).length}`)
  return body
}

/**
 * 向 OpenCode Server 发送热重载请求
 */
export async function hotReloadConfig(port: number, config: OhMyOpenCodeConfig): Promise<void> {
  const body = buildOpenCodeAgentConfig(config)
  log(`[opencodeApi] hotReloadConfig: port=${port}, body=${JSON.stringify(body).substring(0, 300)}`)
  await invoke('hot_reload_config', { port, config: body })
  log(`[opencodeApi] hotReloadConfig: invoke 完成`)
}

/**
 * 触发 OpenCode 实例重建（dispose + lazy rebuild）
 * 流程：POST /instance/dispose → GET /config/
 * 重建时 OhMyOpenCode 插件会重新调用 loadPluginConfig() 读取更新后的 oh-my-opencode.json
 */
export async function disposeInstance(port: number): Promise<void> {
  log(`[opencodeApi] disposeInstance: port=${port}`)
  await invoke('dispose_instance', { port })
  log(`[opencodeApi] disposeInstance: 完成`)
}

/**
 * 获取 OpenCode Server 上所有处于 busy 状态的会话 ID
 * 用于热重载前记录活跃会话，以便恢复
 */
export async function getActiveSessions(port: number): Promise<string[]> {
  try {
    log(`[opencodeApi] getActiveSessions: port=${port}`)
    const result = await invoke<string[]>('get_active_sessions', { port })
    log(`[opencodeApi] getActiveSessions: 发现 ${result.length} 个活跃会话`)
    return result
  } catch (e) {
    warn(`[opencodeApi] getActiveSessions 异常:`, e)
    return []
  }
}

/**
 * 向指定会话发送恢复消息（异步，不等待 agent 回复完成）
 * 通过 POST /session/:id/prompt_async 发送
 */
export async function resumeSession(port: number, sessionId: string, message: string): Promise<boolean> {
  try {
    log(`[opencodeApi] resumeSession: port=${port}, session=${sessionId}`)
    const result = await invoke<boolean>('resume_session', { port, sessionId, message })
    log(`[opencodeApi] resumeSession: 结果=${result}`)
    return result
  } catch (e) {
    warn(`[opencodeApi] resumeSession 异常:`, e)
    return false
  }
}

/**
 * 带会话恢复的热重载：dispose 前记录活跃会话，dispose + rebuild 后自动恢复
 * 返回成功恢复的会话数量
 */
export async function disposeAndResume(port: number): Promise<number> {
  try {
    log(`[opencodeApi] disposeAndResume: port=${port}`)
    const result = await invoke<number>('dispose_and_resume', { port })
    log(`[opencodeApi] disposeAndResume: 恢复了 ${result} 个会话`)
    return result
  } catch (e) {
    warn(`[opencodeApi] disposeAndResume 异常:`, e)
    throw e
  }
}