/**
 * 额度类型定义
 * 归一化显示模型，不是原始 API 响应
 */

/**
 * 额度查询状态
 */
export type QuotaStatus = 'loading' | 'success' | 'error'

/**
 * 额度类型
 */
export type QuotaType = 'balance' | 'token_limit' | 'unsupported'

/**
 * 智谱 limits 数组中的单项
 */
export interface ZhipuLimitItem {
  /** 限制类型: TOKENS_LIMIT | TIME_LIMIT | RATE_LIMIT | TIMES_LIMIT | SESSION_LIMIT */
  type: string
  /** 已用百分比 (0-100) */
  percentage?: number
  /** 总额度 */
  usage?: number
  /** 当前已用 */
  currentValue?: number
  /** 剩余 */
  remaining?: number
  /** 下次重置时间 (ISO 时间戳字符串或毫秒时间戳) */
  nextResetTime?: string | number
}

/**
 * OpenCode Go limits 数组中的单项（三维度：rolling/weekly/monthly）
 */
export interface OpenCodeGoLimitItem {
  type: string
  label: string
  usagePercent: number
  resetInSec: number
  resetTime?: string
}

/**
 * 单个供应商的额度信息（统一格式）
 * 与 Rust 端 ProviderQuota 结构体一一对应
 */
export interface ProviderQuota {
  /** 供应商 ID */
  providerId: string
  /** 供应商显示名 */
  providerName: string
  /** 额度类型 */
  quotaType: QuotaType
  /** 查询状态 */
  status: QuotaStatus
  /** 错误信息 */
  errorMessage?: string | null

  // 余额型字段
  /** 总余额 */
  totalBalance?: number | null
  /** 可用余额 */
  availableBalance?: number | null
  /** 已用余额 */
  usedBalance?: number | null
  /** 货币类型 (CNY / USD) */
  currency?: string | null

  // 配额型字段
  /** 已用配额百分比 (0-100) */
  quotaPercentage?: number | null
  /** 已用 token */
  quotaUsed?: number | null
  /** 总限额 token */
  quotaLimit?: number | null
  /** 下次重置时间 */
  resetTime?: string | null

  // OpenRouter 专用
  /** 每日用量 (USD) */
  dailyUsage?: number | null
  /** 每周用量 (USD) */
  weeklyUsage?: number | null
  /** 每月用量 (USD) */
  monthlyUsage?: number | null
  /** 消费上限 (USD) */
  spendingLimit?: number | null
  /** 剩余额度 (USD) */
  limitRemaining?: number | null

  // 智谱 / OpenCode Go 专用
  /** 完整 limits 数组，供详情弹窗使用（字段因供应商而异） */
  limits?: any[] | null

  // Kimi Code 专用
  /** 是否为 Kimi Code 平台 */
  isKimiCode?: boolean | null
  /** Kimi Code 额度详情（5小时、周、总） */
  kimiCodeUsage?: {
    /** 5小时滚动窗口额度 */
    fiveHour?: {
      limit: number
      used: number
      remaining: number
      resetTime: string
    }
    /** 周额度 */
    weekly?: {
      limit: number
      used: number
      remaining: number
      resetTime: string
    }
    /** 总额度 */
    total?: {
      limit: number
      remaining: number
    }
  } | null
}

/**
 * 单个模型的 7 天 token 汇总
 */
export interface ModelSummaryItem {
  /** 模型名称 (如 "GLM-5.1") */
  modelName: string
  /** 该模型 7 天总 token 数 */
  totalTokens: number
}

/**
 * 模型用量汇总（来自 /api/monitor/usage/model-usage 的 totalUsage）
 */
export interface ModelUsageSummary {
  /** 总调用次数 */
  totalCalls: number
  /** 总 token 消耗 */
  totalTokens: number
  /** 各模型 token 消耗明细 */
  modelList: ModelSummaryItem[]
}

/**
 * 智谱用量详情（点击卡片时查询）
 */
export interface ZhipuUsageDetails {
  /** 供应商 ID */
  providerId: string
  /** 今日模型用量汇总 */
  todayModelUsage: ModelUsageSummary
  /** 模型用量汇总 (近 7 天) */
  modelUsage: ModelUsageSummary
}