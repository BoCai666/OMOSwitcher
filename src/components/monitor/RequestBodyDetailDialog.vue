<script setup lang="ts">
/**
 * 请求体详情弹窗
 * 以可视化友好的方式展示大模型请求内容
 */
import { computed, ref } from 'vue'
import { User, ChatDotRound, Setting, Document, Tools, VideoPlay, ArrowRight, Cpu } from '@element-plus/icons-vue'

const props = defineProps<{
  visible: boolean
  requestBody: any
  actualTokens?: number
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 解析请求体
const parsedBody = computed(() => {
  if (!props.requestBody) return null
  
  // 如果是字符串，尝试解析
  let body = props.requestBody
  if (typeof body === 'string') {
    try {
      body = JSON.parse(body)
    } catch {
      return null
    }
  }
  
  return body
})

// 模型名称
const modelName = computed(() => parsedBody.value?.model || '-')

// 估算文本的 token 数量
function estimateTokens(text: string): number {
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
function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return (tokens / 1000000).toFixed(1).replace(/\.0$/, '') + 'M'
  }
  if (tokens >= 1000) {
    return (tokens / 1000).toFixed(1).replace(/\.0$/, '') + 'K'
  }
  return tokens.toString()
}

// 消息列表
const messages = computed(() => {
  const msgs = parsedBody.value?.messages || []
  return msgs.map((msg: any, index: number) => {
    // 解析 content，支持字符串和数组格式
    let contentStr = ''
    let thinkingBlocks: Array<{ type: string; text: string }> = []
    
    // 处理 null/undefined
    if (msg.content === null || msg.content === undefined) {
      contentStr = ''
    } else if (typeof msg.content === 'string') {
      contentStr = msg.content
    } else if (Array.isArray(msg.content)) {
      // Anthropic 格式：content 数组可能包含 thinking 和 text 块
      const textParts: string[] = []
      for (const block of msg.content) {
        if (block.type === 'thinking' && block.thinking) {
          thinkingBlocks.push({ type: 'thinking', text: block.thinking })
        } else if (block.type === 'text' && block.text) {
          textParts.push(block.text)
        } else if (block.type === 'redacted_thinking') {
          thinkingBlocks.push({ type: 'redacted_thinking', text: '[思考内容已隐藏]' })
        } else if (block.type && block.content !== undefined) {
          // 其他类型的块（如 image）
          textParts.push(`[${block.type}]`)
        }
      }
      contentStr = textParts.join('\n')
    } else if (typeof msg.content === 'object') {
      // 单个 content 对象（可能是一个 block）
      if (msg.content.type === 'thinking' && msg.content.thinking) {
        thinkingBlocks.push({ type: 'thinking', text: msg.content.thinking })
      } else if (msg.content.type === 'text' && msg.content.text) {
        contentStr = msg.content.text
      } else {
        contentStr = JSON.stringify(msg.content, null, 2)
      }
    } else {
      contentStr = String(msg.content)
    }
    
    return {
      index: index + 1,
      role: msg.role || 'unknown',
      content: contentStr,
      thinkingBlocks, // 思考块
      name: msg.name,
      toolCallId: msg.tool_call_id,
      tokens: estimateTokens(contentStr + thinkingBlocks.map(b => b.text).join(''))
    }
  })
})

// 工具定义
const tools = computed(() => {
  return parsedBody.value?.tools || []
})

// 工具调用列表
const toolCalls = computed(() => {
  // 从最后一条 assistant 消息中获取 tool_calls
  const msgs = parsedBody.value?.messages || []
  const lastAssistantMsg = [...msgs].reverse().find((m: any) => m.role === 'assistant')
  return lastAssistantMsg?.tool_calls || []
})

// 参数设置
const parameters = computed(() => {
  const body = parsedBody.value || {}
  const params: { key: string; value: any; type: string }[] = []
  
  const paramKeys = [
    'temperature', 'max_tokens', 'top_p', 'top_k', 'frequency_penalty',
    'presence_penalty', 'stream', 'n', 'stop', 'seed', 'response_format',
    'logprobs', 'top_logprobs'
  ]
  
  for (const key of paramKeys) {
    if (body[key] !== undefined) {
      let type = 'value'
      if (typeof body[key] === 'boolean') type = 'boolean'
      if (Array.isArray(body[key])) type = 'array'
      if (key === 'response_format') type = 'object'
      
      params.push({
        key,
        value: body[key],
        type
      })
    }
  }
  
  return params
})

