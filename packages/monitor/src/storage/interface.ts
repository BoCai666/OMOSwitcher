import { LLMRequest, LLMResponse, LLMMetrics, MCPCall, DailyRecord } from '../types.js';

export interface MetricsStats {
  count: number;
  totalTokens: number;
  totalCost: number;
  modelStats: Record<string, { count: number; tokens: number; cost: number }>;
}

export interface DomainStats {
  domain: string;
  count: number;
  tokens: number;
  cost: number;
  avgLatency: number;
  models: Record<string, { count: number; tokens: number; cost: number }>;
}

export interface DomainStatsResult {
  domains: DomainStats[];
}

/**
 * 列表用的精简请求类型（不含大型字段）
 */
export interface RequestListItem {
  id: string;
  timestamp: number;
  provider: string;
  model: string;
  method: string;
  url: string;
  domain?: string;
  tokens?: number;
  cost?: number;
  duration?: number;
  statusCode?: number;
}

/**
 * 完整请求类型（含 metrics）
 * @deprecated 列表接口请使用 RequestListItem
 */
export interface RequestWithMetrics extends LLMRequest {
  tokens?: number;
  cost?: number;
  duration?: number;
  statusCode?: number;
}

export interface DeltaResult {
  newRequests: RequestListItem[];
  updatedRequests: RequestListItem[];
}

export interface StorageInterface {
  saveRequest(request: LLMRequest): void;
  saveResponse(response: LLMResponse): void;
  saveMetrics(metrics: LLMMetrics): void;
  getRecentRequests(limit: number): LLMRequest[];
  getRequestById(id: string): LLMRequest | null;
  getResponseByRequestId(requestId: string): LLMResponse | null;
  getMetricsByRequestId(requestId: string): LLMMetrics | null;
  getMetricsStats(startTime: number, endTime: number): MetricsStats;
  clear(): void;

  // MCP 相关方法
  saveMcpCall(mcpCall: MCPCall): void;
  getMcpCallsByRequestId(requestId: string): MCPCall[];

  // 每日记录方法
  getDailyRecords(startDate: string, endDate: string): DailyRecord[];
  getDailyRecord(date: string): DailyRecord | null;

  // 日期范围查询
  getRequestsByDateRange(startDate: string, endDate: string, limit?: number): LLMRequest[];

  // 数据管理
  hasData(): boolean;
  getAllModels(): string[];

  // 增量查询
  getDelta(since: number, limit: number): DeltaResult;

  // 获取带关联数据的请求列表（精简版，用于列表显示）
  getRecentRequestsWithMetrics(limit: number): RequestListItem[];
  getRequestsByDateRangeWithMetrics(startDate: string, endDate: string, limit?: number): RequestListItem[];
  getRequestsByTimestampRangeWithMetrics(startTime: number, endTime: number, limit?: number): RequestListItem[];

  // 获取域名统计
  getDomainStats(startTime: number, endTime: number): DomainStatsResult;
}
