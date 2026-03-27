#!/usr/bin/env node

/**
 * OpenCode LLM Monitor
 * 代理服务器模式 - 监控 LLM API 调用
 */

import { ProxyServer, CertificateManager } from './proxy/index.js';
import { memoryStore } from './storage/memory-store.js';
import { sqliteStorage } from './storage/sqlite-store.js';
import { StorageInterface } from './storage/interface.js';
import { DatabaseBackup } from './db/backup.js';
import { DataCleanupTask } from './tasks/data-cleanup.js';
import { startServer, broadcastNewRequest, broadcastResponse, broadcastMetrics } from './server/index.js';
import { config, updatePortsFromConfig } from './config.js';
import { ConfigManager } from './config-manager.js';
import { LLMRequest, LLMResponse, LLMMetrics, MCPCall } from './types.js';
import { detectMcpCall, extractMcpResult } from './parsers/mcp-detector.js';
import { v4 as uuidv4 } from 'uuid';

// 当前使用的存储
let storage: StorageInterface = memoryStore;
let dbBackup: DatabaseBackup | null = null;
let dataCleanup: DataCleanupTask | null = null;

// 计算指标
function calculateMetrics(response: LLMResponse, modelFromRequest?: string, requestBody?: any): LLMMetrics {
  const parsedBody = response.parsedBody;
  const usage = parsedBody?.usage;
  const content = parsedBody?.content || '';
  

  
  // 优先使用 API 返回的 usage 数据
  let promptTokens = usage?.prompt_tokens || 0;
  let completionTokens = usage?.completion_tokens || 0;
  let totalTokens = usage?.total_tokens || 0;
  
  // 如果没有 usage 数据但有响应内容，进行估算
  if (totalTokens === 0 && content && content.length > 0) {
    // 估算规则：平均每个 token 约 4 个字符
    completionTokens = Math.ceil(content.length / 4);
    
    // 尝试从请求中估算 prompt tokens
    const messages = requestBody?.messages || [];
    if (messages.length > 0) {
      const promptText = messages.map((m: any) => m.content || '').join(' ');
      promptTokens = Math.ceil(promptText.length / 4);
    }
    
    totalTokens = promptTokens + completionTokens;
    
    console.log(`[Metrics] Estimated tokens: prompt=${promptTokens}, completion=${completionTokens}, total=${totalTokens}, contentLength=${content.length}`);
  } else if (totalTokens === 0) {
    console.log('[Metrics] No content available for estimation. Content:', content, 'Length:', content?.length);
  }
  
  // 优先使用请求中的 model，其次使用响应中的 model
  const model = modelFromRequest || response.parsedBody?.model || 'unknown';
  
  // 计算成本 - 支持多种模型（单位：美元/1M tokens）
  let costPer1M = 1; // 默认成本
  if (model.includes('gpt-4')) {
    costPer1M = 30;
  } else if (model.includes('gpt-3.5')) {
    costPer1M = 1.5;
  } else if (model.includes('kimi')) {
    costPer1M = 1; // Kimi 模型成本
  } else if (model.includes('doubao') || model.startsWith('ep-')) {
    costPer1M = 0.1; // 火山引擎豆包模型默认成本
  }
  const estimatedCost = totalTokens > 0 ? (totalTokens / 1000000) * costPer1M : 0;
  
  console.log(`[Metrics] Model: ${model}, Tokens: ${totalTokens}, Cost: $${estimatedCost.toFixed(6)}, Usage:`, usage || 'N/A (estimated)');
  
  return {
    id: `metrics-${response.requestId}`,
    requestId: response.requestId,
    model,
    provider: 'unknown',
    promptTokens,
    completionTokens,
    totalTokens,
    estimatedCost,
    duration: response.duration,
    timestamp: response.timestamp,
  };
}

