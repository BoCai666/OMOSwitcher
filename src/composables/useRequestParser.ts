/**
 * 请求体解析工具函数
 * 提供消息相关的通用计算和格式化能力
 */
import { User, ChatDotRound, Setting, Document, Tools } from '@element-plus/icons-vue'
import type { Component } from 'vue'

// 估算文本的 token 数量
export function estimateTokens(text: string): number {
  if (!text) return 0

  // 简单估算规则：
  // - 中文字符：约 1.5 tokens/字符
  // - 英文/数字/符号：约 0.25 tokens/字符 (4字符=1token)
  let tokenCount = 0
  for (const char of text) {
    const code = char.charCodeAt(0)
    // 中文字符范围
    if (code >= 0x4e00 && code <= 0x9fff) {
      tokenCount += 1.5
    } else if (code >= 0x3400 && code <= 0x4dbf) {
      // 扩展中文字符
      tokenCount += 1.5
    } else {
      tokenCount += 0.25
    }
  }
  return Math.ceil(tokenCount)
}

// 格式化 token 数量显示
export function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return (tokens / 1000000).toFixed(1).replace(/\.0$/, '') + 'M'
  }
  if (tokens >= 1000) {
    return (tokens / 1000).toFixed(1).replace(/\.0$/, '') + 'K'
  }
  return tokens.toString()
}

// 获取角色标签类型
export function getRoleTagType(role: string): string {
  const types: Record<string, string> = {
    system: 'warning',
    user: 'primary',
    assistant: 'success',
    tool: 'info'
  }
  return types[role] || 'info'
}

// 获取角色图标
export function getRoleIcon(role: string): Component {
  const icons: Record<string, Component> = {
    system: Setting,
    user: User,
    assistant: ChatDotRound,
    tool: Tools
  }
  return icons[role] || Document
}

// 获取角色显示名称
export function getRoleDisplayName(role: string): string {
  const names: Record<string, string> = {
    system: '系统',
    user: '用户',
    assistant: '助手',
    tool: '工具'
  }
  return names[role] || role
}
