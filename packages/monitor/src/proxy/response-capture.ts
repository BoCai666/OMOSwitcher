/**
 * HTTP 响应捕获模块
 *
 * 功能：
 * 1. 捕获完整的 HTTP 响应信息
 * 2. 处理流式响应(SSE)
 * 3. 提取 Token 使用量
 * 4. 计算请求耗时
 */

import http from 'http';
import { LLMResponse } from '../types.js';

export interface CaptureOptions {
  requestId: string;
  startTime: number;
  isStreaming?: boolean;
}

/**
 * 主函数：捕获完整的 HTTP 响应
 */
export async function captureResponse(
  proxyRes: http.IncomingMessage,
  options: CaptureOptions
): Promise<LLMResponse> {
  const id = generateResponseId();
  const timestamp = Date.now();
  const duration = timestamp - options.startTime;
  const statusCode = proxyRes.statusCode || 500;
  const headers = convertHeaders(proxyRes.headers);

  // 检查是否是流式响应
  const isStreaming = options.isStreaming || isStreamingResponse(proxyRes);

  if (isStreaming) {
    return captureStreamingResponse(proxyRes, { ...options, isStreaming: true });
  }

  // 处理普通响应
  const body = await readFullBody(proxyRes);
  const parsedBody = parseResponseBody(body);

  return {
    id,
    requestId: options.requestId,
    timestamp,
    statusCode,
    headers,
    body,
    parsedBody,
    duration
  };
}

/**
 * 流式响应捕获配置
 */
const MAX_STREAM_CHUNKS = 10000; // 最大累积的 chunks 数量，防止内存无限增长

/**
 * 处理流式响应(SSE)
 * 注意：此函数会立即返回一个基础响应对象，在后台异步完成完整捕获
 */
async function captureStreamingResponse(
  proxyRes: http.IncomingMessage,
  options: CaptureOptions
): Promise<LLMResponse> {
  const id = generateResponseId();
  const timestamp = Date.now();
  const statusCode = proxyRes.statusCode || 200;
  const headers = convertHeaders(proxyRes.headers);

  // 立即返回基础响应，让流式传输不被阻塞
  const baseResponse: LLMResponse = {
    id,
    requestId: options.requestId,
    timestamp,
    statusCode,
    headers,
    body: '',  // 将在后台更新
    parsedBody: { content: '', streaming: true },
    duration: 0  // 将在完成时更新
  };

  // 后台异步捕获完整响应
  captureStreamingResponseAsync(proxyRes, options, id, timestamp);

  return baseResponse;
}

/**
 * 后台异步捕获流式响应
 * 不阻塞主响应流程
 */
