/**
 * 模型状态管理
 * 管理用户自定义的模型列表
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Model } from '@/types'
import {
  listModels,
  addModel as addModelToStorage,
  updateModel as updateModelInStorage,
  deleteModel as deleteModelFromStorage,
  getModelById,
  resetModels as resetModelsInStorage,
  validateModelId
} from '@/services'

export const useModelStore = defineStore('model', () => {
  // ========== 状态 ==========

  // 模型列表
  const models = ref<Model[]>([])

  // 是否已加载
  const isLoaded = ref(false)

  // 是否正在加载
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // ========== 计算属性 ==========

  // 模型数量
  const modelCount = computed(() => models.value.length)

  // 是否有模型
  const hasModels = computed(() => models.value.length > 0)

  // 所有提供商列表（去重）
  const providers = computed(() => {
    const providerSet = new Set(models.value.map(m => m.provider))
    return Array.from(providerSet).sort()
  })

  // 按提供商分组的模型
  const modelsByProvider = computed(() => {
    const grouped: Record<string, Model[]> = {}
    for (const model of models.value) {
      if (!grouped[model.provider]) {
        grouped[model.provider] = []
      }
      grouped[model.provider].push(model)
    }
    return grouped
  })

  // ========== 方法 ==========

  /**
   * 加载所有模型
   */
  async function loadModels(): Promise<void> {
    if (isLoading.value) return
    
    try {
      isLoading.value = true
      error.value = null
      models.value = await listModels()
      isLoaded.value = true
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 添加模型
   * @param model 模型信息
   * @returns 添加后的模型
   * @throws 如果模型 ID 格式无效或模型已存在
   */
  async function addModel(model: Model): Promise<Model> {
    try {
      error.value = null

      // 验证模型 ID 格式
      if (!validateModelId(model.id)) {
        throw new Error('模型 ID 格式无效，应为 provider/model-name 格式')
      }

      const newModel = await addModelToStorage(model)
      models.value = await listModels()
      return newModel
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  /**
   * 更新模型
   * @param id 模型 ID
   * @param updates 更新内容
   * @returns 更新后的模型，不存在时返回 null
   */
  async function updateModel(id: string, updates: Partial<Model>): Promise<Model | null> {
    try {
      error.value = null
      const updated = await updateModelInStorage(id, updates)
      if (updated) {
        models.value = await listModels()
      }
      return updated
    } catch (e) {
      error.value = (e as Error).message
      return null
    }
  }

  /**
   * 删除模型
   * @param id 模型 ID
   * @returns 是否删除成功
   */
  async function deleteModel(id: string): Promise<boolean> {
    try {
      error.value = null
      const success = await deleteModelFromStorage(id)
      if (success) {
        models.value = await listModels()
      }
      return success
    } catch (e) {
      error.value = (e as Error).message
      return false
    }
  }

  /**
   * 根据 ID 获取模型
   * @param id 模型 ID
   */
  async function getModel(id: string): Promise<Model | undefined> {
    return await getModelById(id)
  }

  /**
   * 检查模型是否存在
   * @param id 模型 ID
   */
  function modelExists(id: string): boolean {
    return models.value.some(m => m.id === id)
  }

  /**
   * 按提供商筛选模型
   * @param provider 提供商名称
   */
  function getModelsByProvider(provider: string): Model[] {
    return models.value.filter(m => m.provider === provider)
  }

  /**
   * 重置为默认模型列表
   */
  async function resetModels(): Promise<void> {
    try {
      error.value = null
      models.value = await resetModelsInStorage()
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  /**
   * 重置 store 状态
   */
  function reset(): void {
    models.value = []
    isLoaded.value = false
    error.value = null
  }

  /**
   * 清除错误状态
   */
  function clearError(): void {
    error.value = null
  }

  return {
    // 状态
    models,
    isLoaded,
    isLoading,
    error,

    // 计算属性
    modelCount,
    hasModels,
    providers,
    modelsByProvider,

    // 方法
    loadModels,
    addModel,
    updateModel,
    deleteModel,
    getModel,
    modelExists,
    getModelsByProvider,
    resetModels,
    reset,
    clearError
  }
})
