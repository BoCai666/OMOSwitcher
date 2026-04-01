import { Router, Response } from 'express';
import { config } from '../config.js';
import { LLMRequest } from '../types.js';
import { ConfigManager } from '../config-manager.js';
import { RequestListItem } from '../storage/interface.js';
import { getStorage } from '../storage/storage-manager.js';
import { existsSync } from 'fs';
import { CA_CERT_FILE } from '../paths.js';

// 创建 ConfigManager 实例
const configManager = new ConfigManager();
// 异步加载配置
let configManagerLoaded = false;

async function ensureConfigManagerLoaded() {
  if (!configManagerLoaded) {
    await configManager.load();
    configManagerLoaded = true;
  }
}

// 存储 SSE 连接的客户端
let clients: Response[] = [];

/**
 * 获取当前连接的客户端列表（用于测试和调试）
 */
export function getClients(): Response[] {
  return clients;
}

/**
 * 设置客户端列表（主要用于测试）
 */
export function setClients(newClients: Response[]): void {
  clients = newClients;
}

/**
 * 广播新请求到所有连接的客户端
 */
export function broadcastNewRequest(request: any): void {
  const data = JSON.stringify({ type: 'new-request', request });
  const message = `data: ${data}\n\n`;

  clients.forEach((client) => {
    client.write(message);
  });
}

/**
 * 广播响应数据到所有连接的客户端
 */
export function broadcastResponse(response: any): void {
  const data = JSON.stringify({ type: 'response', response });
  const message = `data: ${data}\n\n`;

  clients.forEach((client) => {
    client.write(message);
  });
}

/**
 * 广播指标数据到所有连接的客户端
 */
export function broadcastMetrics(metrics: any): void {
  const data = JSON.stringify({ type: 'metrics', metrics });
  const message = `data: ${data}\n\n`;

  clients.forEach((client) => {
    client.write(message);
  });
}

const router = Router();

// 最大返回记录数限制
const MAX_REQUEST_LIMIT = 500;

// GET /api/cert-status - 获取 CA 证书状态
router.get('/cert-status', (req, res) => {
  const exists = existsSync(CA_CERT_FILE);
  res.json({ exists });
});

// GET /api/requests - 获取最近请求列表
router.get('/requests', async (req, res) => {
  try {
    const storage = getStorage();
    
    // 支持日期范围查询（兼容旧参数和新参数）
    const startDate = req.query.startDate as string;
    const endDate = req.query.endDate as string;
    const startTimestamp = req.query.startTimestamp ? parseInt(req.query.startTimestamp as string) : null;
    const endTimestamp = req.query.endTimestamp ? parseInt(req.query.endTimestamp as string) : null;
    const model = req.query.model as string;
    // 强制限制最大数量
    const limit = Math.min(parseInt(req.query.limit as string) || 100, MAX_REQUEST_LIMIT);
    
    let requests: RequestListItem[];
    
    if (startTimestamp !== null && endTimestamp !== null && !isNaN(startTimestamp) && !isNaN(endTimestamp)) {
      // 新方式：使用时间戳范围（前端计算，避免时区问题）
      requests = storage.getRequestsByTimestampRangeWithMetrics(startTimestamp, endTimestamp, limit);
    } else if (startDate && endDate) {
      // 旧方式：使用日期字符串（后端计算）
      requests = storage.getRequestsByDateRangeWithMetrics(startDate, endDate, limit);
    } else {
      requests = storage.getRecentRequestsWithMetrics(limit);
    }
    
    // 模型筛选
    if (model) {
      requests = requests.filter((r: RequestListItem) => r.model === model);
    }
    
    res.json(requests);
  } catch (error) {
    console.error('[ERROR] /api/requests failed:', error);
    res.status(500).json({ error: 'Internal server error', message: (error as Error).message });
  }
});

// GET /api/requests/delta - 获取增量更新（必须在 /:id 之前定义）
router.get('/requests/delta', async (req, res) => {
  try {
    const since = parseInt(req.query.since as string) || 0;
    const limit = Math.min(parseInt(req.query.limit as string) || 50, 100);
    const storage = getStorage();
    
    // 检查是否需要全量刷新（超过5分钟）
    const needsFullRefresh = Date.now() - since > 5 * 60 * 1000;
    
    let newRequests: any[] = [];
    let updatedRequests: any[] = [];
    
    if (!needsFullRefresh && 'getDelta' in storage) {
      const delta = storage.getDelta(since, limit);
      newRequests = delta.newRequests;
      updatedRequests = delta.updatedRequests;
    }
    
    res.json({
      serverTime: Date.now(),
      newRequests,
      updatedRequests,
      needsFullRefresh
    });
  } catch (error) {
    console.error('Failed to get delta:', error);
    res.status(500).json({ error: 'Failed to get delta' });
  }
});

