/**
 * 配置文件读写模块
 * 通过 Tauri 命令读写配置文件
 */

import type { OhMyOpenCodeConfig } from '@/types'
import { createDefaultConfig, sortConfigKeys, AGENT_NAMES, CATEGORY_NAMES } from '@/types'

/**
 * 读取配置
 * 调用 Tauri 命令读取配置文件
 * @returns 配置对象，如果配置不存在则返回默认配置
 */
export async function readConfig(): Promise<OhMyOpenCodeConfig> {
  try {
    // 动态导入 Tauri API
    const { invoke } = await import('@tauri-apps/api/core')
    
    // 调用 Tauri 命令读取配置
    const content = await invoke<string>('read_config')
    
    // 解析 JSON
    return JSON.parse(content) as OhMyOpenCodeConfig
  } catch (error) {
    // 配置文件不存在或解析失败，返回默认配置
    console.warn('读取配置文件失败，使用默认配置:', error)
    return createDefaultConfig()
  }
}

/**
 * 检查配置是否存在
 * @returns 配置是否存在
 */
export async function configExists(): Promise<boolean> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke<string>('read_config')
    return true
  } catch {
    return false
  }
}

/**
 * 写入配置
 * 保留配置文件中的其他字段（如 google_auth, auto_update 等）
 * 只更新 agents 和 categories 部分
 * @param config 配置对象
 */
export async function writeConfig(config: OhMyOpenCodeConfig): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    
    // 先读取现有配置，保留其他字段
    let fullConfig: Record<string, unknown>
    try {
      const existingContent = await invoke<string>('read_config')
      fullConfig = JSON.parse(existingContent) as Record<string, unknown>
    } catch {
      // 文件不存在，使用传入的配置作为基础
      fullConfig = { ...config }
    }
    
    // 只更新 agents 和 categories，保留其他字段
    // 按固定顺序重排 agents 和 categories 内部键，防止预设切换打乱字段顺序
    const sortedAgents = sortConfigKeys(config.agents, AGENT_NAMES)
    const sortedCategories = sortConfigKeys(config.categories, CATEGORY_NAMES)
    
    fullConfig = {
      ...fullConfig,
      $schema: config.$schema,
      agents: sortedAgents,
      categories: sortedCategories
    }
    
    // 序列化为 JSON 字符串
    const content = JSON.stringify(fullConfig, null, 2)
    
    // 调用 Tauri 命令写入配置
    await invoke('write_config', { content })
  } catch (error) {
    throw new Error('配置文件写入失败: ' + (error as Error).message)
  }
}

/**
 * 删除配置（重置为默认配置）
 * 注意：不是真正删除文件，而是写入默认配置，但保留其他字段
 */
export async function deleteConfig(): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    
    // 读取现有配置，保留其他字段
    let fullConfig: Record<string, unknown>
    try {
      const existingContent = await invoke<string>('read_config')
      fullConfig = JSON.parse(existingContent) as Record<string, unknown>
    } catch {
      fullConfig = {}
    }
    
    // 重置 agents 和 categories 为默认值
    const defaultConf = createDefaultConfig()
    const sortedAgents = sortConfigKeys(defaultConf.agents, AGENT_NAMES)
    const sortedCategories = sortConfigKeys(defaultConf.categories, CATEGORY_NAMES)
    
    fullConfig = {
      ...fullConfig,
      $schema: defaultConf.$schema,
      agents: sortedAgents,
      categories: sortedCategories
    }
    
    const content = JSON.stringify(fullConfig, null, 2)
    await invoke('write_config', { content })
  } catch (error) {
    throw new Error('重置配置失败: ' + (error as Error).message)
  }
}
