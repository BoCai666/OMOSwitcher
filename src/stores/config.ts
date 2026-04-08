/**
 * 配置状态管理
 * 使用 Pinia defineStore 组合式 API
 */

import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { OhMyOpenCodeConfig, AgentName, CategoryName } from '@/types'
import { readConfig, writeConfig, savePreset, getCurrentPreset } from '@/services'

// 保存状态类型
export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'

// 防抖保存延迟时间（毫秒）
const DEBOUNCE_DELAY = 1000

// 状态提示自动清除时间（毫秒）
const STATUS_CLEAR_DELAY = 5000

export const useConfigStore = defineStore('config', () => {
  // ========== 状态 ==========

  // 配置文件路径（后续用于 Tauri 文件操作）
  const configPath = ref<string>('C:\\Users\\Administrator\\.config\\opencode\\oh-my-opencode.json')

  // 配置数据
  const config = ref<OhMyOpenCodeConfig | null>(null)

  // 原始配置（用于检测未保存更改和重置）
  const originalConfig = ref<OhMyOpenCodeConfig | null>(null)

  // 当前预设名称
  const currentPresetName = ref<string | null>(null)

  // 保存状态
  const saveStatus = ref<SaveStatus>('idle')

  // 是否有未保存的更改
  const isDirty = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 防抖定时器
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  // 状态自动清除定时器
  let statusClearTimer: ReturnType<typeof setTimeout> | null = null

  // ========== 计算属性 ==========

  // 是否已加载
  const isLoaded = computed(() => config.value !== null)

  // 是否正在保存
  const isSaving = computed(() => saveStatus.value === 'saving')

  // 是否保存成功
  const isSaved = computed(() => saveStatus.value === 'saved')

  // 是否有错误
  const hasError = computed(() => saveStatus.value === 'error')

  // 是否有未保存的更改（通过比较原始配置）
  const hasUnsavedChanges = computed(() => {
    if (!config.value || !originalConfig.value) return isDirty.value
    return JSON.stringify(config.value) !== JSON.stringify(originalConfig.value)
  })

  // 是否有当前预设
  const hasCurrentPreset = computed(() => currentPresetName.value !== null)

  // ========== 内部方法 ==========

  // 清除状态自动清理定时器
  function clearStatusClearTimer(): void {
    if (statusClearTimer) {
      clearTimeout(statusClearTimer)
      statusClearTimer = null
    }
  }

  // 安排状态自动清理
  function scheduleStatusCleanup(): void {
    clearStatusClearTimer()
    statusClearTimer = setTimeout(() => {
      if (saveStatus.value === 'saved') {
        saveStatus.value = 'idle'
      }
    }, STATUS_CLEAR_DELAY)
  }

  // 实际保存配置
  async function performSave(): Promise<void> {
    if (!config.value) return

    try {
      saveStatus.value = 'saving'
      error.value = null
      clearStatusClearTimer()
      
      // 保存主配置文件
      await writeConfig(config.value)
      
      // 如果有当前预设，同步更新预设文件
      if (currentPresetName.value) {
        await savePreset(currentPresetName.value, config.value)
      }
      
      saveStatus.value = 'saved'
      isDirty.value = false
      // 更新原始配置为当前配置
      originalConfig.value = JSON.parse(JSON.stringify(config.value))

      scheduleStatusCleanup()
    } catch (e) {
      saveStatus.value = 'error'
      error.value = (e as Error).message
      throw e
    }
  }

  // 防抖保存 - 配置变更后延迟保存
  function debouncedSave(): void {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }
    debounceTimer = setTimeout(() => {
      performSave()
      debounceTimer = null
    }, DEBOUNCE_DELAY)
  }

  // ========== 公共方法 ==========

  /**
   * 加载配置
   * 从文件系统读取配置，不存在则创建默认配置
   */
  async function loadConfig(): Promise<void> {
    try {
      error.value = null
      config.value = await readConfig()
      // 保存原始配置副本
      originalConfig.value = JSON.parse(JSON.stringify(config.value))
      // 从持久化存储加载当前预设名称
      currentPresetName.value = (await getCurrentPreset()) || null
      isDirty.value = false
      saveStatus.value = 'idle'
    } catch (e) {
      error.value = (e as Error).message
      saveStatus.value = 'error'
      throw e
    }
  }

  /**
   * 更新配置（局部更新）
   * @param updates 部分配置更新
   */
  function updateConfig(updates: Partial<OhMyOpenCodeConfig>): void {
    if (!config.value) {
      console.warn('配置未加载，无法更新')
      return
    }
    config.value = { ...config.value, ...updates }
    isDirty.value = true
    saveStatus.value = 'idle'
  }

  /**
   * 更新指定 Agent 的模型
   * @param agentName Agent 名称
   * @param model 模型 ID
   */
  function updateAgentModel(agentName: AgentName, model: string): void {
    if (!config.value) {
      console.warn('配置未加载，无法更新 Agent')
      return
    }
    config.value.agents[agentName] = { model }
    isDirty.value = true
    saveStatus.value = 'idle'
  }

  /**
   * 更新指定 Category 的模型
   * @param categoryName Category 名称
   * @param model 模型 ID
   */
  function updateCategoryModel(categoryName: CategoryName, model: string): void {
    if (!config.value) {
      console.warn('配置未加载，无法更新 Category')
      return
    }
    config.value.categories[categoryName] = { model }
    isDirty.value = true
    saveStatus.value = 'idle'
  }

  /**
   * 保存配置（立即保存，取消防抖）
   */
  async function saveConfig(): Promise<void> {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
      debounceTimer = null
    }
    await performSave()
  }

  /**
   * 防抖保存 - 配置变更后延迟保存
   * 用于自动保存场景
   */
  function autoSave(): void {
    if (isDirty.value && config.value) {
      debouncedSave()
    }
  }

  /**
   * 应用预设配置
   * @param presetConfig 预设的配置内容
   * @param presetName 预设名称（可选，用于跟踪当前预设）
   */
  function applyPreset(presetConfig: OhMyOpenCodeConfig, presetName?: string): void {
    // 深拷贝预设配置到当前配置
    config.value = JSON.parse(JSON.stringify(presetConfig))
    // 设置当前预设名称
    currentPresetName.value = presetName || null
    // 注意：不更新 originalConfig，这样用户可以检测到相对于原始配置的更改
    isDirty.value = true
    saveStatus.value = 'idle'
  }

  /**
   * 设置当前预设名称
   * @param name 预设名称，传入 null 清除当前预设
   */
  function setCurrentPreset(name: string | null): void {
    currentPresetName.value = name
  }

  /**
   * 清除当前预设（切换到无预设状态）
   */
  function clearCurrentPreset(): void {
    currentPresetName.value = null
  }

  /**
   * 重置到原始配置（放弃未保存的更改）
   */
  function resetToOriginal(): void {
    if (originalConfig.value) {
      config.value = JSON.parse(JSON.stringify(originalConfig.value))
      isDirty.value = false
      saveStatus.value = 'idle'
    }
  }

  /**
   * 重置配置状态
   */
  function reset(): void {
    config.value = null
    isDirty.value = false
    saveStatus.value = 'idle'
    error.value = null
    currentPresetName.value = null
    if (debounceTimer) {
      clearTimeout(debounceTimer)
      debounceTimer = null
    }
    clearStatusClearTimer()
  }

  /**
   * 清除错误状态
   */
  function clearError(): void {
    error.value = null
    if (saveStatus.value === 'error') {
      saveStatus.value = 'idle'
    }
  }

  // 监听配置变化，标记为脏数据（不自动保存，由用户手动保存）
  watch(
    config,
    (newValue, oldValue) => {
      // 仅在配置已加载且有实际变化时标记为脏
      if (newValue && oldValue && JSON.stringify(newValue) !== JSON.stringify(oldValue)) {
        isDirty.value = true
        saveStatus.value = 'idle'
      }
    },
    { deep: true }
  )

  return {
    // 状态
    configPath,
    config,
    originalConfig,
    currentPresetName,
    saveStatus,
    isDirty,
    error,

    // 计算属性
    isLoaded,
    isSaving,
    isSaved,
    hasError,
    hasUnsavedChanges,
    hasCurrentPreset,

    // 方法
    loadConfig,
    updateConfig,
    updateAgentModel,
    updateCategoryModel,
    saveConfig,
    autoSave,
    applyPreset,
    setCurrentPreset,
    clearCurrentPreset,
    resetToOriginal,
    reset,
    clearError
  }
})
