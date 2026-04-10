/**
 * 监控服务 API 封装
 * 通过 Tauri invoke 与 Rust 后端监控模块通信
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  RequestListItem,
  StatsSummary,
  LLMRequest,
  LLMResponse,
  MCPCall,
  LLMMetrics,
  SSEEventCallbacks
} from '@/types/monitor'

/**
 * 监控服务 API
 * 所有方法通过 Tauri invoke 调用 Rust 后端
 */
export const monitorApi = {
  // ========== 数据查询 API ==========

  /**
   * 获取请求列表
   * @param limit 返回数量限制
   */
  async getRequests(limit = 100): Promise<RequestListItem[]> {
    return invoke<RequestListItem[]>('monitor_get_requests', { limit })
  },

  /**
   * 获取请求详情
   * @param id 请求 ID
   */
  async getRequest(id: string): Promise<LLMRequest> {
    return invoke<LLMRequest>('monitor_get_request', { id })
  },

  /**
   * 获取响应详情
   * @param requestId 请求 ID
   */
  async getResponse(requestId: string): Promise<LLMResponse | null> {
    return invoke<LLMResponse | null>('monitor_get_response', { requestId })
  },

  /**
   * 获取 MCP 调用记录
   * @param requestId 请求 ID
   */
  async getMcpCalls(requestId: string): Promise<MCPCall[]> {
    return invoke<MCPCall[]>('monitor_get_mcp_calls', { requestId })
  },

  /**
   * 获取指标详情
   * @param requestId 请求 ID
   */
  async getMetrics(requestId: string): Promise<LLMMetrics | null> {
    return invoke<LLMMetrics | null>('monitor_get_metrics', { requestId })
  },

  /**
   * 获取统计汇总
   */
  async getStatsSummary(): Promise<StatsSummary> {
    return invoke<StatsSummary>('monitor_get_stats_summary')
  },

  /**
   * 健康检查
   * @returns 服务是否可用
   */
  async healthCheck(): Promise<boolean> {
    try {
      return await invoke<boolean>('monitor_health')
    } catch {
      return false
    }
  },

  /**
   * 清空监控数据
   */
  async clearData(): Promise<void> {
    await invoke('monitor_clear_data')
  },

  // ========== SSE 实时推送 ==========

  // 存储已注册的事件取消监听函数
  _unlistenFns: [] as UnlistenFn[],

  /**
   * 连接 SSE 实时推送
   * 通过 Tauri event 系统监听后端事件
   * @param callbacks 事件回调函数
   * @returns 断开连接函数
   */
  async connectSSE(callbacks: SSEEventCallbacks): Promise<() => void> {
    // 如果已有连接，先断开
    await this.disconnectSSE()

    console.log('[Monitor] Registering Tauri event listeners')

    // 注册新请求事件
    const unlistenNewRequest = await listen<LLMRequest>('monitor:new-request', (event) => {
      callbacks.onNewRequest?.(event.payload)
    })
    this._unlistenFns.push(unlistenNewRequest)

    // 注册响应事件
    const unlistenResponse = await listen<LLMResponse>('monitor:response', (event) => {
      callbacks.onResponse?.(event.payload)
    })
    this._unlistenFns.push(unlistenResponse)

    // 注册指标事件
    const unlistenMetrics = await listen<LLMMetrics>('monitor:metrics', (event) => {
      callbacks.onMetrics?.(event.payload)
    })
    this._unlistenFns.push(unlistenMetrics)

    console.log('[Monitor] Event listeners registered')

    // 连接成功回调
    callbacks.onConnected?.(Date.now())

    // 返回断开连接函数
    return () => {
      this.disconnectSSE()
    }
  },

  /**
   * 断开 SSE 连接
   * 取消所有已注册的事件监听
   */
  async disconnectSSE(): Promise<void> {
    if (this._unlistenFns.length > 0) {
      console.log('[Monitor] Unregistering event listeners')
      for (const unlisten of this._unlistenFns) {
        unlisten()
      }
      this._unlistenFns = []
      console.log('[Monitor] Event listeners unregistered')
    }
  },

  /**
   * 检查 SSE 是否已连接
   */
  isSSEConnected(): boolean {
    return this._unlistenFns.length > 0
  }
}