function captureStreamingResponseAsync(
  proxyRes: http.IncomingMessage,
  options: CaptureOptions,
  responseId: string,
  startTimestamp: number
): void {
  const chunks: string[] = [];
  const contents: string[] = [];
  let chunkCount = 0;

  proxyRes.on('data', (chunk) => {
    const text = chunk.toString('utf8');
    
    // 限制内存使用：超过最大 chunks 时，只保留最近的 80%
    if (chunkCount >= MAX_STREAM_CHUNKS) {
      const retainCount = Math.floor(MAX_STREAM_CHUNKS * 0.8);
      chunks.splice(0, chunks.length - retainCount);
      console.warn(`[ResponseCapture] Stream chunks exceeded limit, trimming buffer for response ${responseId}`);
    }
    
    chunks.push(text);
    chunkCount++;

    // 实时解析 SSE 数据
    const lines = text.split('\n');
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6).trim();
        if (data === '[DONE]') {
          continue;
        }
        try {
          const json = JSON.parse(data);
          // 提取内容
          if (json.choices && json.choices[0]?.delta?.content) {
            contents.push(json.choices[0].delta.content);
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
  });

  proxyRes.on('end', () => {
    const duration = Date.now() - options.startTime;
    const fullContent = contents.join('');
    const promptTokens = 0;
    const completionTokens = estimateTokens(fullContent);

    console.log(`[ResponseCapture] Streaming response completed: ${responseId}, ` +
                `${chunkCount} chunks, ${completionTokens} tokens estimated, ${duration}ms`);
    
    // 这里可以将完整数据保存到存储中（如果实现了存储更新接口）
    // 目前只是记录日志，实际数据已通过流式传输给客户端
  });

  proxyRes.on('error', (err) => {
    console.error(`[ResponseCapture] Error capturing streaming response ${responseId}:`, err);
  });
}

/**
 * 判断是否是流式响应
 */
function isStreamingResponse(res: http.IncomingMessage): boolean {
  const contentType = res.headers['content-type'] || '';
  return contentType.includes('text/event-stream') || contentType.includes('stream');
}

/**
 * 解析响应体
 */
function parseResponseBody(body: string): any {
  if (!body || body.trim().length === 0) {
    return undefined;
  }

  try {
    const parsed = JSON.parse(body);

    // 提取关键信息
    const result: any = {};

    // 提取内容
    if (parsed.choices && parsed.choices.length > 0) {
      const choice = parsed.choices[0];
      if (choice.message?.content) {
        result.content = choice.message.content;
      } else if (choice.text) {
        result.content = choice.text;
      } else if (choice.delta?.content) {
        result.content = choice.delta.content;
      }
    }

    // 提取 usage - 支持多种字段命名格式
    if (parsed.usage) {
      result.usage = {
        prompt_tokens: parsed.usage.prompt_tokens || parsed.usage.input_tokens || 0,
        completion_tokens: parsed.usage.completion_tokens || parsed.usage.output_tokens || 0,
        total_tokens: parsed.usage.total_tokens || parsed.usage.total || 0
      };
      // 如果 total_tokens 为 0 但有 input/output，计算 total
      if (result.usage.total_tokens === 0 && (result.usage.prompt_tokens > 0 || result.usage.completion_tokens > 0)) {
        result.usage.total_tokens = result.usage.prompt_tokens + result.usage.completion_tokens;
      }
    }
    
    // 一些提供商（如火山引擎）可能在根级别返回 token 使用情况
    if (!result.usage) {
      if (parsed.input_tokens !== undefined || parsed.output_tokens !== undefined || parsed.total_tokens !== undefined) {
        result.usage = {
          prompt_tokens: parsed.input_tokens || 0,
          completion_tokens: parsed.output_tokens || 0,
          total_tokens: parsed.total_tokens || (parsed.input_tokens || 0) + (parsed.output_tokens || 0)
        };
      }
    }

    // 保留原始 choices
    if (parsed.choices) {
      result.choices = parsed.choices;
    }

    // 保留其他字段
    for (const key of Object.keys(parsed)) {
      if (!result[key] && key !== 'choices' && key !== 'usage') {
        result[key] = parsed[key];
      }
    }

    return Object.keys(result).length > 0 ? result : undefined;
  } catch {
    return undefined;
  }
}

/**
 * 估算 Token 数量（约4字符=1 Token）
 */
function estimateTokens(text: string): number {
  if (!text) return 0;
  return Math.ceil(text.length / 4);
}

/**
 * 读取完整的响应体
 */
function readFullBody(res: http.IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];

    res.on('data', (chunk) => {
      chunks.push(chunk);
    });

    res.on('end', () => {
      const body = Buffer.concat(chunks).toString('utf8');
      resolve(body);
    });

    res.on('error', reject);
  });
}

/**
 * 转换 headers 格式
 */
function convertHeaders(headers: http.IncomingHttpHeaders): Record<string, string> {
  const result: Record<string, string> = {};

  for (const [key, value] of Object.entries(headers)) {
    if (value === undefined) continue;
    if (Array.isArray(value)) {
      result[key] = value.join(', ');
    } else {
      result[key] = value;
    }
  }

  return result;
}

/**
 * 生成唯一响应 ID
 */
function generateResponseId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `res-${timestamp}-${random}`;
}
