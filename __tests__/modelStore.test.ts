/**
 * modelStore 测试文件
 */

import { describe, it, expect, beforeEach } from 'vitest'
import {
  listModels,
  addModel,
  updateModel,
  deleteModel,
  resetModels,
  validateModelId,
  parseProvider,
  getModelById,
  modelExists,
  getDefaultModels
} from '@/services/modelStore'
import type { Model } from '@/types'

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

describe('modelStore', () => {
  beforeEach(() => {
    // 每次测试前清空 localStorage
    localStorageMock.clear()
  })

  describe('validateModelId', () => {
    it('有效的模型 ID 应该通过验证', () => {
      expect(validateModelId('wuwen/glm-5')).toBe(true)
      expect(validateModelId('openai/gpt-4')).toBe(true)
      expect(validateModelId('anthropic/claude-3-opus')).toBe(true)
      expect(validateModelId('provider-1/model_v2.0')).toBe(true)
    })

    it('无效的模型 ID 应该被拒绝', () => {
      // 缺少斜杠
      expect(validateModelId('invalid-model-id')).toBe(false)
      // 多个斜杠
      expect(validateModelId('provider/model/name')).toBe(false)
      // 特殊字符
      expect(validateModelId('provider/model@name')).toBe(false)
      // 空字符串
      expect(validateModelId('')).toBe(false)
      // 只有斜杠
      expect(validateModelId('/')).toBe(false)
      // 斜杠开头
      expect(validateModelId('/model')).toBe(false)
      // 斜杠结尾
      expect(validateModelId('provider/')).toBe(false)
    })
  })

  describe('parseProvider', () => {
    it('正确解析 provider', () => {
      expect(parseProvider('wuwen/glm-5')).toBe('wuwen')
      expect(parseProvider('openai/gpt-4')).toBe('openai')
      expect(parseProvider('provider-1/model')).toBe('provider-1')
    })

    it('无效 ID 返回 null', () => {
      expect(parseProvider('invalid')).toBe(null)
      expect(parseProvider('')).toBe(null)
    })
  })

  describe('listModels', () => {
    it('没有存储时返回默认模型列表', () => {
      const models = listModels()
      
      expect(models.length).toBe(4)
      expect(models[0].id).toBe('wuwen/glm-5')
      expect(models[1].id).toBe('wuwen/minimax-m2.5')
      expect(models[2].id).toBe('wuwen/minimax-m2.7')
      expect(models[3].id).toBe('wuwen/kimi-k2.5')
    })

    it('返回存储的模型列表', () => {
      const customModels: Model[] = [
        { id: 'custom/model-1', name: 'Custom Model', provider: 'custom' }
      ]
      localStorageMock.setItem('omo-models', JSON.stringify(customModels))
      
      const models = listModels()
      
      expect(models.length).toBe(1)
      expect(models[0].id).toBe('custom/model-1')
    })

    it('JSON 解析错误时返回默认模型列表', () => {
      localStorageMock.setItem('omo-models', 'invalid-json{')
      
      const models = listModels()
      
      expect(models.length).toBe(4)
      expect(models[0].id).toBe('wuwen/glm-5')
    })
  })

  describe('addModel', () => {
    it('成功添加模型', () => {
      const model: Model = {
        id: 'openai/gpt-4',
        name: 'GPT-4',
        provider: 'openai'
      }
      
      const result = addModel(model)
      
      expect(result).toEqual(model)
      
      const models = listModels()
      expect(models.length).toBe(5)
      expect(models.find(m => m.id === 'openai/gpt-4')).toBeDefined()
    })

    it('无效 ID 格式时抛出错误', () => {
      const model: Model = {
        id: 'invalid-id',
        name: 'Invalid Model',
        provider: 'invalid'
      }
      
      expect(() => addModel(model)).toThrow('模型 ID 格式无效')
    })

    it('模型已存在时抛出错误', () => {
      const model: Model = {
        id: 'wuwen/glm-5',
        name: 'GLM-5 Duplicate',
        provider: 'wuwen'
      }
      
      expect(() => addModel(model)).toThrow('该模型 ID 已存在')
    })
  })

  describe('updateModel', () => {
    it('成功更新模型', () => {
      const updated = updateModel('wuwen/glm-5', { name: 'GLM-5 Updated' })
      
      expect(updated).toBeDefined()
      expect(updated!.name).toBe('GLM-5 Updated')
      expect(updated!.id).toBe('wuwen/glm-5')
      expect(updated!.provider).toBe('wuwen')
    })

    it('模型不存在时返回 null', () => {
      const result = updateModel('nonexistent/model', { name: 'New Name' })
      expect(result).toBeNull()
    })
  })

  describe('deleteModel', () => {
    it('成功删除模型', () => {
      const result = deleteModel('wuwen/glm-5')
      
      expect(result).toBe(true)
      
      const models = listModels()
      expect(models.length).toBe(3)
      expect(models.find(m => m.id === 'wuwen/glm-5')).toBeUndefined()
    })

    it('模型不存在时返回 false', () => {
      const result = deleteModel('nonexistent/model')
      expect(result).toBe(false)
    })
  })

  describe('resetModels', () => {
    it('重置为默认模型列表', () => {
      // 先添加一些自定义模型
      addModel({ id: 'custom/model', name: 'Custom', provider: 'custom' })
      expect(listModels().length).toBe(5)
      
      // 重置
      const models = resetModels()
      
      expect(models.length).toBe(4)
      expect(models[0].id).toBe('wuwen/glm-5')
      
      // 确认存储也被重置
      const storedModels = listModels()
      expect(storedModels.length).toBe(4)
    })
  })

  describe('getModelById', () => {
    it('根据 ID 获取模型', () => {
      const model = getModelById('wuwen/glm-5')
      
      expect(model).toBeDefined()
      expect(model!.name).toBe('GLM-5')
    })

    it('模型不存在时返回 undefined', () => {
      const model = getModelById('nonexistent/model')
      expect(model).toBeUndefined()
    })
  })

  describe('modelExists', () => {
    it('模型存在时返回 true', () => {
      expect(modelExists('wuwen/glm-5')).toBe(true)
    })

    it('模型不存在时返回 false', () => {
      expect(modelExists('nonexistent/model')).toBe(false)
    })
  })

  describe('getDefaultModels', () => {
    it('返回默认模型列表', () => {
      const models = getDefaultModels()
      
      expect(models.length).toBe(4)
      expect(models[0].id).toBe('wuwen/glm-5')
    })

    it('返回的是副本，修改不影响原数组', () => {
      const models1 = getDefaultModels()
      const models2 = getDefaultModels()
      
      models1.push({ id: 'test/model', name: 'Test', provider: 'test' })
      
      expect(models1.length).toBe(5)
      expect(models2.length).toBe(4)
    })
  })

  describe('完整流程', () => {
    it('添加、更新、删除模型流程', () => {
      // 初始状态
      expect(listModels().length).toBe(4)
      
      // 添加
      const newModel: Model = {
        id: 'openai/gpt-4-turbo',
        name: 'GPT-4 Turbo',
        provider: 'openai'
      }
      addModel(newModel)
      expect(listModels().length).toBe(5)
      expect(modelExists('openai/gpt-4-turbo')).toBe(true)
      
      // 更新
      updateModel('openai/gpt-4-turbo', { name: 'GPT-4 Turbo Updated' })
      expect(getModelById('openai/gpt-4-turbo')!.name).toBe('GPT-4 Turbo Updated')
      
      // 删除
      deleteModel('openai/gpt-4-turbo')
      expect(listModels().length).toBe(4)
      expect(modelExists('openai/gpt-4-turbo')).toBe(false)
    })

    it('重置后恢复默认模型', () => {
      // 清空所有默认模型
      deleteModel('wuwen/glm-5')
      deleteModel('wuwen/minimax-m2.5')
      deleteModel('wuwen/minimax-m2.7')
      deleteModel('wuwen/kimi-k2.5')
      expect(listModels().length).toBe(0)
      
      // 重置
      resetModels()
      expect(listModels().length).toBe(4)
      expect(modelExists('wuwen/glm-5')).toBe(true)
    })
  })
})
