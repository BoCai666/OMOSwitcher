/**
 * configReader 测试文件
 */

import { describe, it, expect, beforeEach } from 'vitest'
import {
  readConfig,
  configExists,
  writeConfig,
  deleteConfig
} from '@/services/configReader'
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

describe('configReader', () => {
  beforeEach(() => {
    // 每次测试前清空 localStorage
    localStorageMock.clear()
  })

  describe('readConfig', () => {
    it('配置不存在时返回默认配置', async () => {
      const config = await readConfig()
      
      expect(config).toBeDefined()
      expect(config.$schema).toBeDefined()
      expect(config.agents).toBeDefined()
      expect(config.categories).toBeDefined()
      expect(Object.keys(config.agents).length).toBe(10)
      expect(Object.keys(config.categories).length).toBe(8)
    })

    it('配置存在时返回存储的配置', async () => {
      const customConfig = createDefaultConfig()
      customConfig.agents.oracle.model = 'custom-model'
      
      localStorageMock.setItem('omo-config', JSON.stringify(customConfig))
      
      const config = await readConfig()
      
      expect(config.agents.oracle.model).toBe('custom-model')
    })

    it('JSON 解析错误时抛出可理解的错误', async () => {
      // 存储无效 JSON
      localStorageMock.setItem('omo-config', 'invalid-json{')
      
      await expect(readConfig()).rejects.toThrow('配置文件读取失败')
    })
  })

  describe('configExists', () => {
    it('配置不存在时返回 false', () => {
      expect(configExists()).toBe(false)
    })

    it('配置存在时返回 true', () => {
      localStorageMock.setItem('omo-config', '{}')
      expect(configExists()).toBe(true)
    })
  })

  describe('writeConfig', () => {
    it('成功写入配置', async () => {
      const config = createDefaultConfig()
      config.agents.oracle.model = 'test-model'
      
      await writeConfig(config)
      
      const stored = localStorageMock.getItem('omo-config')
      expect(stored).toBeDefined()
      
      const parsed = JSON.parse(stored!)
      expect(parsed.agents.oracle.model).toBe('test-model')
    })
  })

  describe('deleteConfig', () => {
    it('成功删除配置', () => {
      localStorageMock.setItem('omo-config', '{}')
      expect(configExists()).toBe(true)
      
      deleteConfig()
      
      expect(configExists()).toBe(false)
    })
  })

  describe('完整流程', () => {
    it('写入后读取应该返回相同配置', async () => {
      const config = createDefaultConfig()
      config.agents.oracle.model = 'test-model'
      config.categories['visual-engineering'].model = 'visual-model'
      
      await writeConfig(config)
      const read = await readConfig()
      
      expect(read.agents.oracle.model).toBe('test-model')
      expect(read.categories['visual-engineering'].model).toBe('visual-model')
    })
  })
})
