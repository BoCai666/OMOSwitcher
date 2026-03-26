/**
 * 流式响应处理模块
 * 
 * 性能优化说明：
 * 1. 流式响应不再完全缓冲，避免阻塞 token 级实时传输
 * 2. 后台异步捕获内容，不影响主响应流程
 * 3. 添加内存限制防止长时间会话导致内存溢出
 */

/**
 * 流式响应配置
 */
const MAX_STREAM_CHUNKS = 5000;  // 最大累积 chunks 数量
const MAX_STREAM_SIZE_MB = 10;   // 最大累积数据大小(MB)

/**
 * 流式响应捕获结果
 */
export interface StreamCaptureResult {
  content: string;           // 已捕获的内容（可能不完整）
  isComplete: boolean;       // 是否完成捕获
  chunksProcessed: number;   // 处理的 chunks 数量
  totalTokens: number;       // 预估的 token 数量
}

/**
 * 捕获 SSE 流式响应
 * 
 * 优化策略：
 * - 如果 streamCollect 为 true：后台异步收集完整内容（用于监控）
 * - 如果 streamCollect 为 false：立即返回，只解析已接收的数据
 * 
 * 注意：此函数不阻塞流式传输，数据会实时传递给调用方
 */
export async function captureStreamResponse(
  response: Response, 
  options: { streamCollect?: boolean; maxWaitMs?: number } = {}
): Promise<string> {
  const { streamCollect = false, maxWaitMs = 5000 } = options;
  
  const reader = response.body?.getReader();
  if (!reader) {
    return '{}';
  }

  // 如果不收集完整流，立即返回基础响应
  if (!streamCollect) {
    reader.releaseLock();
    return JSON.stringify({
      choices: [{
        message: { role: 'assistant', content: '' },
        delta: { content: '' }
      }],
      streaming: true,
      note: 'Streaming response - content captured asynchronously'
    });
  }

  const chunks: string[] = [];
  const decoder = new TextDecoder();
  let totalSize = 0;
  const maxSize = MAX_STREAM_SIZE_MB * 1024 * 1024;
  
  try {
    // 设置超时，避免无限等待
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => reject(new Error('Stream capture timeout')), maxWaitMs);
    });

    const readPromise = (async () => {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        
        const text = decoder.decode(value, { stream: true });
        
        // 内存限制检查
        totalSize += value.length;
        if (totalSize > maxSize) {
          console.warn(`[StreamHandler] Stream size exceeded ${MAX_STREAM_SIZE_MB}MB limit, truncating`);
          break;
        }
        
        if (chunks.length >= MAX_STREAM_CHUNKS) {
          // 保留最近的 80% 数据
          const retainCount = Math.floor(MAX_STREAM_CHUNKS * 0.8);
          chunks.splice(0, chunks.length - retainCount);
          console.warn(`[StreamHandler] Stream chunks exceeded limit, trimming buffer`);
        }
        
        chunks.push(text);
      }
    })();

    // 等待读取完成或超时
    await Promise.race([readPromise, timeoutPromise]);
    
  } catch (err) {
    if ((err as Error).message === 'Stream capture timeout') {
      console.warn('[StreamHandler] Stream capture timed out, returning partial content');
    } else {
      console.error(`[Monitor] Error reading stream: ${err}`);
    }
  } finally {
    reader.releaseLock();
  }

  // 合并所有 chunks 并解析 SSE 格式
  const fullText = chunks.join('');
  return parseSSEStream(fullText);
}

/**
 * 后台异步捕获流式响应（不阻塞主流程）
 * 
 * 使用场景：需要监控流式响应但不影响实时传输性能
 */
