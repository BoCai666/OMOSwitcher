import { LLMRequest, LLMResponse, LLMMetrics, MCPCall, DailyRecord } from '../types.js';
import { StorageInterface, MetricsStats, DeltaResult, RequestWithMetrics, RequestListItem, DomainStatsResult, DomainStats } from './interface.js';

class MemoryStore implements StorageInterface {
  private requests: Map<string, LLMRequest> = new Map();
  private responses: Map<string, LLMResponse> = new Map();
  private metrics: Map<string, LLMMetrics> = new Map();
  private mcpCalls: Map<string, MCPCall> = new Map();
  private requestOrder: string[] = [];  // 保持插入顺序
  
  // 反向索引：requestId -> responseId/metricsId，实现O(1)删除
  private requestIdToResponseId: Map<string, string> = new Map();
  private requestIdToMetricsId: Map<string, string> = new Map();
  private requestIdToMcpCallIds: Map<string, string[]> = new Map();
  
  // 最大存储数量限制（防止内存溢出）
  private maxSize: number = 1000;
  
  saveRequest(request: LLMRequest): void {
    this.requests.set(request.id, request);
    this.requestOrder.push(request.id);
    
    // 超过限制时删除最旧的数据
    if (this.requestOrder.length > this.maxSize) {
      const oldestId = this.requestOrder.shift();
      if (oldestId) {
        this.deleteRequest(oldestId);
      }
    }
  }
  
  saveResponse(response: LLMResponse): void {
    this.responses.set(response.id, response);
    // 建立反向索引
    this.requestIdToResponseId.set(response.requestId, response.id);
  }
  
  saveMetrics(metrics: LLMMetrics): void {
    this.metrics.set(metrics.id, metrics);
    // 建立反向索引
    this.requestIdToMetricsId.set(metrics.requestId, metrics.id);
  }
  
  getRecentRequests(limit: number): LLMRequest[] {
    return this.requestOrder
      .slice(-limit)
      .map(id => this.requests.get(id))
      .filter((req): req is LLMRequest => req !== undefined);
  }
  
  getRequestById(id: string): LLMRequest | null {
    return this.requests.get(id) || null;
  }
  
  getResponseByRequestId(requestId: string): LLMResponse | null {
    // 使用反向索引 O(1) 查找
    const responseId = this.requestIdToResponseId.get(requestId);
    if (responseId) {
      return this.responses.get(responseId) || null;
    }
    return null;
  }
  
  getMetricsByRequestId(requestId: string): LLMMetrics | null {
    // 使用反向索引 O(1) 查找
    const metricsId = this.requestIdToMetricsId.get(requestId);
    if (metricsId) {
      return this.metrics.get(metricsId) || null;
    }
    return null;
  }
  
  getMetricsStats(startTime: number, endTime: number): MetricsStats {
    let totalTokens = 0;
    let totalCost = 0;
    let count = 0;
    const modelStats: Record<string, { count: number; tokens: number; cost: number }> = {};
    
    for (const metrics of this.metrics.values()) {
      if (metrics.timestamp >= startTime && metrics.timestamp <= endTime) {
        totalTokens += metrics.totalTokens;
        totalCost += metrics.estimatedCost;
        count++;
        
        if (!modelStats[metrics.model]) {
          modelStats[metrics.model] = { count: 0, tokens: 0, cost: 0 };
        }
        modelStats[metrics.model].count++;
        modelStats[metrics.model].tokens += metrics.totalTokens;
        modelStats[metrics.model].cost += metrics.estimatedCost;
      }
    }
    
    return { count, totalTokens, totalCost, modelStats };
  }
  
  clear(): void {
    this.requests.clear();
    this.responses.clear();
    this.metrics.clear();
    this.requestOrder = [];
    this.requestIdToResponseId.clear();
    this.requestIdToMetricsId.clear();
  }
  
  private deleteRequest(id: string): void {
    this.requests.delete(id);
    
    // O(1) 删除关联数据（使用反向索引）
    const responseId = this.requestIdToResponseId.get(id);
    if (responseId) {
      this.responses.delete(responseId);
      this.requestIdToResponseId.delete(id);
    }
    
    const metricsId = this.requestIdToMetricsId.get(id);
    if (metricsId) {
      this.metrics.delete(metricsId);
      this.requestIdToMetricsId.delete(id);
    }
    
    // 删除关联的 MCP 调用
    const mcpCallIds = this.requestIdToMcpCallIds.get(id);
    if (mcpCallIds) {
      for (const mcpCallId of mcpCallIds) {
        this.mcpCalls.delete(mcpCallId);
      }
      this.requestIdToMcpCallIds.delete(id);
    }
  }
  