// GET /api/requests/:id - 获取单个请求详情
router.get('/requests/:id', async (req, res) => {
  const storage = getStorage();
  const request = storage.getRequestById(req.params.id);
  if (!request) {
    return res.status(404).json({ error: 'Request not found' });
  }
  
  // 同时返回关联的 MCP 调用
  const mcpCalls = storage.getMcpCallsByRequestId?.(req.params.id) || [];
  
  res.json({
    ...request,
    mcpCalls
  });
});

// GET /api/requests/:id/response - 获取请求的响应
router.get('/requests/:id/response', async (req, res) => {
  const storage = getStorage();
  const response = storage.getResponseByRequestId(req.params.id);
  if (!response) {
    return res.status(404).json({ error: 'Response not found' });
  }
  res.json(response);
});

// GET /api/requests/:id/mcp-calls - 获取请求的 MCP 调用
router.get('/requests/:id/mcp-calls', async (req, res) => {
  const storage = getStorage();
  
  if (!storage.getMcpCallsByRequestId) {
    return res.json({ requestId: req.params.id, calls: [] });
  }
  
  const calls = storage.getMcpCallsByRequestId(req.params.id);
  res.json({
    requestId: req.params.id,
    calls
  });
});

// GET /api/requests/:id/metrics - 获取请求的指标
router.get('/requests/:id/metrics', async (req, res) => {
  const storage = getStorage();
  
  if (!storage.getMetricsByRequestId) {
    return res.status(404).json({ error: 'Metrics not found' });
  }
  
  const metrics = storage.getMetricsByRequestId(req.params.id);
  if (!metrics) {
    return res.status(404).json({ error: 'Metrics not found' });
  }
  res.json(metrics);
});

// GET /api/metrics - 获取统计指标
router.get('/metrics', async (req, res) => {
  const storage = getStorage();
  const startTime = parseInt(req.query.startTime as string) || 0;
  const endTime = parseInt(req.query.endTime as string) || Date.now();
  const stats = storage.getMetricsStats(startTime, endTime);
  res.json(stats);
});

// GET /api/stats/summary - 汇总统计
router.get('/stats/summary', async (req, res) => {
  const storage = getStorage();
  const now = Date.now();
  
  // 使用本地日期计算，与前端筛选保持一致
  const today = new Date();
  const todayStart = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 0, 0, 0, 0).getTime();
  
  const weekStart = todayStart - 7 * 24 * 60 * 60 * 1000;
  const monthStart = todayStart - 30 * 24 * 60 * 60 * 1000;
  
  res.json({
    today: storage.getMetricsStats(todayStart, now),
    thisWeek: storage.getMetricsStats(weekStart, now),
    thisMonth: storage.getMetricsStats(monthStart, now),
    allTime: storage.getMetricsStats(0, now),
  });
});

// GET /api/stats/by-domain - 域名统计
router.get('/stats/by-domain', async (req, res) => {
  const storage = getStorage();
  
  // 支持 startDate 和 endDate 查询参数（YYYY-MM-DD 格式）
  const startDate = req.query.startDate as string;
  const endDate = req.query.endDate as string;
  
  let startTime: number;
  let endTime: number;
  
  if (startDate && endDate) {
    // 将 YYYY-MM-DD 转换为本地时间戳范围（考虑时区）
    // 使用本地时间 00:00:00 和 23:59:59.999
    startTime = new Date(startDate + 'T00:00:00').getTime();
    endTime = new Date(endDate + 'T23:59:59.999').getTime();
  } else {
    // 默认返回所有时间的数据
    startTime = 0;
    endTime = Date.now();
  }
  
  if (!('getDomainStats' in storage)) {
    return res.json({ domains: [] });
  }
  
  const stats = storage.getDomainStats(startTime, endTime);
  res.json(stats);
});

// GET /api/daily-records - 获取每日记录
router.get('/daily-records', async (req, res) => {
  const storage = getStorage();
  
  if (!storage.getDailyRecords) {
    return res.json({ records: [] });
  }
  
  const startDate = req.query.startDate as string;
  const endDate = req.query.endDate as string;
  
  // 默认返回最近30天
  const now = new Date();
  const defaultEndDate = now.toISOString().split('T')[0];
  const defaultStartDate = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];
  
  const records = storage.getDailyRecords(
    startDate || defaultStartDate,
    endDate || defaultEndDate
  );
  
  res.json({ records });
});

