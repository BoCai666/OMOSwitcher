import { randomUUID } from 'crypto';
import { LLMRequest, LLMResponse, LLMMetrics } from '../types.js';
import { memoryStore } from '../storage/memory-store.js';
import { captureStreamResponse, captureStreamResponseAsync } from './stream-handler.js';
import { calculateCost } from '../parsers/cost-calculator.js';

export async function handleResponse(
  request: Request,
  response: Response
): Promise<void> {
  const requestId = (request as any).__monitorId;
  if (!requestId) {
    console.warn('[Monitor] Response received but no requestId found');
    return;
  }

  const startTime = (request as any).__startTime || Date.now();
  const duration = Date.now() - startTime;

  // 从存储中获取对应的请求信息
  const llmRequest = memoryStore.getRequestById(requestId);
  if (!llmRequest) {
    console.warn(`[Monitor] Request ${requestId} not found in store`);
  }

  // 处理流式响应 - 优化策略：流式响应不阻塞，后台异步捕获
  let bodyText: string;
  let isStreaming = false;
  
  try {
    const contentType = response.headers.get('content-type') || '';
    isStreaming = contentType.includes('text/event-stream') || contentType.includes('stream');
    
    if (isStreaming) {
      // 流式响应：立即返回基础响应，后台异步捕获内容
      console.log(`[Monitor] Streaming response detected for ${requestId}`);
      bodyText = await captureStreamResponse(response, { streamCollect: false });
      
      // 使用克隆的响应进行后台捕获（不阻塞主流程）
      try {
        const clonedResponse = response.clone();
        captureStreamResponseAsync(clonedResponse, (result) => {
          console.log(`[Monitor] Stream capture completed for ${requestId}: ${result.chunksProcessed} chunks, ~${result.totalTokens} tokens`);
          // 更新响应内容和 metrics
          updateResponseContent(requestId, result.content, result.totalTokens);
        });
      } catch (cloneErr) {
        console.warn(`[Monitor] Failed to clone response: ${cloneErr}`);
      }
    } else {
      // 普通响应：正常克隆并读取
      bodyText = await response.clone().text();
    }
  } catch (err) {
    console.error(`[Monitor] Failed to capture response: ${err}`);
    bodyText = '{}';
  }

  let parsedBody: any = {};
  try {
    if (bodyText.trim()) {
      parsedBody = JSON.parse(bodyText);
    }
  } catch (err) {
    // 非JSON响应（如纯文本流）
    parsedBody = { raw: bodyText };
  }

  const llmResponse: LLMResponse = {
    id: randomUUID(),
    requestId,
    timestamp: Date.now(),
    statusCode: response.status,
    headers: Object.fromEntries(response.headers.entries()),
    body: parsedBody,
    parsedBody: extractResponseFields(parsedBody),
    duration,
  };

  memoryStore.saveResponse(llmResponse);

  // 计算并保存指标（仅非流式响应立即计算）
  if (llmRequest && !isStreaming) {
    const metrics = calculateMetrics(llmRequest, llmResponse);
    memoryStore.saveMetrics(metrics);
  }
}

function extractResponseFields(body: any): any {
  // 如果 body 是数组，合并所有事件的 content
  if (Array.isArray(body)) {
    let combinedContent = '';
    let lastUsage = null;
    
    for (const event of body) {
      if (event.choices?.[0]?.message?.content) {
        combinedContent += event.choices[0].message.content;
      } else if (event.choices?.[0]?.text) {
        combinedContent += event.choices[0].text;
      } else if (event.choices?.[0]?.delta?.content) {
        combinedContent += event.choices[0].delta.content;
      } else if (Array.isArray(event.content)) {
        combinedContent += event.content.map((block: any) => block.text || '').join('');
      } else if (event.content?.text) {
        combinedContent += event.content.text;
      } else if (typeof event.content === 'string') {
        combinedContent += event.content;
      } else if (event.delta?.text) {
        combinedContent += event.delta.text;
      }
      
      if (event.usage) {
        lastUsage = event.usage;
      }
    }
    
    const normalizedUsage = lastUsage ? {
      prompt_tokens: lastUsage.prompt_tokens ?? lastUsage.promptTokens ?? lastUsage.input_tokens ?? 0,
      completion_tokens: lastUsage.completion_tokens ?? lastUsage.completionTokens ?? lastUsage.output_tokens ?? 0,
      total_tokens: lastUsage.total_tokens ?? lastUsage.totalTokens ?? 0
    } : undefined;
    
    return {
      content: combinedContent,
      choices: body[body.length - 1]?.choices,
      usage: normalizedUsage,
      rawUsage: lastUsage,
    };
  }
  
  // 普通对象格式
  const usage = body.usage || body.usage_info || body.token_usage || null;
  
  const normalizedUsage = usage ? {
    prompt_tokens: usage.prompt_tokens ?? usage.promptTokens ?? usage.input_tokens ?? 0,
    completion_tokens: usage.completion_tokens ?? usage.completionTokens ?? usage.output_tokens ?? 0,
    total_tokens: usage.total_tokens ?? usage.totalTokens ?? 0
  } : undefined;
  
  let content = '';
  if (body.choices?.[0]?.message?.content) {
    content = body.choices[0].message.content;
  } else if (body.choices?.[0]?.text) {
    content = body.choices[0].text;
  } else if (body.choices?.[0]?.delta?.content) {
    content = body.choices[0].delta.content;
  } else if (Array.isArray(body.content)) {
    content = body.content.map((block: any) => block.text || '').join('');
  } else if (body.content?.text) {
    content = body.content.text;
  } else if (typeof body.content === 'string') {
    content = body.content;
  } else if (body.delta?.text) {
    content = body.delta.text;
  }
  
  return {
    content,
    choices: body.choices,
    usage: normalizedUsage,
    rawUsage: usage,
  };
}

