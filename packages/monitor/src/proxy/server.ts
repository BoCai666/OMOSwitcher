/**
 * HTTP/HTTPS 代理服务器
 * 
 * 功能：
 * 1. 接收来自 OpenCode 的 HTTP/HTTPS 请求
 * 2. 识别 LLM API 请求
 * 3. 记录请求/响应
 * 4. 转发到目标服务器
 * 5. 返回响应给 OpenCode
 */

import http from 'http';
import https from 'https';
import net from 'net';
import httpProxy from 'http-proxy';
import { EventEmitter } from 'events';
import { captureRequest, updateProviderPatterns } from './request-capture.js';
import { captureResponse } from './response-capture.js';
import { MITMHandler } from './mitm.js';
import type { LLMRequest, LLMResponse } from '../types.js';
import { ConfigManager } from '../config-manager.js';

// 临时的 CertificateManager 接口（后续任务实现）
export interface CertificateManager {
  getCertificateForDomain(domain: string): { key: string; cert: string; ca: string };
  getCACertPath(): string;
  getCACertContent(): string;
}

export interface ProxyOptions {
  port: number;
  httpsPort?: number;
  enableMITM: boolean;
  certManager?: CertificateManager;
  configManager: ConfigManager;
}

export class ProxyServer extends EventEmitter {
  private httpServer: http.Server;
  private proxy: httpProxy;
  private options: ProxyOptions;
  private isRunning: boolean = false;

  constructor(options: ProxyOptions) {
    super();
    this.options = options;
    this.proxy = this.createProxy();
    this.httpServer = this.createServer();
    
    // 初始化 provider patterns
    this.updateProviderPatterns();
    
    // 监听配置变更，动态更新域名列表
    this.options.configManager.watch((key) => {
      if (key === 'domains') {
        console.log('[Proxy] Domain configuration updated');
        this.updateProviderPatterns();
      }
    });
  }

  /**
   * 更新 request-capture 中的 provider patterns
   */
  private updateProviderPatterns(): void {
    const domains = this.options.configManager.get<{domain: string; provider: string; enabled: boolean}[]>('domains') || [];
    const patterns = domains
      .filter(d => d.enabled !== false)
      .map(d => ({
        pattern: d.domain,
        provider: this.normalizeProvider(d.provider)
      }));
    
    updateProviderPatterns(patterns);
    console.log(`[Proxy] Loaded ${patterns.length} domain patterns from config`);
  }

  private createProxy(): httpProxy {
    return httpProxy.createProxyServer({
      secure: false,
      changeOrigin: true,
      ws: true
    });
  }

  private createServer(): http.Server {
    const server = http.createServer((req, res) => {
      this.handleRequest(req, res);
    });

    server.on('connect', (req, socket, head) => {
      this.handleConnect(req, socket as any, head);
    });

    return server;
  }

  private async handleRequest(
    req: http.IncomingMessage,
    res: http.ServerResponse
  ): Promise<void> {
    const target = req.headers.host;
    
    if (!target) {
      this.sendError(res, 400, 'Bad Request: No Host header');
      return;
    }

    await this.handleHttpRequest(req, res, target);
  }

  private async handleHttpRequest(
    req: http.IncomingMessage,
    res: http.ServerResponse,
    target: string
  ): Promise<void> {
    const isLLM = this.isLLMRequest(target);
    
    if (isLLM) {
      const llmRequest = await captureRequest(req);
      this.emit('llm-request:detected', { host: target, url: req.url, request: llmRequest });
      this.emit('llm-request:captured', llmRequest);
      
      await this.forwardWithCapture(req, res, target, llmRequest);
    } else {
      this.proxy.web(req, res, { target: `http://${target}` });
    }
  }

