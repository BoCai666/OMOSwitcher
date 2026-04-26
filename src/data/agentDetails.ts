/**
 * Agent 详情数据
 * 系统提示词已迁移至 public/prompts/agents/ 目录下动态加载
 * 数据来源: oh-my-openagent 官方源码
 */

import type { AgentName } from '@/types'

export interface FallbackModel {
  providers: string[]
  model: string
  variant?: string
}

export interface AgentDetail {
  name: AgentName
  displayName: string
  description: string
  recommendedModel: string
  fallbackChain: FallbackModel[]
  triggers: string[]
  useWhen: string[]
  avoidWhen: string[]
}

export const AGENT_DETAILS: Record<AgentName, AgentDetail> = {
  sisyphus: {
    name: 'sisyphus',
    displayName: 'Sisyphus',
    description: 'Powerful AI orchestrator. Plans obsessively with todos, assesses search complexity before exploration, delegates strategically via category+skills combinations. Uses explore for internal code (parallel-friendly), librarian for external docs. (Sisyphus - OhMyOpenCode)',
    recommendedModel: 'anthropic/claude-opus-4-7',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['opencode-go', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['kimi-for-coding'], model: 'k2p5' },
      { providers: ['opencode', 'moonshotai', 'moonshotai-cn', 'firmware', 'ollama-cloud', 'aihubmix', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'medium' },
      { providers: ['zai-coding-plan', 'opencode', 'vercel'], model: 'glm-5' },
      { providers: ['opencode'], model: 'big-pickle' }
    ],
    triggers: [
      '外部库/源被提及时启动 librarian 后台任务',
      '涉及 2 个以上模块时启动 explore 后台任务',
      '模糊或复杂的请求在 Prometheus 之前咨询 Metis',
      '工作计划保存到 .sisyphus/plans/*.md 时调用 Momus'
    ],
    useWhen: [
      '需要编排多个子代理的复杂任务',
      '需要并行执行以提高吞吐量',
      '需要解析隐式需求并做出路由决策'
    ],
    avoidWhen: [
      '简单的单文件操作',
      '用户明确想要自己控制流程'
    ]
  },

  hephaestus: {
    name: 'hephaestus',
    displayName: 'Hephaestus',
    description: 'Autonomous Deep Worker - goal-oriented execution with GPT Codex. Explores thoroughly before acting, uses explore/librarian agents for comprehensive context, completes tasks end-to-end. Inspired by AmpCode deep mode. (Hephaestus - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'github-copilot', 'venice', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'medium' }
    ],
    triggers: [
      '端到端任务完成，不提前停止',
      '多步骤实现需要彻底探索'
    ],
    useWhen: [
      '任务需要在实现前深度探索',
      '用户想要自主端到端完成',
      '需要复杂的多文件更改'
    ],
    avoidWhen: [
      '简单的单步任务',
      '需要在每一步确认的任务',
      '需要跨多个代理编排时（使用 Atlas）'
    ]
  },

  oracle: {
    name: 'oracle',
    displayName: 'Oracle',
    description: 'Read-only consultation agent. High-IQ reasoning specialist for debugging hard problems and high-difficulty architecture design. (Oracle - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'high' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' }
    ],
    triggers: [
      '架构决策、多系统权衡、不熟悉的模式',
      '完成重要实现后的自我审查',
      '2 次以上修复尝试失败后的调试'
    ],
    useWhen: [
      '复杂的架构设计',
      '完成重要工作后',
      '2 次以上修复尝试失败',
      '不熟悉的代码模式',
      '安全/性能问题',
      '多系统权衡'
    ],
    avoidWhen: [
      '简单的文件操作（使用直接工具）',
      '首次修复尝试（先自己尝试）',
      '可以从已读代码中回答的问题',
      '琐碎决策（变量名、格式化）',
      '可以从现有代码模式推断的事情'
    ]
  },

  librarian: {
    name: 'librarian',
    displayName: 'Librarian',
    description: 'Specialized codebase understanding agent for multi-repository analysis, searching remote codebases, retrieving official documentation, and finding implementation examples using GitHub CLI, Context7, and Web Search. MUST BE USED when users ask to look up code in remote repositories, explain library internals, or find usage examples in open source. (Librarian - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.4-mini-fast',
    fallbackChain: [
      { providers: ['openai'], model: 'gpt-5.4-mini-fast' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7-highspeed' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' },
      { providers: ['anthropic', 'opencode', 'vercel'], model: 'claude-haiku-4-5' },
      { providers: ['openai', 'opencode', 'vercel'], model: 'gpt-5.4-nano' }
    ],
    triggers: [
      '不熟悉的包/库，在奇怪行为时查找现有开源实现'
    ],
    useWhen: [
      '"如何使用 [库]？"',
      '"[框架功能] 的最佳实践是什么？"',
      '"为什么 [外部依赖] 会这样行为？"',
      '"查找 [库] 使用示例"',
      '使用不熟悉的 npm/pip/cargo 包'
    ],
    avoidWhen: [
      '已知位置的简单文件读取',
      '不需要外部参考的问题'
    ]
  },

  explore: {
    name: 'explore',
    displayName: 'Explore',
    description: 'Contextual grep for codebases. Answers "Where is X?", "Which file has Y?", "Find the code that does Z". Fire multiple in parallel for broad searches. Specify thoroughness: "quick" for basic, "medium" for moderate, "very thorough" for comprehensive analysis. (Explore - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.4-mini-fast',
    fallbackChain: [
      { providers: ['openai'], model: 'gpt-5.4-mini-fast' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7-highspeed' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' },
      { providers: ['anthropic', 'opencode', 'vercel'], model: 'claude-haiku-4-5' },
      { providers: ['openai', 'opencode', 'vercel'], model: 'gpt-5.4-nano' }
    ],
    triggers: [
      '查找现有代码库结构、模式和样式'
    ],
    useWhen: [
      '需要多个搜索角度',
      '不熟悉的模块结构',
      '跨层模式发现'
    ],
    avoidWhen: [
      '你确切知道要搜索什么',
      '单个关键词/模式就足够',
      '已知文件位置'
    ]
  },

  'multimodal-looker': {
    name: 'multimodal-looker',
    displayName: 'Multimodal Looker',
    description: 'Analyze media files (PDFs, images, diagrams) that require interpretation beyond raw text. Extracts specific information or summaries from documents, describes visual content. Use when you need analyzed/extracted data rather than literal file contents. (Multimodal-Looker - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'medium' },
      { providers: ['opencode-go', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['zai-coding-plan', 'vercel'], model: 'glm-4.6v' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5-nano' }
    ],
    triggers: [],
    useWhen: [
      '需要分析无法作为纯文本读取的媒体文件',
      '从文档中提取特定信息或摘要',
      '描述图像或图表中的视觉内容'
    ],
    avoidWhen: [
      '需要读取精确内容的源代码或纯文本文件（使用 Read）',
      '之后需要编辑的文件（需要 Read 的原始内容）'
    ]
  },

  prometheus: {
    name: 'prometheus',
    displayName: 'Prometheus',
    description: 'Strategic planning consultant. Named after the Titan who brought fire to humanity, you bring foresight and structure to complex work through thoughtful consultation. (Prometheus - OhMyOpenCode)',
    recommendedModel: 'anthropic/claude-opus-4-7',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'high' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro' }
    ],
    triggers: [
      '需要创建详细工作计划',
      '复杂任务需要分解'
    ],
    useWhen: [
      '需要创建工作计划',
      '复杂任务需要迭代澄清',
      '需要生成带有依赖图的任务分解'
    ],
    avoidWhen: [
      '简单的单任务请求',
      '用户明确想跳过计划'
    ]
  },

  metis: {
    name: 'metis',
    displayName: 'Metis',
    description: 'Pre-planning consultant that analyzes requests to identify hidden intentions, ambiguities, and AI failure points. (Metis - OhMyOpenCode)',
    recommendedModel: 'anthropic/claude-opus-4-7',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'high' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' },
      { providers: ['kimi-for-coding'], model: 'k2p5' }
    ],
    triggers: [
      '复杂任务需要范围澄清，需求模糊'
    ],
    useWhen: [
      '在规划非平凡任务之前',
      '用户请求模糊或开放式时',
      '防止 AI 过度工程模式'
    ],
    avoidWhen: [
      '简单、明确定义的任务',
      '用户已提供详细需求'
    ]
  },

  momus: {
    name: 'momus',
    displayName: 'Momus',
    description: 'Expert reviewer for evaluating work plans against rigorous clarity, verifiability, and completeness standards. (Momus - OhMyOpenCode)',
    recommendedModel: 'openai/gpt-5.5',
    fallbackChain: [
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'xhigh' },
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-opus-4-7', variant: 'max' },
      { providers: ['google', 'github-copilot', 'opencode', 'vercel'], model: 'gemini-3.1-pro', variant: 'high' },
      { providers: ['opencode-go', 'vercel'], model: 'glm-5' }
    ],
    triggers: [
      '根据清晰度、可验证性和完整性标准评估工作计划',
      '在实现之前捕获差距、歧义和缺失上下文'
    ],
    useWhen: [
      'Prometheus 创建工作计划后',
      '执行复杂待办列表之前',
      '委托给执行者之前验证计划质量',
      '当计划需要严格审查以发现 ADHD 驱动的遗漏时'
    ],
    avoidWhen: [
      '简单、单任务请求',
      '用户明确想跳过审查',
      '不需要正式审查的琐碎计划'
    ]
  },

  atlas: {
    name: 'atlas',
    displayName: 'Atlas',
    description: 'Orchestrates work via task() to complete ALL tasks in a todo list until fully done. (Atlas - OhMyOpenCode)',
    recommendedModel: 'anthropic/claude-sonnet-4-6',
    fallbackChain: [
      { providers: ['anthropic', 'github-copilot', 'opencode', 'vercel'], model: 'claude-sonnet-4-6' },
      { providers: ['opencode-go', 'vercel'], model: 'kimi-k2.5' },
      { providers: ['openai', 'github-copilot', 'opencode', 'vercel'], model: 'gpt-5.5', variant: 'medium' },
      { providers: ['opencode-go', 'vercel'], model: 'minimax-m2.7' }
    ],
    triggers: [
      '完成待办列表中的所有任务并进行验证',
      '跨专业代理的并行任务执行'
    ],
    useWhen: [
      '用户提供了待办列表路径',
      '多个任务需要按顺序或并行完成',
      '工作需要跨多个专业代理协调'
    ],
    avoidWhen: [
      '不需要编排的单个简单任务',
      '可以由一个代理直接处理的任务',
      '用户想要手动执行任务'
    ]
  }
}
