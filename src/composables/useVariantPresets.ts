/**
 * 变体预设配置
 * 基于 OpenCode 源码 packages/opencode/src/provider/transform.ts
 */

// Variant 参数字段定义
export interface VariantFieldDef {
  key: string
  label: string
  type: 'select' | 'number'
  options?: { value: string; label: string }[]
  min?: number
  max?: number
  step?: number
  default: unknown
}

// Variant 选项定义
export interface VariantOption {
  key: string
  label: string
  description: string
  fields: VariantFieldDef[]
  defaults: Record<string, unknown>
}

// 根据 API 格式定义可用的 variant 选项及其可配置参数
// 参考: https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/provider/transform.ts
export const VARIANT_PRESETS: Record<string, VariantOption[]> = {
  // OpenAI 兼容格式 (OpenRouter, Venice, DeepInfra, Cerebras, TogetherAI, xAI 等)
  '@ai-sdk/openai-compatible': [
    {
      key: 'none',
      label: 'none',
      description: '无推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'none', label: 'none（无）' },
        ], default: 'none' },
      ],
      defaults: { reasoningEffort: 'none' },
    },
    {
      key: 'minimal',
      label: 'minimal',
      description: '最小推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'minimal', label: 'minimal（最小）' },
        ], default: 'minimal' },
      ],
      defaults: { reasoningEffort: 'minimal' },
    },
    {
      key: 'low',
      label: 'low',
      description: '低推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'low', label: 'low（低）' },
        ], default: 'low' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
      ],
      defaults: { reasoningEffort: 'low', textVerbosity: 'low' },
    },
    {
      key: 'medium',
      label: 'medium',
      description: '中等推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'medium', label: 'medium（中）' },
        ], default: 'medium' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
      ],
      defaults: { reasoningEffort: 'medium', textVerbosity: 'low' },
    },
    {
      key: 'high',
      label: 'high',
      description: '高推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'high', label: 'high（高）' },
        ], default: 'high' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
      ],
      defaults: { reasoningEffort: 'high', textVerbosity: 'low' },
    },
    {
      key: 'xhigh',
      label: 'xhigh',
      description: '极高推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'xhigh', label: 'xhigh（极高）' },
        ], default: 'xhigh' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
      ],
      defaults: { reasoningEffort: 'xhigh', textVerbosity: 'low' },
    },
  ],

  // OpenAI 原生格式 (支持更多 variant 级别)
  '@ai-sdk/openai': [
    {
      key: 'none',
      label: 'none',
      description: '无推理（最快）',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'none', label: 'none（无）' },
        ], default: 'none' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'none', reasoningSummary: 'auto' },
    },
    {
      key: 'minimal',
      label: 'minimal',
      description: '最小推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'minimal', label: 'minimal（最小）' },
        ], default: 'minimal' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'minimal', reasoningSummary: 'auto' },
    },
    {
      key: 'low',
      label: 'low',
      description: '低推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'low', label: 'low（低）' },
        ], default: 'low' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'low', textVerbosity: 'low', reasoningSummary: 'auto' },
    },
    {
      key: 'medium',
      label: 'medium',
      description: '中等推理（默认）',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'medium', label: 'medium（中）' },
        ], default: 'medium' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'medium', textVerbosity: 'low', reasoningSummary: 'auto' },
    },
    {
      key: 'high',
      label: 'high',
      description: '高推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'high', label: 'high（高）' },
        ], default: 'high' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'high', textVerbosity: 'low', reasoningSummary: 'auto' },
    },
    {
      key: 'xhigh',
      label: 'xhigh',
      description: '极高推理',
      fields: [
        { key: 'reasoningEffort', label: 'reasoningEffort（推理努力）', type: 'select', options: [
          { value: 'xhigh', label: 'xhigh（极高）' },
        ], default: 'xhigh' },
        { key: 'textVerbosity', label: 'textVerbosity（文本详细度）', type: 'select', options: [
          { value: 'low', label: 'low（简洁）' },
          { value: 'medium', label: 'medium（中等）' },
          { value: 'high', label: 'high（详细）' },
        ], default: 'low' },
        { key: 'reasoningSummary', label: 'reasoningSummary（推理摘要）', type: 'select', options: [
          { value: 'auto', label: 'auto（自动）' },
          { value: 'concise', label: 'concise（简洁）' },
          { value: 'detailed', label: 'detailed（详细）' },
        ], default: 'auto' },
      ],
      defaults: { reasoningEffort: 'xhigh', textVerbosity: 'low', reasoningSummary: 'auto' },
    },
  ],

  // Anthropic 格式 (Claude 系列模型)
  '@ai-sdk/anthropic': [
    {
      key: 'low',
      label: 'low',
      description: '低思考预算',
      fields: [
        { key: 'budgetTokens', label: 'budgetTokens（思考预算）', type: 'number', min: 1000, max: 31999, step: 1000, default: 8000 },
      ],
      defaults: { thinking: { type: 'enabled', budgetTokens: 8000 } },
    },
    {
      key: 'medium',
      label: 'medium',
      description: '中等思考预算',
      fields: [
        { key: 'budgetTokens', label: 'budgetTokens（思考预算）', type: 'number', min: 1000, max: 31999, step: 1000, default: 12000 },
      ],
      defaults: { thinking: { type: 'enabled', budgetTokens: 12000 } },
    },
    {
      key: 'high',
      label: 'high',
      description: '高思考预算',
      fields: [
        { key: 'budgetTokens', label: 'budgetTokens（思考预算）', type: 'number', min: 1000, max: 31999, step: 1000, default: 16000 },
      ],
      defaults: { thinking: { type: 'enabled', budgetTokens: 16000 } },
    },
    {
      key: 'max',
      label: 'max',
      description: '最大思考预算',
      fields: [
        { key: 'budgetTokens', label: 'budgetTokens（思考预算）', type: 'number', min: 1000, max: 31999, step: 1000, default: 31999 },
      ],
      defaults: { thinking: { type: 'enabled', budgetTokens: 31999 } },
    },
  ],

  // Google Gemini 格式
  '@ai-sdk/google': [
    {
      key: 'low',
      label: 'low',
      description: '低思考级别',
      fields: [
        { key: 'thinkingLevel', label: 'thinkingLevel（思考级别）', type: 'select', options: [
          { value: 'low', label: 'low（低）' },
        ], default: 'low' },
      ],
      defaults: { thinkingConfig: { includeThoughts: true, thinkingLevel: 'low' } },
    },
    {
      key: 'medium',
      label: 'medium',
      description: '中等思考级别',
      fields: [
        { key: 'thinkingLevel', label: 'thinkingLevel（思考级别）', type: 'select', options: [
          { value: 'medium', label: 'medium（中）' },
        ], default: 'medium' },
      ],
      defaults: { thinkingConfig: { includeThoughts: true, thinkingLevel: 'medium' } },
    },
    {
      key: 'high',
      label: 'high',
      description: '高思考级别',
      fields: [
        { key: 'thinkingLevel', label: 'thinkingLevel（思考级别）', type: 'select', options: [
          { value: 'high', label: 'high（高）' },
        ], default: 'high' },
        { key: 'thinkingBudget', label: 'thinkingBudget（思考预算）', type: 'number', min: 1000, max: 24576, step: 1000, default: 16000 },
      ],
      defaults: { thinkingConfig: { includeThoughts: true, thinkingLevel: 'high', thinkingBudget: 16000 } },
    },
    {
      key: 'max',
      label: 'max',
      description: '最大思考预算',
      fields: [
        { key: 'thinkingBudget', label: 'thinkingBudget（思考预算）', type: 'number', min: 1000, max: 24576, step: 1000, default: 24576 },
      ],
      defaults: { thinkingConfig: { includeThoughts: true, thinkingBudget: 24576 } },
    },
  ],
}

// 获取当前 API 格式下的可用变体选项
export function getVariantOptions(npm: string): VariantOption[] {
  return VARIANT_PRESETS[npm] || []
}
