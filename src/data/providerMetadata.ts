/**
 * 供应商视觉元数据
 * 包含品牌色、图标 SVG path、显示名称等
 */

export interface ProviderMetadata {
  /** 显示名称 */
  displayName: string
  /** 品牌主色 */
  color: string
  /** 品牌渐变色（可选） */
  gradient?: string
  /** 图标 SVG path */
  iconPath: string
  /** 图标 viewBox */
  iconViewBox?: string
  /** 描述 */
  description?: string
}

// 常见供应商的元数据
export const PROVIDER_METADATA: Record<string, ProviderMetadata> = {
  anthropic: {
    displayName: 'Anthropic',
    color: '#d4a574',
    gradient: 'linear-gradient(135deg, #d4a574 0%, #c9956c 100%)',
    iconPath: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z',
    iconViewBox: '0 0 24 24',
    description: 'Claude AI 模型'
  },
  openai: {
    displayName: 'OpenAI',
    color: '#10a37f',
    gradient: 'linear-gradient(135deg, #10a37f 0%, #0d8a6a 100%)',
    iconPath: 'M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364l2.0201-1.1685a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.4043-.6813zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z',
    iconViewBox: '0 0 24 24',
    description: 'GPT 系列模型'
  },
  google: {
    displayName: 'Google',
    color: '#4285f4',
    gradient: 'linear-gradient(135deg, #4285f4 0%, #34a853 50%, #fbbc05 75%, #ea4335 100%)',
    iconPath: 'M12.545,10.239v3.821h5.445c-0.712,2.315-2.647,3.972-5.445,3.972c-3.332,0-6.033-2.701-6.033-6.032s2.701-6.032,6.033-6.032c1.498,0,2.866,0.549,3.921,1.453l2.814-2.814C17.503,2.988,15.139,2,12.545,2C7.021,2,2.543,6.477,2.543,12s4.478,10,10.002,10c8.396,0,10.249-7.85,9.426-11.748L12.545,10.239z',
    iconViewBox: '0 0 24 24',
    description: 'Gemini 模型'
  },
  deepseek: {
    displayName: 'DeepSeek',
    color: '#0066ff',
    gradient: 'linear-gradient(135deg, #0066ff 0%, #0052cc 100%)',
    iconPath: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
    iconViewBox: '0 0 24 24',
    description: '深度求索'
  },
  'x-ai': {
    displayName: 'xAI',
    color: '#000000',
    gradient: 'linear-gradient(135deg, #1a1a1a 0%, #333333 100%)',
    iconPath: 'M18.901 1.153h3.68l-8.04 9.19L24 22.846h-7.406l-5.8-7.584-6.638 7.584H.474l8.6-9.83L0 1.154h7.594l5.243 6.932ZM17.61 20.644h2.039L6.486 3.24H4.298Z',
    iconViewBox: '0 0 24 24',
    description: 'Grok 模型'
  },
  moonshotai: {
    displayName: 'Moonshot',
    color: '#6366f1',
    gradient: 'linear-gradient(135deg, #6366f1 0%, #4f46e5 100%)',
    iconPath: 'M12 3L1 9l11 6 9-4.91V17h2V9L12 3z',
    iconViewBox: '0 0 24 24',
    description: '月之暗面'
  },
  ollama: {
    displayName: 'Ollama',
    color: '#00d4aa',
    gradient: 'linear-gradient(135deg, #00d4aa 0%, #00b894 100%)',
    iconPath: 'M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 18a8 8 0 1 1 8-8 8 8 0 0 1-8 8zm4-8a4 4 0 1 1-4-4 4 4 0 0 1 4 4z',
    iconViewBox: '0 0 24 24',
    description: '本地模型运行'
  },
  'github-copilot': {
    displayName: 'GitHub Copilot',
    color: '#24292f',
    gradient: 'linear-gradient(135deg, #24292f 0%, #404040 100%)',
    iconPath: 'M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.463-1.11-1.463-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.268 2.75 1.026A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.026 2.747-1.026.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.163 22 16.418 22 12c0-5.523-4.477-10-10-10z',
    iconViewBox: '0 0 24 24',
    description: 'GitHub AI 助手'
  },
  zhipu: {
    displayName: '智谱AI',
    color: '#3370ff',
    gradient: 'linear-gradient(135deg, #3370ff 0%, #2860e1 100%)',
    iconPath: 'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z',
    iconViewBox: '0 0 24 24',
    description: 'GLM 模型'
  },
  qianfan: {
    displayName: '百度千帆',
    color: '#2932e1',
    gradient: 'linear-gradient(135deg, #2932e1 0%, #1e26c9 100%)',
    iconPath: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
    iconViewBox: '0 0 24 24',
    description: '文心一言'
  },
  aliyun: {
    displayName: '阿里云',
    color: '#ff6a00',
    gradient: 'linear-gradient(135deg, #ff6a00 0%, #ee5a00 100%)',
    iconPath: 'M12 2L2 7v10l10 5 10-5V7L12 2zm0 2.18l6.9 3.45L12 11.09 5.1 7.63 12 4.18zM4 16.54V9.09l7 3.5v7.45l-7-3.5zm9 3.5v-7.45l7-3.5v7.45l-7 3.5z',
    iconViewBox: '0 0 24 24',
    description: '通义千问'
  },
  'aws-bedrock': {
    displayName: 'AWS Bedrock',
    color: '#ff9900',
    gradient: 'linear-gradient(135deg, #ff9900 0%, #e68a00 100%)',
    iconPath: 'M12 2L2 7v10l10 5 10-5V7L12 2zm0 18.5L4 16V8.5l8 4v8zm8-4.5l-8 4v-8l8-4V16z',
    iconViewBox: '0 0 24 24',
    description: 'AWS 基础模型服务'
  },
  azure: {
    displayName: 'Azure OpenAI',
    color: '#0078d4',
    gradient: 'linear-gradient(135deg, #0078d4 0%, #006cbd 100%)',
    iconPath: 'M5.48 10.45h5.47l1.87-5.83h3.7l1.87 5.83h5.47L20.9 2H3.1l2.38 8.45zM22.86 12H1.14l2.38 8.45h18.96l2.38-8.45z',
    iconViewBox: '0 0 24 24',
    description: 'Azure OpenAI 服务'
  },
  cohere: {
    displayName: 'Cohere',
    color: '#39594d',
    gradient: 'linear-gradient(135deg, #39594d 0%, #2d473d 100%)',
    iconPath: 'M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 18a8 8 0 1 1 8-8 8 8 0 0 1-8 8z',
    iconViewBox: '0 0 24 24',
    description: 'Cohere 模型'
  },
  mistral: {
    displayName: 'Mistral',
    color: '#ff7000',
    gradient: 'linear-gradient(135deg, #ff7000 0%, #e56300 100%)',
    iconPath: 'M12 2L2 7v10l10 5 10-5V7L12 2z',
    iconViewBox: '0 0 24 24',
    description: 'Mistral AI'
  },
  'openrouter': {
    displayName: 'OpenRouter',
    color: '#6366f1',
    gradient: 'linear-gradient(135deg, #6366f1 0%, #4f46e5 100%)',
    iconPath: 'M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z',
    iconViewBox: '0 0 24 24',
    description: '多模型聚合'
  },
  minimax: {
    displayName: 'MiniMax',
    color: '#2563eb',
    gradient: 'linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%)',
    iconPath: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z',
    iconViewBox: '0 0 24 24',
    description: 'MiniMax AI'
  }
}

