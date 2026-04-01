/**
 * 监控服务状态管理
 * 使用 Pinia Composition API
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { monitorApi } from '@/services/monitorApi'
import type {
  RequestListItem,
  StatsSummary,
  MonitorStatus,
  LLMRequest,
  LLMResponse,
  MCPCall,
  LLMMetrics
} from '@/types/monitor'

export const useMonitorStore = defineStore('monitor', () => {
  // ========== 状态 ==========

  // 请求列表
  const requests = ref<RequestListItem[]>([])

  // 统计汇总
  const stats = ref<StatsSummary | null>(null)

  // 服务状态
  const status = ref<MonitorStatus>({
    is_running: false,
    port: 7100
  })

  // 加载状态
  const loading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 当前选中的请求 ID
  const selectedRequestId = ref<string | null>(null)

  // 请求详情缓存
  const requestDetails = ref<Map<string, LLMRequest>>(new Map())
  const responseDetails = ref<Map<string, LLMResponse>>(new Map())
  const mcpCallsCache = ref<Map<string, MCPCall[]>>(new Map())
  const metricsCache = ref<Map<string, LLMMetrics>>(new Map())

  // 自动刷新定时器（保留用于降级场景）
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  // SSE 连接状态
  const sseConnected = ref(false)
  let sseDisconnect: (() => void) | null = null

  // ========== 计算属性 ==========

  // 服务是否运行中
  const isRunning = computed(() => status.value.is_running)

  // 今日统计
  const todayStats = computed(() => {
    return stats.value?.today ?? {
      count: 0,
      totalTokens: 0,
      totalCost: 0,
      modelStats: {}
    }
  })

  // 本周统计
  const weekStats = computed(() => {
    return stats.value?.thisWeek ?? {
      count: 0,
      totalTokens: 0,
      totalCost: 0,
      modelStats: {}
    }
  })

  // 本月统计
  const monthStats = computed(() => {
    return stats.value?.thisMonth ?? {
      count: 0,
      totalTokens: 0,
      totalCost: 0,
      modelStats: {}
    }
  })

  // 全部统计
  const allTimeStats = computed(() => {
    return stats.value?.allTime ?? {
      count: 0,
      totalTokens: 0,
      totalCost: 0,
      modelStats: {}
    }
  })

  // 当前选中的请求
  const selectedRequest = computed(() => {
    if (!selectedRequestId.value) return null
    return requestDetails.value.get(selectedRequestId.value) ?? null
  })

  // 当前选中请求的响应
  const selectedResponse = computed(() => {
    if (!selectedRequestId.value) return null
    return responseDetails.value.get(selectedRequestId.value) ?? null
  })

  // 当前选中请求的 MCP 调用
  const selectedMcpCalls = computed(() => {
    if (!selectedRequestId.value) return []
    return mcpCallsCache.value.get(selectedRequestId.value) ?? []
  })

  // 当前选中请求的指标
  const selectedMetrics = computed(() => {
    if (!selectedRequestId.value) return null
    return metricsCache.value.get(selectedRequestId.value) ?? null
  })

  // ========== 内部方法 ==========

  // 清空缓存
  function clearCaches(): void {
    requestDetails.value.clear()
    responseDetails.value.clear()
    mcpCallsCache.value.clear()
    metricsCache.value.clear()
  }

  // ========== 公共方法 ==========

  /**
   * 启动监控服务
   */
  async function startMonitor(): Promise<string> {
    try {
      error.value = null
      const result = await invoke<string>('start_monitor_service')
      status.value.is_running = true
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 停止监控服务
   */
  async function stopMonitor(): Promise<void> {
    try {
      error.value = null
      await invoke('stop_monitor_service')
      status.value.is_running = false
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 检查服务状态
   */
  async function checkStatus(): Promise<void> {
    try {
      error.value = null
      const result = await invoke<MonitorStatus>('get_monitor_status')
      status.value = result
      
      // 如果 Tauri 认为服务没有运行，但 API 健康检查通过，也认为服务在运行
      if (!result.is_running) {
        const isHealthy = await monitorApi.healthCheck()
        if (isHealthy) {
          status.value.is_running = true
        }
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  /**
   * 检查服务健康状态
   */
  async function checkHealth(): Promise<boolean> {
    return monitorApi.healthCheck()
  }

  /**
   * 获取请求列表
   */
  async function fetchRequests(): Promise<void> {
    loading.value = true
    try {
      error.value = null
      requests.value = await monitorApi.getRequests()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取统计汇总
   */
  async function fetchStats(): Promise<void> {
    try {
      error.value = null
      stats.value = await monitorApi.getStatsSummary()
    } catch (e) {
      error.value = String(e)
    }
  }

  /**
   * 刷新所有数据
   */
  async function refresh(): Promise<void> {
    await Promise.all([fetchRequests(), fetchStats()])
  }

  /**
   * 选择请求
   */
  function selectRequest(id: string | null): void {
    selectedRequestId.value = id
  }

  /**
   * 加载请求详情
   */
  async function loadRequestDetail(id: string): Promise<LLMRequest> {
    // 检查缓存
    if (requestDetails.value.has(id)) {
      return requestDetails.value.get(id)!
    }

    try {
      const request = await monitorApi.getRequest(id)
      requestDetails.value.set(id, request)
      return request
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 加载响应详情
   */
  async function loadResponseDetail(requestId: string): Promise<LLMResponse | null> {
    // 检查缓存
    if (responseDetails.value.has(requestId)) {
      return responseDetails.value.get(requestId)!
    }

    try {
      const response = await monitorApi.getResponse(requestId)
      if (response) {
        responseDetails.value.set(requestId, response)
      }
      return response
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 加载 MCP 调用
   */
  async function loadMcpCalls(requestId: string): Promise<MCPCall[]> {
    // 检查缓存
    if (mcpCallsCache.value.has(requestId)) {
      return mcpCallsCache.value.get(requestId)!
    }

    try {
      const calls = await monitorApi.getMcpCalls(requestId)
      mcpCallsCache.value.set(requestId, calls)
      return calls
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 加载指标
   */
  async function loadMetrics(requestId: string): Promise<LLMMetrics | null> {
    // 检查缓存
    if (metricsCache.value.has(requestId)) {
      return metricsCache.value.get(requestId)!
    }

    try {
      const metrics = await monitorApi.getMetrics(requestId)
      if (metrics) {
        metricsCache.value.set(requestId, metrics)
      }
      return metrics
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 加载选中请求的所有详情
   */
  async function loadSelectedRequestDetails(): Promise<void> {
    if (!selectedRequestId.value) return

    const id = selectedRequestId.value

    try {
      // 并行加载所有详情
      await Promise.all([
        loadRequestDetail(id),
        loadResponseDetail(id),
        loadMcpCalls(id),
        loadMetrics(id)
      ])
    } catch (e) {
      error.value = String(e)
    }
  }

  /**
   * 清空监控数据
   */
  async function clearData(): Promise<void> {
    try {
      error.value = null
      await monitorApi.clearData()
      // 清空本地状态
      requests.value = []
      stats.value = null
      selectedRequestId.value = null
      clearCaches()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /**
   * 启动自动刷新
   * @param interval 刷新间隔（毫秒）
   */
  function startAutoRefresh(interval = 5000): void {
    stopAutoRefresh()
    refreshTimer = setInterval(() => {
      if (status.value.is_running) {
        refresh()
      }
    }, interval)
  }

  /**
   * 停止自动刷新
   */
  function stopAutoRefresh(): void {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }

  // ========== SSE 实时推送 ==========

  /**
   * 启动 SSE 实时推送
   * 替代轮询机制，实现服务端主动推送数据更新
   */
  function startSSE(): void {
    // 如果已连接，先断开
    stopSSE()

    console.log('[Monitor] Starting SSE connection')
    sseConnected.value = false

    // 先获取一次全量数据
    refresh()

    // 连接 SSE
    sseDisconnect = monitorApi.connectSSE({
      onConnected: (timestamp) => {
        console.log('[Monitor] SSE connected at', new Date(timestamp).toISOString())
        sseConnected.value = true
        error.value = null
      },

      onNewRequest: (request) => {
        console.log('[Monitor] SSE: new request', request.id)
        // 将新请求添加到列表头部
        const newItem: RequestListItem = {
          id: request.id,
          timestamp: request.timestamp,
          provider: request.provider,
          model: request.model,
          method: request.method,
          url: request.url,
          domain: request.domain
        }
        // 避免重复
        const exists = requests.value.some(r => r.id === request.id)
        if (!exists) {
          requests.value = [newItem, ...requests.value]
        }
      },

      onResponse: (response) => {
        console.log('[Monitor] SSE: response', response.requestId)
        // 更新列表中对应请求的状态码和时长
        const index = requests.value.findIndex(r => r.id === response.requestId)
        if (index !== -1) {
          requests.value[index] = {
            ...requests.value[index],
            statusCode: response.statusCode,
            duration: response.duration
          }
        }
        // 缓存响应详情
        responseDetails.value.set(response.requestId, response)
      },

      onMetrics: (metrics) => {
        console.log('[Monitor] SSE: metrics', metrics.requestId, metrics.totalTokens, 'tokens')
        // 更新列表中对应请求的 tokens 和 cost
        const index = requests.value.findIndex(r => r.id === metrics.requestId)
        if (index !== -1) {
          requests.value[index] = {
            ...requests.value[index],
            tokens: metrics.totalTokens,
            cost: metrics.estimatedCost,
            duration: metrics.duration
          }
        }
        // 缓存指标
        metricsCache.value.set(metrics.requestId, metrics)
        // 刷新统计数据
        fetchStats()
      },

      onError: (err) => {
        console.error('[Monitor] SSE error:', err)
        sseConnected.value = false
        error.value = 'SSE 连接错误，尝试重连中...'
      }
    })
  }

  /**
   * 停止 SSE 实时推送
   */
  function stopSSE(): void {
    if (sseDisconnect) {
      sseDisconnect()
      sseDisconnect = null
    }
    sseConnected.value = false
    monitorApi.disconnectSSE()
    console.log('[Monitor] SSE stopped')
  }

  /**
   * 重置状态
   */
  function reset(): void {
    stopAutoRefresh()
    stopSSE()
    requests.value = []
    stats.value = null
    status.value = { is_running: false, port: 7100 }
    loading.value = false
    error.value = null
    selectedRequestId.value = null
    clearCaches()
  }

  return {
    // 状态
    requests,
    stats,
    status,
    loading,
    error,
    selectedRequestId,
    requestDetails,
    responseDetails,
    mcpCallsCache,
    metricsCache,
    sseConnected,

    // 计算属性
    isRunning,
    todayStats,
    weekStats,
    monthStats,
    allTimeStats,
    selectedRequest,
    selectedResponse,
    selectedMcpCalls,
    selectedMetrics,

    // 方法
    startMonitor,
    stopMonitor,
    checkStatus,
    checkHealth,
    fetchRequests,
    fetchStats,
    refresh,
    selectRequest,
    loadRequestDetail,
    loadResponseDetail,
    loadMcpCalls,
    loadMetrics,
    loadSelectedRequestDetails,
    clearData,
    startAutoRefresh,
    stopAutoRefresh,
    startSSE,
    stopSSE,
    reset
  }
})
