/**
 * OhMyOpenCode 配置类型定义
 */

// Agent 名称类型（10个）
export type AgentName =
  | 'sisyphus'
  | 'hephaestus'
  | 'oracle'
  | 'librarian'
  | 'explore'
  | 'multimodal-looker'
  | 'prometheus'
  | 'metis'
  | 'momus'
  | 'atlas'

// Category 名称类型（8个）
export type CategoryName =
  | 'visual-engineering'
  | 'ultrabrain'
  | 'deep'
  | 'artistry'
  | 'quick'
  | 'unspecified-low'
  | 'unspecified-high'
  | 'writing'

// Agent 配置
export interface AgentConfig {
  model: string
}

// Category 配置
export interface CategoryConfig {
  model: string
}

// OhMyOpenCode 完整配置
export interface OhMyOpenCodeConfig {
  $schema?: string
  agents: Record<AgentName, AgentConfig>
  categories: Record<CategoryName, CategoryConfig>
}

// 预设类型
export interface Preset {
  name: string
  description?: string
  config: OhMyOpenCodeConfig
  createdAt: string
  updatedAt: string
}

// 模型类型
export interface Model {
  id: string // 格式: provider/model-name
  name: string // 显示名称
  provider: string // 提供商
}

// 默认 Agent 名称列表
export const AGENT_NAMES: AgentName[] = [
  'sisyphus',
  'hephaestus',
  'oracle',
  'librarian',
  'explore',
  'multimodal-looker',
  'prometheus',
  'metis',
  'momus',
  'atlas'
]

// 默认 Category 名称列表
export const CATEGORY_NAMES: CategoryName[] = [
  'visual-engineering',
  'ultrabrain',
  'deep',
  'artistry',
  'quick',
  'unspecified-low',
  'unspecified-high',
  'writing'
]

// Agent 显示名称和描述（来源：oh-my-opencode 官方文档）
export const AGENT_INFO: Record<AgentName, { displayName: string; description: string }> = {
  sisyphus: {
    displayName: 'Sisyphus',
    description: '默认主编排器。计划、委托并执行复杂任务，使用专门的子代理进行积极的并行执行。'
  },
  hephaestus: {
    displayName: 'Hephaestus',
    description: '自主深度工作者。给它目标而非步骤，它将自主探索代码库、研究模式并端到端执行。'
  },
  oracle: {
    displayName: 'Oracle',
    description: '架构决策、代码审查、调试。只读咨询代理，出色的逻辑推理和深度分析能力。'
  },
  librarian: {
    displayName: 'Librarian',
    description: '多仓库分析、文档查找、开源实现示例。当用户询问远程仓库代码、解释库内部原理或查找开源示例时必须使用。'
  },
  explore: {
    displayName: 'Explore',
    description: '快速代码库探索和上下文 grep。回答"X 在哪里？"、"哪个文件包含 Y？"等问题。'
  },
  'multimodal-looker': {
    displayName: 'Multimodal Looker',
    description: '分析需要超越原始文本解读的媒体文件（PDF、图像、图表）。当需要分析/提取数据而非原始文件内容时使用。'
  },
  prometheus: {
    displayName: 'Prometheus',
    description: '战略规划代理。通过迭代提问创建详细工作计划。'
  },
  metis: {
    displayName: 'Metis',
    description: '计划顾问。预规划分析，识别隐藏意图、歧义和 AI 失败点。'
  },
  momus: {
    displayName: 'Momus',
    description: '计划审查代理。根据清晰度、可验证性和完整性标准验证计划。'
  },
  atlas: {
    displayName: 'Atlas',
    description: '执行 Prometheus 计划。分发任务给专门的子代理，验证完成情况。指挥家而非演奏者。'
  }
}

// Category 显示名称和描述（来源：oh-my-opencode 官方文档）
export const CATEGORY_INFO: Record<CategoryName, { displayName: string; description: string }> = {
  'visual-engineering': {
    displayName: 'Visual Engineering',
    description: '前端、UI/UX、设计、样式、动画。'
  },
  ultrabrain: {
    displayName: 'Ultra Brain',
    description: '深度逻辑推理、复杂架构决策，需要大量分析。'
  },
  deep: {
    displayName: 'Deep',
    description: '目标导向的自主问题解决，行动前深入研究。'
  },
  artistry: {
    displayName: 'Artistry',
    description: '高度创意/艺术性任务、新颖想法。'
  },
  quick: {
    displayName: 'Quick',
    description: '琐碎任务 - 单文件更改、拼写修正、简单修改。'
  },
  'unspecified-low': {
    displayName: 'Unspecified Low',
    description: '不适合其他类别的任务，低工作量。'
  },
  'unspecified-high': {
    displayName: 'Unspecified High',
    description: '不适合其他类别的任务，高工作量。'
  },
  writing: {
    displayName: 'Writing',
    description: '文档、散文、技术写作。'
  }
}

// 创建默认配置
export function createDefaultConfig(): OhMyOpenCodeConfig {
  const defaultModel = 'wuwen/glm-5'

  const agents: Record<AgentName, AgentConfig> = {} as Record<AgentName, AgentConfig>
  const categories: Record<CategoryName, CategoryConfig> = {} as Record<CategoryName, CategoryConfig>

  for (const name of AGENT_NAMES) {
    agents[name] = { model: defaultModel }
  }

  for (const name of CATEGORY_NAMES) {
    categories[name] = { model: defaultModel }
  }

  return {
    $schema: 'https://raw.githubusercontent.com/code-yeongyu/oh-my-opencode/master/assets/oh-my-opencode.schema.json',
    agents,
    categories
  }
}

// 默认配置实例
export const defaultConfig: OhMyOpenCodeConfig = createDefaultConfig()

// ============ OpenCode 配置类型 (opencode.json) ============

/**
 * OpenCode 单个模型的配置
 * 来自 opencode.json 的 provider.{provider}.models.{modelId}
 */
export interface OpenCodeProviderModel {
  name: string
  limit?: {
    context?: number
    output?: number
  }
  modalities?: {
    input?: string[]
    output?: string[]
  }
  options?: Record<string, unknown>
  variants?: Record<string, unknown>
}

/**
 * OpenCode 单个 Provider 的配置
 * 来自 opencode.json 的 provider.{provider}
 */
export interface OpenCodeProvider {
  npm?: string
  name?: string
  options?: Record<string, unknown>
  models: Record<string, OpenCodeProviderModel>
}

/**
 * OpenCode 完整配置 (opencode.json)
 * 包含 provider 字段，用于提取可用模型列表
 */
export interface OpenCodeConfig {
  $schema?: string
  plugin?: string[]
  autoupdate?: boolean
  mcp?: Record<string, unknown>
  provider: Record<string, OpenCodeProvider>
}

// ============ OpenCode 模型注册表类型 (from ~/.cache/opencode/models.json) ============

/** 注册表中的单个模型元数据 */
export interface RegistryModel {
  id: string
  name: string
  family?: string
  attachment?: boolean
  reasoning?: boolean
  tool_call?: boolean
  release_date?: string
  last_updated?: string
  modalities?: {
    input?: string[]
    output?: string[]
  }
  open_weights?: boolean
  limit?: {
    context?: number
    output?: number
  }
  variants?: Record<string, unknown>
}

/** 注册表中的单个供应商 */
export interface RegistryProvider {
  id: string
  name?: string
  api?: string
  npm?: string
  env?: string[]
  doc?: string
  models: Record<string, RegistryModel>
}

/** 模型注册表整体数据（顶层 key 为 provider ID) */
export type ModelsRegistry = Record<string, RegistryProvider>
