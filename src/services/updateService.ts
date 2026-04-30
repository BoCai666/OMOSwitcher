/**
 * 更新服务 API 封装
 * 封装 @tauri-apps/plugin-updater 和 @tauri-apps/plugin-process 调用
 */

import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

// ============================================================================
// 类型定义
// ============================================================================

/** 更新信息 */
export interface UpdateInfo {
  version: string
  date: string | undefined
  body: string | undefined
}

/** 下载进度事件 */
export interface DownloadProgress {
  event: 'Started' | 'Progress' | 'Finished'
  contentLength?: number
  chunkLength?: number
}

// ============================================================================
// 模块状态
// ============================================================================

/** 缓存的更新对象，用于 checkForUpdate 和 downloadAndInstallUpdate 之间传递 */
let pendingUpdate: Update | null = null

// ============================================================================
// API 函数
// ============================================================================

/**
 * 检查是否有可用更新
 * @returns 更新信息，如果没有更新则返回 null
 */
export async function checkForUpdate(): Promise<UpdateInfo | null> {
  try {
    const update = await check()

    if (update === null) {
      pendingUpdate = null
      return null
    }

    pendingUpdate = update

    return {
      version: update.version,
      date: update.date,
      body: update.body
    }
  } catch (error) {
    console.warn('[Updater] 检查更新失败:', error)
    return null
  }
}

/**
 * 下载并安装更新
 * @param onProgress 进度回调函数
 * @throws Error 如果没有可用的更新或下载失败
 */
export async function downloadAndInstallUpdate(
  onProgress: (progress: DownloadProgress) => void
): Promise<void> {
  if (pendingUpdate === null) {
    throw new Error('没有可用的更新')
  }

  await pendingUpdate.downloadAndInstall((event) => {
    const progress: DownloadProgress = {
      event: event.event
    }
    // The 'Started' and 'Progress' events have data with contentLength/chunkLength
    // The 'Finished' event has no data
    if (event.event === 'Started' || event.event === 'Progress') {
      const data = event.data as { contentLength?: number; chunkLength?: number } | undefined
      progress.contentLength = data?.contentLength
      progress.chunkLength = data?.chunkLength
    }
    onProgress(progress)
  })

  pendingUpdate = null
}

/**
 * 重新启动应用
 * 用于更新完成后重启应用
 */
export async function restartApp(): Promise<void> {
  await relaunch()
}