// 思考参数
const thinkingParams = computed(() => {
  const body = parsedBody.value || {}
  const params: { key: string; value: any; type: string; label: string }[] = []
  
  // reasoning_effort (OpenAI o1/o3)
  if (body.reasoning_effort !== undefined) {
    const labels: Record<string, string> = { low: '低', medium: '中', high: '高' }
    params.push({
      key: 'reasoning_effort',
      value: body.reasoning_effort,
      type: 'select',
      label: labels[body.reasoning_effort] || body.reasoning_effort
    })
  }
  
  // thinking (Anthropic)
  if (body.thinking !== undefined) {
    if (typeof body.thinking === 'object' && body.thinking.type) {
      params.push({
        key: 'thinking',
        value: body.thinking.type,
        type: 'select',
        label: body.thinking.type === 'enabled' ? '已启用' : body.thinking.type
      })
      if (body.thinking.budget_tokens !== undefined) {
        params.push({
          key: 'thinking.budget_tokens',
          value: body.thinking.budget_tokens,
          type: 'tokens',
          label: `${body.thinking.budget_tokens} tokens`
        })
      }
    }
  }
  
  // thinking_budget
  if (body.thinking_budget !== undefined) {
    params.push({
      key: 'thinking_budget',
      value: body.thinking_budget,
      type: 'tokens',
      label: `${body.thinking_budget} tokens`
    })
  }
  
  // extended_thinking
  if (body.extended_thinking !== undefined) {
    const val = typeof body.extended_thinking === 'boolean'
      ? (body.extended_thinking ? '已启用' : '已禁用')
      : String(body.extended_thinking)
    params.push({
      key: 'extended_thinking',
      value: body.extended_thinking,
      type: 'boolean',
      label: val
    })
  }
  
  return params
})

// 获取角色标签类型
function getRoleTagType(role: string): string {
  const types: Record<string, string> = {
    system: 'warning',
    user: 'primary',
    assistant: 'success',
    tool: 'info'
  }
  return types[role] || 'info'
}

// 获取角色图标
function getRoleIcon(role: string) {
  const icons: Record<string, any> = {
    system: Setting,
    user: User,
    assistant: ChatDotRound,
    tool: Tools
  }
  return icons[role] || Document
}

// 获取角色显示名称
function getRoleDisplayName(role: string): string {
  const names: Record<string, string> = {
    system: '系统',
    user: '用户',
    assistant: '助手',
    tool: '工具'
  }
  return names[role] || role
}

// 格式化参数值显示
function formatParamValue(value: any, type: string): string {
  if (type === 'boolean') return value ? '是' : '否'
  if (type === 'array') return value.join(', ')
  if (type === 'object') return JSON.stringify(value)
  return String(value)
}

// 当前展开的消息
const expandedMessages = ref<number[]>([])
const expandedToolIndexes = ref<number[]>([])
const expandedToolCallIndexes = ref<number[]>([])

// 区块折叠状态（默认折叠）
const messagesCollapsed = ref(true)
const toolsCollapsed = ref(true)
const toolCallsCollapsed = ref(true)

// 切换消息展开状态
function toggleMessage(index: number) {
  const idx = expandedMessages.value.indexOf(index)
  if (idx > -1) {
    expandedMessages.value.splice(idx, 1)
  } else {
    expandedMessages.value.push(index)
  }
}

