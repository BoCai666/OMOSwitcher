export function parseGenericRequest(body: any): any {
  return {
    model: body.model || body.model_id,
    messages: body.messages,
    prompt: body.prompt,
  };
}

export function parseGenericResponse(body: any): any {
  return {
    content: body.content || body.choices?.[0]?.message?.content,
    usage: body.usage,
  };
}
