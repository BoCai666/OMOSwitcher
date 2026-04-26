/**
 * Category 详情数据
 * 系统提示词已迁移至 public/prompts/categories/ 目录下动态加载
 * 数据来源: oh-my-openagent 官方源码
 */

import type { CategoryName } from '@/types'

export interface FallbackModel {
  providers: string[]
  model: string
  variant?: string
}

export interface CategoryDetail {
  name: CategoryName
  displayName: string
  description: string
  recommendedModel: string
  fallbackChain: FallbackModel[]
}

export const CATEGORY_DETAILS: Record<CategoryName, CategoryDetail> = {
  'visual-engineering': {
    name: 'visual-engineering',
    displayName: 'Visual Engineering',
    description: 'Frontend, UI/UX, design, styling, animation',
    recommendedModel: 'google/gemini-3.1-pro',
    fallbackChain: [
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' },
      { providers: ['zai-coding-plan', 'opencode', 'vercel'], model: 'glm-5' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' },
      { providers: ['kimi-for-coding'], model: 'k2p5' }
    ]
  },

  ultrabrain: {
    name: 'ultrabrain',
    displayName: 'Ultra Brain',
    description: 'Use ONLY for genuinely hard, logic-heavy tasks. Give clear goals only, not step-by-step instructions.',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'xhigh' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' }
    ]
  },

  deep: {
    name: 'deep',
    displayName: 'Deep',
    description: 'Goal-oriented autonomous problem-solving. Thorough research before action. For hairy problems requiring deep understanding.',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'github-copilot', 'venice', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'medium' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' }
    ]
  },

  artistry: {
    name: 'artistry',
    displayName: 'Artistry',
    description: 'Complex problem-solving with unconventional, creative approaches - beyond standard patterns',
    recommendedModel: 'google/gemini-3.1-pro',
    fallbackChain: [
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5' }
    ]
  },

  quick: {
    name: 'quick',
    displayName: 'Quick',
    description: 'Trivial tasks - single file changes, typo fixes, simple modifications',
    recommendedModel: 'openai/gpt-5.4-mini',
    fallbackChain: [
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.4-mini' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-haiku-4-5' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3-flash' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' },
      { providers: ['opencode', 'vercel'], model: 'gpt-5-nano' }
    ]
  },

  'unspecified-low': {
    name: 'unspecified-low',
    displayName: 'Unspecified Low',
    description: 'Tasks that don\'t fit other categories, low effort required',
    recommendedModel: 'anthropic/claude-sonnet-4-6',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-sonnet-4-6' },
      { providers: ['openai', 'opencode', 'vercel'], model: 'gpt-5.3-codex', variant: 'medium' },
      { providers: ['opencode-go', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3-flash' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' }
    ]
  },

  'unspecified-high': {
    name: 'unspecified-high',
    displayName: 'Unspecified High',
    description: 'Tasks that don\'t fit other categories, high effort required',
    recommendedModel: 'anthropic/claude-opus-4-7',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'high' },
      { providers: ['zai-coding-plan', 'opencode', 'vercel'], model: 'glm-5' },
      { providers: ['kimi-for-coding'], model: 'k2p5' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' },
      { providers: ['opencode', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['opencode', 'moonshotai', 'moonshotai-cn', 'firmware', 'ollama-cloud', 'aihubmix', 'vercel'], model: 'kimi-k2.5' }
    ]
  },

  writing: {
    name: 'writing',
    displayName: 'Writing',
    description: 'Documentation, prose, technical writing',
    recommendedModel: 'kimi-for-coding/k2p5',
    fallbackChain: [
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3-flash' },
      { providers: ['opencode-go', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-sonnet-4-6' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' }
    ]
  }
}
