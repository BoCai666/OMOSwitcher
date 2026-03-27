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
    port: 3030
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

  // 自动刷新定时器
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  // ========== 计算属性 ==========

  // 服务是否运行中
  const isRunning = computed(() => status.value.is_running)

  // 今日统计
  const todayStats = computed(() => {
    return stats.value?.today ?? {
      requestCount: 0,
      totalTokens: 0,
      totalCost: 0
    }
  })

  // 本周统计
  const weekStats = computed(() => {
    return stats.value?.week ?? {
      requestCount: 0,
      totalTokens: 0,
      totalCost: 0
    }
  })

  // 本月统计
  const monthStats = computed(() => {
    return stats.value?.month ?? {
      requestCount: 0,
      totalTokens: 0,
      totalCost: 0
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
   * @param enterpriseCaCertPath 企业代理 CA 证书路径（可选）
   */
  async function startMonitor(enterpriseCaCertPath?: string): Promise<string> {
    try {
      error.value = null
      const result = await invoke<string>('start_monitor_service', {
        enterpriseCaCertPath: enterpriseCaCertPath || ''
      })
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
  async function loadResponseDetail(requestId: string): Promise<LLMResponse> {
    // 检查缓存
    if (responseDetails.value.has(requestId)) {
      return responseDetails.value.get(requestId)!
    }

    try {
      const response = await monitorApi.getResponse(requestId)
      responseDetails.value.set(requestId, response)
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

  /**
   * 重置状态
   */
  function reset(): void {
    stopAutoRefresh()
    requests.value = []
    stats.value = null
    status.value = { is_running: false, port: 3030 }
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

    // 计算属性
    isRunning,
    todayStats,
    weekStats,
    monthStats,
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
    reset
  }
})
