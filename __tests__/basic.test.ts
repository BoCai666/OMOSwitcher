/**
 * 示例测试文件
 * 测试基础设施和类型定义
 */

import { describe, it, expect } from 'vitest'
import {
  type AgentConfig,
  type CategoryConfig,
  type OhMyOpenCodeConfig,
  type Model,
  type Preset,
  AGENT_NAMES,
  CATEGORY_NAMES,
  createDefaultConfig,
  defaultConfig
} from '@/types'

describe('基础测试', () => {
  it('应该正确计算 1 + 1', () => {
    expect(1 + 1).toBe(2)
  })

  it('应该正确处理字符串', () => {
    const str = 'OMOSwitcher'
    expect(str).toContain('Switcher')
    expect(str.length).toBe(11)
  })
})

describe('类型定义测试', () => {
  // 测试 AgentConfig 类型
  it('应该正确创建 AgentConfig 对象', () => {
    const agentConfig: AgentConfig = {
      model: 'wuwen/glm-5'
    }

    expect(agentConfig.model).toBe('wuwen/glm-5')
  })

  // 测试 CategoryConfig 类型
  it('应该正确创建 CategoryConfig 对象', () => {
    const categoryConfig: CategoryConfig = {
      model: 'wuwen/minimax-m2.5'
    }

    expect(categoryConfig.model).toBe('wuwen/minimax-m2.5')
  })

  // 测试 Agent 名称列表
  it('应该包含 10 个 Agent 名称', () => {
    expect(AGENT_NAMES.length).toBe(10)
    expect(AGENT_NAMES).toContain('oracle')
    expect(AGENT_NAMES).toContain('librarian')
    expect(AGENT_NAMES).toContain('explore')
  })

  // 测试 Category 名称列表
  it('应该包含 8 个 Category 名称', () => {
    expect(CATEGORY_NAMES.length).toBe(8)
    expect(CATEGORY_NAMES).toContain('visual-engineering')
    expect(CATEGORY_NAMES).toContain('deep')
    expect(CATEGORY_NAMES).toContain('quick')
  })

  // 测试 createDefaultConfig
  it('应该创建默认配置', () => {
    const config = createDefaultConfig()

    expect(config.$schema).toBeDefined()
    expect(config.agents).toBeDefined()
    expect(config.categories).toBeDefined()
    expect(Object.keys(config.agents).length).toBe(10)
    expect(Object.keys(config.categories).length).toBe(8)
  })

  // 测试 defaultConfig
  it('应该正确导出默认配置', () => {
    expect(defaultConfig.agents).toBeDefined()
    expect(defaultConfig.categories).toBeDefined()
    expect(defaultConfig.agents.oracle.model).toBeDefined()
    expect(defaultConfig.categories['visual-engineering'].model).toBeDefined()
  })

  // 测试 Model 类型
  it('应该正确创建 Model 对象', () => {
    const model: Model = {
      id: 'wuwen/glm-5',
      name: 'GLM-5',
      provider: 'wuwen'
    }

    expect(model.id).toBe('wuwen/glm-5')
    expect(model.name).toBe('GLM-5')
    expect(model.provider).toBe('wuwen')
  })

  // 测试 Preset 类型
  it('应该正确创建 Preset 对象', () => {
    const preset: Preset = {
      name: '默认配置',
      description: '我的默认配置',
      config: createDefaultConfig(),
      createdAt: '2026-03-24T00:00:00.000Z',
      updatedAt: '2026-03-24T00:00:00.000Z'
    }

    expect(preset.name).toBe('默认配置')
    expect(preset.description).toBe('我的默认配置')
    expect(preset.config).toBeDefined()
  })

  // 测试完整的 OhMyOpenCodeConfig
  it('应该正确处理完整的配置结构', () => {
    const fullConfig: OhMyOpenCodeConfig = {
      $schema: 'https://example.com/schema.json',
      agents: {
        oracle: { model: 'wuwen/glm-5' },
        librarian: { model: 'wuwen/minimax-m2.5' },
        explore: { model: 'wuwen/minimax-m2.5' },
        'multimodal-looker': { model: 'wuwen/kimi-k2.5' },
        metis: { model: 'wuwen/minimax-m2.7' },
        momus: { model: 'wuwen/glm-5' },
        sisyphus: { model: 'wuwen/glm-5' },
        hephaestus: { model: 'wuwen/glm-5' },
        prometheus: { model: 'wuwen/glm-5' },
        atlas: { model: 'wuwen/glm-5' }
      },
      categories: {
        'visual-engineering': { model: 'wuwen/kimi-k2.5' },
        ultrabrain: { model: 'wuwen/minimax-m2.7' },
        deep: { model: 'wuwen/glm-5' },
        artistry: { model: 'wuwen/glm-5' },
        quick: { model: 'wuwen/minimax-m2.5' },
        'unspecified-low': { model: 'wuwen/minimax-m2.7' },
        'unspecified-high': { model: 'wuwen/glm-5' },
        writing: { model: 'wuwen/minimax-m2.5' }
      }
    }

    expect(fullConfig.agents.oracle.model).toBe('wuwen/glm-5')
    expect(fullConfig.categories['visual-engineering'].model).toBe('wuwen/kimi-k2.5')
  })
})
