import { ref, onMounted, onUnmounted } from 'vue'
import { quotaApi } from '@/services/quotaApi'
import { getBalancePercentage, formatResetTime } from '@/composables/useQuotaFormatter'
import type { ProviderQuota } from '@/types/quota'
import { getProviderMetadata } from '@/data/providerMetadata'

const REFRESH_INTERVAL = 10 * 60 * 1000 // 10 分钟

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
  // 优先使用简称映射
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

export function useBubbleQuota() {
  const quotas = ref<BubbleQuotaItem[]>([])
  const currentIndex = ref(0)
  const isLoading = ref(true)
  const error = ref<string | null>(null)
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  async function fetchQuotas() {
    isLoading.value = true
    error.value = null

    try {
      const rawQuotas = await quotaApi.fetchAllProviderQuotas()

      quotas.value = rawQuotas
        .filter(q => q.status === 'success' && q.quotaType !== 'unsupported')
        .map(q => {
          let remainingPercentage: number

          if (q.quotaType === 'token_limit') {
            remainingPercentage = 100 - (q.quotaPercentage ?? 0)
          } else if (q.quotaType === 'balance') {
            remainingPercentage = getBalancePercentage(q)
          } else {
            remainingPercentage = 0
          }

          // Clamp to 0-100
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
        })
    } catch (e) {
      error.value = e instanceof Error ? e.message : '未知错误'
      // 保留上次成功数据
    } finally {
      isLoading.value = false
    }
  }

  function startRefresh() {
    stopRefresh()
    fetchQuotas()
    refreshTimer = setInterval(fetchQuotas, REFRESH_INTERVAL)
  }

  function stopRefresh() {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }

  onMounted(() => {
    startRefresh()
  })

  onUnmounted(() => {
    stopRefresh()
  })

  return {
    quotas,
    currentIndex,
    isLoading,
    error,
    startRefresh,
    stopRefresh,
    fetchQuotas,
  }
}