// GET /api/daily-records/:date/requests - 获取指定日期的请求
router.get('/daily-records/:date/requests', async (req, res) => {
  const storage = getStorage();
  const date = req.params.date;
  
  // 获取该日期的请求
  const requests: LLMRequest[] = storage.getRequestsByDateRange?.(date, date) || [];
  
  // 计算汇总数据
  let totalTokens = 0;
  let totalCost = 0;
  requests.forEach((req: LLMRequest) => {
    const metrics = storage.getMetricsByRequestId(req.id);
    if (metrics) {
      totalTokens += metrics.totalTokens;
      totalCost += metrics.estimatedCost;
    }
  });
  
  res.json({
    date,
    requests,
    summary: {
      count: requests.length,
      tokens: totalTokens,
      cost: totalCost
    }
  });
});

// GET /api/models - 获取所有模型列表
router.get('/models', async (req, res) => {
  const storage = getStorage();
  
  if (!storage.getAllModels) {
    return res.json({ models: [] });
  }
  
  const models = storage.getAllModels();
  res.json({ models });
});

// POST /api/clear - 清空历史数据
router.post('/clear', async (req, res) => {
  const storage = getStorage();
  storage.clear();
  res.json({ success: true });
});

// GET /api/health - 健康检查
router.get('/health', async (req, res) => {
  const storage = getStorage();
  const storageType = config.storage.type;
  
  const health: any = {
    status: 'ok',
    timestamp: Date.now(),
    storage: {
      type: storageType,
      hasData: storage.hasData?.() || false
    }
  };
  
  if (storageType === 'sqlite') {
    health.database = 'connected';
  }
  
  res.json(health);
});

// POST /api/backup - 创建数据库备份
router.post('/backup', async (req, res) => {
  if (config.storage.type !== 'sqlite') {
    return res.status(400).json({ error: 'Backup only available with SQLite storage' });
  }
  
  try {
    const { DatabaseBackup } = await import('../db/backup.js');
    const { sqliteStorage } = await import('../storage/sqlite-store.js');
    const backup = new DatabaseBackup(sqliteStorage.getDatabase());
    await backup.initialize();
    const backupPath = await backup.createBackup();
    
    res.json({
      status: 'ok',
      backupPath,
      timestamp: new Date().toISOString()
    });
  } catch (err) {
    res.status(500).json({ error: 'Backup failed', message: (err as Error).message });
  }
});

// GET /api/backups - 获取备份列表
router.get('/backups', async (req, res) => {
  if (config.storage.type !== 'sqlite') {
    return res.json({ backups: [], totalSize: 0 });
  }
  
  try {
    const { DatabaseBackup } = await import('../db/backup.js');
    const { sqliteStorage } = await import('../storage/sqlite-store.js');
    const backup = new DatabaseBackup(sqliteStorage.getDatabase());
    const backups = await backup.getBackupList();
    
    const totalSize = backups.reduce((sum: number, b: { size: number }) => sum + b.size, 0);
    
    res.json({
      backups,
      totalSize
    });
  } catch (err) {
    res.status(500).json({ error: 'Failed to get backups', message: (err as Error).message });
  }
});

// GET /api/export - 导出数据
router.get('/export', async (req, res) => {
  const storage = getStorage();
  const format = (req.query.format as string) || 'json';
  const startDate = req.query.startDate as string;
  const endDate = req.query.endDate as string;
  
  // 获取数据
  let requests: LLMRequest[];
  if (startDate && endDate) {
    requests = storage.getRequestsByDateRange?.(startDate, endDate) || 
               storage.getRecentRequests(10000);
  } else {
    requests = storage.getRecentRequests(10000);
  }
  
  // 组装完整数据
  interface ExportData {
    [key: string]: any;
    response: any;
    metrics: any;
    mcpCalls: any[];
  }
  
  const data: ExportData[] = requests.map((req: LLMRequest) => ({
    ...req,
    response: storage.getResponseByRequestId(req.id),
    metrics: storage.getMetricsByRequestId(req.id),
    mcpCalls: storage.getMcpCallsByRequestId?.(req.id) || []
  }));
  
  if (format === 'csv') {
    // CSV 导出
    res.setHeader('Content-Type', 'text/csv');
    res.setHeader('Content-Disposition', 'attachment; filename="export.csv"');
    
    // 简化的 CSV 格式
    const headers = 'timestamp,model,provider,prompt_tokens,completion_tokens,total_tokens,duration';
    const rows = data.map((d: ExportData) => {
      const m = d.metrics || {};
      return `${d.timestamp},"${d.model}","${d.provider}",${m.promptTokens || 0},${m.completionTokens || 0},${m.totalTokens || 0},${m.duration || 0}`;
    });
    
    res.send([headers, ...rows].join('\n'));
  } else {
    // JSON 导出
    res.setHeader('Content-Type', 'application/json');
    res.setHeader('Content-Disposition', 'attachment; filename="export.json"');
    res.json(data);
  }
});