// 判断消息是否展开
function isMessageExpanded(index: number): boolean {
  return expandedMessages.value.includes(index)
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="请求体详情"
    width="900px"
    append-to=".app-main"
    align-center
    class="body-detail-dialog"
    destroy-on-close
  >
    <div class="body-detail-content">
      <!-- 模型信息 -->
      <div class="section model-section">
        <div class="section-header">
          <el-icon class="section-icon"><VideoPlay /></el-icon>
          <span class="section-title">模型</span>
        </div>
        <div class="section-body">
          <el-tag size="large" effect="dark" class="model-tag">
            {{ modelName }}
          </el-tag>
        </div>
      </div>

      <!-- 参数设置 -->
      <div v-if="parameters.length > 0" class="section params-section">
        <div class="section-header">
          <el-icon class="section-icon"><Setting /></el-icon>
          <span class="section-title">参数设置</span>
        </div>
        <div class="section-body">
          <div class="params-grid">
            <div v-for="param in parameters" :key="param.key" class="param-item">
              <span class="param-key">{{ param.key }}</span>
              <span class="param-value" :class="param.type">
                {{ formatParamValue(param.value, param.type) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- 思考参数 -->
      <div v-if="thinkingParams.length > 0" class="section thinking-params-section">
        <div class="section-header">
          <el-icon class="section-icon thinking-icon"><Cpu /></el-icon>
          <span class="section-title">思考参数</span>
        </div>
        <div class="section-body">
          <div class="params-grid">
            <div v-for="param in thinkingParams" :key="param.key" class="param-item thinking-param">
              <span class="param-key">{{ param.key }}</span>
              <span class="param-value thinking-value">{{ param.label }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 消息列表（可折叠） -->
      <div v-if="messages.length > 0" class="section messages-section collapsible-section">
        <div class="section-header clickable" @click="messagesCollapsed = !messagesCollapsed">
          <el-icon class="section-icon"><ChatDotRound /></el-icon>
          <span class="section-title">消息列表</span>
          <span class="section-badge">{{ messages.length }}</span>
          <el-icon class="collapse-arrow" :class="{ expanded: !messagesCollapsed }">
            <ArrowRight />
          </el-icon>
        </div>
        <Transition name="collapse-section">
          <div v-show="!messagesCollapsed" class="section-body">
            <div class="messages-list">
              <div
                v-for="msg in messages"
                :key="msg.index"
                class="message-item"
                :class="[`role-${msg.role}`]"
              >
                <div class="message-header" @click="toggleMessage(msg.index)">
                  <div class="message-role">
                    <el-icon class="role-icon"><component :is="getRoleIcon(msg.role)" /></el-icon>
                    <el-tag :type="getRoleTagType(msg.role)" size="small" effect="dark">
                      {{ getRoleDisplayName(msg.role) }}
                    </el-tag>
                    <span v-if="msg.name" class="message-name">{{ msg.name }}</span>
                  </div>
                  <div class="message-meta">
                    <span class="message-tokens">~{{ formatTokens(msg.tokens) }} token</span>
                    <span class="message-index">#{{ msg.index }}</span>
                    <el-icon class="expand-icon" :class="{ expanded: isMessageExpanded(msg.index) }">
                      <ArrowRight />
                    </el-icon>
                  </div>
                </div>
                <Transition name="collapse">
                  <div v-if="isMessageExpanded(msg.index)" class="message-content-wrapper">
                    <!-- 思考块 -->
                    <div v-if="msg.thinkingBlocks && msg.thinkingBlocks.length > 0" class="thinking-blocks">
                      <div v-for="(block, blockIndex) in msg.thinkingBlocks" :key="blockIndex" class="thinking-block">
                        <div class="thinking-header">
                          <el-icon class="thinking-icon"><Cpu /></el-icon>
                          <span class="thinking-label">{{ block.type === 'thinking' ? '思考过程' : '隐藏思考' }}</span>
                        </div>
                        <pre class="thinking-content">{{ block.text }}</pre>
                      </div>
                    </div>
                    <!-- 消息内容 -->
                    <div v-if="msg.content" class="message-content">
                      <pre>{{ msg.content }}</pre>
                    </div>
                  </div>
                </Transition>
              </div>
            </div>
          </div>
        </Transition>
      </div>

      <!-- 工具调用（可折叠，放在工具定义上方） -->
      <div v-if="toolCalls.length > 0" class="section tool-calls-section collapsible-section">
        <div class="section-header clickable" @click="toolCallsCollapsed = !toolCallsCollapsed">
          <el-icon class="section-icon"><Tools /></el-icon>
          <span class="section-title">工具调用</span>
          <span class="section-badge">{{ toolCalls.length }}</span>
          <el-icon class="collapse-arrow" :class="{ expanded: !toolCallsCollapsed }">
            <ArrowRight />
          </el-icon>
        </div>
        <Transition name="collapse-section">
          <div v-show="!toolCallsCollapsed" class="section-body">
            <el-collapse v-model="expandedToolCallIndexes" class="tool-calls-collapse">
              <el-collapse-item
                v-for="(call, index) in toolCalls"
                :key="index"
                :name="index"
                class="tool-call-item"
              >
                <template #title>
                  <div class="tool-call-title">
                    <span class="call-id">{{ call.id?.slice(0, 8) || '#' + (index + 1) }}</span>
                    <span class="call-name">{{ call.function?.name || '未知工具' }}</span>
                  </div>
                </template>
                <div class="tool-call-details">
                  <div v-if="call.id" class="call-detail-item">
                    <span class="detail-label">调用 ID</span>
                    <code class="detail-code">{{ call.id }}</code>
                  </div>
                  <div v-if="call.type" class="call-detail-item">
                    <span class="detail-label">类型</span>
                    <el-tag size="small">{{ call.type }}</el-tag>
                  </div>
                  <div v-if="call.function?.arguments" class="call-detail-item">
                    <span class="detail-label">参数</span>
                    <div class="code-block">
                      <pre>{{ typeof call.function.arguments === 'string' 
                        ? JSON.stringify(JSON.parse(call.function.arguments), null, 2) 
                        : JSON.stringify(call.function.arguments, null, 2) }}</pre>
                    </div>
                  </div>
                </div>
              </el-collapse-item>
            </el-collapse>
          </div>
        </Transition>
      </div>

      <!-- 工具定义（可折叠） -->
      <div v-if="tools.length > 0" class="section tools-section collapsible-section">
        <div class="section-header clickable" @click="toolsCollapsed = !toolsCollapsed">
          <el-icon class="section-icon"><Tools /></el-icon>
          <span class="section-title">工具定义</span>
          <span class="section-badge">{{ tools.length }}</span>
          <el-icon class="collapse-arrow" :class="{ expanded: !toolsCollapsed }">
            <ArrowRight />
          </el-icon>
        </div>
        <Transition name="collapse-section">
          <div v-show="!toolsCollapsed" class="section-body">
            <el-collapse v-model="expandedToolIndexes" class="tools-collapse">
              <el-collapse-item
                v-for="(tool, index) in tools"
                :key="index"
                :name="index"
                class="tool-item"
              >
                <template #title>
                  <div class="tool-title">
                    <span class="tool-name">{{ tool.function?.name || tool.name || '未命名工具' }}</span>
                    <el-tag v-if="tool.type" size="small" type="info">{{ tool.type }}</el-tag>
                  </div>
                </template>
                <div class="tool-details">
                  <div v-if="tool.function?.description" class="tool-detail-item">
                    <span class="detail-label">描述</span>
                    <span class="detail-value">{{ tool.function.description }}</span>
                  </div>
                  <div v-if="tool.function?.parameters" class="tool-detail-item">
                    <span class="detail-label">参数定义</span>
                    <div class="code-block">
                      <pre>{{ JSON.stringify(tool.function.parameters, null, 2) }}</pre>
                    </div>
                  </div>
                </div>
              </el-collapse-item>
            </el-collapse>
          </div>
        </Transition>
      </div>

      <!-- 空状态 -->
      <el-empty v-if="!parsedBody" description="无法解析请求体" />
    </div>
  </el-dialog>
</template>

<style scoped>
.body-detail-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-height: 70vh;
  overflow-y: auto;
  padding-right: 4px;
}

/* 区块样式 */
.section {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--app-border-default);
  border-radius: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid var(--app-border-default);
}

.section-header.clickable {
  cursor: pointer;
  transition: background 0.2s ease;
}

.section-header.clickable:hover {
  background: rgba(0, 212, 255, 0.08);
}

.collapse-arrow {
  margin-left: auto;
  font-size: 14px;
  color: var(--app-text-tertiary);
  transition: transform 0.3s ease;
}

.collapse-arrow.expanded {
  transform: rotate(90deg);
  color: var(--app-color-primary);
}

.section-icon {
  font-size: 18px;
  color: var(--app-color-primary);
}

.section-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--app-text-primary);
  letter-spacing: 0.3px;
}

