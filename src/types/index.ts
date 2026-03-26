// 类型定义导出
export * from './config'

// 重新导出常用类型
export type {
  AgentName,
  CategoryName,
  AgentConfig,
  CategoryConfig,
  OhMyOpenCodeConfig,
  Preset,
  Model
} from './config'

export {
  AGENT_NAMES,
  CATEGORY_NAMES,
  AGENT_INFO,
  CATEGORY_INFO,
  createDefaultConfig,
  defaultConfig
} from './config'
