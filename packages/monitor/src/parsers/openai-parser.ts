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
  return {
    content: body.choices?.[0]?.message?.content || body.choices?.[0]?.text,
    usage: body.usage,
    model: body.model,
  };
}