.section-badge {
  padding: 3px 12px;
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 12px;
  font-size: 13px;
  font-weight: 700;
  color: var(--app-color-primary);
}

.section-tokens {
  margin-left: auto;
  padding: 3px 12px;
  background: rgba(0, 245, 160, 0.15);
  border: 1px solid rgba(0, 245, 160, 0.3);
  border-radius: 12px;
  font-size: 13px;
  font-weight: 700;
  color: var(--app-color-success);
}

.section-body {
  padding: 16px;
}

/* 模型标签 */
.model-tag {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(139, 92, 246, 0.15)) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-weight: 700;
  font-size: 15px;
}

/* 参数网格 */
.params-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}

.param-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--app-border-default);
  border-radius: 8px;
}

.param-key {
  font-size: 12px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  font-weight: 600;
}

.param-value {
  font-size: 15px;
  color: var(--app-text-primary);
  font-weight: 700;
}

.param-value.boolean {
  color: var(--app-color-success);
}

.param-value.array {
  font-size: 12px;
  color: var(--app-text-secondary);
}

/* 思考参数样式 */
.thinking-params-section .section-header {
  background: linear-gradient(90deg, rgba(168, 85, 247, 0.15), transparent);
}

.thinking-params-section .section-icon {
  color: #a855f7;
}