export function captureStreamResponseAsync(
  response: Response,
  onComplete?: (result: StreamCaptureResult) => void
): void {
  const reader = response.body?.getReader();
  if (!reader) {
    onComplete?.({
      content: '',
      isComplete: false,
      chunksProcessed: 0,
      totalTokens: 0
    });
    return;
  }

  const chunks: string[] = [];
  const contents: string[] = [];
  const decoder = new TextDecoder();
  let chunkCount = 0;
  let totalSize = 0;
  const maxSize = MAX_STREAM_SIZE_MB * 1024 * 1024;

  const readChunk = async () => {
    try {
      while (true) {
        // 内存限制检查
        if (totalSize > maxSize || chunkCount >= MAX_STREAM_CHUNKS) {
          console.warn('[StreamHandler] Async capture limits reached, stopping');
          break;
        }

        const { done, value } = await reader.read();
        if (done) break;

        const text = decoder.decode(value, { stream: true });
        chunks.push(text);
        chunkCount++;
        totalSize += value.length;

        // 实时解析 SSE 数据
        const lines = text.split('\n');
        for (const line of lines) {
          const trimmed = line.trim();
          if (trimmed.startsWith('data: ')) {
            const jsonStr = trimmed.slice(6);
            if (jsonStr === '[DONE]') continue;
            
            try {
              const data = JSON.parse(jsonStr);
              // OpenAI 格式
              if (data.choices?.[0]?.delta?.content) {
                contents.push(data.choices[0].delta.content);
              }
              if (data.choices?.[0]?.text) {
                contents.push(data.choices[0].text);
              }
              // Anthropic/Kimi 格式
              if (data.delta?.text) {
                contents.push(data.delta.text);
              }
              if (data.content_block?.text) {
                contents.push(data.content_block.text);
              }
            } catch {
              // 忽略解析失败的行
            }
          }
        }
      }

      const fullContent = contents.join('');
      onComplete?.({
        content: fullContent,
        isComplete: true,
        chunksProcessed: chunkCount,
        totalTokens: Math.ceil(fullContent.length / 4)  // 粗略估算
      });

      console.log(`[StreamHandler] Async capture completed: ${chunkCount} chunks, ` +
                  `${fullContent.length} chars, ~${Math.ceil(fullContent.length / 4)} tokens`);
    } catch (err) {
      console.error(`[StreamHandler] Async capture error: ${err}`);
      onComplete?.({
        content: contents.join(''),
        isComplete: false,
        chunksProcessed: chunkCount,
        totalTokens: Math.ceil(contents.join('').length / 4)
      });
    } finally {
      reader.releaseLock();
    }
  };

  // 启动后台读取，不 await
  readChunk();
}

/**
 * 解析 SSE 流数据，提取完整内容
 * OpenAI格式: data: {...}\n\ndata: {...}\n\n
 * Anthropic格式类似
 */
function parseSSEStream(streamText: string): string {
  const lines = streamText.split('\n');
  const contents: string[] = [];
  let usage: any = null;
  let finishReason: string | null = null;
  
  for (const line of lines) {
    const trimmed = line.trim();
    
    // SSE格式: data: {...}
    if (trimmed.startsWith('data: ')) {
      const jsonStr = trimmed.slice(6); // 去掉"data: "
      
      if (jsonStr === '[DONE]') {
        continue;
      }
      
      try {
        const data = JSON.parse(jsonStr);
        
        // OpenAI Chat Completions格式: choices[0].delta.content
        if (data.choices?.[0]?.delta?.content) {
          contents.push(data.choices[0].delta.content);
        }
        
        // OpenAI旧版Completions格式: choices[0].text
        if (data.choices?.[0]?.text) {
          contents.push(data.choices[0].text);
        }
        
        // Anthropic/Kimi 格式
        if (data.delta?.text) {
          contents.push(data.delta.text);
        }
        if (data.content_block?.text) {
          contents.push(data.content_block.text);
        }
        
        // 提取finish_reason
        if (data.choices?.[0]?.finish_reason) {
          finishReason = data.choices[0].finish_reason;
        }
        
        // 某些流式响应在最后一个chunk包含usage
        if (data.usage) {
          usage = data.usage;
        }
      } catch (err) {
        // 忽略解析失败的行（如空行或注释）
      }
    }
  }
  
  // 重构完整的响应对象
  const reconstructed: any = {
    choices: [{
      message: {
        role: 'assistant',
        content: contents.join('')
      },
      finish_reason: finishReason || 'stop'
    }],
    usage: usage || {
      prompt_tokens: 0,
      completion_tokens: 0,
      total_tokens: 0
    }
  };
  
  return JSON.stringify(reconstructed);
}
