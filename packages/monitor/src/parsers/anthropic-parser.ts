export function parseAnthropicRequest(body: any): any {
  return {
    model: body.model,
    messages: body.messages,
    max_tokens: body.max_tokens,
    temperature: body.temperature,
  };
}

export function parseAnthropicResponse(body: any): any {
  return {
    content: body.content?.[0]?.text || body.completion,
    usage: body.usage,
    model: body.model,
  };
}