.thinking-param {
  background: rgba(168, 85, 247, 0.08) !important;
  border-color: rgba(168, 85, 247, 0.2) !important;
}

.thinking-param:hover {
  background: rgba(168, 85, 247, 0.12) !important;
  border-color: rgba(168, 85, 247, 0.3) !important;
}

.thinking-param .param-key {
  color: #c084fc;
}

.thinking-value {
  color: #e879f9 !important;
  font-weight: 700;
}

/* 消息列表 */
.messages-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.message-item {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--app-border-default);
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.message-item:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.message-item.role-system {
  border-left: 3px solid var(--app-color-warning);
}

.message-item.role-user {
  border-left: 3px solid var(--app-color-primary);
}

.message-item.role-assistant {
  border-left: 3px solid var(--app-color-success);
}

.message-item.role-tool {
  border-left: 3px solid var(--app-color-info);
}

.message-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.message-header:hover {
  background: rgba(0, 212, 255, 0.05);
}

.message-role {
  display: flex;
  align-items: center;
  gap: 10px;
}

.role-icon {
  font-size: 16px;
  color: var(--app-text-tertiary);
}

.message-name {
  font-size: 13px;
  color: var(--app-text-tertiary);
  font-weight: 500;
}

.message-meta {
  display: flex;
  align-items: center;
  gap: 10px;
}

.message-tokens {
  font-size: 12px;
  color: var(--app-color-success);
  font-weight: 700;
  padding: 3px 10px;
  background: rgba(0, 245, 160, 0.1);
  border-radius: 4px;
}

.message-index {
  font-size: 12px;
  color: var(--app-text-tertiary);
  font-weight: 500;
}

.expand-icon {
  font-size: 14px;
  color: var(--app-text-tertiary);
  transition: transform 0.3s ease;
}

.expand-icon.expanded {
  transform: rotate(90deg);
  color: var(--app-color-primary);
}

.message-content {
  padding: 0 16px 16px;
}

/* 消息内容包装器 */
.message-content-wrapper {
  padding: 0 16px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 思考块样式 */
.thinking-blocks {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.1), rgba(139, 92, 246, 0.05));
  border: 1px solid rgba(168, 85, 247, 0.3);
  border-radius: 10px;
  overflow: hidden;
}

.thinking-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(168, 85, 247, 0.15);
  border-bottom: 1px solid rgba(168, 85, 247, 0.2);
}

.thinking-header .thinking-icon {
  font-size: 16px;
  color: #a855f7;
}

.thinking-label {
  font-size: 13px;
  font-weight: 600;
  color: #c084fc;
  letter-spacing: 0.3px;
}

.thinking-content {
  margin: 0;
  padding: 14px;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.7;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-weight: 450;
  background: rgba(168, 85, 247, 0.03);
}

.message-content pre {
  margin: 0;
  padding: 16px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  line-height: 1.7;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-weight: 450;
}

/* 折叠过渡动画 */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 500px;
}

/* 区块折叠过渡动画 */
.collapse-section-enter-active,
.collapse-section-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.collapse-section-enter-from,
.collapse-section-leave-to {
  opacity: 0;
  max-height: 0;
}

.collapse-section-enter-to,
.collapse-section-leave-from {
  opacity: 1;
  max-height: 2000px;
}

/* 工具折叠面板 */
.tools-collapse,
.tool-calls-collapse {
  --el-collapse-border-color: transparent;
  --el-collapse-header-bg-color: transparent;
  --el-collapse-content-bg-color: transparent;
  border: none;
  background: transparent;
}

.tools-collapse :deep(.el-collapse-item__header),
.tool-calls-collapse :deep(.el-collapse-item__header) {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--app-border-default);
  border-radius: 10px;
  color: var(--app-text-primary);
  padding: 14px 16px;
  height: auto;
  line-height: 1.5;
  font-size: 14px;
}

