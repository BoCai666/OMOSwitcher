import { LLMRequest } from '../types.js';

export interface McpDetectionResult {
  isMcpCall: boolean;
  toolName?: string;
  arguments?: Record<string, unknown>;
  transportType?: 'stdio' | 'sse' | 'http';
  jsonrpcVersion?: string;
  rpcId?: string;
  serverInfo?: {
    name?: string;
    version?: string;
  };
  meta?: Record<string, unknown>;
}

/**
 * 检测请求是否为 MCP 调用
 * 
 * 支持三种检测方式：
 * 1. JSON-RPC 2.0 格式 (标准 MCP)
 * 2. OpenAI Function Calling 格式
 * 3. URL 路径模式
 */
export function detectMcpCall(request: LLMRequest): McpDetectionResult {
  const body = request.body;
  
  // 检测1: JSON-RPC 2.0 格式 (标准 MCP)
  if (body?.jsonrpc === '2.0' && body?.method) {
    // tools/call 请求
    if (body.method === 'tools/call' && body.params) {
      return {
        isMcpCall: true,
        jsonrpcVersion: '2.0',
        rpcId: body.id?.toString(),
        toolName: body.params.name,
        arguments: body.params.arguments,
        transportType: detectTransportType(request),
        meta: body.params._meta
      };
    }
    
    // tools/list 请求
    if (body.method === 'tools/list') {
      return {
        isMcpCall: true,
        jsonrpcVersion: '2.0',
        rpcId: body.id?.toString(),
        toolName: 'tools/list',
        transportType: detectTransportType(request)
      };
    }
  }
  
  // 检测2: OpenAI Function Calling 格式
  if (body?.tool_choice || body?.tools) {
    const toolCall = body.tool_choice?.function || body.tools?.[0]?.function;
    if (toolCall) {
      return {
        isMcpCall: true,
        toolName: toolCall.name,
        arguments: typeof toolCall.arguments === 'string' 
          ? JSON.parse(toolCall.arguments) 
          : toolCall.arguments,
        transportType: 'http'
      };
    }
  }
  
  // 检测3: 通过请求路径判断
  if (request.url?.includes('/mcp') || request.url?.includes('/tools/call')) {
    return {
      isMcpCall: true,
      transportType: 'http'
    };
  }
  
  return { isMcpCall: false };
}

/**
 * 检测传输类型
 */
function detectTransportType(request: LLMRequest): 'stdio' | 'sse' | 'http' {
  const contentType = request.headers?.['content-type'] || '';
  const accept = request.headers?.['accept'] || '';
  
  if (accept.includes('text/event-stream')) {
    return 'sse';
  }
  if (request.url?.includes('/mcp') || request.url?.includes('/jsonrpc')) {
    return 'http';
  }
  if (contentType.includes('application/json')) {
    return 'http';
  }
  return 'stdio'; // 默认
}

/**
 * 从响应中提取 MCP 结果
 */
export function extractMcpResult(responseBody: any): {
  content?: any;
  isError: boolean;
  errorMessage?: string;
} {
  if (!responseBody) {
    return { isError: false };
  }
  
  // JSON-RPC 2.0 响应格式
  if (responseBody.jsonrpc === '2.0') {
    if (responseBody.error) {
      return {
        isError: true,
        errorMessage: responseBody.error.message || JSON.stringify(responseBody.error),
        content: responseBody.error
      };
    }
    return {
      isError: false,
      content: responseBody.result
    };
  }
  
  // OpenAI function calling 结果
  if (responseBody.choices?.[0]?.message?.function_call) {
    const funcCall = responseBody.choices[0].message.function_call;
    return {
      isError: false,
      content: typeof funcCall.arguments === 'string' 
        ? JSON.parse(funcCall.arguments) 
        : funcCall.arguments
    };
  }
  
  return { isError: false, content: responseBody };
}
