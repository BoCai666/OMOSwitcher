import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useQuotaStore } from '@/stores/quota'
import { getBalancePercentage, formatResetTime } from '@/composables/useQuotaFormatter'
import type { ProviderQuota } from '@/types/quota'
import { getProviderMetadata } from '@/data/providerMetadata'

const REFRESH_INTERVAL = 10 * 60 * 1000 // 10 分钟（悬浮球自己的轮询阈值）
const CACHE_TTL = REFRESH_INTERVAL

export interface BubbleQuotaItem {
  providerId: string
  providerName: string
  /** 剩余百分比 0-100 */
  remainingPercentage: number
  /** 品牌色 */
  color: string
  /** Provider 简称（3-4 字符） */
  label: string
  /** 重置倒计时文本 */
  resetTimeText: string
  /** 原始数据 */
  raw: ProviderQuota
}

function computeLabel(providerId: string, providerName: string): string {
  const shortNames: Record<string, string> = {
    'deepseek': 'DS',
    'zhipu': 'GLM',
    'openrouter': 'OR',
    'kimi-code': 'KCD',
    'minimax': 'MM',
    'moonshot': 'MS',
    'opencode-go': 'OCG',
    'infini': 'INF',
    'siliconflow': 'SF',
  }
  return shortNames[providerId] ?? providerName.slice(0, 3).toUpperCase()
}

function toBubbleItem(q: ProviderQuota): BubbleQuotaItem {
  let remainingPercentage: number
  if (q.quotaType === 'token_limit') {
    remainingPercentage = 100 - (q.quotaPercentage ?? 0)
  } else if (q.quotaType === 'balance') {
    remainingPercentage = getBalancePercentage(q)
  } else {
    remainingPercentage = 0
  }
  remainingPercentage = Math.max(0, Math.min(100, remainingPercentage))

  return {
    providerId: q.providerId,
    providerName: q.providerName,
    remainingPercentage,
    color: getProviderMetadata(q.providerId).color,
    label: computeLabel(q.providerId, q.providerName),
    resetTimeText: q.resetTime ? formatResetTime(q.resetTime) : '',
    raw: q,
  }
}

export function useBubbleQuota() {
  const quotaStore = useQuotaStore()
  const { data: rawData, isLoading, lastFetchTime } = storeToRefs(quotaStore)

  // 派生 BubbleQuotaItem[]：从共享 store 的原始数据计算
  const quotas = computed<BubbleQuotaItem[]>(() =>
    rawData.value
      .filter(q => q.status === 'success' && q.quotaType !== 'unsupported')
      .map(toBubbleItem)
  )

  const currentIndex = ref(0)
  const error = ref<string | null>(null)  // 保留兼容旧 API
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  /** 10 分钟定时拉取（10 分钟内有数据则跳过） */
  async function pollFetch() {
    try {
      await quotaStore.fetchIfStale(CACHE_TTL)
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : '未知错误'
    }
  }

  /** 展开面板时强制刷新 */
  async function fetchQuotas() {
    try {
      await quotaStore.refresh()
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : '未知错误'
    }
  }

  function startRefresh() {
    stopRefresh()
    pollFetch()
    refreshTimer = setInterval(pollFetch, REFRESH_INTERVAL)
  }

  function stopRefresh() {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }

  onMounted(() => {
    // 初始化 store（订阅跨 webview 同步事件），然后开始 10 分钟轮询
    quotaStore.init().then(() => startRefresh())
  })

  onUnmounted(() => {
    stopRefresh()
  })

  return {
    quotas,
    currentIndex,
    isLoading,
    error,
    lastFetchTime,
    startRefresh,
    stopRefresh,
    fetchQuotas,
  }
}
