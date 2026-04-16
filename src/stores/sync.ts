/**
 * 同步状态管理
 * 管理 GitHub 认证和配置同步状态
 * 使用 Pinia defineStore 组合式 API
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as syncApi from '@/services/syncApi'
import { invalidatePresetsCache } from '@/services/presetStore'
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
      if (!state) return
      authState.value = state
      if (state.type === 'LoggedIn') {
        // 已登录，加载同步元数据
        const meta = await syncApi.getSyncStatus()
        if (meta) syncMetadata.value = meta
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
    if (!result) {
      throw new Error('启动 Device Flow 登录失败，请检查网络连接后重试')
    }
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
    if (!user) {
      throw new Error('完成 Device Flow 登录失败，请重试')
    }
    authState.value = { type: 'LoggedIn', user }
    // 登录成功后自动触发首次同步
    await sync()
    return user
  }

  /**
   * 使用 OAuth Web Flow 登录（推荐）
   * 自动打开浏览器，用户只需点击授权按钮
   * @returns 登录的 GitHub 用户信息
   */
  async function loginWithOAuth(): Promise<GitHubUser> {
    lastError.value = null
    authState.value = { type: 'OAuthLoggingIn' }
    let user: GitHubUser
    try {
      const result = await syncApi.startOAuthLogin()
      if (!result) {
        authState.value = { type: 'LoggedOut' }
        throw new Error('GitHub 登录失败，请重试')
      }
      user = result
      authState.value = { type: 'LoggedIn', user }
    } catch (e) {
      authState.value = { type: 'LoggedOut' }
      throw e
    }
    // 登录成功后自动触发首次同步（失败不影响登录状态）
    try {
      await sync()
    } catch (e) {
      // 同步失败只记录错误，不重置登录状态
      lastError.value = String(e)
    }
    return user
  }

  /**
   * 使用 Personal Access Token 登录
   * @param pat GitHub Personal Access Token
   * @returns 登录的 GitHub 用户信息
   */
  async function loginWithPat(pat: string): Promise<GitHubUser> {
    lastError.value = null
    let user: GitHubUser
    try {
      const result = await syncApi.loginWithPat(pat)
      if (!result) {
        throw new Error('PAT 登录失败，请检查 Token 是否正确')
      }
      user = result
      authState.value = { type: 'LoggedIn', user }
    } catch (e) {
      throw e
    }
    // 登录成功后自动触发首次同步（失败不影响登录状态）
    try {
      await sync()
    } catch (e) {
      lastError.value = String(e)
    }
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
      if (!result) {
        throw new Error('执行同步失败')
      }
      if (result.type === 'Conflict') {
        // 检测到冲突，设置待处理冲突供 UI 处理
        pendingConflict.value = result
      } else {
        // UpToDate / Uploaded / Downloaded
        const meta = await syncApi.getSyncStatus()
        if (meta) syncMetadata.value = meta
        // 远程预设已下载到本地，需使缓存失效以便 UI 刷新
        if (result.type === 'Downloaded') {
          invalidatePresetsCache()
        }
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
      if (!result) {
        throw new Error('上传同步失败')
      }
      const meta = await syncApi.getSyncStatus()
      if (meta) syncMetadata.value = meta
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
      if (!result) {
        throw new Error('下载同步失败')
      }
      const meta = await syncApi.getSyncStatus()
      if (meta) syncMetadata.value = meta
      // 远程预设已下载到本地，使缓存失效
      invalidatePresetsCache()
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
    // 冲突解决可能选择了远程版本，本地预设已变更
    invalidatePresetsCache()
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
   * 取消 OAuth Web Flow 登录
   */
  async function cancelOAuthLogin(): Promise<void> {
    lastError.value = null
    await syncApi.cancelOAuthLogin()
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
    loginWithOAuth,
    loginWithPat,
    logout,
    sync,
    upload,
    download,
    resolveConflict,
    cancelDeviceLogin,
    cancelOAuthLogin,
    clearError,
    clearPendingConflict,
    reset,
  }
})