  // MCP 调用相关方法
  saveMcpCall(mcpCall: MCPCall): void {
    this.mcpCalls.set(mcpCall.id, mcpCall);
    
    // 建立反向索引
    const existingIds = this.requestIdToMcpCallIds.get(mcpCall.requestId) || [];
    existingIds.push(mcpCall.id);
    this.requestIdToMcpCallIds.set(mcpCall.requestId, existingIds);
  }
  
  getMcpCallsByRequestId(requestId: string): MCPCall[] {
    const mcpCallIds = this.requestIdToMcpCallIds.get(requestId) || [];
    return mcpCallIds
      .map(id => this.mcpCalls.get(id))
      .filter((call): call is MCPCall => call !== undefined)
      .sort((a, b) => a.timestamp - b.timestamp);
  }
  
  // 每日记录方法
  getDailyRecords(startDate: string, endDate: string): DailyRecord[] {
    const records: DailyRecord[] = [];
    const start = new Date(startDate).getTime();
    const end = new Date(endDate).getTime() + 24 * 60 * 60 * 1000 - 1; // 包含结束日期
    
    // 按日期分组统计
    const dailyData = new Map<string, {
      count: number;
      tokens: number;
      cost: number;
      models: Set<string>;
      modelStats: Record<string, { count: number; tokens: number; cost: number }>;
    }>();
    
    for (const metrics of this.metrics.values()) {
      if (metrics.timestamp >= start && metrics.timestamp <= end) {
        const date = new Date(metrics.timestamp).toISOString().split('T')[0];
        
        if (!dailyData.has(date)) {
          dailyData.set(date, {
            count: 0,
            tokens: 0,
            cost: 0,
            models: new Set(),
            modelStats: {}
          });
        }
        
        const day = dailyData.get(date)!;
        day.count++;
        day.tokens += metrics.totalTokens;
        day.cost += metrics.estimatedCost;
        day.models.add(metrics.model);
        
        if (!day.modelStats[metrics.model]) {
          day.modelStats[metrics.model] = { count: 0, tokens: 0, cost: 0 };
        }
        day.modelStats[metrics.model].count++;
        day.modelStats[metrics.model].tokens += metrics.totalTokens;
        day.modelStats[metrics.model].cost += metrics.estimatedCost;
      }
    }
    
    // 转换为 DailyRecord 数组
    for (const [date, data] of dailyData) {
      records.push({
        date,
        requestCount: data.count,
        totalTokens: data.tokens,
        totalCost: data.cost,
        models: Array.from(data.models),
        modelStats: data.modelStats
      });
    }
    
    return records.sort((a, b) => a.date.localeCompare(b.date));
  }
  
  getDailyRecord(date: string): DailyRecord | null {
    const records = this.getDailyRecords(date, date);
    return records.length > 0 ? records[0] : null;
  }
  
  // 日期范围查询
  getRequestsByDateRange(startDate: string, endDate: string, limit?: number): LLMRequest[] {
    const start = new Date(startDate).getTime();
    const end = new Date(endDate).getTime() + 24 * 60 * 60 * 1000 - 1;
    
    const requests: LLMRequest[] = [];
    for (const request of this.requests.values()) {
      if (request.timestamp >= start && request.timestamp <= end) {
        requests.push(request);
      }
    }
    
    // 按时间降序排列
    requests.sort((a, b) => b.timestamp - a.timestamp);
    
    if (limit) {
      return requests.slice(0, limit);
    }
    return requests;
  }
  
  // 数据管理
  hasData(): boolean {
    return this.requests.size > 0;
  }
  
  getAllModels(): string[] {
    const models = new Set<string>();
    for (const metrics of this.metrics.values()) {
      if (metrics.model) {
        models.add(metrics.model);
      }
    }
    return Array.from(models).sort();
  }

  /**
   * 获取增量更新数据（内存存储实现）
   */
  getDelta(since: number, limit: number): DeltaResult {
    const newRequests: RequestListItem[] = [];
    const updatedRequests: RequestListItem[] = [];
    
    // 内存存储简化实现：使用 timestamp 判断
    // 注意：内存存储没有 updated_at 字段，仅使用 timestamp
    const allRequests = Array.from(this.requests.values())
      .sort((a, b) => b.timestamp - a.timestamp);
    
    for (const request of allRequests) {
      const metrics = this.getMetricsByRequestId(request.id);
      const response = this.getResponseByRequestId(request.id);
      
      const requestItem: RequestListItem = {
        id: request.id,
        timestamp: request.timestamp,
        provider: request.provider,
        model: request.model,
        method: request.method,
        url: request.url,
        domain: request.domain,
        tokens: metrics?.totalTokens,
        cost: metrics?.estimatedCost,
        duration: metrics?.duration,
        statusCode: response?.statusCode
      };
      
      if (request.timestamp > since) {
        newRequests.push(requestItem);
      }
    }
    
    return {
      newRequests: newRequests.slice(0, limit),
      updatedRequests: updatedRequests.slice(0, limit)
    };
  }

