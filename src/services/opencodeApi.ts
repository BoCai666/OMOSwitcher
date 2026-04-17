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