.tools-collapse :deep(.el-collapse-item__header:hover),
.tool-calls-collapse :deep(.el-collapse-item__header:hover) {
  background: rgba(0, 212, 255, 0.05);
  border-color: rgba(0, 212, 255, 0.3);
}

.tools-collapse :deep(.el-collapse-item__wrap),
.tool-calls-collapse :deep(.el-collapse-item__wrap) {
  background: transparent;
  border: none;
  border-bottom-left-radius: 10px;
  border-bottom-right-radius: 10px;
}

.tools-collapse :deep(.el-collapse-item__content),
.tool-calls-collapse :deep(.el-collapse-item__content) {
  padding: 14px;
  background: rgba(0, 0, 0, 0.1);
  border: 1px solid var(--app-border-default);
  border-top: none;
  border-bottom-left-radius: 10px;
  border-bottom-right-radius: 10px;
}

.tools-collapse :deep(.el-collapse-item__arrow),
.tool-calls-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-text-tertiary);
  font-size: 14px;
  transition: transform 0.3s ease;
}

.tools-collapse :deep(.el-collapse-item.is-active .el-collapse-item__arrow),
.tool-calls-collapse :deep(.el-collapse-item.is-active .el-collapse-item__arrow) {
  color: var(--app-color-primary);
}

.tool-item,
.tool-call-item {
  margin-bottom: 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 10px;
}

.tool-title,
.tool-call-title {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.tool-name,
.call-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--app-text-primary);
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
}

.call-id {
  font-size: 12px;
  color: var(--app-text-tertiary);
  background: rgba(0, 212, 255, 0.1);
  padding: 3px 10px;
  border-radius: 4px;
  font-weight: 600;
}

.tool-details,
.tool-call-details {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tool-detail-item,
.call-detail-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  font-weight: 600;
}

.detail-value {
  font-size: 14px;
  color: var(--app-text-secondary);
  line-height: 1.6;
  font-weight: 450;
}

.detail-code {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.1);
  padding: 4px 12px;
  border-radius: 4px;
  font-weight: 600;
}

.code-block {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--app-border-default);
  border-radius: 8px;
  padding: 14px;
  overflow-x: auto;
}

.code-block pre {
  margin: 0;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  font-weight: 450;
}

/* 滚动条 */
.body-detail-content::-webkit-scrollbar,
.message-content pre::-webkit-scrollbar,
.code-block pre::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.body-detail-content::-webkit-scrollbar-track,
.message-content pre::-webkit-scrollbar-track,
.code-block pre::-webkit-scrollbar-track {
  background: transparent;
}

.body-detail-content::-webkit-scrollbar-thumb,
.message-content pre::-webkit-scrollbar-thumb,
.code-block pre::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

.body-detail-content::-webkit-scrollbar-thumb:hover,
.message-content pre::-webkit-scrollbar-thumb:hover,
.code-block pre::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
}

/* 对话框样式 */
:deep(.el-dialog) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
}

:deep(.el-dialog__header) {
  padding: 16px 24px;
  border-bottom: 1px solid var(--app-border-default);
  margin-right: 0;
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
}

:deep(.el-dialog__body) {
  padding: 20px 24px;
  max-height: calc(100vh - 200px);
  overflow: visible;
}

:deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid var(--app-border-default);
}

/* 赛博朋克主题 */
html.cyberpunk .section {
  border-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .section-header {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.1), transparent);
  border-bottom-color: rgba(0, 212, 255, 0.2);
}

html.cyberpunk .model-tag {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(255, 0, 255, 0.15)) !important;
  border-color: rgba(0, 255, 255, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .message-item:hover {
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .expand-icon.expanded {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 255, 255, 0.5));
}

html.cyberpunk .tools-collapse :deep(.el-collapse-item__header),
html.cyberpunk .tool-calls-collapse :deep(.el-collapse-item__header) {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .tools-collapse :deep(.el-collapse-item__header:hover),
html.cyberpunk .tool-calls-collapse :deep(.el-collapse-item__header:hover) {
  background: rgba(0, 212, 255, 0.12);
  border-color: rgba(0, 212, 255, 0.5);
}

html.cyberpunk .tools-collapse :deep(.el-collapse-item__arrow),
html.cyberpunk .tool-calls-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-color-primary);
}

