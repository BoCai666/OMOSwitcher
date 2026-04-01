export function parseOpenAIRequest(body: any): any {
  return {
    model: body.model,
    messages: body.messages,
    prompt: body.prompt,
    temperature: body.temperature,
    max_tokens: body.max_tokens,
    stream: body.stream,
  };
}

export function parseOpenAIResponse(body: any): any {
  const choice = body.choices?.[0];
  const message = choice?.message;
  
  // 提取 thinking/reasoning 内容
  let thinking: string | undefined;
  let content: string | undefined;
  
  // DeepSeek R1 格式：reasoning_content 字段
  if (message?.reasoning_content) {
    thinking = message.reasoning_content;
  }
  
  // 提取主内容
  content = message?.content || choice?.text;
  
  return {
    content,
    thinking,
    usage: body.usage,
    model: body.model,
  };
}
