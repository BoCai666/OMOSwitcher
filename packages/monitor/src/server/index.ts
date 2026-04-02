import express from 'express';
import cors from 'cors';
import http from 'http';
import routes, { broadcastNewRequest, broadcastResponse, broadcastMetrics, getClients, setClients } from './routes.js';
import { config } from '../config.js';

// 导出 routes 中的功能，供其他模块使用
export { broadcastNewRequest, broadcastResponse, broadcastMetrics, getClients, setClients };

/**
 * 启动 Web API 服务器
 * @returns Promise，在服务器开始监听后 resolve，启动失败时 reject
 */
export function startServer(): Promise<void> {
  return new Promise((resolve, reject) => {
    const app = express();
    
    // 中间件
    app.use(cors());
    app.use(express.json());
    
    // API 路由
    app.use('/api', routes);
    
    // 启动服务器
    const port = config.port || 3030;
    
    // 创建 HTTP 服务器
    const server = http.createServer(app);
    
    // 错误处理（使用 once 避免重复绑定，直接 reject 传播错误）
    server.once('error', (err: NodeJS.ErrnoException) => {
      if (err.code === 'EADDRINUSE') {
        console.error(`[Server] 端口 ${port} 已被占用`);
        console.error(`[Server] 请检查是否有其他 Monitor 进程在运行`);
        reject(new Error(`端口 ${port} 已被占用，请检查是否有其他 Monitor 进程在运行`));
      } else {
        console.error('[Server] 服务器启动失败:', err);
        reject(new Error(`服务器启动失败: ${err.message} (code: ${err.code})`));
      }
    });
    
    // 启动监听
    server.listen({
      port,
      host: '0.0.0.0',
      exclusive: false,
    }, () => {
      console.log(`[Server] API server running on http://localhost:${port}`);
      resolve();
    });
  });
}
