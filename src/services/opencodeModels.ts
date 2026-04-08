/**
 * OpenCode 模型注册表服务
 * 从 ~/.cache/opencode/models.json 读取全量供应商和模型信息
 * 同时合并 opencode.json 中的自定义供应商
 */

import type { RegistryProvider, RegistryModel } from '@/types/config'

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
let customProvidersCache: Map<string, RegistryProvider> | null = null

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
 * 获取已配置（可用)的供应商 ID 列表
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

/**
 * 从 opencode.json 读取自定义供应商配置并转换为 RegistryProvider 格式
 */
async function readCustomProviders(): Promise<Map<string, RegistryProvider>> {
  if (customProvidersCache) return customProvidersCache

  const invoke = await getTauriInvoke()
  if (!invoke) return new Map()

  try {
    const content = await invoke<string>('read_opencode_config')
    const config = JSON.parse(content)
    const providers = new Map<string, RegistryProvider>()

    if (config.provider && typeof config.provider === 'object') {
      for (const [providerId, providerConfig] of Object.entries(config.provider) as [string, any][]) {
        // 检查是否有 apiKey（自定义配置的标志）
        const hasApiKey = providerConfig?.apiKey || providerConfig?.options?.apiKey
        if (!hasApiKey) continue

        // 将 opencode.json 中的 provider 转换为 RegistryProvider 格式
        const models: Record<string, RegistryModel> = {}
        if (providerConfig?.models && typeof providerConfig.models === 'object') {
          for (const [modelId, modelConfig] of Object.entries(providerConfig.models) as [string, any][]) {
            models[modelId] = {
              id: modelId,
              name: modelConfig?.name || modelId,
              family: modelConfig?.family,
              tool_call: modelConfig?.tool_call ?? false,
              reasoning: modelConfig?.reasoning ?? false,
              attachment: modelConfig?.attachment ?? false,
              limit: modelConfig?.limit,
              modalities: modelConfig?.modalities,
              release_date: modelConfig?.release_date,
              open_weights: modelConfig?.open_weights
            }
          }
        }

        providers.set(providerId, {
          id: providerId,
          name: providerConfig?.name || providerId,
          api: providerConfig?.options?.baseURL,
          npm: providerConfig?.npm,
          models
        })
      }
    }

    customProvidersCache = providers
    return providers
  } catch (error) {
    console.error('读取自定义供应商配置失败:', error)
    return new Map()
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
 * 合并 models.json 注册表和 opencode.json 中的自定义供应商
 */
export async function getProvidersWithAvailability(): Promise<ProviderWithAvailability[]> {
  const [registry, availableIds, customIds, customProviders] = await Promise.all([
    readModelsRegistry(),
    getAvailableProviderIds(),
    getCustomProviderIds(),
    readCustomProviders()
  ])

  const availableSet = new Set(availableIds)
  const customSet = new Set(customIds)

  console.log('[opencodeModels] availableSet:', [...availableSet])
  console.log('[opencodeModels] customSet:', [...customSet])
  console.log('[opencodeModels] customProviders:', [...customProviders.keys()])

  // 合并注册表和自定义供应商
  const allProviders = new Map<string, RegistryProvider>()

  // 1. 添加注册表中的供应商
  for (const [id, provider] of Object.entries(registry)) {
    allProviders.set(id, provider)
  }

  // 2. 添加自定义供应商（如果不在注册表中）
  for (const [id, provider] of customProviders) {
    if (!allProviders.has(id)) {
      allProviders.set(id, provider)
    }
  }

  console.log('[opencodeModels] 合并后供应商数量:', allProviders.size)

  return Array.from(allProviders.values())
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
  customProvidersCache = null
}

/**
 * 删除 opencode.json 中指定的自定义 provider
 * 调用后端 Tauri 命令删除并清除缓存
 * @param providerId 要删除的 provider ID
 */
export async function deleteCustomProvider(providerId: string): Promise<void> {
  const invoke = await getTauriInvoke()
  if (!invoke) {
    throw new Error('Tauri API 不可用')
  }

  await invoke('delete_custom_provider', { providerId })
  // 清除缓存以便下次加载时获取最新数据
  clearRegistryCache()
}

/** 自定义 provider 配置参数 */
export interface CustomProviderConfig {
  /** npm 包名（如 @ai-sdk/openai-compatible） */
  npm?: string
  /** 供应商显示名称 */
  name?: string
  /** 通用选项 */
  options?: {
    apiKey?: string
    baseURL?: string
    timeout?: number
    headers?: Record<string, string>
  }
  /** 模型配置（key 为模型 ID） */
  models?: Record<string, {
    name?: string
    disabled?: boolean
    reasoning?: boolean | { type: string; budgetTokens?: number }
    limit?: { context?: number; output?: number }
    modalities?: { input?: string[]; output?: string[] }
    variants?: Record<string, Record<string, unknown>>
  }>
}

/**
 * 添加自定义 provider 到 opencode.json
 * 调用后端 Tauri 命令写入并清除缓存
 * @param providerId 供应商 ID（kebab-case）
 * @param config provider 配置
 */
export async function addCustomProvider(providerId: string, config: CustomProviderConfig): Promise<void> {
  const invoke = await getTauriInvoke()
  if (!invoke) {
    throw new Error('Tauri API 不可用')
  }

  const configJson = JSON.stringify(config)
  await invoke('add_custom_provider', { providerId, configJson })
  // 清除缓存以便下次加载时获取最新数据
  clearRegistryCache()
}

/** 可用模型类型 */
export interface AvailableModel {
  id: string
  name: string
  provider: string
  providerName: string
  available: boolean
  custom: boolean
  tool_call: boolean
  reasoning: boolean
  attachment: boolean
  limit?: { context?: number; output?: number }
}

/**
 * 获取所有可用的模型（来自可用的供应商)
 * 返回带可用性标记的模型列表
 */
export async function getAvailableModels(): Promise<AvailableModel[]> {
  const providers = await getProvidersWithAvailability()
  
  const models: AvailableModel[] = []
  
  for (const provider of providers) {
    if (!provider.available) continue
    
    const providerModels = Object.values(provider.models || {})
    for (const model of providerModels) {
      models.push({
        id: `${provider.id}/${model.id}`,
        name: model.name || model.id,
        provider: provider.id,
        providerName: provider.name || provider.id,
        available: true,
        custom: provider.custom,
        tool_call: model.tool_call || false,
        reasoning: model.reasoning || false,
        attachment: model.attachment || false,
        limit: model.limit
      })
    }
  }
  
  // 按供应商和模型名称排序
  return models.sort((a, b) => {
    if (a.provider !== b.provider) {
      return a.provider.localeCompare(b.provider)
    }
    return a.name.localeCompare(b.name)
  })
}
