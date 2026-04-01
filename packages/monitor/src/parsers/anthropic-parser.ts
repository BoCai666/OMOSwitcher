export function parseAnthropicRequest(body: any): any {
  return {
    model: body.model,
    messages: body.messages,
    max_tokens: body.max_tokens,
    temperature: body.temperature,
  };
}

export function parseAnthropicResponse(body: any): any {
  // 提取 thinking 和 text 内容
  let thinking: string | undefined;
  let content: string | undefined;
  
  if (Array.isArray(body.content)) {
    const textParts: string[] = [];
    
    for (const block of body.content) {
      if (block.type === 'thinking' && block.thinking) {
        thinking = block.thinking;
      } else if (block.type === 'text' && block.text) {
        textParts.push(block.text);
      } else if (block.type === 'redacted_thinking') {
        thinking = '[思考内容已隐藏]';
      }
    }
    
    content = textParts.join('\n');
  } else {
    content = body.content?.[0]?.text || body.completion;
  }
  
  return {
    content,
    thinking,
    usage: body.usage,
    model: body.model,
  };
}
