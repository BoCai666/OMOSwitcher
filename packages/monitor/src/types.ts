export interface LLMRequest {
  id: string;
  timestamp: number;
  provider: 'openai' | 'anthropic' | 'kimi' | 'unknown';
  model: string;
  method: string;
  url: string;
  domain?: string;
  headers: Record<string, string>;
  body: any; // 完整的请求体
  parsedBody?: {
    messages?: Array<{ role: string; content: string | any[] }>;
    prompt?: string;
    temperature?: number;
    max_tokens?: number;
    // 思考相关字段
    thinking?: { type: string; budget_tokens?: number } | any;
    thinking_budget?: number;
    reasoning_effort?: 'low' | 'medium' | 'high';
    reasoning?: any;
    extended_thinking?: any;
    [key: string]: any;
  };
}

export interface LLMResponse {
  id: string;
  requestId: string;
  timestamp: number;
  statusCode: number;
  headers: Record<string, string>;
  body: any; // 完整的响应体
  parsedBody?: {
    content?: string;
    thinking?: string; // 思考内容（DeepSeek R1, Anthropic 等）
    choices?: Array<any>;
    usage?: {
      prompt_tokens: number;
      completion_tokens: number;
      total_tokens: number;
    };
    [key: string]: any;
  };
  duration: number; // 请求耗时(ms)
}

export interface LLMMetrics {
  id: string;
  requestId: string;
  model: string;
  provider: string;
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  estimatedCost: number; // USD
  duration: number;
  timestamp: number;
}

export interface MetricsStats {
  count: number;
  totalTokens: number;
  totalCost: number;
  modelStats: Record<string, { count: number; tokens: number; cost: number }>;
}

export interface MCPCall {
  id: string;
  requestId: string;
  jsonrpcVersion?: string;
  rpcId?: string;
  toolName: string;
  toolTitle?: string;
  toolDescription?: string;
  arguments?: Record<string, unknown>;
  resultContent?: any;
  resultIsError: boolean;
  errorMessage?: string;
  executionDuration?: number;
  transportType?: 'stdio' | 'sse' | 'http';
  serverName?: string;
  traceId?: string;
  timestamp: number;
}

export interface DailyRecord {
  date: string; // YYYY-MM-DD format
  requestCount: number;
  totalTokens: number;
  totalCost: number;
  models: string[];
  modelStats: Record<string, {
    count: number;
    tokens: number;
    cost: number;
  }>;
}

// 域名配置接口
export interface DomainConfig {
  pattern: string; // 域名匹配模式
  matchType: 'exact' | 'glob' | 'regex'; // 匹配类型
  provider: string; // 服务商名称
  isEnabled: boolean; // 是否启用
}

// 模型定价配置接口
export interface ModelPricingConfig {
  model: string; // 模型匹配模式（前缀匹配）
  input: number; // 输入价格（美元/1M tokens）
  output: number; // 输出价格（美元/1M tokens）
}

// 端口配置接口
export interface PortConfig {
  portType: 'http' | 'https'; // 端口类型
  portNumber: number; // 端口号
  isEnabled: boolean; // 是否启用
}

// 监控配置接口
export interface MonitorConfig {
  refreshInterval: number; // 刷新间隔（毫秒）
  maxRequestsInList: number; // 列表中最大请求数
  enableRealtime: boolean; // 是否启用实时监控
  enableNotifications: boolean; // 是否启用通知
}

// 星空特效配置接口
export interface NebulaConfig {
  theme: 'dark' | 'light' | 'auto'; // 主题
  starCount: number; // 星星数量
  connectionCount: number; // 连接线条数量
  enableParallax: boolean; // 是否启用视差效果
  enableShootingStars: boolean; // 是否启用流星效果
  fpsLimit: number; // FPS限制
}

// 总配置接口
export interface AppConfig {
  domains: DomainConfig[]; // 域名配置列表
  pricing: ModelPricingConfig[]; // 定价配置列表
  ports: PortConfig[]; // 端口配置列表
  monitor: MonitorConfig; // 监控配置
  nebula: NebulaConfig; // 星空特效配置
}
