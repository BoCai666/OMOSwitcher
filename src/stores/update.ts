/**
 * 更新状态管理
 * 管理应用自动更新的检查、下载、安装状态
 * 使用 Pinia Composition API
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { checkForUpdate, downloadAndInstallUpdate, restartApp, type UpdateInfo, type DownloadProgress } from '@/services/updateService'
import { savePendingChangelog } from '@/services/settingsStore'
import { useMonitorStore } from '@/stores/monitor'

export const useUpdateStore = defineStore('update', () => {
  // ========== 状态 ==========

  /** 更新信息 */
  const updateInfo = ref<UpdateInfo | null>(null)

  /** 是否正在检查更新 */
  const isChecking = ref(false)

  /** 是否正在下载 */
  const isDownloading = ref(false)

  /** 下载进度 (0-100) */
  const downloadProgress = ref<number>(0)

  /** 错误信息 */
  const error = ref<string | null>(null)

  // ========== 计算属性 ==========

  /** 是否有可用更新 */
  const hasUpdate = computed(() => updateInfo.value !== null)

  // ========== 操作 ==========

  /**
   * 静默检查更新
   * 不显示 UI 提示，仅更新状态
   */
  async function check(): Promise<void> {
    isChecking.value = true
    error.value = null

    try {
      const result = await checkForUpdate()
      if (result !== null) {
        updateInfo.value = result
      } else {
        updateInfo.value = null
      }
    } catch (e) {
      console.warn('[Updater] 检查更新失败:', e)
      updateInfo.value = null
    } finally {
      isChecking.value = false
    }
  }

  /**
   * 下载并安装更新
   * 会先停止 Monitor 服务，然后下载更新并重启应用
   */
  async function install(): Promise<void> {
    isDownloading.value = true
    downloadProgress.value = 0
    error.value = null

    try {
      // 停止 Monitor 服务（如果正在运行）
      const monitorStore = useMonitorStore()
      if (monitorStore.status.is_running) {
        await monitorStore.stopMonitor()
      }

      // 下载并安装更新
      let downloadedBytes = 0

      await downloadAndInstallUpdate((progress: DownloadProgress) => {
        if (progress.event === 'Started') {
          downloadedBytes = 0
          downloadProgress.value = 0
        } else if (progress.event === 'Progress') {
          const chunkLength = progress.chunkLength ?? 0
          const contentLength = progress.contentLength

          downloadedBytes += chunkLength

          if (contentLength !== undefined && contentLength > 0) {
            downloadProgress.value = Math.min(99, Math.round((downloadedBytes / contentLength) * 100))
          } else {
            // 如果没有 contentLength，每次增加小量进度
            downloadProgress.value = Math.min(99, downloadProgress.value + 5)
          }
        } else if (progress.event === 'Finished') {
          downloadProgress.value = 100
        }
      })

      // 下载完成，保存更新日志后重启
      if (updateInfo.value) {
        await savePendingChangelog({
          version: updateInfo.value.version,
          date: updateInfo.value.date,
          body: updateInfo.value.body
        })
      }
      await restartApp()
    } catch (e) {
      error.value = String(e)
      isDownloading.value = false
    }
  }

  /**
   * 关闭更新对话框
   * 仅清除更新信息，不影响正在进行的下载
   */
  function dismiss(): void {
    updateInfo.value = null
    error.value = null
  }

  return {
    // 状态
    updateInfo,
    isChecking,
    isDownloading,
    downloadProgress,
    error,

    // 计算属性
    hasUpdate,

    // 操作
    check,
    install,
    dismiss
  }
})