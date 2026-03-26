import express from 'express';
import cors from 'cors';
import routes, { broadcastNewRequest, broadcastResponse, broadcastMetrics, getClients, setClients } from './routes.js';
import { memoryStore } from '../storage/memory-store.js';
import { config } from '../config.js';

// 导出 routes 中的功能，供其他模块使用
export { broadcastNewRequest, broadcastResponse, broadcastMetrics, getClients, setClients };

export function startServer(): void {
  const app = express();
  
  // 中间件
  app.use(cors());
  app.use(express.json());
  
  // API 路由
  app.use('/api', routes);
  
  // 静态文件
  app.use(express.static('dist/public'));
  
  // SPA 路由回退 - 处理所有未匹配的路由
  app.use((req, res) => {
    res.sendFile('index.html', { root: 'dist/public' });
  });
  
  // 启动服务器
  const port = config.port || 3000;
  app.listen(port, () => {
    console.log(`[Server] Web server running on http://localhost:${port}`);
  });
}