  /**
   * 获取最近的请求列表（带关联数据）
   */
  getRecentRequestsWithMetrics(limit: number): RequestListItem[] {
    const recentRequests = this.getRecentRequests(limit);
    return recentRequests.map(request => {
      const metrics = this.getMetricsByRequestId(request.id);
      const response = this.getResponseByRequestId(request.id);
      
      return {
        id: request.id,
        timestamp: request.timestamp,
        provider: request.provider,
        model: request.model,
        method: request.method,
        url: request.url,
        domain: request.domain,
        tokens: metrics?.totalTokens,
        cost: metrics?.estimatedCost,
        duration: metrics?.duration,
        statusCode: response?.statusCode
      };
    });
  }

  /**
   * 获取指定日期范围内的请求列表（带关联数据）
   */
  getRequestsByDateRangeWithMetrics(startDate: string, endDate: string, limit?: number): RequestListItem[] {
    const requests = this.getRequestsByDateRange(startDate, endDate, limit);
    return requests.map(request => {
      const metrics = this.getMetricsByRequestId(request.id);
      const response = this.getResponseByRequestId(request.id);
      
      return {
        id: request.id,
        timestamp: request.timestamp,
        provider: request.provider,
        model: request.model,
        method: request.method,
        url: request.url,
        domain: request.domain,
        tokens: metrics?.totalTokens,
        cost: metrics?.estimatedCost,
        duration: metrics?.duration,
        statusCode: response?.statusCode
      };
    });
  }

  /**
   * 按时间戳范围获取请求（带关联数据）
   */
  getRequestsByTimestampRangeWithMetrics(startTime: number, endTime: number, limit?: number): RequestListItem[] {
    // 按时间戳过滤
    let filteredRequests = Array.from(this.requests.values())
      .filter(req => req.timestamp >= startTime && req.timestamp <= endTime)
      .sort((a, b) => b.timestamp - a.timestamp);
    
    if (limit !== undefined) {
      filteredRequests = filteredRequests.slice(0, limit);
    }
    
    return filteredRequests.map(request => {
      const metrics = this.getMetricsByRequestId(request.id);
      const response = this.getResponseByRequestId(request.id);
      
      return {
        id: request.id,
        timestamp: request.timestamp,
        provider: request.provider,
        model: request.model,
        method: request.method,
        url: request.url,
        domain: request.domain,
        tokens: metrics?.totalTokens,
        cost: metrics?.estimatedCost,
        duration: metrics?.duration,
        statusCode: response?.statusCode
      };
    });
  }

  /**
   * 获取域名统计（内存存储实现）
   */
  getDomainStats(startTime: number, endTime: number): DomainStatsResult {
    // 用于存储按域名分组的统计数据
    const domainData = new Map<string, {
      count: number;
      tokens: number;
      cost: number;
      latencySum: number;
      models: Record<string, { count: number; tokens: number; cost: number }>;
    }>();

    // 遍历所有请求，按域名分组统计
    for (const request of this.requests.values()) {
      if (request.timestamp >= startTime && request.timestamp <= endTime) {
        const domain = request.domain || 'unknown';
        const metrics = this.getMetricsByRequestId(request.id);

        if (!domainData.has(domain)) {
          domainData.set(domain, {
            count: 0,
            tokens: 0,
            cost: 0,
            latencySum: 0,
            models: {}
          });
        }

        const domainStats = domainData.get(domain)!;
        domainStats.count++;

        if (metrics) {
          domainStats.tokens += metrics.totalTokens;
          domainStats.cost += metrics.estimatedCost;
          domainStats.latencySum += metrics.duration;
        }

        // 按模型统计（包括没有 metrics 的请求，归类为 unknown）
        const modelName = metrics?.model || 'unknown';
        if (!domainStats.models[modelName]) {
          domainStats.models[modelName] = { count: 0, tokens: 0, cost: 0 };
        }
        domainStats.models[modelName].count++;
        if (metrics) {
          domainStats.models[modelName].tokens += metrics.totalTokens;
          domainStats.models[modelName].cost += metrics.estimatedCost;
        }
      }
    }

    // 转换为最终格式
    const domains: DomainStats[] = Array.from(domainData.entries())
      .map(([domain, data]) => ({
        domain,
        count: data.count,
        tokens: data.tokens,
        cost: data.cost,
        avgLatency: data.count > 0 ? data.latencySum / data.count : 0,
        models: data.models
      }))
      .sort((a, b) => b.count - a.count);

    return { domains };
  }
}

// 导出单例实例
export const memoryStore = new MemoryStore();
