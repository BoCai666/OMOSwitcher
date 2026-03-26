import { BatchInterceptor } from '@mswjs/interceptors';
import { ClientRequestInterceptor } from '@mswjs/interceptors/ClientRequest';
import { FetchInterceptor } from '@mswjs/interceptors/fetch';
import { handleRequest } from './request-handler.js';
import { handleResponse } from './response-handler.js';

const interceptor = new BatchInterceptor({
  name: 'opencode-monitor',
  interceptors: [
    new ClientRequestInterceptor(),
    new FetchInterceptor(),
  ],
});

// 只拦截LLM API
const LLM_HOSTS = [
  'api.openai.com',
  'api.anthropic.com',
  'api.groq.com',
  'openrouter.ai',
  'api.kimi.com',  // Kimi for Coding
];

export function startInterceptor(): void {
  interceptor.on('request', ({ request }) => {
    const url = new URL(request.url);
    if (LLM_HOSTS.some(host => url.hostname.includes(host))) {
      handleRequest(request);
    }
  });

  interceptor.on('response', ({ response, request }) => {
    const url = new URL(request.url);
    if (LLM_HOSTS.some(host => url.hostname.includes(host))) {
      handleResponse(request, response);
    }
  });

  interceptor.apply();
  console.log('[Monitor] HTTP interceptor started');
}

export function stopInterceptor(): void {
  interceptor.dispose();
  console.log('[Monitor] HTTP interceptor stopped');
}
