/**
 * 同步状态管理
 * 管理 GitHub 认证和配置同步状态
 * 使用 Pinia defineStore 组合式 API
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as syncApi from '@/services/syncApi'
import type {
  AuthState,
  GitHubUser,
  SyncMetadata,
  SyncResult,
  ConflictResolution
} from '@/services/syncApi'

export const useSyncStore = defineStore('sync', () => {
  // ========== 状态 ==========

  /** 认证状态 */
  const authState = ref<AuthState>({ type: 'LoggedOut' })

  /** 同步元数据 */
  const syncMetadata = ref<SyncMetadata | null>(null)

  /** 是否正在同步 */
  const isSyncing = ref(false)

  /** 最后一次错误信息 */
  const lastError = ref<string | null>(null)

  /** 待处理的冲突 */
  const pendingConflict = ref<SyncResult | null>(null)

  // ========== 计算属性 ==========

  /** 是否已登录 */
  const isLoggedIn = computed(() => authState.value.type === 'LoggedIn')

  /** 当前用户信息 */
  const currentUser = computed<GitHubUser | null>(() =>
    authState.value.type === 'LoggedIn' ? authState.value.user : null
  )

  /** 最后同步时间 */
  const lastSyncTime = computed(() => syncMetadata.value?.last_sync_at)

  // ========== 操作 ==========

  /**
   * 检查认证状态
   * 从后端获取当前认证状态并更新本地状态
   */
  async function checkAuth(): Promise<void> {
    try {
      lastError.value = null
      const state = await syncApi.getAuthState()
      authState.value = state
      if (state.type === 'LoggedIn') {
        // 已登录，加载同步元数据
        const meta = await syncApi.getSyncStatus()
        syncMetadata.value = meta
      }
    } catch (e) {
      lastError.value = String(e)
    }
  }

  /**
   * 启动 Device Flow 登录
   * @returns 用户码和验证链接
   */
  async function startDeviceLogin(): Promise<{ user_code: string; verification_uri: string }> {
    lastError.value = null
    const result = await syncApi.startDeviceLogin()
    authState.value = {
      type: 'LoggingIn',
      user_code: result.user_code,
      verification_uri: result.verification_uri,
    }
    return result
  }

  /**
   * 完成 Device Flow 登录
   * @returns 登录的 GitHub 用户信息
   */
  async function completeDeviceLogin(): Promise<GitHubUser> {
    lastError.value = null
    const user = await syncApi.completeDeviceLogin()
    authState.value = { type: 'LoggedIn', user }
    // 登录成功后自动触发首次同步
    await sync()
    return user
  }

  /**
   * 使用 Personal Access Token 登录
   * @param pat GitHub Personal Access Token
   * @returns 登录的 GitHub 用户信息
   */
  async function loginWithPat(pat: string): Promise<GitHubUser> {
    lastError.value = null
    const user = await syncApi.loginWithPat(pat)
    authState.value = { type: 'LoggedIn', user }
    // 登录成功后自动触发首次同步
    await sync()
    return user
  }

  /**
   * 登出
   * 清除 GitHub 认证状态和本地同步数据
   */
  async function logout(): Promise<void> {
    lastError.value = null
    await syncApi.logout()
    authState.value = { type: 'LoggedOut' }
    syncMetadata.value = null
    pendingConflict.value = null
  }

  /**
   * 执行同步
   * 根据本地和远程状态自动判断上传或下载
   * @returns 同步结果
   */
  async function sync(): Promise<SyncResult> {
    isSyncing.value = true
    lastError.value = null
    try {
      const result = await syncApi.performSync()
      if (result.type === 'Conflict') {
        // 检测到冲突，设置待处理冲突供 UI 处理
        pendingConflict.value = result
      } else if (result.type === 'Downloaded') {
        // 下载后需要刷新配置 — 但这里不直接引用 configStore
        // 留给调用方处理刷新逻辑（或通过事件）
        syncMetadata.value = await syncApi.getSyncStatus()
      } else {
        // UpToDate 或 Uploaded
        syncMetadata.value = await syncApi.getSyncStatus()
      }
      return result
    } catch (e) {
      lastError.value = String(e)
      throw e
    } finally {
      isSyncing.value = false
    }
  }

  /**
   * 手动上传
   * 强制上传本地配置到 GitHub Gist
   * @returns 同步结果
   */
  async function upload(): Promise<SyncResult> {
    isSyncing.value = true
    lastError.value = null
    try {
      const result = await syncApi.uploadSync()
      syncMetadata.value = await syncApi.getSyncStatus()
      return result
    } catch (e) {
      lastError.value = String(e)
      throw e
    } finally {
      isSyncing.value = false
    }
  }

  /**
   * 手动下载
   * 从 GitHub Gist 下载配置到本地
   * @returns 同步结果
   */
  async function download(): Promise<SyncResult> {
    isSyncing.value = true
    lastError.value = null
    try {
      const result = await syncApi.downloadSync()
      syncMetadata.value = await syncApi.getSyncStatus()
      return result
    } catch (e) {
      lastError.value = String(e)
      throw e
    } finally {
      isSyncing.value = false
    }
  }

  /**
   * 解决冲突
   * @param resolution 冲突解决策略
   */
  async function resolveConflict(resolution: ConflictResolution): Promise<void> {
    lastError.value = null
    await syncApi.resolveConflict(resolution)
    pendingConflict.value = null
    syncMetadata.value = await syncApi.getSyncStatus()
  }

  /**
   * 取消 Device Flow 登录
   */
  async function cancelDeviceLogin(): Promise<void> {
    lastError.value = null
    await syncApi.cancelDeviceLogin()
    authState.value = { type: 'LoggedOut' }
  }

  /**
   * 清除错误状态
   */
  function clearError(): void {
    lastError.value = null
  }

  /**
   * 清除待处理冲突
   */
  function clearPendingConflict(): void {
    pendingConflict.value = null
  }

  /**
   * 重置状态
   */
  function reset(): void {
    authState.value = { type: 'LoggedOut' }
    syncMetadata.value = null
    isSyncing.value = false
    lastError.value = null
    pendingConflict.value = null
  }

  return {
    // 状态
    authState,
    syncMetadata,
    isSyncing,
    lastError,
    pendingConflict,

    // 计算属性
    isLoggedIn,
    currentUser,
    lastSyncTime,

    // 操作
    checkAuth,
    startDeviceLogin,
    completeDeviceLogin,
    loginWithPat,
    logout,
    sync,
    upload,
    download,
    resolveConflict,
    cancelDeviceLogin,
    clearError,
    clearPendingConflict,
    reset,
  }
})
