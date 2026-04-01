/**
 * 监控服务 API 封装
 * 与 Sidecar 监控服务通信
 */

import type {
  RequestListItem,
  StatsSummary,
  LLMRequest,
  LLMResponse,
  MCPCall,
  LLMMetrics,
  SSEEventCallbacks
} from '@/types/monitor'

// 端口缓存
let cachedPort: number | null = null

/**
 * 获取 Monitor Web API 端口
 */
async function getMonitorPort(): Promise<number> {
  if (cachedPort !== null) {
    return cachedPort
  }
  
  try {
    // 动态导入避免循环依赖
    const { getMonitorWebPort } = await import('./settingsStore')
    cachedPort = await getMonitorWebPort()
    return cachedPort
  } catch {
    // 默认端口
    return 7100
  }
}

/**
 * 获取 API 基础 URL
 */
async function getBaseUrl(): Promise<string> {
  const port = await getMonitorPort()
  return `http://localhost:${port}/api`
}

/**
 * 监控服务 API
 */
export const monitorApi = {
  /**
   * 获取请求列表
   * @param limit 返回数量限制
   */
  async getRequests(limit = 100): Promise<RequestListItem[]> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/requests?limit=${limit}`)
    if (!response.ok) {
      throw new Error(`获取请求列表失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取请求详情
   * @param id 请求 ID
   */
  async getRequest(id: string): Promise<LLMRequest> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/requests/${id}`)
    if (!response.ok) {
      throw new Error(`获取请求详情失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取响应详情
   * @param requestId 请求 ID
   */
  async getResponse(requestId: string): Promise<LLMResponse | null> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/requests/${requestId}/response`)
    if (!response.ok) {
      if (response.status === 404) {
        return null
      }
      throw new Error(`获取响应详情失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取 MCP 调用记录
   * @param requestId 请求 ID
   */
  async getMcpCalls(requestId: string): Promise<MCPCall[]> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/requests/${requestId}/mcp-calls`)
    if (!response.ok) {
      if (response.status === 404) {
        return []
      }
      throw new Error(`获取 MCP 调用失败: ${response.statusText}`)
    }
    const data = await response.json()
    return data.calls || []
  },

  /**
   * 获取指标详情
   * @param requestId 请求 ID
   */
  async getMetrics(requestId: string): Promise<LLMMetrics | null> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/requests/${requestId}/metrics`)
    if (!response.ok) {
      if (response.status === 404) {
        return null
      }
      throw new Error(`获取指标失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取统计汇总
   */
  async getStatsSummary(): Promise<StatsSummary> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/stats/summary`)
    if (!response.ok) {
      throw new Error(`获取统计汇总失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 健康检查
   * @returns 服务是否可用
   */
  async healthCheck(): Promise<boolean> {
    try {
      const baseUrl = await getBaseUrl()
      const response = await fetch(`${baseUrl}/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(3000) // 3秒超时
      })
      return response.ok
    } catch {
      return false
    }
  },

  /**
   * 清空监控数据
   */
  async clearData(): Promise<void> {
    const baseUrl = await getBaseUrl()
    const response = await fetch(`${baseUrl}/clear`, {
      method: 'POST'
    })
    if (!response.ok) {
      throw new Error(`清空数据失败: ${response.statusText}`)
    }
  },
  
  /**
   * 清除端口缓存（当端口配置变更时调用）
   */
  clearPortCache(): void {
    cachedPort = null
  },

  // ========== SSE 实时推送 ==========

  // SSE 连接实例
  _eventSource: null as EventSource | null,

  /**
   * 连接 SSE 实时推送
   * @param callbacks 事件回调函数
   * @returns 断开连接函数
   */
  connectSSE(callbacks: SSEEventCallbacks): () => void {
    // 如果已有连接，先断开
    if (this._eventSource) {
      this._eventSource.close()
      this._eventSource = null
    }

    // 创建 EventSource 连接
    getBaseUrl().then(baseUrl => {
      const sseUrl = baseUrl.replace('/api', '') + '/api/events'
      console.log('[Monitor] Connecting to SSE:', sseUrl)

      const eventSource = new EventSource(sseUrl)
      this._eventSource = eventSource

      // 连接成功
      eventSource.onopen = () => {
        console.log('[Monitor] SSE connected')
      }

      // 接收消息
      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)

          switch (data.type) {
            case 'connected':
              callbacks.onConnected?.(data.timestamp as number)
              break

            case 'new-request':
              callbacks.onNewRequest?.(data.request as LLMRequest)
              break

            case 'response':
              callbacks.onResponse?.(data.response as LLMResponse)
              break

            case 'metrics':
              callbacks.onMetrics?.(data.metrics as LLMMetrics)
              break

            default:
              console.warn('[Monitor] Unknown SSE event type:', data.type)
          }
        } catch (err) {
          console.error('[Monitor] Failed to parse SSE message:', err)
        }
      }

      // 连接错误
      eventSource.onerror = (err) => {
        console.error('[Monitor] SSE connection error:', err)
        callbacks.onError?.(new Error('SSE connection error'))

        // 自动重连逻辑由 EventSource 内置处理
        // 如果连接彻底失败，EventSource 会自动尝试重连
      }
    }).catch(err => {
      console.error('[Monitor] Failed to get monitor port:', err)
      callbacks.onError?.(err)
    })

    // 返回断开连接函数
    return () => {
      if (this._eventSource) {
        console.log('[Monitor] Disconnecting SSE')
        this._eventSource.close()
        this._eventSource = null
      }
    }
  },

  /**
   * 断开 SSE 连接
   */
  disconnectSSE(): void {
    if (this._eventSource) {
      this._eventSource.close()
      this._eventSource = null
      console.log('[Monitor] SSE disconnected')
    }
  },

  /**
   * 检查 SSE 是否已连接
   */
  isSSEConnected(): boolean {
    return this._eventSource !== null && this._eventSource.readyState === EventSource.OPEN
  }
}
