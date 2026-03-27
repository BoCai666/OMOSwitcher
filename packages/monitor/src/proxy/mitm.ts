/**
 * MITM (Man-In-The-Middle) 处理模块
 * 
 * 功能：
 * 1. 处理 HTTPS CONNECT 请求
 * 2. 建立 MITM 连接，拦截和解密 HTTPS 流量
 * 3. 转发并捕获流量
 * 
 * 工作原理：
 * Client <--TLS(伪造证书)--> Proxy <--TLS(真实证书)--> Server
 */

import net from 'net';
import tls from 'tls';
import http from 'http';
import https from 'https';
import { EventEmitter } from 'events';
import type { CertificateManager } from './server.js';

// 流量捕获钩子接口
export interface TrafficCaptureHooks {
  onRequest?: (data: { host: string; port: number; data: Buffer }) => void;
  onResponse?: (data: { host: string; port: number; data: Buffer }) => void;
  onConnect?: (data: { host: string; port: number }) => void;
  onDisconnect?: (data: { host: string; port: number }) => void;
  onError?: (data: { host: string; port: number; error: Error }) => void;
}

// MITM 处理选项
export interface MITMHandlerOptions {
  certManager: CertificateManager;
  hooks?: TrafficCaptureHooks;
  enableDebug?: boolean;
}

// 活跃连接信息
interface ActiveConnection {
  clientSocket: net.Socket;
  serverSocket?: tls.TLSSocket;
  tlsSocket?: tls.TLSSocket;
  host: string;
  port: number;
  connectTime: number;
}


/**
 * MITM 处理器类
 * 负责处理 HTTPS 连接的中间人攻击，以拦截和解密流量
 */
export class MITMHandler extends EventEmitter {
  private certManager: CertificateManager;
  private hooks?: TrafficCaptureHooks;
  private enableDebug: boolean;
  private activeConnections: Map<string, ActiveConnection> = new Map();
  private connectionCounter: number = 0;

  constructor(options: MITMHandlerOptions) {
    super();
    this.certManager = options.certManager;
    this.hooks = options.hooks;
    this.enableDebug = options.enableDebug ?? false;
  }

  /**
   * 处理 CONNECT 请求
   * 这是 MITM 处理的入口点
   */
  public async handleConnect(
    req: http.IncomingMessage,
    clientSocket: net.Socket,
    head: Buffer
  ): Promise<void> {
    const targetHost = req.url;

    if (!targetHost) {
      this.logError('CONNECT request missing target host');
      clientSocket.end('HTTP/1.1 400 Bad Request\r\n\r\n');
      return;
    }

    const [hostname, portStr] = targetHost.split(':');
    const port = parseInt(portStr) || 443;

    this.logDebug(`[MITM] CONNECT request to ${targetHost}`);

    this.hooks?.onConnect?.({ host: hostname, port });
    this.emit('connect', { host: hostname, port });

    try {
      await this.createMITMConnection(hostname, port, clientSocket, head);
    } catch (error) {
      this.logError(`[MITM] Failed to create MITM connection: ${error}`);
      this.hooks?.onError?.({ host: hostname, port, error: error as Error });
      this.emit('error', { host: hostname, port, error });
      
      this.establishTunnel(hostname, port, clientSocket, head);
    }
  }

