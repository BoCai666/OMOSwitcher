/**
 * 预设存储服务
 * 管理用户保存的配置预设
 * 数据存储在 ~/.config/opencode/presets/ 目录
 */

import type { Preset, OhMyOpenCodeConfig } from '@/types'
import { sortConfigKeys, AGENT_NAMES, CATEGORY_NAMES } from '@/types'
import { AppError, ErrorCode } from '@/utils/errorHandler'
import {
  getCurrentPreset as getSettingsCurrentPreset,
  setCurrentPreset as setSettingsCurrentPreset,
  getLastUsedPreset as getSettingsLastUsedPreset,
  setLastUsedPreset as setSettingsLastUsedPreset,
  getPresetHistory as getSettingsPresetHistory,
  recordPresetUsage as recordSettingsPresetUsage
} from './settingsStore'

// 内存缓存
let presetsCache: Preset[] | null = null

/**
 * 动态导入 Tauri API
 */
async function getTauriInvoke() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke
  } catch {
    return null
  }
}

/**
 * 获取所有预设列表（使用合并命令，避免 N+1 问题）
 * @returns 预设数组，失败时返回空数组
 */
export async function listPresets(): Promise<Preset[]> {
  // 返回缓存
  if (presetsCache) {
    return [...presetsCache]
  }

  try {
    const invoke = await getTauriInvoke()
    if (!invoke) {
      return []
    }

    // 使用新的合并命令，一次性获取所有预设
    const content = await invoke<string>('read_all_presets')
    if (content) {
      const presets = JSON.parse(content) as Preset[]
      presetsCache = presets
      return [...presets]
    }
  } catch (error) {
    console.error('读取预设列表失败:', error)
  }

  return []
}

/**
 * 同步版本的获取预设列表（用于兼容旧代码）
 * 注意：首次调用可能返回空数组，建议使用异步版本
 */
export function listPresetsSync(): Preset[] {
  return presetsCache || []
}

/**
 * 保存预设（新建或更新）
 * @param name 预设名称
 * @param config 配置内容
 * @param description 可选的描述信息
 * @returns 保存后的预设对象
 * @throws {AppError} 保存失败时抛出错误
 */
export async function savePreset(
  name: string,
  config: OhMyOpenCodeConfig,
  description?: string
): Promise<Preset> {
  const presets = await listPresets()
  const now = new Date().toISOString()

  // 检查是否已存在同名预设
  const existingIndex = presets.findIndex((p) => p.name === name)

  // 如果描述为空且预设已存在，保留原有描述
  const finalDescription = description ?? (existingIndex >= 0 ? presets[existingIndex].description : undefined)

  const preset: Preset = {
    name,
    description: finalDescription,
    config,
    createdAt: existingIndex >= 0 ? presets[existingIndex].createdAt : now,
    updatedAt: now
  }

  try {
    const invoke = await getTauriInvoke()
    if (!invoke) {
      throw new AppError('无法保存预设：Tauri API 不可用', ErrorCode.PRESET_SAVE_FAILED)
    }

    // 保存到文件系统，按固定顺序排列 agents 和 categories 内部键
    const sortedConfig: OhMyOpenCodeConfig = {
      ...preset.config,
      agents: sortConfigKeys(preset.config.agents, AGENT_NAMES),
      categories: sortConfigKeys(preset.config.categories, CATEGORY_NAMES)
    }
    
    const content = JSON.stringify({
      config: sortedConfig,
      description: preset.description,
      createdAt: preset.createdAt,
      updatedAt: preset.updatedAt
    }, null, 2)

    await invoke('save_preset', { name, content })

    // 更新缓存
    if (existingIndex >= 0) {
      presets[existingIndex] = preset
    } else {
      presets.push(preset)
    }
    presetsCache = presets

    return preset
  } catch (error) {
    throw new AppError(
      '预设保存失败: ' + (error instanceof Error ? error.message : '存储错误'),
      ErrorCode.PRESET_SAVE_FAILED
    )
  }
}

/**
 * 加载指定名称的预设
 * @param name 预设名称
 * @returns 预设对象，不存在时返回 null
 */