html.cyberpunk .message-tokens {
  color: var(--app-color-success);
  background: rgba(0, 255, 136, 0.15);
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

html.cyberpunk .section-badge {
  background: rgba(0, 255, 255, 0.15);
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .section-tokens {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.4);
  color: var(--app-color-success);
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

/* 玻璃拟态主题 */
html.glassmorphism .section {
  background: rgba(255, 255, 255, 0.9);
  border-color: #e5e7eb;
}

html.glassmorphism .section-header {
  background: #f9fafb;
  border-bottom-color: #e5e7eb;
}

html.glassmorphism .model-tag {
  background: rgba(37, 99, 235, 0.1) !important;
  border-color: rgba(37, 99, 235, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.glassmorphism .section-badge {
  background: rgba(37, 99, 235, 0.1);
  border-color: rgba(37, 99, 235, 0.3);
  color: var(--app-color-primary);
}

html.glassmorphism .section-tokens {
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.3);
  color: var(--app-color-success);
}

html.glassmorphism .param-item {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.08), rgba(59, 130, 246, 0.04));
  border-color: rgba(37, 99, 235, 0.2);
}

html.glassmorphism .param-item:hover {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.12), rgba(59, 130, 246, 0.06));
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .param-key {
  color: #6b7280;
}

html.glassmorphism .param-value {
  color: #1f2937;
}

html.glassmorphism .param-value.boolean {
  color: #059669;
}

html.glassmorphism .message-item {
  background: #ffffff;
}

html.glassmorphism .message-tokens {
  color: #059669;
  background: rgba(16, 185, 129, 0.1);
}

html.glassmorphism .message-content pre,
html.glassmorphism .code-block {
  background: #f9fafb;
}

html.glassmorphism .tools-collapse :deep(.el-collapse-item__header),
html.glassmorphism .tool-calls-collapse :deep(.el-collapse-item__header) {
  background: #f9fafb;
  border-color: #e5e7eb;
}

html.glassmorphism .tools-collapse :deep(.el-collapse-item__header:hover),
html.glassmorphism .tool-calls-collapse :deep(.el-collapse-item__header:hover) {
  background: #f3f4f6;
  border-color: #93c5fd;
}

/* 暗色主题 */
html.dark .section {
  background: rgba(26, 26, 46, 0.6);
  border-color: var(--app-border-default);
}

html.dark .section-header {
  background: rgba(0, 212, 255, 0.05);
}

html.dark .section-badge {
  background: rgba(0, 212, 255, 0.12);
  border-color: rgba(0, 212, 255, 0.35);
}

html.dark .section-tokens {
  background: rgba(0, 245, 160, 0.12);
  border-color: rgba(0, 245, 160, 0.35);
}

html.dark .model-tag {
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.2);
}

html.dark .param-item {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.2);
}

html.dark .param-item:hover {
  background: rgba(0, 212, 255, 0.12);
  border-color: rgba(0, 212, 255, 0.3);
}

html.dark .message-item:hover {
  border-color: rgba(0, 212, 255, 0.4);
}

html.dark .message-tokens {
  color: var(--app-color-success);
  background: rgba(0, 245, 160, 0.12);
}

html.dark .expand-icon.expanded {
  color: var(--app-color-primary);
}

html.dark .tools-collapse :deep(.el-collapse-item__header),
html.dark .tool-calls-collapse :deep(.el-collapse-item__header) {
  background: rgba(26, 26, 46, 0.6);
  border-color: var(--app-border-default);
}

html.dark .tools-collapse :deep(.el-collapse-item__header:hover),
html.dark .tool-calls-collapse :deep(.el-collapse-item__header:hover) {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.4);
}

html.dark .tools-collapse :deep(.el-collapse-item__arrow),
html.tool-calls-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-color-primary);
}

/* 暗色主题思考块 */
html.dark .thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.12), rgba(139, 92, 246, 0.06));
  border-color: rgba(168, 85, 247, 0.35);
  box-shadow: 0 0 20px rgba(168, 85, 247, 0.1);
}

html.dark .thinking-header {
  background: rgba(168, 85, 247, 0.18);
}

html.dark .thinking-content {
  background: rgba(168, 85, 247, 0.05);
}

html.dark .thinking-param {
  background: rgba(168, 85, 247, 0.1) !important;
  border-color: rgba(168, 85, 247, 0.25) !important;
  box-shadow: 0 0 10px rgba(168, 85, 247, 0.08);
}

/* 明色主题思考块 */
html.light:not(.cyberpunk):not(.dark) .thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.08), rgba(139, 92, 246, 0.04));
  border-color: rgba(168, 85, 247, 0.25);
}

