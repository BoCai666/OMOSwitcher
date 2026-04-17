/**
 * 额度格式化工具函数
 * 从 QuotaView.vue 中提取的可复用格式化逻辑
 */
import type { ProviderQuota } from '@/types/quota'
import { getProviderMetadata } from '@/data/providerMetadata'

// 格式化余额显示
export function formatBalance(value: number | null | undefined, currency?: string | null): string {
  if (value == null) return '--'
  const prefix = currency === 'USD' ? '$' : '¥'
  return `${prefix}${value.toFixed(2)}`
}

// 格式化 token 数量
export function formatTokens(value: number | null | undefined): string {
  if (value == null) return '--'
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return String(value)
}

// 格式化重置时间为人类可读倒计时
export function formatResetTime(resetTime: string | number | null | undefined): string {
  if (resetTime == null) return ''

  // 如果已经是可读字符串（非数字、非ISO格式），直接返回
  if (typeof resetTime === 'string') {
    const trimmed = resetTime.trim()
    // 特殊标记：滚动窗口
    if (trimmed === '5h-rolling') return '5小时滚动窗口'
    // 尝试解析为日期
    const parsed = new Date(trimmed)
    if (isNaN(parsed.getTime())) {
      // 无法解析为日期，当作已格式化的字符串直接返回
      return trimmed
    }
    // 成功解析为日期，继续走倒计时逻辑
    return formatCountdown(parsed)
  }

  // 数字类型（毫秒时间戳）
  if (typeof resetTime === 'number') {
    const date = new Date(resetTime)
    if (isNaN(date.getTime())) return ''
    return formatCountdown(date)
  }

  return ''
}

// 将目标日期格式化为倒计时或日期字符串
export function formatCountdown(targetDate: Date): string {
  const now = Date.now()
  const diff = targetDate.getTime() - now

  // 已过期
  if (diff <= 0) {
    return '已重置'
  }

  // 不到1小时
  if (diff < 3600_000) {
    const minutes = Math.ceil(diff / 60_000)
    return `${minutes}分钟后重置`
  }

  // 不到24小时
  if (diff < 86_400_000) {
    const hours = Math.floor(diff / 3600_000)
    const minutes = Math.ceil((diff % 3600_000) / 60_000)
    if (minutes >= 60) {
      return `${hours + 1}小时后重置`
    }
    return minutes > 0 ? `${hours}小时${minutes}分钟后重置` : `${hours}小时后重置`
  }

  // 超过24小时，显示日期
  const month = String(targetDate.getMonth() + 1).padStart(2, '0')
  const day = String(targetDate.getDate()).padStart(2, '0')
  const hour = String(targetDate.getHours()).padStart(2, '0')
  const minute = String(targetDate.getMinutes()).padStart(2, '0')
  return `重置于 ${month}-${day} ${hour}:${minute}`
}

// 获取供应商品牌色
export function getProviderColor(providerId: string): string {
  return getProviderMetadata(providerId).color
}

// 获取供应商完整元数据（图标、渐变等）
export function getProviderMeta(providerId: string) {
  return getProviderMetadata(providerId)
}

// 计算余额剩余百分比（用于进度条）
export function getBalancePercentage(quota: ProviderQuota): number {
  if (quota.totalBalance == null || quota.totalBalance === 0) return 0
  const used = quota.usedBalance ?? 0
  return Math.min(100, Math.max(0, ((quota.totalBalance - used) / quota.totalBalance) * 100))
}

// 进度条颜色
export function getProgressColor(percentage: number | null | undefined): string {
  if (percentage == null) return '#909399'
  if (percentage > 70) return '#67c23a'
  if (percentage > 30) return '#e6a23c'
  return '#f56c6c'
}
