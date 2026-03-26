/**
 * presetStore 测试文件
 */

import { describe, it, expect, beforeEach } from 'vitest'
import {
  listPresets,
  savePreset,
  loadPreset,
  deletePreset,
  presetExists,
  clearPresets
} from '@/services/presetStore'
import { createDefaultConfig } from '@/types'

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value
    },
    removeItem: (key: string) => {
      delete store[key]
    },
    clear: () => {
      store = {}
    }
  }
})()

// 使用 globalThis 替代 global
;(globalThis as unknown as { localStorage: typeof localStorageMock }).localStorage = localStorageMock

describe('presetStore', () => {
  beforeEach(() => {
    // 每次测试前清空 localStorage
    localStorageMock.clear()
  })

  describe('listPresets', () => {
    it('无预设时返回空数组', () => {
      const presets = listPresets()
      expect(presets).toEqual([])
    })

    it('有预设时返回所有预设', () => {
      const config = createDefaultConfig()
      savePreset('preset1', config)
      savePreset('preset2', config)

      const presets = listPresets()
      expect(presets.length).toBe(2)
      expect(presets.map((p) => p.name)).toContain('preset1')
      expect(presets.map((p) => p.name)).toContain('preset2')
    })

    it('JSON 解析错误时返回空数组', () => {
      localStorageMock.setItem('omo-presets', 'invalid-json{')
      const presets = listPresets()
      expect(presets).toEqual([])
    })
  })

  describe('savePreset', () => {
    it('创建新预设', () => {
      const config = createDefaultConfig()
      config.agents.oracle.model = 'test-model'

      const preset = savePreset('my-preset', config, '测试预设')

      expect(preset.name).toBe('my-preset')
      expect(preset.description).toBe('测试预设')
      expect(preset.config.agents.oracle.model).toBe('test-model')
      expect(preset.createdAt).toBeDefined()
      expect(preset.updatedAt).toBeDefined()
    })

    it('更新已存在的预设', () => {
      const config = createDefaultConfig()
      const preset1 = savePreset('my-preset', config, '原始描述')

      const originalCreatedAt = preset1.createdAt

      config.agents.oracle.model = 'updated-model'
      const preset2 = savePreset('my-preset', config, '更新描述')

      // createdAt 应该保持不变
      expect(preset2.createdAt).toBe(originalCreatedAt)
      // updatedAt 应该存在且有值
      expect(preset2.updatedAt).toBeDefined()
      expect(preset2.description).toBe('更新描述')
      expect(preset2.config.agents.oracle.model).toBe('updated-model')

      // 应该只有一个预设
      const presets = listPresets()
      expect(presets.length).toBe(1)
    })

    it('描述参数可选', () => {
      const config = createDefaultConfig()
      const preset = savePreset('no-description', config)

      expect(preset.description).toBeUndefined()
    })
  })

  describe('loadPreset', () => {
    it('加载存在的预设', () => {
      const config = createDefaultConfig()
      config.agents.sisyphus.model = 'special-model'
      savePreset('test-preset', config, '测试')

      const loaded = loadPreset('test-preset')

      expect(loaded).not.toBeNull()
      expect(loaded!.name).toBe('test-preset')
      expect(loaded!.description).toBe('测试')
      expect(loaded!.config.agents.sisyphus.model).toBe('special-model')
    })

    it('加载不存在的预设返回 null', () => {
      const loaded = loadPreset('non-existent')
      expect(loaded).toBeNull()
    })
  })

  describe('deletePreset', () => {
    it('删除存在的预设', () => {
      const config = createDefaultConfig()
      savePreset('to-delete', config)

      expect(listPresets().length).toBe(1)

      const result = deletePreset('to-delete')

      expect(result).toBe(true)
      expect(listPresets().length).toBe(0)
    })

    it('删除不存在的预设返回 false', () => {
      const result = deletePreset('non-existent')
      expect(result).toBe(false)
    })
  })

  describe('presetExists', () => {
    it('预设存在时返回 true', () => {
      const config = createDefaultConfig()
      savePreset('existing-preset', config)

      expect(presetExists('existing-preset')).toBe(true)
    })

    it('预设不存在时返回 false', () => {
      expect(presetExists('non-existent')).toBe(false)
    })
  })

  describe('clearPresets', () => {
    it('清空所有预设', () => {
      const config = createDefaultConfig()
      savePreset('preset1', config)
      savePreset('preset2', config)
      savePreset('preset3', config)

      expect(listPresets().length).toBe(3)

      clearPresets()

      expect(listPresets().length).toBe(0)
    })
  })

  describe('完整流程', () => {
    it('创建、读取、更新、删除预设', () => {
      const config = createDefaultConfig()

      // 创建
      savePreset('workflow-preset', config, '工作流预设')
      expect(presetExists('workflow-preset')).toBe(true)

      // 读取
      const loaded = loadPreset('workflow-preset')
      expect(loaded!.description).toBe('工作流预设')

      // 更新
      config.agents.oracle.model = 'new-model'
      savePreset('workflow-preset', config, '更新后的描述')

      const updated = loadPreset('workflow-preset')
      expect(updated!.description).toBe('更新后的描述')
      expect(updated!.config.agents.oracle.model).toBe('new-model')

      // 删除
      deletePreset('workflow-preset')
      expect(presetExists('workflow-preset')).toBe(false)
    })
  })
})
