import { randomUUID } from 'crypto';
import { LLMRequest } from '../types.js';
import { memoryStore } from '../storage/memory-store.js';

const MAX_BODY_SIZE = 10 * 1024 * 1024; // 10MB

/**
 * 处理传入的 LLM 请求
 * 捕获请求数据并异步存储到内存存储中
 * 此函数是非阻塞的，不会干扰原始请求流程
 */
export async function handleRequest(request: Request): Promise<void> {
  const requestId = randomUUID();
  (request as any).__monitorId = requestId;
  (request as any).__startTime = Date.now();
  
  // 检查body大小
  const contentLength = parseInt(request.headers.get('content-length') || '0');
  if (contentLength > MAX_BODY_SIZE) {
    console.warn(`[Monitor] Request body too large: ${contentLength} bytes`);
    return;
  }
  
  let body = '{}';
  let parsedBody: any = {};
  
  try {
    // 克隆请求以读取body
    const clonedRequest = request.clone();
    body = await clonedRequest.text();
    
    if (body.trim()) {
      parsedBody = JSON.parse(body);
    }
  } catch (err) {
    // 非JSON body或读取失败
    console.warn(`[Monitor] Failed to parse request body: ${err}`);
    body = '{}';
  }
  
  const llmRequest: LLMRequest = {
    id: requestId,
    timestamp: Date.now(),
    provider: detectProvider(request.url),
    model: parsedBody.model || 'unknown',
    method: request.method,
    url: request.url,
    headers: Object.fromEntries(request.headers.entries()),
    body: parsedBody,
    parsedBody: extractKeyFields(parsedBody),
  };
  
  // 异步存储，不阻塞请求
  memoryStore.saveRequest(llmRequest);
}

/**
 * 检测 LLM 提供商
 * 根据请求 URL 判断是 OpenAI、Anthropic、Kimi 还是未知提供商
 */
function detectProvider(url: string): 'openai' | 'anthropic' | 'kimi' | 'unknown' {
  if (url.includes('api.openai.com')) return 'openai';
  if (url.includes('api.anthropic.com')) return 'anthropic';
  if (url.includes('api.kimi.com')) return 'kimi';
  return 'unknown';
}

/**
 * 从请求体中提取关键字段
 * 提取 messages、prompt、temperature、max_tokens 等重要参数
 */
function extractKeyFields(body: any): any {
  return {
    messages: body.messages,
    prompt: body.prompt,
    temperature: body.temperature,
    max_tokens: body.max_tokens,
  };
}
