/**
 * 预设状态管理
 * 管理用户保存的配置预设
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Preset, OhMyOpenCodeConfig } from '@/types'
import { listPresets, savePreset as savePresetToStorage, loadPreset as loadPresetFromStorage, deletePreset as deletePresetFromStorage, clearPresets as clearPresetsFromStorage } from '@/services'

export const usePresetStore = defineStore('preset', () => {
  // ========== 状态 ==========

  // 预设列表
  const presets = ref<Preset[]>([])

  // 当前选中的预设
  const currentPreset = ref<Preset | null>(null)

  // 是否已加载
  const isLoaded = ref(false)

  // 是否正在加载
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // ========== 计算属性 ==========

  // 预设数量
  const presetCount = computed(() => presets.value.length)

  // 是否有预设
  const hasPresets = computed(() => presets.value.length > 0)

  // 当前预设名称
  const currentPresetName = computed(() => currentPreset.value?.name ?? null)

  // ========== 方法 ==========

  /**
   * 加载所有预设
   */
  async function loadPresets(): Promise<void> {
    if (isLoading.value) return
    
    try {
      isLoading.value = true
      error.value = null
      presets.value = await listPresets()
      isLoaded.value = true
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 切换预设
   * @param name 预设名称
   * @returns 预设配置，不存在时返回 null
   */
  async function switchPreset(name: string): Promise<OhMyOpenCodeConfig | null> {
    const preset = await loadPresetFromStorage(name)
    if (preset) {
      currentPreset.value = preset
      return preset.config
    }
    return null
  }

  /**
   * 保存当前配置为新预设
   * @param name 预设名称
   * @param config 配置内容
   * @param description 可选的描述信息
   * @returns 保存后的预设对象
   */
  async function savePresetAs(name: string, config: OhMyOpenCodeConfig, description?: string): Promise<Preset> {
    try {
      error.value = null
      const preset = await savePresetToStorage(name, config, description)
      // 刷新预设列表
      presets.value = await listPresets()
      currentPreset.value = preset
      return preset
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  /**
   * 更新现有预设
   * @param name 预设名称
   * @param config 新配置内容
   * @param description 可选的新描述
   * @returns 更新后的预设，不存在时返回 null
   */
  async function updatePreset(name: string, config: OhMyOpenCodeConfig, description?: string): Promise<Preset | null> {
    const existing = presets.value.find(p => p.name === name)
    if (!existing) return null

    return await savePresetAs(name, config, description ?? existing.description)
  }

  /**
   * 删除预设
   * @param name 预设名称
   * @returns 是否删除成功
   */
  async function deletePreset(name: string): Promise<boolean> {
    try {
      error.value = null
      const success = await deletePresetFromStorage(name)
      if (success) {
        presets.value = await listPresets()
        // 如果删除的是当前预设，清除当前预设
        if (currentPreset.value?.name === name) {
          currentPreset.value = null
        }
      }
      return success
    } catch (e) {
      error.value = (e as Error).message
      return false
    }
  }

  /**
   * 检查预设是否存在
   * @param name 预设名称
   */
  function presetExists(name: string): boolean {
    return presets.value.some(p => p.name === name)
  }

  /**
   * 根据名称获取预设
   * @param name 预设名称
   */
  function getPresetByName(name: string): Preset | undefined {
    return presets.value.find(p => p.name === name)
  }

  /**
   * 清空所有预设
   */
  async function clearAllPresets(): Promise<void> {
    try {
      error.value = null
      await clearPresetsFromStorage()
      presets.value = []
      currentPreset.value = null
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  /**
   * 清除当前预设选择
   */
  function clearCurrentPreset(): void {
    currentPreset.value = null
  }

  /**
   * 重置 store 状态
   */
  function reset(): void {
    presets.value = []
    currentPreset.value = null
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
    presets,
    currentPreset,
    isLoaded,
    isLoading,
    error,

    // 计算属性
    presetCount,
    hasPresets,
    currentPresetName,

    // 方法
    loadPresets,
    switchPreset,
    savePresetAs,
    updatePreset,
    deletePreset,
    presetExists,
    getPresetByName,
    clearAllPresets,
    clearCurrentPreset,
    reset,
    clearError
  }
})