  private async handleConnect(
    req: http.IncomingMessage,
    clientSocket: net.Socket,
    head: Buffer
  ): Promise<void> {
    const targetHost = req.url;
    
    if (!targetHost) {
      clientSocket.end('HTTP/1.1 400 Bad Request\r\n\r\n');
      return;
    }

    console.log(`[Proxy] CONNECT request to ${targetHost}`);

    if (this.options.enableMITM && this.options.certManager) {
      const [hostname] = targetHost.split(':');
      if (this.isLLMRequest(hostname)) {
        console.log(`[Proxy] Enabling MITM for ${hostname}`);
        try {
          const mitmHandler = new MITMHandler({
            certManager: this.options.certManager,
            enableDebug: true
          });
          
          mitmHandler.on('http:request', (requestData) => {
            this.handleMitmRequest(requestData, hostname);
          });
          
          mitmHandler.on('http:response', (responseData) => {
            this.handleMitmResponse(responseData, hostname);
          });
          
          await mitmHandler.handleConnect(req, clientSocket, head);
          return;
        } catch (error) {
          console.error('[Proxy] MITM failed, falling back to tunnel:', error);
        }
      }
    }

    this.establishTunnel(targetHost, clientSocket, head);
  }

  private establishTunnel(
    targetHost: string,
    clientSocket: net.Socket,
    head: Buffer
  ): void {
    const [hostname, portStr] = targetHost.split(':');
    const port = parseInt(portStr) || 443;

    console.log(`[Proxy] Establishing tunnel to ${hostname}:${port}`);

    const serverSocket = net.connect(port, hostname, () => {
      clientSocket.write('HTTP/1.1 200 Connection Established\r\n\r\n');
      
      if (head && head.length > 0) {
        serverSocket.write(head);
      }
      
      serverSocket.pipe(clientSocket);
      clientSocket.pipe(serverSocket);
    });

    serverSocket.on('error', (err) => {
      console.error('[Proxy] Tunnel error:', err);
      clientSocket.end();
    });

    clientSocket.on('error', (err) => {
      // ECONNRESET 是客户端正常断开，不需要记录
      if ((err as any).code !== 'ECONNRESET') {
        console.error('[Proxy] Client socket error:', err);
      }
      serverSocket.end();
    });
  }

  private handleMitmRequest(requestData: any, host: string): void {
    const provider = this.detectProvider(requestData.url);
    
    let parsedBody: any = undefined;
    if (requestData.body) {
      try {
        const bodyStr = requestData.body.toString('utf8');
        parsedBody = JSON.parse(bodyStr);
      } catch {
        // 非 JSON 请求体，忽略
      }
    }
    
    const model = parsedBody?.model || 'unknown';
    
    const llmRequest: LLMRequest = {
      id: requestData.id,
      timestamp: requestData.timestamp,
      provider: provider as any,
      model,
      method: requestData.method,
      url: requestData.url,
      headers: requestData.headers as Record<string, string>,
      body: requestData.body,
      parsedBody
    };
    
    this.emit('llm-request:captured', llmRequest);
    console.log(`[Proxy] Captured request: ${requestData.method} ${requestData.url}`);
  }

  private handleMitmResponse(responseData: any, host: string): void {
    let parsedBody: any = undefined;
    if (responseData.body) {
      try {
        const bodyStr = responseData.body.toString('utf8');
        if (responseData.headers['content-type']?.includes('text/event-stream')) {
          parsedBody = this.parseSSEResponse(bodyStr);
        } else {
          parsedBody = JSON.parse(bodyStr);
        }
      } catch {
        // 非 JSON 响应体，忽略
      }
    }
    
    const llmResponse: LLMResponse = {
      id: responseData.id,
      requestId: responseData.requestId,
      timestamp: responseData.timestamp,
      statusCode: responseData.statusCode,
      headers: responseData.headers as Record<string, string>,
      body: responseData.body,
      parsedBody,
      duration: responseData.duration
    };
    
    this.emit('llm-response:captured', llmResponse);
    console.log(`[Proxy] Captured response: ${responseData.statusCode} for ${responseData.requestId}`);
  }

