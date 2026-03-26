import { ConfigManager } from '../config-manager.js';

// 硬编码定价作为回退（单位：美元/1M tokens）
const FALLBACK_PRICING: Record<string, { input: number; output: number }> = {
  'gpt-4': { input: 30, output: 60 },
  'gpt-4-turbo': { input: 10, output: 30 },
  'gpt-3.5-turbo': { input: 0.5, output: 1.5 },
  'gpt-4o': { input: 5, output: 15 },
  'gpt-4o-mini': { input: 0.15, output: 0.6 },
  'claude-3-opus': { input: 15, output: 75 },
  'claude-3-sonnet': { input: 3, output: 15 },
};

// ConfigManager 单例
let configManager: ConfigManager | null = null;
let configLoaded = false;

/**
 * 初始化 ConfigManager（异步）
 */
async function initConfigManager(): Promise<void> {
  if (!configManager) {
    configManager = new ConfigManager();
    try {
      await configManager.load();
      configLoaded = true;
    } catch (error) {
      console.warn('Failed to load config for cost calculator, using fallback pricing:', error);
      configLoaded = false;
    }
  }
}

// 立即尝试初始化配置
initConfigManager().catch(() => {
  // 初始化失败时保持使用回退定价
});

/**
 * 从配置中查找模型定价
 */
function findPricingFromConfig(model: string): { input: number; output: number } | null {
  if (!configManager || !configLoaded) {
    return null;
  }

  try {
    const pricing = configManager.get('pricing');
    if (!pricing || !pricing.models || !Array.isArray(pricing.models)) {
      return null;
    }

    const matchStrategy = pricing.matchStrategy || 'prefix';
    const models = pricing.models;

    // 首先尝试精确匹配
    for (const modelConfig of models) {
      if (modelConfig.model === model) {
        return { input: modelConfig.input, output: modelConfig.output };
      }
    }

    // 如果匹配策略是 prefix，尝试前缀匹配
    if (matchStrategy === 'prefix') {
      for (const modelConfig of models) {
        if (model.startsWith(modelConfig.model)) {
          return { input: modelConfig.input, output: modelConfig.output };
        }
      }
    }

    return null;
  } catch (error) {
    console.warn('Error reading pricing from config:', error);
    return null;
  }
}

/**
 * 从回退定价中查找
 */
function findPricingFromFallback(model: string): { input: number; output: number } {
  let pricing = FALLBACK_PRICING[model];
  if (!pricing) {
    for (const [key, value] of Object.entries(FALLBACK_PRICING)) {
      if (model.startsWith(key)) {
        pricing = value;
        break;
      }
    }
  }
  if (!pricing) {
    // 默认定价（单位：美元/1M tokens）
    pricing = { input: 1, output: 2 };
  }
  return pricing;
}

export function calculateCost(model: string, inputTokens: number, outputTokens: number): number {
  // 优先尝试从配置获取定价
  let pricing = findPricingFromConfig(model);

  // 如果配置中没有找到，使用回退定价
  if (!pricing) {
    pricing = findPricingFromFallback(model);
  }

  // 配置单位为 美元/1M tokens，计算时需除以 1000000
  const inputCost = (inputTokens / 1000000) * pricing.input;
  const outputCost = (outputTokens / 1000000) * pricing.output;
  return inputCost + outputCost;
}
