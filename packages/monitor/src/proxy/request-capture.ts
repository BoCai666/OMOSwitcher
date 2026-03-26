/**
 * HTTP 请求捕获模块
 * 
 * 功能：
 * 1. 捕获完整的 HTTP 请求信息
 * 2. 解析请求体(JSON/表单/原始数据)
 * 3. 识别 LLM 提供商和模型
 * 4. 生成唯一请求 ID
 * 5. 清理敏感信息
 */

import http from 'http';
import { LLMRequest } from '../types.js';

// Provider 检测模式 - 可动态更新
let providerPatterns: { pattern: string; provider: string }[] = [
  { pattern: 'api.openai.com', provider: 'openai' },
  { pattern: 'api.anthropic.com', provider: 'anthropic' },
  { pattern: 'api.groq.com', provider: 'openai' },
  { pattern: 'openrouter.ai', provider: 'openai' },
  { pattern: 'api.kimi.com', provider: 'openai' },
  { pattern: 'api.deepseek.com', provider: 'openai' }
];

/**
 * 更新 provider 检测模式
 * @param patterns 新的模式列表
 */
export function updateProviderPatterns(patterns: { pattern: string; provider: string }[]): void {
  providerPatterns = patterns;
}

/**
 * 主函数：捕获完整的 HTTP 请求
 */
export async function captureRequest(req: http.IncomingMessage): Promise<LLMRequest> {
  const id = generateRequestId();
  const timestamp = Date.now();
  const url = req.url || '/';
  const fullUrl = `http://${req.headers.host}${url}`;
  const provider = detectProvider(fullUrl);
  const headers = sanitizeHeaders(req.headers);
  const body = await parseBody(req);
  const parsedBody = extractKeyFields(body);
  const model = parsedBody?.model || 'unknown';

  return {
    id,
    timestamp,
    provider: provider as 'openai' | 'anthropic' | 'unknown',
    model,
    method: req.method || 'GET',
    url: fullUrl,
    headers,
    body,
    parsedBody
  };
}

/**
 * 解析请求体
 * 支持 JSON、表单数据和原始数据
 */
async function parseBody(req: http.IncomingMessage): Promise<any> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    
    req.on('data', (chunk) => {
      chunks.push(chunk);
    });
    
    req.on('end', () => {
      const rawBody = Buffer.concat(chunks);
      
      if (rawBody.length === 0) {
        resolve(null);
        return;
      }
      
      const contentType = req.headers['content-type'] || '';
      
      try {
        if (contentType.includes('application/json')) {
          resolve(JSON.parse(rawBody.toString('utf8')));
        } else if (contentType.includes('application/x-www-form-urlencoded')) {
          const formData = new URLSearchParams(rawBody.toString('utf8'));
          const result: Record<string, string> = {};
          formData.forEach((value, key) => {
            result[key] = value;
          });
          resolve(result);
        } else {
          resolve(rawBody.toString('utf8'));
        }
      } catch (error) {
        resolve(rawBody.toString('utf8'));
      }
    });
    
    req.on('error', (error) => {
      reject(error);
    });
  });
}

/**
 * 从 URL 检测 LLM 提供商
 */
function detectProvider(url: string): string {
  for (const { pattern, provider } of providerPatterns) {
    if (url.includes(pattern)) {
      return provider;
    }
  }
  
  return 'unknown';
}

/**
 * 从请求体中提取关键字段
 */
function extractKeyFields(body: any): any {
  if (!body || typeof body !== 'object') {
    return undefined;
  }
  
  const result: any = {};
  
  const keyFields = [
    'model',
    'messages',
    'prompt',
    'temperature',
    'max_tokens',
    'max_completion_tokens',
    'top_p',
    'top_k',
    'frequency_penalty',
    'presence_penalty',
    'stop',
    'stream',
    'n',
    'system',
    'tools',
    'tool_choice'
  ];
  
  for (const field of keyFields) {
    if (body[field] !== undefined) {
      result[field] = body[field];
    }
  }
  
  return Object.keys(result).length > 0 ? result : undefined;
}

/**
 * 清理敏感信息，如 Authorization 头等
 */
function sanitizeHeaders(headers: http.IncomingHttpHeaders): Record<string, string> {
  const sanitized: Record<string, string> = {};
  
  for (const [key, value] of Object.entries(headers)) {
    if (value === undefined) continue;
    
    const lowerKey = key.toLowerCase();
    
    if (lowerKey === 'authorization' || lowerKey === 'proxy-authorization') {
      sanitized[key] = '[REDACTED]';
    } else if (lowerKey === 'cookie') {
      sanitized[key] = '[REDACTED]';
    } else if (Array.isArray(value)) {
      sanitized[key] = value.join(', ');
    } else {
      sanitized[key] = value;
    }
  }
  
  return sanitized;
}

/**
 * 生成唯一请求 ID
 */
function generateRequestId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `req-${timestamp}-${random}`;
}