  /**
   * 创建 MITM 连接
   * 使用 tls.TLSSocket 直接包装 clientSocket，解密客户端 TLS 数据
   */
  private async createMITMConnection(
    hostname: string,
    port: number,
    clientSocket: net.Socket,
    head: Buffer
  ): Promise<void> {
    const connectionId = this.generateConnectionId();
    
    const certs = this.certManager.getCertificateForDomain(hostname);
    
    const certChain = certs.ca ? `${certs.cert}\n${certs.ca}` : certs.cert;
    
    this.logDebug(`Creating MITM for ${hostname}, cert chain length: ${certChain.length}`);

    const tlsServer = new tls.Server({
      key: certs.key,
      cert: certChain,
    });

    const handshakeTimeout = setTimeout(() => {
      this.logError(`TLS handshake timeout for ${hostname}:${port}`);
      clientSocket.destroy();
      tlsServer.close();
      this.activeConnections.delete(connectionId);
    }, 10000);

    tlsServer.on('tlsClientError', (err: any, rawSocket) => {
      clearTimeout(handshakeTimeout);
      this.logError(`TLS client error for ${hostname}: ${err.message} (code: ${err.code})`);
      this.hooks?.onError?.({ host: hostname, port, error: err });
      rawSocket.destroy();
      tlsServer.close();
      this.activeConnections.delete(connectionId);
    });

    tlsServer.on('secureConnection', (tlsSocket: tls.TLSSocket) => {
      clearTimeout(handshakeTimeout);
      this.logDebug(`TLS handshake complete for ${hostname}, cipher: ${tlsSocket.getCipher()?.name}`);
      
      tlsServer.close();

      const connection: ActiveConnection = {
        clientSocket,
        tlsSocket,
        host: hostname,
        port,
        connectTime: Date.now(),
      };
      this.activeConnections.set(connectionId, connection);

      const httpServer = http.createServer((clientReq, clientRes) => {
        const requestData: any = {
          id: `req-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          timestamp: Date.now(),
          method: clientReq.method,
          url: `https://${hostname}${clientReq.url}`,
          headers: clientReq.headers,
          host: hostname,
          port,
        };

        const reqChunks: Buffer[] = [];
        let bodyReceived = false;
        
        // 立即设置转发选项并创建请求（不等待完整body）
        const options = {
          hostname,
          port,
          path: clientReq.url,
          method: clientReq.method,
          headers: {
            ...clientReq.headers,
            host: hostname,
          },
          // 跳过 SSL 证书验证，解决企业网络代理自签名证书问题
          rejectUnauthorized: false,
        };

        const proxyReq = https.request(options, (proxyRes) => {
          const responseData: any = {
            id: `res-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            requestId: requestData.id,
            timestamp: Date.now(),
            statusCode: proxyRes.statusCode || 200,
            headers: proxyRes.headers,
            host: hostname,
            port,
          };

          clientRes.writeHead(proxyRes.statusCode || 200, proxyRes.headers);

          const resChunks: Buffer[] = [];
          proxyRes.on('data', (chunk) => {
            resChunks.push(chunk);
            clientRes.write(chunk);  // 立即转发响应数据
          });
          
          proxyRes.on('end', () => {
            responseData.body = Buffer.concat(resChunks);
            responseData.duration = Date.now() - requestData.timestamp;
            
            this.emit('http:response', responseData);
            clientRes.end();
          });
        });

        proxyReq.on('error', (err) => {
          this.logError(`Proxy request error: ${err.message}`);
          clientRes.writeHead(502);
          clientRes.end('Bad Gateway');
        });

        // 流式转发请求体：收到数据立即转发，同时缓存用于监控
        clientReq.on('data', (chunk) => {
          reqChunks.push(chunk);
          proxyReq.write(chunk);  // 立即转发，不等待end事件
        });
        
        clientReq.on('end', () => {
          bodyReceived = true;
          requestData.body = Buffer.concat(reqChunks);
          this.emit('http:request', requestData);
          proxyReq.end();
        });
        
        clientReq.on('error', (err) => {
          this.logError(`Client request error: ${err.message}`);
          proxyReq.destroy();
        });
      });

      tlsSocket.on('error', (err) => {
        this.logError(`TLS socket error for ${hostname}: ${err.message}`);
        httpServer.close();
        this.cleanupConnection(connectionId);
      });

      tlsSocket.on('close', () => {
        httpServer.close();
        this.cleanupConnection(connectionId);
      });

      httpServer.emit('connection', tlsSocket);
    });

    clientSocket.write('HTTP/1.1 200 Connection Established\r\n\r\n');
    tlsServer.emit('connection', clientSocket);

    this.logDebug(`Socket fed to TLS server for ${hostname}, waiting for handshake...`);

    clientSocket.on('error', (err) => {
      clearTimeout(handshakeTimeout);
      this.logError(`Client socket error for ${hostname}: ${err.message}`);
      tlsServer.close();
      this.cleanupConnection(connectionId);
    });
  }

  /**
   * 建立普通隧道（非 MITM 模式）
   * 直接转发数据，不解密
   */
  private establishTunnel(
    hostname: string,
    port: number,
    clientSocket: net.Socket,
    head: Buffer
  ): void {
    this.logDebug(`[MITM] Establishing direct tunnel to ${hostname}:${port}`);

    const serverSocket = net.connect(port, hostname, () => {
      clientSocket.write('HTTP/1.1 200 Connection Established\r\n\r\n');

      if (head && head.length > 0) {
        serverSocket.write(head);
      }

      serverSocket.pipe(clientSocket);
      clientSocket.pipe(serverSocket);
    });

    serverSocket.on('error', (err) => {
      this.logError(`[MITM] Tunnel error: ${err}`);
      clientSocket.end();
    });

    clientSocket.on('error', (err) => {
      this.logError(`[MITM] Client socket error: ${err}`);
      serverSocket.end();
    });
  }

  /**
   * 生成连接 ID
   */
  private generateConnectionId(): string {
    return `conn_${++this.connectionCounter}_${Date.now()}`;
  }

  /**
   * 清理连接资源
   */
  private cleanupConnection(connectionId: string): void {
    const connection = this.activeConnections.get(connectionId);
    if (!connection) return;

    this.logDebug(`[MITM] Cleaning up connection ${connectionId}`);

    if (connection.tlsSocket && !connection.tlsSocket.destroyed) {
      connection.tlsSocket.end();
    }

    if (connection.serverSocket && !connection.serverSocket.destroyed) {
      connection.serverSocket.end();
    }

    if (connection.clientSocket && !connection.clientSocket.destroyed) {
      connection.clientSocket.end();
    }

    this.activeConnections.delete(connectionId);

    this.logDebug(`[MITM] Connection ${connectionId} cleaned up`);
  }

  /**
   * 关闭所有连接
   */
  public closeAll(): void {
    this.logDebug('[MITM] Closing all connections');

    for (const [connectionId] of this.activeConnections) {
      this.cleanupConnection(connectionId);
    }

    this.activeConnections.clear();
    this.logDebug('[MITM] All connections closed');
  }

  /**
   * 获取活跃连接数
   */
  public getActiveConnectionCount(): number {
    return this.activeConnections.size;
  }

  /**
   * 获取所有活跃连接信息
   */
  public getActiveConnections(): Array<{
    id: string;
    host: string;
    port: number;
    connectTime: number;
    duration: number;
  }> {
    const now = Date.now();
    return Array.from(this.activeConnections.entries()).map(([id, conn]) => ({
      id,
      host: conn.host,
      port: conn.port,
      connectTime: conn.connectTime,
      duration: now - conn.connectTime,
    }));
  }

  /**
   * 日志输出（调试模式）
   */
  private logDebug(message: string): void {
    if (this.enableDebug) {
      console.log(`[MITM] ${message}`);
    }
  }

  /**
   * 错误日志输出
   */
  private logError(message: string): void {
    console.error(`[MITM] ${message}`);
  }
}

// 导出默认实例创建函数
export function createMITMHandler(options: MITMHandlerOptions): MITMHandler {
  return new MITMHandler(options);
}
