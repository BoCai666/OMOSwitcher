import express from 'express';
import cors from 'cors';
import routes, { broadcastNewRequest, broadcastResponse, broadcastMetrics, getClients, setClients } from './routes.js';
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
  
  // 启动服务器
  const port = config.port || 3030;
  app.listen(port, () => {
    console.log(`[Server] API server running on http://localhost:${port}`);
  });
}