/**
 * 更新已存储响应的内容（后台异步捕获完成后调用）
 */
function updateResponseContent(requestId: string, content: string, tokens: number): void {
  try {
    console.log(`[Monitor] Updating content for ${requestId}, length: ${content.length}`);
    
    const existingResponse = memoryStore.getResponseByRequestId?.(requestId);
    if (!existingResponse) {
      console.warn(`[Monitor] Response not found: ${requestId}`);
      return;
    }
    
    // 更新响应内容
    existingResponse.parsedBody = {
      ...existingResponse.parsedBody,
      content,
      streaming: false,
      completed: true
    };
    
    // 获取请求信息用于计算 metrics
    const request = memoryStore.getRequestById?.(requestId);
    if (!request) {
      console.warn(`[Monitor] Request not found: ${requestId}`);
      return;
    }
    
    // 从请求中估算 prompt tokens
    const messages = request.parsedBody?.messages || [];
    let promptTokens = 0;
    if (messages.length > 0) {
      const promptText = messages.map((m: any) => m.content || '').join(' ');
      promptTokens = Math.ceil(promptText.length / 4);
    }
    
    // completion tokens 从捕获的内容估算
    const completionTokens = tokens || Math.ceil(content.length / 4);
    const totalTokens = promptTokens + completionTokens;
    
    console.log(`[Monitor] Calculating metrics for ${requestId}: prompt=${promptTokens}, completion=${completionTokens}, total=${totalTokens}`);
    
    // 保存 metrics
    if (totalTokens > 0) {
      const metrics: LLMMetrics = {
        id: randomUUID(),
        requestId: requestId,
        model: request.model,
        provider: request.provider,
        promptTokens,
        completionTokens,
        totalTokens,
        estimatedCost: calculateCost(request.model, promptTokens, completionTokens),
        duration: existingResponse.duration,
        timestamp: Date.now(),
      };
      memoryStore.saveMetrics(metrics);
      console.log(`[Monitor] Saved metrics for ${requestId}: ${totalTokens} tokens`);
    }
  } catch (err) {
    console.warn(`[Monitor] Failed to update content: ${err}`);
  }
}

function calculateMetrics(
  request: LLMRequest,
  response: LLMResponse
): LLMMetrics {
  const usage = response.parsedBody?.usage;
  const content = response.parsedBody?.content || '';
  
  // 优先使用 API 返回的 usage 数据
  let promptTokens = usage?.prompt_tokens || 0;
  let completionTokens = usage?.completion_tokens || 0;
  let totalTokens = usage?.total_tokens || 0;
  
  // 如果没有 usage 数据但有响应内容，进行估算
  if (totalTokens === 0 && content.length > 0) {
    completionTokens = Math.ceil(content.length / 4);
    
    const messages = request.parsedBody?.messages || [];
    if (messages.length > 0) {
      const promptText = messages.map((m: any) => m.content || '').join(' ');
      promptTokens = Math.ceil(promptText.length / 4);
    }
    
    totalTokens = promptTokens + completionTokens;
  }

  return {
    id: randomUUID(),
    requestId: request.id,
    model: request.model,
    provider: request.provider,
    promptTokens,
    completionTokens,
    totalTokens,
    estimatedCost: calculateCost(request.model, promptTokens, completionTokens),
    duration: response.duration,
    timestamp: Date.now(),
  };
}