export async function loadPreset(name: string): Promise<Preset | null> {
  const presets = await listPresets()
  return presets.find((p) => p.name === name) || null
}

/**
 * 删除指定名称的预设
 * @param name 预设名称
 * @returns 删除成功返回 true，预设不存在返回 false
 */
export async function deletePreset(name: string): Promise<boolean> {
  const presets = await listPresets()
  const index = presets.findIndex((p) => p.name === name)

  if (index < 0) {
    return false
  }

  try {
    const invoke = await getTauriInvoke()
    if (!invoke) {
      throw new Error('Tauri API 不可用')
    }

    await invoke('delete_preset', { name })

    // 更新缓存
    presets.splice(index, 1)
    presetsCache = presets

    return true
  } catch (error) {
    throw new AppError(
      '预设删除失败: ' + (error instanceof Error ? error.message : '存储错误'),
      ErrorCode.PRESET_DELETE_FAILED
    )
  }
}

/**
 * 检查预设是否存在
 * @param name 预设名称
 * @returns 存在返回 true，否则返回 false
 */
export async function presetExists(name: string): Promise<boolean> {
  const presets = await listPresets()
  return presets.some((p) => p.name === name)
}

/**
 * 清空所有预设
 */
export async function clearPresets(): Promise<void> {
  const presets = await listPresets()
  const invoke = await getTauriInvoke()
  
  if (!invoke) return

  for (const preset of presets) {
    try {
      await invoke('delete_preset', { name: preset.name })
    } catch (e) {
      console.error(`删除预设 ${preset.name} 失败:`, e)
    }
  }

  presetsCache = []
}

// ==================== 预设切换相关函数 ====================

/**
 * 获取当前激活的预设名称
 * @returns 当前预设名称，未设置时返回 undefined
 */
export async function getCurrentPreset(): Promise<string | undefined> {
  return await getSettingsCurrentPreset()
}

/**
 * 设置当前激活的预设
 * @param name 预设名称，传入 undefined 清除当前预设
 */
export async function setCurrentPreset(name: string | undefined): Promise<void> {
  await setSettingsCurrentPreset(name)
}

/**
 * 获取最近使用的预设名称
 * @returns 最近使用的预设名称，不存在时返回 undefined
 */
export async function getLastUsedPreset(): Promise<string | undefined> {
  return await getSettingsLastUsedPreset()
}

/**
 * 设置最近使用的预设
 * @param name 预设名称
 */
export async function setLastUsedPreset(name: string): Promise<void> {
  await setSettingsLastUsedPreset(name)
}

/**
 * 预设切换结果
 */
export interface SwitchPresetResult {
  success: boolean
  preset: Preset | null
  error?: string
}

/**
 * 切换预设的核心逻辑
 * 此函数仅负责加载预设配置，不负责检查未保存更改（由调用方负责）
 * 
 * @param name 目标预设名称
 * @returns 切换结果
 */
export async function switchPreset(name: string): Promise<SwitchPresetResult> {
  // 加载预设
  const preset = await loadPreset(name)
  
  if (!preset) {
    return {
      success: false,
      preset: null,
      error: `预设 "${name}" 不存在`
    }
  }
  
  // 更新当前预设状态
  await setCurrentPreset(name)
  
  // 记录为最近使用的预设
  await setLastUsedPreset(name)
  
  // 记录使用历史
  await recordPresetUsage(name)
  
  return {
    success: true,
    preset
  }
}

/**
 * 获取最近使用的预设列表（按使用时间倒序）
 * @param limit 返回数量限制，默认 5 个
 * @returns 预设名称数组
 */
export async function getRecentPresets(limit: number = 5): Promise<string[]> {
  return await getSettingsPresetHistory(limit)
}

/**
 * 记录预设使用历史
 * @param name 预设名称
 */
export async function recordPresetUsage(name: string): Promise<void> {
  await recordSettingsPresetUsage(name)
}

/**
 * 初始化预设存储
 * 在应用启动时调用
 */
export async function initPresetStore(): Promise<void> {
  await listPresets()
}
