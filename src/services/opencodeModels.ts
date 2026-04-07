/**
 * OpenCode 模型注册表服务
 * 从 ~/.cache/opencode/models.json 读取全量供应商和模型信息
 */

import type { RegistryProvider } from '@/types/config'

async function getTauriInvoke() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke
  } catch {
    return null
  }
}

/** 缓存 */
let registryCache: Record<string, RegistryProvider> | null = null
let availableIdsCache: string[] | null = null
let customIdsCache: string[] | null = null

/**
 * 读取模型注册表（全量）
 * 数据来自 ~/.cache/opencode/models.json
 * @throws 如果读取失败则抛出错误
 */
export async function readModelsRegistry(): Promise<Record<string, RegistryProvider>> {
  if (registryCache) return registryCache

  const invoke = await getTauriInvoke()
  if (!invoke) {
    throw new Error('Tauri API 不可用')
  }

  const content = await invoke<string>('read_opencode_models_cache')
  const data = JSON.parse(content) as Record<string, RegistryProvider>
  registryCache = data
  return data
}

/**
 * 获取已配置（可用）的供应商 ID 列表
 */
export async function getAvailableProviderIds(): Promise<string[]> {
  if (availableIdsCache) return availableIdsCache

  const invoke = await getTauriInvoke()
  if (!invoke) return []

  try {
    const ids = await invoke<string[]>('get_available_provider_ids')
    availableIdsCache = ids
    return ids
  } catch (error) {
    console.error('获取可用供应商失败:', error)
    return []
  }
}

/**
 * 获取 opencode.json 中手动配置的供应商 ID 列表
 */
export async function getCustomProviderIds(): Promise<string[]> {
  if (customIdsCache) return customIdsCache

  const invoke = await getTauriInvoke()
  if (!invoke) return []

  try {
    const ids = await invoke<string[]>('get_custom_provider_ids')
    customIdsCache = ids
    return ids
  } catch (error) {
    console.error('获取自定义供应商失败:', error)
    return []
  }
}

/** 带可用标记和自定义标记的供应商信息 */
export interface ProviderWithAvailability extends RegistryProvider {
  available: boolean
  custom: boolean
  modelCount: number
}

/**
 * 获取供应商列表（带可用标记和自定义标记）
 */
export async function getProvidersWithAvailability(): Promise<ProviderWithAvailability[]> {
  const [registry, availableIds, customIds] = await Promise.all([
    readModelsRegistry(),
    getAvailableProviderIds(),
    getCustomProviderIds()
  ])
  const availableSet = new Set(availableIds)
  const customSet = new Set(customIds)

  return Object.values(registry)
    .map(p => ({
      ...p,
      name: p.name || p.id,
      available: availableSet.has(p.id),
      custom: customSet.has(p.id),
      modelCount: Object.keys(p.models || {}).length
    }))
    .sort((a, b) => {
      // 可用的排前面，然后按名称排序
      if (a.available !== b.available) return a.available ? -1 : 1
      return (a.name || a.id).localeCompare(b.name || b.id)
    })
}

/** 清除缓存（配置变更时调用） */
export function clearRegistryCache(): void {
  registryCache = null
  availableIdsCache = null
  customIdsCache = null
}
