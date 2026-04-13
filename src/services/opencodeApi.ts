/**
 * OpenCode 热重载 API 服务
 * 通过 Tauri 命令与 OpenCode Server 交互
 */
import { invoke } from '@tauri-apps/api/core'
import type { OhMyOpenCodeConfig } from '@/types'

/**
 * 检测 OpenCode Server 是否在指定端口运行
 */
export async function detectOpenCodeServer(port: number): Promise<boolean> {
  try {
    return await invoke<boolean>('detect_opencode_server', { port })
  } catch {
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
  
  return { agent: agentConfig }
}

/**
 * 向 OpenCode Server 发送热重载请求
 */
export async function hotReloadConfig(port: number, config: OhMyOpenCodeConfig): Promise<void> {
  const body = buildOpenCodeAgentConfig(config)
  await invoke('hot_reload_config', { port, config: body })
}