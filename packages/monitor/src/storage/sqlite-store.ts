import { LLMRequest, LLMResponse, LLMMetrics, MCPCall, DailyRecord } from '../types.js';
import { StorageInterface, MetricsStats, DeltaResult, RequestWithMetrics, RequestListItem, DomainStatsResult, DomainStats } from './interface.js';
import { dbManager } from '../db/index.js';
import Database from 'better-sqlite3';

/**
 * SQLite 存储实现类
 * 
 * 实现 StorageInterface 接口，使用 SQLite 数据库存储 LLM 请求、响应、指标和 MCP 调用。
 * 特点：
 * - 使用预编译语句提高性能和安全性
 * - 自动处理 JSON 序列化/反序列化
 * - 支持批量操作的事务处理
 */
export class SQLiteStorage implements StorageInterface {
  private db: Database.Database | null = null;

  /**
   * 初始化数据库连接
   */
  async initialize(): Promise<void> {
    this.db = await dbManager.initialize();
  }

  /**
   * 获取数据库实例（用于备份等操作）
   */
  getDatabase(): Database.Database {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db;
  }

  /**
   * 保存 LLM 请求
   * @param request - LLM 请求对象
   */
  saveRequest(request: LLMRequest): void {
    if (!this.db) throw new Error('Database not initialized');
    
    // 提取域名：优先使用 request.domain，否则从 URL 中提取
    const domain = request.domain || this.extractDomain(request.url || '');
    
    const stmt = this.db.prepare(`
      INSERT OR REPLACE INTO requests 
      (id, timestamp, provider, model, method, url, domain, headers, body, parsed_body) 
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);
    
    stmt.run(
      request.id,
      request.timestamp,
      request.provider || 'unknown',
      request.model || '',
      request.method || '',
      request.url || '',
      domain,
      JSON.stringify(request.headers || {}),
      JSON.stringify(request.body),
      JSON.stringify(request.parsedBody || null)
    );
  }

  /**
   * 保存 LLM 响应
   * @param response - LLM 响应对象
   */
  saveResponse(response: LLMResponse): void {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      INSERT OR REPLACE INTO responses 
      (id, request_id, timestamp, status_code, headers, body, parsed_body, duration) 
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `);
    
    stmt.run(
      response.id,
      response.requestId,
      response.timestamp,
      response.statusCode || 0,
      JSON.stringify(response.headers || {}),
      JSON.stringify(response.body),
      JSON.stringify(response.parsedBody || null),
      response.duration || 0
    );
  }

  /**
   * 保存指标数据
   * @param metrics - LLM 指标对象
   */
  saveMetrics(metrics: LLMMetrics): void {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      INSERT OR REPLACE INTO metrics 
      (id, request_id, model, provider, prompt_tokens, completion_tokens, total_tokens, estimated_cost, duration, timestamp) 
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);
    