// 默认供应商元数据
const DEFAULT_METADATA: ProviderMetadata = {
  displayName: '',
  color: '#6b7280',
  gradient: 'linear-gradient(135deg, #6b7280 0%, #4b5563 100%)',
  iconPath: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
  iconViewBox: '0 0 24 24'
}

/**
 * 获取供应商元数据
 * @param providerId 供应商 ID
 * @returns 供应商元数据
 */
export function getProviderMetadata(providerId: string): ProviderMetadata {
  // 直接匹配
  if (PROVIDER_METADATA[providerId]) {
    return PROVIDER_METADATA[providerId]
  }
  
  // 模糊匹配（处理 kebab-case 变体）
  const normalizedId = providerId.toLowerCase().replace(/[_\s]/g, '-')
  for (const [key, value] of Object.entries(PROVIDER_METADATA)) {
    if (key.toLowerCase().replace(/[_\s]/g, '-') === normalizedId) {
      return value
    }
  }
  
  // 返回默认值，displayName 使用 providerId
  return {
    ...DEFAULT_METADATA,
    displayName: providerId.charAt(0).toUpperCase() + providerId.slice(1).replace(/-/g, ' ')
  }
}

/**
 * 获取供应商品牌色
 */
export function getProviderColor(providerId: string): string {
  return getProviderMetadata(providerId).color
}

/**
 * 获取供应商渐变色
 */
export function getProviderGradient(providerId: string): string {
  return getProviderMetadata(providerId).gradient || getProviderColor(providerId)
}