  private parseSSEResponse(body: string): any {
    const contents: string[] = [];
    let usage: any = undefined;
    const lines = body.split('\n');
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6).trim();
        if (data === '[DONE]') continue;
        try {
          const json = JSON.parse(data);
          // 提取内容
          if (json.choices?.[0]?.delta?.content) {
            contents.push(json.choices[0].delta.content);
          }
          // 提取 usage（通常在最后一个 chunk）
          if (json.usage) {
            usage = {
              prompt_tokens: json.usage.prompt_tokens || json.usage.input_tokens || 0,
              completion_tokens: json.usage.completion_tokens || json.usage.output_tokens || 0,
              total_tokens: json.usage.total_tokens || json.usage.total || 0
            };
            // 如果 total_tokens 为 0 但有 input/output，计算 total
            if (usage.total_tokens === 0 && (usage.prompt_tokens > 0 || usage.completion_tokens > 0)) {
              usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;
            }
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
    
    const result: any = {
      content: contents.join(''),
      choices: [{ delta: { content: contents.join('') } }]
    };
    
    // 如果捕获到 usage，添加到结果中
    if (usage) {
      result.usage = usage;
    }
    
    return result;
  }

  private detectProvider(url: string): string {
    // 从配置中读取域名和提供商映射
    const domains = this.options.configManager.get<{domain: string; provider: string; enabled: boolean}[]>('domains') || [];
    
    // 查找匹配的域名
    for (const { domain, provider, enabled } of domains) {
      if (enabled !== false && url.includes(domain)) {
        // 将配置中的 provider 名称标准化为内部使用的 provider 类型
        return this.normalizeProvider(provider);
      }
    }
    
    return 'unknown';
  }

  /**
   * 标准化提供商名称
   */
  private normalizeProvider(provider: string): string {
    const providerLower = provider.toLowerCase();
    
    // 已知的提供商直接映射
    const providerMap: Record<string, string> = {
      'openai': 'openai',
      'anthropic': 'anthropic',
      'groq': 'openai',
      'openrouter': 'openai',
      'kimi': 'openai',
      'deepseek': 'openai',
      'volces': 'openai',
    };
    
    return providerMap[providerLower] || 'openai';
  }

  private isLLMRequest(host: string): boolean {
    // 从配置中读取域名列表
    const domains = this.options.configManager.get<{domain: string; provider: string; enabled: boolean}[]>('domains') || [];
    
    // 只匹配启用的域名
    return domains
      .filter(d => d.enabled !== false)
      .some(d => host.includes(d.domain));
  }

  private async forwardWithCapture(
    req: http.IncomingMessage,
    res: http.ServerResponse,
    target: string,
    llmRequest?: LLMRequest
  ): Promise<void> {
    const startTime = llmRequest?.timestamp || Date.now();
    
    this.proxy.once('proxyRes', async (proxyRes) => {
      try {
        // 检测是否为流式响应
        const contentType = proxyRes.headers['content-type'] || '';
        const isStreaming = contentType.includes('text/event-stream') || 
                           contentType.includes('stream');
        
        if (isStreaming) {
          console.log('[Proxy] Detected streaming response, using optimized forwarding');
        }
        
        const responseInfo = await captureResponse(proxyRes, {
          requestId: llmRequest?.id || 'unknown',
          startTime,
          isStreaming
        });
        
        this.emit('llm-response:captured', responseInfo);
      } catch (error) {
        console.error('[Proxy] Error capturing response:', error);
      }
    });
    
    this.proxy.web(req, res, { target: `http://${target}` }, (err) => {
      if (err) {
        console.error('[Proxy] Forward error:', err);
      }
    });
  }

  private sendError(res: http.ServerResponse, statusCode: number, message: string): void {
    res.writeHead(statusCode, { 'Content-Type': 'text/plain' });
    res.end(message);
  }

  public start(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.httpServer.listen(this.options.port, () => {
        this.isRunning = true;
        console.log(`[Proxy] Server started on port ${this.options.port}`);
        resolve();
      });

      this.httpServer.on('error', reject);
    });
  }

  public stop(): Promise<void> {
    return new Promise((resolve) => {
      this.httpServer.close(() => {
        this.proxy.close();
        this.isRunning = false;
        console.log('[Proxy] Server stopped');
        resolve();
      });
    });
  }
}
