/**
 * 监控服务类型定义
 * 基于后端 packages/monitor/src/types.ts 适配前端环境
 */

// LLM 请求信息
export interface LLMRequest {
  id: string
  timestamp: number
  provider: string
  model: string
  method: string
  url: string
  domain?: string
  headers: Record<string, string>
  body: unknown
  parsedBody?: {
    messages?: Array<{ role: string; content: string | any[] }>
    prompt?: string
    temperature?: number
    max_tokens?: number
    // 思考相关字段
    thinking?: { type: string; budget_tokens?: number } | any
    thinking_budget?: number
    reasoning_effort?: 'low' | 'medium' | 'high'
    reasoning?: any
    extended_thinking?: any
    [key: string]: unknown
  }
}

// LLM 响应信息
export interface LLMResponse {
  id: string
  requestId: string
  timestamp: number
  statusCode: number
  headers: Record<string, string>
  body: unknown
  parsedBody?: {
    content?: string
    thinking?: string
    choices?: unknown[]
    usage?: {
      prompt_tokens: number
      completion_tokens: number
      total_tokens: number
    }
    [key: string]: unknown
  }
  duration: number
}

// LLM 指标
export interface LLMMetrics {
  id: string
  requestId: string
  model: string
  provider: string
  promptTokens: number
  completionTokens: number
  totalTokens: number
  estimatedCost: number
  duration: number
  timestamp: number
}

// MCP 调用记录
export interface MCPCall {
  id: string
  requestId: string
  jsonrpcVersion?: string
  rpcId?: string
  toolName: string
  toolTitle?: string
  toolDescription?: string
  arguments?: Record<string, unknown>
  resultContent?: unknown
  resultIsError: boolean
  errorMessage?: string
  executionDuration?: number
  transportType?: 'stdio' | 'sse' | 'http'
  serverName?: string
  traceId?: string
  timestamp: number
}

// 请求列表项（简化版用于列表展示）
export interface RequestListItem {
  id: string
  timestamp: number
  provider: string
  model: string
  method: string
  url: string
  domain?: string
  tokens?: number
  cost?: number
  duration?: number
  statusCode?: number
}

// 监控服务状态
export interface MonitorStatus {
  is_running: boolean
  port: number
}

// 统计汇总
export interface StatsSummary {
  today: {
    count: number
    totalTokens: number
    totalCost: number
    modelStats: Record<string, { count: number; tokens: number; cost: number }>
  }
  thisWeek: {
    count: number
    totalTokens: number
    totalCost: number
    modelStats: Record<string, { count: number; tokens: number; cost: number }>
  }
  thisMonth: {
    count: number
    totalTokens: number
    totalCost: number
    modelStats: Record<string, { count: number; tokens: number; cost: number }>
  }
  allTime: {
    count: number
    totalTokens: number
    totalCost: number
    modelStats: Record<string, { count: number; tokens: number; cost: number }>
  }
}

// 指标统计
export interface MetricsStats {
  count: number
  totalTokens: number
  totalCost: number
  modelStats: Record<string, { count: number; tokens: number; cost: number }>
}

// 每日记录
export interface DailyRecord {
  date: string
  requestCount: number
  totalTokens: number
  totalCost: number
  models: string[]
  modelStats: Record<string, {
    count: number
    tokens: number
    cost: number
  }>
}

// ========== SSE 事件类型 ==========

// SSE 连接成功事件
export interface SSEConnectedEvent {
  type: 'connected'
  timestamp: number
}

// SSE 新请求事件
export interface SSENewRequestEvent {
  type: 'new-request'
  request: LLMRequest
}

// SSE 响应事件
export interface SSEResponseEvent {
  type: 'response'
  response: LLMResponse
}

// SSE 指标事件
export interface SSEMetricsEvent {
  type: 'metrics'
  metrics: LLMMetrics
}

// SSE 事件联合类型
export type SSEEvent = SSEConnectedEvent | SSENewRequestEvent | SSEResponseEvent | SSEMetricsEvent

// SSE 事件回调
export interface SSEEventCallbacks {
  onConnected?: (timestamp: number) => void
  onNewRequest?: (request: LLMRequest) => void
  onResponse?: (response: LLMResponse) => void
  onMetrics?: (metrics: LLMMetrics) => void
  onError?: (error: Error) => void
}