html.light:not(.cyberpunk):not(.dark) .thinking-header {
  background: rgba(168, 85, 247, 0.1);
}

html.light:not(.cyberpunk):not(.dark) .thinking-content {
  background: rgba(168, 85, 247, 0.02);
}

html.light:not(.cyberpunk):not(.dark) .thinking-param {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.08), rgba(139, 92, 246, 0.04)) !important;
  border-color: rgba(168, 85, 247, 0.2) !important;
}

/* 赛博朋克主题思考块 */
html.cyberpunk .thinking-block {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.08), rgba(255, 0, 255, 0.04));
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .thinking-header {
  background: linear-gradient(90deg, rgba(0, 255, 255, 0.15), transparent);
  border-bottom-color: rgba(0, 255, 255, 0.3);
}

html.cyberpunk .thinking-header .thinking-icon {
  color: #00ffff;
  filter: drop-shadow(0 0 5px rgba(0, 255, 255, 0.5));
}

html.cyberpunk .thinking-label {
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .thinking-content {
  background: rgba(0, 255, 255, 0.02);
}

html.cyberpunk .thinking-param {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.1), rgba(255, 0, 255, 0.05)) !important;
  border-color: rgba(0, 255, 255, 0.3) !important;
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .thinking-param .param-key {
  color: #00ffff;
}

html.cyberpunk .thinking-value {
  color: #ff00ff !important;
  text-shadow: 0 0 8px rgba(255, 0, 255, 0.4);
}

/* 明色主题 (html.light - 非玻璃拟态/非暗色) */
html.light:not(.cyberpunk):not(.dark) .section {
  background: #ffffff;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .section-header {
  background: #f9fafb;
  border-bottom-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .model-tag {
  background: rgba(0, 168, 232, 0.1) !important;
  border-color: rgba(0, 168, 232, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .section-badge {
  background: rgba(0, 168, 232, 0.1);
  border-color: rgba(0, 168, 232, 0.3);
  color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .section-tokens {
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.3);
  color: #059669;
}

html.light:not(.cyberpunk):not(.dark) .param-item {
  background: linear-gradient(135deg, rgba(0, 168, 232, 0.08), rgba(0, 212, 255, 0.04));
  border-color: rgba(0, 168, 232, 0.2);
}

html.light:not(.cyberpunk):not(.dark) .param-item:hover {
  background: linear-gradient(135deg, rgba(0, 168, 232, 0.12), rgba(0, 212, 255, 0.06));
  border-color: rgba(0, 168, 232, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .param-key {
  color: #6b7280;
}

html.light:not(.cyberpunk):not(.dark) .param-value {
  color: #1f2937;
}

html.light:not(.cyberpunk):not(.dark) .param-value.boolean {
  color: #059669;
}

html.light:not(.cyberpunk):not(.dark) .message-item {
  background: #ffffff;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .message-item:hover {
  border-color: rgba(0, 168, 232, 0.4);
}

html.light:not(.cyberpunk):not(.dark) .message-tokens {
  color: #059669;
  background: rgba(16, 185, 129, 0.1);
}

html.light:not(.cyberpunk):not(.dark) .message-content pre,
html.light:not(.cyberpunk):not(.dark) .code-block {
  background: #f9fafb;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .tools-collapse :deep(.el-collapse-item__header),
html.light:not(.cyberpunk):not(.dark) .tool-calls-collapse :deep(.el-collapse-item__header) {
  background: #f9fafb;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .tools-collapse :deep(.el-collapse-item__header:hover),
html.light:not(.cyberpunk):not(.dark) .tool-calls-collapse :deep(.el-collapse-item__header:hover) {
  background: #f3f4f6;
  border-color: rgba(0, 168, 232, 0.4);
}

html.light:not(.cyberpunk):not(.dark) .tools-collapse :deep(.el-collapse-item__content),
html.light:not(.cyberpunk):not(.dark) .tool-calls-collapse :deep(.el-collapse-item__content) {
  background: linear-gradient(135deg, rgba(0, 168, 232, 0.04), rgba(0, 196, 180, 0.02));
  border-color: rgba(0, 168, 232, 0.15);
}

html.light:not(.cyberpunk):not(.dark) .tool-item,
html.light:not(.cyberpunk):not(.dark) .tool-call-item {
  background: rgba(0, 168, 232, 0.02);
}

html.light:not(.cyberpunk):not(.dark) .expand-icon.expanded {
  color: var(--app-color-primary);
}
</style>
