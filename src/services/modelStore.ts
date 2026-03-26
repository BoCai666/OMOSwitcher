/**
 * 模型列表存储模块
 * 管理用户自定义的模型列表，格式为 provider/model-name
 * 数据存储在 ~/.config/omoswitcher/models.json
 * 若 models.json 不存在，则从 opencode.json 的 provider 字段读取默认模型
 */

import type { Model, OpenCodeConfig } from '@/types'
import { AppError, ErrorCode } from '@/utils/errorHandler'

// 默认模型列表
const DEFAULT_MODELS: Model[] = [
  { id: 'wuwen/glm-5', name: 'GLM-5', provider: 'wuwen' },
  { id: 'wuwen/minimax-m2.5', name: 'MiniMax M2.5', provider: 'wuwen' },
  { id: 'wuwen/minimax-m2.7', name: 'MiniMax M2.7', provider: 'wuwen' },
  { id: 'wuwen/kimi-k2.5', name: 'Kimi K2.5', provider: 'wuwen' },
]

// 内存缓存
let modelsCache: Model[] | null = null

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
 * 验证模型 ID 格式
 * 格式要求: provider/model-name
 * provider: 字母、数字、下划线、连字符
 * model-name: 字母、数字、下划线、点、连字符
 */
export function validateModelId(id: string): boolean {
  return /^[a-zA-Z0-9_-]+\/[a-zA-Z0-9_.-]+$/.test(id)
}

/**
 * 从模型 ID 解析 provider
 */
export function parseProvider(id: string): string | null {
  const match = id.match(/^([a-zA-Z0-9_-]+)\//)
  return match ? match[1] : null
}

/**
 * 获取所有模型
 */
export async function listModels(): Promise<Model[]> {
  // 返回缓存
  if (modelsCache) {
    return [...modelsCache]
  }

  try {
    const invoke = await getTauriInvoke()
    if (invoke) {
      // 尝试读取 models.json
      const content = await invoke<string>('read_models')
      if (content) {
        modelsCache = JSON.parse(content)
        if (modelsCache && modelsCache.length > 0) {
          return [...modelsCache]
        }
      }
    }
  } catch (error) {
    // models.json 不存在，尝试从 opencode.json 读取
    console.log('读取模型列表失败，尝试从 opencode.json 读取:', error)
  }

  // models.json 不存在或为空，从 opencode.json 的 provider 字段读取默认模型
  const modelsFromProvider = await loadModelsFromOpenCodeProvider()
  if (modelsFromProvider.length > 0) {
    modelsCache = modelsFromProvider
    return [...modelsFromProvider]
  }

  // 如果 opencode.json 也没有 provider 字段，使用硬编码默认值
  return [...DEFAULT_MODELS]
}

/**
 * 从 opencode.json 的 provider 字段加载模型列表
 * 遍历所有 provider，提取其中的模型信息
 */
async function loadModelsFromOpenCodeProvider(): Promise<Model[]> {
  try {
    const invoke = await getTauriInvoke()
    if (!invoke) {
      return []
    }

    // 读取 opencode.json
    const content = await invoke<string>('read_opencode_config')
    if (!content) {
      return []
    }

    const config = JSON.parse(content) as OpenCodeConfig
    if (!config.provider) {
      return []
    }

    const models: Model[] = []

    // 遍历所有 provider
    for (const [providerName, providerConfig] of Object.entries(config.provider)) {
      if (!providerConfig.models) {
        continue
      }

      // 遍历该 provider 下的所有模型
      for (const [modelId, modelConfig] of Object.entries(providerConfig.models)) {
        // 模型 ID 格式: provider/model-name
        const fullId = `${providerName}/${modelId}`
        models.push({
          id: fullId,
          name: modelConfig.name || modelId,
          provider: providerName
        })
      }
    }

    return models
  } catch (error) {
    console.warn('从 opencode.json 读取 provider 失败:', error)
    return []
  }
}

/**
 * 同步版本的获取模型列表（用于兼容旧代码）
 * 注意：首次调用可能返回默认值，建议使用异步版本
 */
export function listModelsSync(): Model[] {
  return modelsCache ? [...modelsCache] : [...DEFAULT_MODELS]
}

/**
 * 保存模型列表到文件
 */
async function saveModels(models: Model[]): Promise<void> {
  const invoke = await getTauriInvoke()
  if (!invoke) {
    throw new AppError('无法保存模型：Tauri API 不可用', ErrorCode.MODEL_ADD_FAILED)
  }

  await invoke('write_models', { content: JSON.stringify(models, null, 2) })
  modelsCache = models
}

/**
 * 添加模型
 * @throws {AppError} 如果模型 ID 格式无效或模型已存在
 */
export async function addModel(model: Model): Promise<Model> {
  if (!validateModelId(model.id)) {
    throw new AppError(
      '模型 ID 格式无效，应为 provider/model-name 格式',
      ErrorCode.MODEL_ADD_FAILED
    )
  }

  const models = await listModels()
  if (models.find(m => m.id === model.id)) {
    throw new AppError(
      '该模型 ID 已存在',
      ErrorCode.MODEL_DUPLICATE
    )
  }

  try {
    models.push(model)
    await saveModels(models)
    return model
  } catch (error) {
    throw new AppError(
      '模型保存失败: ' + (error instanceof Error ? error.message : '未知错误'),
      ErrorCode.MODEL_ADD_FAILED
    )
  }
}

/**
 * 更新模型
 * @returns 更新后的模型，如果模型不存在则返回 null
 */
export async function updateModel(id: string, updates: Partial<Model>): Promise<Model | null> {
  const models = await listModels()
  const index = models.findIndex(m => m.id === id)
  if (index < 0) return null

  models[index] = { ...models[index], ...updates }
  await saveModels(models)
  return models[index]
}

/**
 * 删除模型
 * @returns 是否删除成功
 */
export async function deleteModel(id: string): Promise<boolean> {
  const models = await listModels()
  const index = models.findIndex(m => m.id === id)
  if (index < 0) return false

  models.splice(index, 1)
  await saveModels(models)
  return true
}

/**
 * 重置为默认模型
 */
export async function resetModels(): Promise<Model[]> {
  await saveModels([...DEFAULT_MODELS])
  return [...DEFAULT_MODELS]
}

/**
 * 根据 ID 获取单个模型
 */
export async function getModelById(id: string): Promise<Model | undefined> {
  const models = await listModels()
  return models.find(m => m.id === id)
}

/**
 * 检查模型是否存在
 */
export async function modelExists(id: string): Promise<boolean> {
  const models = await listModels()
  return models.some(m => m.id === id)
}

/**
 * 获取默认模型列表（只读副本）
 */
export function getDefaultModels(): Model[] {
  return [...DEFAULT_MODELS]
}

/**
 * 按供应商分组模型
 * 返回 Map<provider, Model[]>
 */
export async function groupModelsByProvider(models?: Model[]): Promise<Map<string, Model[]>> {
  const modelList = models || await listModels()
  const groups = new Map<string, Model[]>()

  for (const model of modelList) {
    const provider = model.provider
    if (!groups.has(provider)) {
      groups.set(provider, [])
    }
    groups.get(provider)!.push(model)
  }

  return groups
}

/**
 * 获取所有供应商列表（去重）
 */
export async function getProviders(): Promise<string[]> {
  const models = await listModels()
  const providers = new Set(models.map(m => m.provider))
  return Array.from(providers).sort()
}

/**
 * 初始化模型存储
 * 在应用启动时调用
 */
export async function initModelStore(): Promise<void> {
  await listModels()
}