    stmt.run(
      metrics.id,
      metrics.requestId,
      metrics.model || '',
      metrics.provider || '',
      metrics.promptTokens || 0,
      metrics.completionTokens || 0,
      metrics.totalTokens || 0,
      metrics.estimatedCost || 0,
      metrics.duration || 0,
      metrics.timestamp || Date.now()
    );
  }

  /**
   * 保存 MCP 调用记录
   * @param mcpCall - MCP 调用对象
   */
  saveMcpCall(mcpCall: MCPCall): void {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      INSERT OR REPLACE INTO mcp_calls 
      (id, request_id, jsonrpc_version, rpc_id, tool_name, tool_title, tool_description, arguments, result_content, result_is_error, error_message, execution_duration, transport_type, server_name, trace_id, timestamp) 
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);
    
    stmt.run(
      mcpCall.id,
      mcpCall.requestId,
      mcpCall.jsonrpcVersion || '2.0',
      mcpCall.rpcId || null,
      mcpCall.toolName,
      mcpCall.toolTitle || null,
      mcpCall.toolDescription || null,
      JSON.stringify(mcpCall.arguments || {}),
      JSON.stringify(mcpCall.resultContent || null),
      mcpCall.resultIsError ? 1 : 0,
      mcpCall.errorMessage || null,
      mcpCall.executionDuration || null,
      mcpCall.transportType || null,
      mcpCall.serverName || null,
      mcpCall.traceId || null,
      mcpCall.timestamp
    );
  }

  /**
   * 获取最近的请求列表
   * @param limit - 返回的最大数量
   * @returns LLM 请求数组
   */
  getRecentRequests(limit: number): LLMRequest[] {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      SELECT * FROM requests 
      ORDER BY timestamp DESC 
      LIMIT ?
    `);
    
    const rows = stmt.all(limit) as any[];
    return rows.map(row => this.parseRequest(row));
  }

  /**
   * 获取最近的请求列表（带关联数据）
   * @param limit - 返回的最大数量
   * @returns 带 metrics 的请求数组
   */
  getRecentRequestsWithMetrics(limit: number): RequestListItem[] {
    if (!this.db) throw new Error('Database not initialized');
    
    // 只选择列表显示需要的字段，排除大型字段（body, headers）
    const stmt = this.db.prepare(`
      SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
             res.status_code as status_code,
             m.total_tokens as tokens,
             m.estimated_cost as cost,
             m.duration as duration
      FROM requests r
      LEFT JOIN responses res ON r.id = res.request_id
      LEFT JOIN metrics m ON r.id = m.request_id
      ORDER BY r.timestamp DESC
      LIMIT ?
    `);
    
    const rows = stmt.all(limit) as any[];
    return rows.map(row => this.parseRequestWithMetrics(row));
  }

  /**
   * 根据 ID 获取请求
   * @param id - 请求 ID
   * @returns LLM 请求对象或 null
   */
  getRequestById(id: string): LLMRequest | null {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT * FROM requests WHERE id = ?');
    const row = stmt.get(id) as any;
    
    return row ? this.parseRequest(row) : null;
  }

  /**
   * 根据请求 ID 获取响应
   * @param requestId - 请求 ID
   * @returns LLM 响应对象或 null
   */
  getResponseByRequestId(requestId: string): LLMResponse | null {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT * FROM responses WHERE request_id = ?');
    const row = stmt.get(requestId) as any;
    
    return row ? this.parseResponse(row) : null;
  }

  /**
   * 根据请求 ID 获取指标
   * @param requestId - 请求 ID
   * @returns LLM 指标对象或 null
   */
  getMetricsByRequestId(requestId: string): LLMMetrics | null {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT * FROM metrics WHERE request_id = ?');
    const row = stmt.get(requestId) as any;
    
    return row ? this.parseMetrics(row) : null;
  }

  /**
   * 根据请求 ID 获取 MCP 调用列表
   * @param requestId - 请求 ID
   * @returns MCP 调用数组
   */
  getMcpCallsByRequestId(requestId: string): MCPCall[] {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT * FROM mcp_calls WHERE request_id = ? ORDER BY timestamp ASC');
    const rows = stmt.all(requestId) as any[];
    
    return rows.map(row => this.parseMcpCall(row));
  }

  /**
   * 获取指定时间范围内的指标统计
   * @param startTime - 开始时间戳（毫秒）
   * @param endTime - 结束时间戳（毫秒）
   * @returns 指标统计对象
   */
  getMetricsStats(startTime: number, endTime: number): MetricsStats {
    if (!this.db) throw new Error('Database not initialized');
    
    // 获取基本统计数据
    const statsStmt = this.db.prepare(`
      SELECT 
        COUNT(*) as count,
        COALESCE(SUM(total_tokens), 0) as total_tokens,
        COALESCE(SUM(estimated_cost), 0) as total_cost
      FROM metrics 
      WHERE timestamp >= ? AND timestamp <= ?
    `);
    const statsRow = statsStmt.get(startTime, endTime) as any;
    
    // 获取按模型分组的统计数据
    const modelStmt = this.db.prepare(`
      SELECT 
        model,
        COUNT(*) as count,
        COALESCE(SUM(total_tokens), 0) as tokens,
        COALESCE(SUM(estimated_cost), 0) as cost
      FROM metrics 
      WHERE timestamp >= ? AND timestamp <= ?
      GROUP BY model
    `);
    const modelRows = modelStmt.all(startTime, endTime) as any[];
    
    // 构建模型统计对象
    const modelStats: Record<string, { count: number; tokens: number; cost: number }> = {};
    for (const row of modelRows) {
      modelStats[row.model] = {
        count: Number(row.count),
        tokens: Number(row.tokens),
        cost: Number(row.cost)
      };
    }
    
    return {
      count: Number(statsRow.count),
      totalTokens: Number(statsRow.total_tokens),
      totalCost: Number(statsRow.total_cost),
      modelStats
    };
  }

  /**
   * 获取指定日期范围内的每日记录
   * @param startDate - 开始日期（YYYY-MM-DD）
   * @param endDate - 结束日期（YYYY-MM-DD）
   * @returns 每日记录数组
   */
  getDailyRecords(startDate: string, endDate: string): DailyRecord[] {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      SELECT * FROM daily_records 
      WHERE date >= ? AND date <= ?
      ORDER BY date ASC
    `);
    
    const rows = stmt.all(startDate, endDate) as any[];
    return rows.map(row => this.parseDailyRecord(row));
  }

  /**
   * 获取指定日期的记录
   * @param date - 日期（YYYY-MM-DD）
   * @returns 每日记录对象或 null
   */
  getDailyRecord(date: string): DailyRecord | null {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT * FROM daily_records WHERE date = ?');
    const row = stmt.get(date) as any;
    
    return row ? this.parseDailyRecord(row) : null;
  }

  /**
   * 获取指定日期范围内的请求列表
   * @param startDate - 开始日期（YYYY-MM-DD）
   * @param endDate - 结束日期（YYYY-MM-DD）
   * @param limit - 可选的最大返回数量
   * @returns LLM 请求数组
   */
  getRequestsByDateRange(startDate: string, endDate: string, limit?: number): LLMRequest[] {
    if (!this.db) throw new Error('Database not initialized');
    
    // 将 YYYY-MM-DD 转换为本地时间戳范围（考虑时区）
    // 使用本地时间 00:00:00 和 23:59:59.999
    const startTimestamp = new Date(startDate + 'T00:00:00').getTime();
    const endTimestamp = new Date(endDate + 'T23:59:59.999').getTime();
    
    let sql = `
      SELECT * FROM requests 
      WHERE timestamp >= ? AND timestamp <= ?
      ORDER BY timestamp DESC
    `;
    const params: any[] = [startTimestamp, endTimestamp];
    
    if (limit !== undefined) {
      sql += ' LIMIT ?';
      params.push(limit);
    }
    
    const stmt = this.db.prepare(sql);
    const rows = stmt.all(...params) as any[];
    return rows.map(row => this.parseRequest(row));
  }

  /**
   * 获取指定日期范围内的请求列表（带关联数据）
   * @param startDate - 开始日期（YYYY-MM-DD）
   * @param endDate - 结束日期（YYYY-MM-DD）
   * @param limit - 可选的最大返回数量
   * @returns 带 metrics 的请求数组
   */
  getRequestsByDateRangeWithMetrics(startDate: string, endDate: string, limit?: number): RequestListItem[] {
    if (!this.db) throw new Error('Database not initialized');
    
    // 将 YYYY-MM-DD 转换为本地时间戳范围（考虑时区）
    // 使用本地时间 00:00:00 和 23:59:59.999
    const startTimestamp = new Date(startDate + 'T00:00:00').getTime();
    const endTimestamp = new Date(endDate + 'T23:59:59.999').getTime();
    
    // 只选择列表显示需要的字段，排除大型字段（body, headers）
    let sql = `
      SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
             res.status_code as status_code,
             m.total_tokens as tokens,
             m.estimated_cost as cost,
             m.duration as duration
      FROM requests r
      LEFT JOIN responses res ON r.id = res.request_id
      LEFT JOIN metrics m ON r.id = m.request_id
      WHERE r.timestamp >= ? AND r.timestamp <= ?
      ORDER BY r.timestamp DESC
    `;
    const params: any[] = [startTimestamp, endTimestamp];
    
    if (limit !== undefined) {
      sql += ' LIMIT ?';
      params.push(limit);
    }
    
    const stmt = this.db.prepare(sql);
    const rows = stmt.all(...params) as any[];
    return rows.map(row => this.parseRequestWithMetrics(row));
  }

  /**
   * 按时间戳范围获取请求（带 metrics）
   * @param startTime - 开始时间戳（毫秒）
   * @param endTime - 结束时间戳（毫秒）
   * @param limit - 限制数量
   * @returns 带 metrics 的请求数组
   */
  getRequestsByTimestampRangeWithMetrics(startTime: number, endTime: number, limit?: number): RequestListItem[] {
    if (!this.db) throw new Error('Database not initialized');
    
    // 只选择列表显示需要的字段，排除大型字段（body, headers）
    let sql = `
      SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
             res.status_code as status_code,
             m.total_tokens as tokens,
             m.estimated_cost as cost,
             m.duration as duration
      FROM requests r
      LEFT JOIN responses res ON r.id = res.request_id
      LEFT JOIN metrics m ON r.id = m.request_id
      WHERE r.timestamp >= ? AND r.timestamp <= ?
      ORDER BY r.timestamp DESC
    `;
    const params: any[] = [startTime, endTime];
    
    if (limit !== undefined) {
      sql += ' LIMIT ?';
      params.push(limit);
    }
    
    const stmt = this.db.prepare(sql);
    const rows = stmt.all(...params) as any[];
    return rows.map(row => this.parseRequestWithMetrics(row));
  }

  /**
   * 检查是否有数据
   * @returns 是否有数据
   */
  hasData(): boolean {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT COUNT(*) as count FROM requests LIMIT 1');
    const row = stmt.get() as any;
    return row && row.count > 0;
  }

  /**
   * 获取所有模型列表
   * @returns 模型名称数组
   */
  getAllModels(): string[] {
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('SELECT DISTINCT model FROM metrics WHERE model IS NOT NULL AND model != \'\' ORDER BY model');
    const rows = stmt.all() as any[];
    return rows.map(row => row.model);
  }

  /**
   * 获取域名统计
   * @param startTime - 开始时间戳（毫秒）
   * @param endTime - 结束时间戳（毫秒）
   * @returns 域名统计结果
   */
  getDomainStats(startTime: number, endTime: number): DomainStatsResult {
    if (!this.db) throw new Error('Database not initialized');
    
    // 获取域名基础统计
    const domainStmt = this.db.prepare(`
      SELECT 
        r.domain,
        COUNT(*) as count,
        COALESCE(SUM(m.total_tokens), 0) as tokens,
        COALESCE(SUM(m.estimated_cost), 0) as cost,
        COALESCE(AVG(m.duration), 0) as avg_latency
      FROM requests r
      LEFT JOIN metrics m ON r.id = m.request_id
      WHERE r.timestamp >= ? AND r.timestamp <= ?
      GROUP BY r.domain
      ORDER BY count DESC
    `);
    const domainRows = domainStmt.all(startTime, endTime) as any[];
    
    // 获取域名+模型组合的统计（包括没有 model 的请求，归类为 unknown）
    const modelStmt = this.db.prepare(`
      SELECT 
        r.domain,
        COALESCE(NULLIF(m.model, ''), 'unknown') as model_name,
        COUNT(*) as count,
        COALESCE(SUM(m.total_tokens), 0) as tokens,
        COALESCE(SUM(m.estimated_cost), 0) as cost
      FROM requests r
      LEFT JOIN metrics m ON r.id = m.request_id
      WHERE r.timestamp >= ? AND r.timestamp <= ?
      GROUP BY r.domain, model_name
    `);
    const modelRows = modelStmt.all(startTime, endTime) as any[];
    
    // 构建模型统计映射
    const modelStatsByDomain: Record<string, Record<string, { count: number; tokens: number; cost: number }>> = {};
    for (const row of modelRows) {
      if (!modelStatsByDomain[row.domain]) {
        modelStatsByDomain[row.domain] = {};
      }
      modelStatsByDomain[row.domain][row.model_name] = {
        count: Number(row.count),
        tokens: Number(row.tokens),
        cost: Number(row.cost)
      };
    }
    
    // 构建最终的域名统计结果
    const domains: DomainStats[] = domainRows.map(row => ({
      domain: row.domain || 'unknown',
      count: Number(row.count),
      tokens: Number(row.tokens),
      cost: Number(row.cost),
      avgLatency: Number(row.avg_latency),
      models: modelStatsByDomain[row.domain] || {}
    }));
    
    return { domains };
  }

  /**
   * 清空所有数据
   */
  clear(): void {
    if (!this.db) throw new Error('Database not initialized');
    
    // 由于外键约束设置了 ON DELETE CASCADE，删除 requests 会自动删除关联数据
    this.db.exec('DELETE FROM requests');
  }

  /**
   * 获取增量更新数据
   * @param since - 起始时间戳（毫秒）
   * @param limit - 返回的最大数量
   * @returns 新增和更新的请求
   */
  getDelta(since: number, limit: number): DeltaResult {
    if (!this.db) throw new Error('Database not initialized');
    
    // 查询新增请求（timestamp > since）
    // 只选择列表显示需要的字段，排除大型字段（body, headers）
    const newStmt = this.db.prepare(`
      SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
             m.total_tokens as tokens, 
             m.estimated_cost as cost, 
             m.duration,
             res.status_code as statusCode
      FROM requests r
      LEFT JOIN metrics m ON r.id = m.request_id
      LEFT JOIN responses res ON r.id = res.request_id
      WHERE r.timestamp > ?
      ORDER BY r.timestamp DESC
      LIMIT ?
    `);
    const newRows = newStmt.all(since, limit) as any[];
    
    // 查询更新的请求（updated_at > since 但 timestamp <= since）
    const updatedStmt = this.db.prepare(`
      SELECT r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
             m.total_tokens as tokens, 
             m.estimated_cost as cost, 
             m.duration,
             res.status_code as statusCode
      FROM requests r
      LEFT JOIN metrics m ON r.id = m.request_id
      LEFT JOIN responses res ON r.id = res.request_id
      WHERE r.updated_at > ? AND r.timestamp <= ?
      ORDER BY r.updated_at DESC
      LIMIT ?
    `);
    const updatedRows = updatedStmt.all(since, since, limit) as any[];
    
    return {
      newRequests: newRows.map(row => this.parseRequestWithMetrics(row)),
      updatedRequests: updatedRows.map(row => this.parseRequestWithMetrics(row))
    };
  }

  /**
   * 解析包含 metrics 的请求（列表用，不含大型字段）
   */
  private parseRequestWithMetrics(row: any): RequestListItem {
    return {
      id: row.id,
      timestamp: Number(row.timestamp),
      provider: row.provider || 'unknown',
      model: row.model || '',
      method: row.method || '',
      url: row.url || '',
      domain: row.domain || '',
      tokens: row.tokens !== null && row.tokens !== undefined ? Number(row.tokens) : undefined,
      cost: row.cost !== null && row.cost !== undefined ? Number(row.cost) : undefined,
      duration: row.duration !== null && row.duration !== undefined ? Number(row.duration) : undefined,
      statusCode: (row.status_code || row.statusCode) !== null && (row.status_code || row.statusCode) !== undefined 
        ? Number(row.status_code || row.statusCode) 
        : undefined
    };
  }

  // ============ 私有辅助方法 ============

  /**
   * 解析数据库行到 LLMRequest
   */
  private parseRequest(row: any): LLMRequest {
    return {
      id: row.id,
      timestamp: Number(row.timestamp),
      provider: row.provider || 'unknown',
      model: row.model || '',
      method: row.method || '',
      url: row.url || '',
      domain: row.domain || '',
      headers: this.safeJsonParse(row.headers, {}),
      body: this.safeJsonParse(row.body, null),
      parsedBody: this.safeJsonParse(row.parsed_body, undefined)
    };
  }

  /**
   * 从 URL 中提取域名
   * @param url - 完整的 URL 字符串
   * @returns 域名或空字符串
   */
  private extractDomain(url: string): string {
    try {
      const urlObj = new URL(url);
      return urlObj.hostname;
    } catch {
      return '';
    }
  }

  /**
   * 解析数据库行到 LLMResponse
   */
  private parseResponse(row: any): LLMResponse {
    return {
      id: row.id,
      requestId: row.request_id,
      timestamp: Number(row.timestamp),
      statusCode: Number(row.status_code) || 0,
      headers: this.safeJsonParse(row.headers, {}),
      body: this.safeJsonParse(row.body, null),
      parsedBody: this.safeJsonParse(row.parsed_body, undefined),
      duration: Number(row.duration) || 0
    };
  }

  /**
   * 解析数据库行到 LLMMetrics
   */
  private parseMetrics(row: any): LLMMetrics {
    return {
      id: row.id,
      requestId: row.request_id,
      model: row.model || '',
      provider: row.provider || '',
      promptTokens: Number(row.prompt_tokens) || 0,
      completionTokens: Number(row.completion_tokens) || 0,
      totalTokens: Number(row.total_tokens) || 0,
      estimatedCost: Number(row.estimated_cost) || 0,
      duration: Number(row.duration) || 0,
      timestamp: Number(row.timestamp) || Date.now()
    };
  }

  /**
   * 解析数据库行到 MCPCall
   */
  private parseMcpCall(row: any): MCPCall {
    return {
      id: row.id,
      requestId: row.request_id,
      jsonrpcVersion: row.jsonrpc_version || '2.0',
      rpcId: row.rpc_id || undefined,
      toolName: row.tool_name,
      toolTitle: row.tool_title || undefined,
      toolDescription: row.tool_description || undefined,
      arguments: this.safeJsonParse(row.arguments, {}),
      resultContent: this.safeJsonParse(row.result_content, null),
      resultIsError: Boolean(row.result_is_error),
      errorMessage: row.error_message || undefined,
      executionDuration: row.execution_duration || undefined,
      transportType: row.transport_type || undefined,
      serverName: row.server_name || undefined,
      traceId: row.trace_id || undefined,
      timestamp: Number(row.timestamp)
    };
  }

  /**
   * 解析数据库行到 DailyRecord
   */
  private parseDailyRecord(row: any): DailyRecord {
    const models = this.safeJsonParse(row.models, []);
    const modelStats = this.safeJsonParse(row.model_stats, {});
    
    return {
      date: row.date,
      requestCount: Number(row.request_count) || 0,
      totalTokens: Number(row.total_tokens) || 0,
      totalCost: Number(row.total_cost) || 0,
      models: Array.isArray(models) ? models : [],
      modelStats: modelStats || {}
    };
  }

  /**
   * 安全解析 JSON 字符串
   * @param value - 要解析的字符串
   * @param defaultValue - 解析失败时的默认值
   * @returns 解析后的值或默认值
   */
  private safeJsonParse<T>(value: string | null | undefined, defaultValue: T): T {
    if (!value || value === 'null' || value === 'undefined') {
      return defaultValue;
    }
    try {
      return JSON.parse(value) as T;
    } catch {
      return defaultValue;
    }
  }
}

/**
 * SQLite 存储单例实例
 */
export const sqliteStorage = new SQLiteStorage();