// ========== 配置管理 API ==========

// GET /api/config - 获取当前配置
router.get('/config', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const config = configManager.getConfig();
    res.json({ success: true, data: config });
  } catch (error) {
    console.error('Failed to get config:', error);
    res.status(500).json({ error: 'Failed to get config' });
  }
});

// GET /api/config/raw - 获取原始 JSONC 文件内容
router.get('/config/raw', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    // 读取原始文件内容
    const fs = await import('fs');
    const path = await import('path');
    const { fileURLToPath } = await import('url');
    // pkg 环境中 __dirname 已存在
    const pkgDirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));
    const configPath = path.resolve(pkgDirname, '..', '..', 'config.jsonc');
    
    if (fs.existsSync(configPath)) {
      const content = await fs.promises.readFile(configPath, 'utf-8');
      res.json({ success: true, data: content });
    } else {
      res.status(404).json({ error: 'Config file not found' });
    }
  } catch (error) {
    console.error('Failed to get raw config:', error);
    res.status(500).json({ error: 'Failed to get raw config' });
  }
});

// POST /api/config - 更新完整配置
router.post('/config', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const newConfig = req.body;
    
    if (!newConfig || typeof newConfig !== 'object') {
      return res.status(400).json({ error: 'Invalid config format' });
    }
    
    // 直接设置完整配置
    const fs = await import('fs');
    const path = await import('path');
    const { fileURLToPath } = await import('url');
    // pkg 环境中 __dirname 已存在
    const pkgDirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));
    const configPath = path.resolve(pkgDirname, '..', '..', 'config.jsonc');
    
    await fs.promises.writeFile(configPath, JSON.stringify(newConfig, null, 2), 'utf-8');
    
    // 重新加载配置
    await configManager.load();
    
    res.json({ success: true, data: newConfig });
  } catch (error) {
    console.error('Failed to update config:', error);
    res.status(500).json({ error: 'Failed to update config' });
  }
});

// POST /api/config/validate - 验证配置格式
router.post('/config/validate', async (req, res) => {
  try {
    const configToValidate = req.body;
    
    if (!configToValidate || typeof configToValidate !== 'object') {
      return res.status(400).json({ valid: false, error: 'Config must be an object' });
    }
    
    // 基本验证
    const errors: string[] = [];
    
    if (!Array.isArray(configToValidate.domains)) {
      errors.push('domains must be an array');
    }
    
    if (!configToValidate.pricing || typeof configToValidate.pricing !== 'object') {
      errors.push('pricing must be an object');
    } else if (!Array.isArray(configToValidate.pricing.models)) {
      errors.push('pricing.models must be an array');
    }
    
    if (!configToValidate.ports || typeof configToValidate.ports !== 'object') {
      errors.push('ports must be an object');
    }
    
    res.json({ 
      valid: errors.length === 0, 
      errors 
    });
  } catch (error) {
    console.error('Failed to validate config:', error);
    res.status(500).json({ valid: false, error: 'Failed to validate config' });
  }
});

// GET /api/config/domains - 获取域名配置列表
router.get('/config/domains', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const domains = configManager.get('domains') || [];
    res.json({ success: true, data: domains });
  } catch (error) {
    console.error('Failed to get domains:', error);
    res.status(500).json({ error: 'Failed to get domains' });
  }
});

// POST /api/config/domains - 添加域名配置
router.post('/config/domains', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const { domain, provider, enabled } = req.body;
    
    if (!domain || !provider) {
      return res.status(400).json({ error: 'Domain and provider are required' });
    }
    
    const domains = configManager.get('domains') || [];
    const newDomain = { domain, provider, enabled: enabled !== false };
    domains.push(newDomain);
    await configManager.set('domains', domains);
    
    res.json({ success: true, data: newDomain });
  } catch (error) {
    console.error('Failed to add domain:', error);
    res.status(500).json({ error: 'Failed to add domain' });
  }
});

