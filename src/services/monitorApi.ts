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
  LLMMetrics
} from '@/types/monitor'

// 监控服务端口
const MONITOR_PORT = 3030
const BASE_URL = `http://localhost:${MONITOR_PORT}/api`

/**
 * 监控服务 API
 */
export const monitorApi = {
  /**
   * 获取请求列表
   * @param limit 返回数量限制
   */
  async getRequests(limit = 100): Promise<RequestListItem[]> {
    const response = await fetch(`${BASE_URL}/requests?limit=${limit}`)
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
    const response = await fetch(`${BASE_URL}/requests/${id}`)
    if (!response.ok) {
      throw new Error(`获取请求详情失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取响应详情
   * @param requestId 请求 ID
   */
  async getResponse(requestId: string): Promise<LLMResponse> {
    const response = await fetch(`${BASE_URL}/requests/${requestId}/response`)
    if (!response.ok) {
      throw new Error(`获取响应详情失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取 MCP 调用记录
   * @param requestId 请求 ID
   */
  async getMcpCalls(requestId: string): Promise<MCPCall[]> {
    const response = await fetch(`${BASE_URL}/requests/${requestId}/mcp-calls`)
    if (!response.ok) {
      throw new Error(`获取 MCP 调用失败: ${response.statusText}`)
    }
    return response.json()
  },

  /**
   * 获取指标详情
   * @param requestId 请求 ID
   */
  async getMetrics(requestId: string): Promise<LLMMetrics | null> {
    const response = await fetch(`${BASE_URL}/requests/${requestId}/metrics`)
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
    const response = await fetch(`${BASE_URL}/stats/summary`)
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
      const response = await fetch(`${BASE_URL}/health`, {
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
    const response = await fetch(`${BASE_URL}/clear`, {
      method: 'POST'
    })
    if (!response.ok) {
      throw new Error(`清空数据失败: ${response.statusText}`)
    }
  }
}