// 创建 MCP 调用记录
function createMcpCall(request: LLMRequest, response?: LLMResponse): MCPCall | null {
  const detection = detectMcpCall(request);
  if (!detection.isMcpCall) return null;
  
  let resultContent = null;
  let resultIsError = false;
  let errorMessage = undefined;
  
  if (response) {
    const result = extractMcpResult(response.body);
    resultContent = result.content;
    resultIsError = result.isError;
    errorMessage = result.errorMessage;
  }
  
  return {
    id: uuidv4(),
    requestId: request.id,
    jsonrpcVersion: detection.jsonrpcVersion,
    rpcId: detection.rpcId,
    toolName: detection.toolName || 'unknown',
    toolTitle: detection.serverInfo?.name,
    toolDescription: undefined,
    arguments: detection.arguments,
    resultContent,
    resultIsError,
    errorMessage,
    executionDuration: response ? response.duration : undefined,
    transportType: detection.transportType,
    serverName: detection.serverInfo?.name,
    traceId: undefined,
    timestamp: Date.now(),
  };
}

// 显示使用说明
function showUsageInstructions(caCertPath: string, proxyPort: number, webPort: number): void {
  console.log('\n==============================================');
  console.log('  OpenCode LLM Monitor - 代理服务器已启动');
  console.log('==============================================\n');
  
  console.log('📋 使用说明:\n');
  
  console.log('1. 安装 CA 证书 (仅首次需要):');
  console.log(`   证书路径: ${caCertPath}`);
  console.log('   - macOS: 双击证书 -> 添加到系统钥匙串 -> 始终信任');
  console.log('   - Windows: 双击证书 -> 安装到受信任的根证书颁发机构');
  console.log('   - Linux: 复制到 /usr/local/share/ca-certificates/ 并运行 update-ca-certificates\n');
  
  console.log('2. 配置代理环境变量:');
  console.log(`   export HTTP_PROXY=http://localhost:${proxyPort}`);
  console.log(`   export HTTPS_PROXY=http://localhost:${proxyPort}`);
  console.log(`   export http_proxy=http://localhost:${proxyPort}`);
  console.log(`   export https_proxy=http://localhost:${proxyPort}\n`);
  
  console.log('   或在命令前临时设置:');
  console.log(`   HTTP_PROXY=http://localhost:${proxyPort} opencode --enable-interceptor\n`);
  
  console.log('3. 访问 Web 界面查看监控数据:');
  console.log(`   http://localhost:${webPort}\n`);
  
  console.log('4. 启动 OpenCode (在另一个终端):');
  console.log('   opencode --enable-interceptor\n');
  
  console.log('==============================================');
  console.log('  按 Ctrl+C 停止代理服务器');
  console.log('==============================================\n');
}

// 初始化存储
async function initializeStorage(): Promise<void> {
  const storageType = config.storage.type;
  
  if (storageType === 'sqlite') {
    console.log('[Monitor] Initializing SQLite storage...');
    await sqliteStorage.initialize();
    
    // 检查是否需要迁移
    const memoryData = memoryStore.getRecentRequests(1);
    const sqliteHasData = sqliteStorage.hasData();
    
    if (memoryData.length > 0 && !sqliteHasData) {
      console.log('[Migration] 检测到内存数据，开始迁移...');
      await migrateFromMemory();
    }
    
    storage = sqliteStorage;
    
    // 初始化备份
    dbBackup = new DatabaseBackup(sqliteStorage.getDatabase());
    await dbBackup.initialize();
    dbBackup.scheduleDailyBackup();
    
    // 初始化清理任务
    dataCleanup = new DataCleanupTask(sqliteStorage.getDatabase());
    await dataCleanup.initialize();
    dataCleanup.scheduleDailyCleanup();
    
    console.log('[Monitor] SQLite storage initialized');
  } else {
    console.log('[Monitor] Using memory storage');
    storage = memoryStore;
  }
}

