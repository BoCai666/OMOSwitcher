/**
 * 额度数据全局共享 Store（跨 webview 同步）
 *
 * 架构说明：
 * - Tauri 2 的两个 webview 各自有独立 JS runtime，Pinia store 实例不共享
 * - 通过 Tauri 事件机制（emit/listen）实现"数据共用一份"：
 *   - Rust 端的 "quota-progress" 事件已是 broadcast，所有 webview 自动收到
 *   - 任何 webview 完成 fetch 后 emit "quota:store-updated" 携带完整数据
 *   - 其他 webview 监听该事件并更新本地 store
 * - 各调用方（QuotaView、useBubbleQuota）决定何时调 fetch
 *
 * 注意：首次启动时两端可能并发 fetch（短期开销），后续通过 lastFetchTime 复用
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import { quotaApi } from '@/services/quotaApi'
import type { ProviderQuota } from '@/types/quota'

/** 跨 webview 同步事件名：fetch 完成后 emit 完整数据 */
const SYNC_EVENT = 'quota:store-updated'
/** Rust 端流式进度事件（已是 broadcast） */
const PROGRESS_EVENT = 'quota:progress'

/** OpenCodeGo Provider id 用于"保留卡片显示设置按钮" */
function isOpenCodeGoProvider(providerId: string): boolean {
  const id = providerId.toLowerCase()
  return id.includes('opencode') && id.includes('go')
}

/** 决定是否保留某个 provider 的过滤函数 */
function shouldKeepProvider(q: ProviderQuota): boolean {
  if (q.quotaType === 'balance' || q.quotaType === 'token_limit') return true
  if (q.status === 'error') return true
  if (isOpenCodeGoProvider(q.providerId)) return true
  return false
}

export const useQuotaStore = defineStore('quota', () => {
  // ========== 状态 ==========
  const data = ref<ProviderQuota[]>([])
  const lastFetchTime = ref(0)
  const isFetching = ref(false)
  const isLoading = ref(false)

  let progressUnlisten: UnlistenFn | null = null
  let syncUnlisten: UnlistenFn | null = null
  let initialized = false

  /**
   * 初始化：订阅跨 webview 同步事件 + Rust 端流式进度
   * 必须在 setup 中调用一次
   */
  async function init(): Promise<void> {
    if (initialized) return
    initialized = true

    // 清理可能残留的旧监听
    syncUnlisten?.()
    progressUnlisten?.()
    syncUnlisten = null
    progressUnlisten = null

    // 监听其他 webview fetch 完成后 emit 的完整数据
    syncUnlisten = await listen<{ data: ProviderQuota[]; lastFetchTime: number }>(
      SYNC_EVENT,
      (event) => {
        data.value = event.payload.data
        lastFetchTime.value = event.payload.lastFetchTime
      }
    )

    // 监听 Rust 端流式进度（broadcast 到所有 webview）
    progressUnlisten = await listen<ProviderQuota>(PROGRESS_EVENT, (event) => {
      const q = event.payload
      if (!shouldKeepProvider(q)) return
      upsertOne(q)
    })
  }

  /**
   * 拉取所有供应商额度。多个调用方同时触发只会有一个真正的网络请求。
   * fetch 完成后会通过 emit 同步给其他 webview。
   */
  async function fetch(): Promise<void> {
    if (isFetching.value) return
    isFetching.value = true
    isLoading.value = true

    try {
      const all = await quotaApi.fetchAllProviderQuotas()
      // 过滤 + 排序
      data.value = all
        .filter(shouldKeepProvider)
        .sort((a, b) => a.providerId.localeCompare(b.providerId))
      lastFetchTime.value = Date.now()

      // 同步给其他 webview（emit 是 broadcast）
      await emit(SYNC_EVENT, {
        data: data.value,
        lastFetchTime: lastFetchTime.value,
      })
    } finally {
      isLoading.value = false
      isFetching.value = false
    }
  }

  /** 增量更新单个 provider（来自 progress 事件） */
  function upsertOne(q: ProviderQuota) {
    const idx = data.value.findIndex(item => item.providerId === q.providerId)
    if (idx !== -1) {
      data.value[idx] = q
    } else {
      data.value.push(q)
    }
  }

  /** 判断当前数据是否新鲜 */
  function isFresh(maxAgeMs: number): boolean {
    return data.value.length > 0 && Date.now() - lastFetchTime.value < maxAgeMs
  }

  /** 数据不够新鲜就 fetch，否则跳过 */
  async function fetchIfStale(maxAgeMs: number): Promise<void> {
    if (isFresh(maxAgeMs)) return
    await fetch()
  }

  /** 强制刷新 */
  async function refresh(): Promise<void> {
    await fetch()
  }

  return {
    // 状态
    data,
    lastFetchTime,
    isFetching,
    isLoading,
    // 方法
    init,
    fetch,
    refresh,
    isFresh,
    fetchIfStale,
    upsertOne,
  }
})