// DELETE /api/config/domains/:index - 删除域名配置（按索引）
router.delete('/config/domains/:index', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const index = parseInt(req.params.index, 10);
    
    if (isNaN(index)) {
      return res.status(400).json({ error: 'Invalid index' });
    }
    
    const domains = configManager.get('domains') || [];
    
    if (index < 0 || index >= domains.length) {
      return res.status(404).json({ error: 'Domain not found' });
    }
    
    const deletedDomain = domains.splice(index, 1)[0];
    await configManager.set('domains', domains);
    
    res.json({ success: true, data: deletedDomain });
  } catch (error) {
    console.error('Failed to delete domain:', error);
    res.status(500).json({ error: 'Failed to delete domain' });
  }
});

// GET /api/config/pricing - 获取定价配置列表
router.get('/config/pricing', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const pricing = configManager.get('pricing') || { matchStrategy: 'prefix', models: [] };
    res.json({ success: true, data: pricing });
  } catch (error) {
    console.error('Failed to get pricing:', error);
    res.status(500).json({ error: 'Failed to get pricing' });
  }
});

// POST /api/config/pricing - 添加/更新定价配置
router.post('/config/pricing', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const { model, input, output } = req.body;
    
    if (!model || input === undefined || output === undefined) {
      return res.status(400).json({ error: 'Model, input, and output prices are required' });
    }
    
    const pricing = configManager.get('pricing') || { matchStrategy: 'prefix', models: [] };
    const models = pricing.models || [];
    
    // 检查是否已存在，更新或添加
    const existingIndex = models.findIndex((m: any) => m.model === model);
    const newPricing = { model, input: parseFloat(input), output: parseFloat(output) };
    
    if (existingIndex >= 0) {
      models[existingIndex] = newPricing;
    } else {
      models.push(newPricing);
    }
    
    pricing.models = models;
    await configManager.set('pricing', pricing);
    
    res.json({ success: true, data: newPricing });
  } catch (error) {
    console.error('Failed to update pricing:', error);
    res.status(500).json({ error: 'Failed to update pricing' });
  }
});

// GET /api/config/ports - 获取端口配置列表
router.get('/config/ports', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const ports = configManager.get('ports') || {};
    res.json({ success: true, data: ports });
  } catch (error) {
    console.error('Failed to get ports:', error);
    res.status(500).json({ error: 'Failed to get ports' });
  }
});

// POST /api/config/ports - 添加/更新端口配置
router.post('/config/ports', async (req, res) => {
  try {
    await ensureConfigManagerLoaded();
    const { name, port } = req.body;
    
    if (!name || port === undefined) {
      return res.status(400).json({ error: 'Port name and value are required' });
    }
    
    const ports = configManager.get('ports') || {};
    ports[name] = parseInt(port, 10);
    await configManager.set('ports', ports);
    
    res.json({ success: true, data: { [name]: ports[name] } });
  } catch (error) {
    console.error('Failed to update ports:', error);
    res.status(500).json({ error: 'Failed to update ports' });
  }
});

// GET /api/events - SSE 端点，用于实时推送新请求
router.get('/events', (req, res) => {
  // 设置 SSE 所需的 HTTP 头
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
    'X-Accel-Buffering': 'no' // 禁用 Nginx 缓冲
  });
  
  // 发送初始连接成功消息
  res.write(`data: ${JSON.stringify({ type: 'connected', timestamp: Date.now() })}\n\n`);
  
  // 将客户端响应对象添加到客户端列表
  clients.push(res);
  console.log(`[SSE] Client connected. Total clients: ${clients.length}`);
  
  // 发送心跳包以保持连接
  const heartbeatInterval = setInterval(() => {
    res.write(`:heartbeat\n\n`); // 以冒号开头的注释行，不会触发 EventSource 的 onmessage
  }, 30000); // 每30秒发送一次心跳
  
  // 客户端断开连接时清理
  req.on('close', () => {
    clearInterval(heartbeatInterval);
    const index = clients.indexOf(res);
    if (index > -1) {
      clients.splice(index, 1);
    }
    console.log(`[SSE] Client disconnected. Total clients: ${clients.length}`);
  });
  
  // 处理客户端错误（ECONNRESET 是刷新页面时的正常行为，静默处理）
  req.on('error', (err: NodeJS.ErrnoException) => {
    clearInterval(heartbeatInterval);
    // ECONNRESET、ECONNABORTED 等是客户端主动断开的正常情况，不打印错误
    if (err.code !== 'ECONNRESET' && err.code !== 'ECONNABORTED' && err.code !== 'ETIMEDOUT') {
      console.warn('[SSE] Client error:', err.message);
    }
  });
});

// router 已在文件顶部创建并配置了所有路由
// broadcastNewRequest, getClients, setClients 已在文件顶部导出
export { router as default };