// 从内存存储迁移数据
async function migrateFromMemory(): Promise<void> {
  const batchSize = 50;
  let migrated = 0;
  
  const allRequests = memoryStore.getRecentRequests(10000); // 获取尽可能多的数据
  
  console.log(`[Migration] 发现 ${allRequests.length} 条记录需要迁移`);
  
  for (let i = 0; i < allRequests.length; i += batchSize) {
    const batch = allRequests.slice(i, i + batchSize);
    
    for (const request of batch) {
      sqliteStorage.saveRequest(request);
      
      const response = memoryStore.getResponseByRequestId(request.id);
      if (response) sqliteStorage.saveResponse(response);
      
      const metrics = memoryStore.getMetricsByRequestId(request.id);
      if (metrics) sqliteStorage.saveMetrics(metrics);
    }
    
    migrated += batch.length;
    console.log(`[Migration] 已迁移 ${migrated}/${allRequests.length} 条记录`);
  }
  
  console.log(`[Migration] 完成！共迁移 ${migrated} 条记录`);
}

// 全局配置管理器
let configManager: ConfigManager;

// 主函数
async function main() {
  console.log('[Monitor] Starting OpenCode LLM Monitor (Proxy Mode)...\n');
  
  let proxyServer: ProxyServer | null = null;
  
  try {
    // 1. 初始化配置管理器并加载配置
    configManager = new ConfigManager();
    await configManager.load();
    console.log('[Monitor] Configuration loaded');
    
    // 1.1 从配置文件更新端口（环境变量优先级更高）
    updatePortsFromConfig(configManager);
    
    // 2. 初始化存储
    await initializeStorage();
    
    // 3. 初始化 CertificateManager
    const certManager = new CertificateManager();
    const caCertPath = certManager.getCACertPath();
    
    // 4. 创建 ProxyServer 实例
    const proxyPort = config.proxyPort || 7101;
    proxyServer = new ProxyServer({
      port: proxyPort,
      enableMITM: true,
      certManager,
      configManager,
    });
    
    // 5. 监听 llm-request:captured 事件
    proxyServer.on('llm-request:captured', (request: LLMRequest) => {
      storage.saveRequest(request);
      
      // 检测并保存 MCP 调用
      const mcpCall = createMcpCall(request);
      if (mcpCall) {
        storage.saveMcpCall(mcpCall);
      }
      
      // 广播新请求到所有连接的 Web 客户端
      broadcastNewRequest(request);
      
      console.log(`[Monitor] Captured LLM request: ${request.provider} - ${request.model}`);
    });
    
    // 6. 监听 llm-response:captured 事件
    proxyServer.on('llm-response:captured', (response: LLMResponse) => {
      storage.saveResponse(response);
      
      // 从 storage 获取对应的 request 来获取 model
      const request = storage.getRequestById(response.requestId);
      const modelFromRequest = request?.model;
      
      // 计算并保存指标
      const metrics = calculateMetrics(response, modelFromRequest, request?.parsedBody);
      storage.saveMetrics(metrics);
      
      // 广播响应和指标到所有连接的 Web 客户端
      broadcastResponse(response);
      broadcastMetrics(metrics);
      
      // 更新 MCP 调用结果（如果存在）
      // 注意：这里简化处理，实际应该通过 requestId 关联
      
      console.log(`[Monitor] Captured LLM response: ${metrics.totalTokens} tokens, ${metrics.duration}ms`);
    });
    
    // 7. 启动代理服务器
    await proxyServer.start();
    
    // 8. 启动 Web 服务器
    startServer();
    
    // 9. 显示使用说明
    showUsageInstructions(caCertPath, proxyPort, config.port);
    
    // 10. 处理关闭信号
    process.on('SIGINT', async () => {
      console.log('\n[Monitor] Shutting down...');
      
      if (proxyServer) {
        await proxyServer.stop();
      }
      
      if (configManager) {
        configManager.destroy();
      }
      
      console.log('[Monitor] Goodbye!');
      process.exit(0);
    });
    
    process.on('SIGTERM', async () => {
      console.log('\n[Monitor] Shutting down...');
      
      if (proxyServer) {
        await proxyServer.stop();
      }
      
      if (configManager) {
        configManager.destroy();
      }
      
      console.log('[Monitor] Goodbye!');
      process.exit(0);
    });
    
  } catch (err) {
    console.error('[Monitor] Failed to start:', err);
    
    if (proxyServer) {
      await proxyServer.stop();
    }
    
    process.exit(1);
  }
}

// 启动
main();
