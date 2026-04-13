<script setup lang="ts">
/**
 * 响应体详情弹窗
 * 以可视化友好的方式展示大模型响应内容
 */
import { computed } from 'vue'
import { ChatDotRound, Cpu, Document } from '@element-plus/icons-vue'

const props = defineProps<{
  visible: boolean
  responseBody: any
  parsedBody?: {
    content?: string
    thinking?: string
    usage?: {
      promptTokens: number
      completionTokens: number
      totalTokens: number
    }
  }
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 解析响应体
const parsedResponse = computed(() => {
  // 只有 parsedBody 是有效对象（非空、有实际内容）时才优先使用
  if (props.parsedBody && typeof props.parsedBody === 'object' && Object.keys(props.parsedBody).length > 0) {
    return props.parsedBody
  }
  
  if (!props.responseBody) return null
  
  // 如果是字符串，尝试解析
  let body = props.responseBody
  if (typeof body === 'string') {
    try {
      body = JSON.parse(body)
    } catch {
      return null
    }
  }
  
  // 如果解析结果是 null 或空对象，返回 null
  if (!body || (typeof body === 'object' && !Array.isArray(body) && Object.keys(body).length === 0)) {
    return null
  }
  
  return body
})

// 思考内容
const thinkingContent = computed(() => {
  // 优先使用解析后的 thinking
  if (props.parsedBody?.thinking) {
    return props.parsedBody.thinking
  }
  
  // 尝试从原始 body 中提取
  const body = parsedResponse.value
  if (!body) return null
  
  // DeepSeek R1 格式
  if (body.choices?.[0]?.message?.reasoning_content) {
    return body.choices[0].message.reasoning_content
  }
  
  // Anthropic 格式
  if (Array.isArray(body.content)) {
    const thinkingBlock = body.content.find((b: any) => b.type === 'thinking')
    if (thinkingBlock?.thinking) {
      return thinkingBlock.thinking
    }
  }
  
  return null
})

// 主内容
const mainContent = computed(() => {
  // 优先使用解析后的 content
  if (props.parsedBody?.content) {
    return props.parsedBody.content
  }
  
  const body = parsedResponse.value
  if (!body) return null
  
  // OpenAI 格式
  if (body.choices?.[0]?.message?.content) {
    return body.choices[0].message.content
  }
  
  // Anthropic 格式
  if (Array.isArray(body.content)) {
    const textParts = body.content
      .filter((b: any) => b.type === 'text' && b.text)
      .map((b: any) => b.text)
    return textParts.join('\n') || null
  }
  
  return null
})

// 使用量
const usage = computed(() => {
  if (props.parsedBody?.usage) {
    return props.parsedBody.usage
  }
  
  return parsedResponse.value?.usage || null
})

// 是否有思考内容
const hasThinking = computed(() => !!thinkingContent.value)

// 是否有主内容
const hasContent = computed(() => !!mainContent.value)

// 格式化数字
function formatNumber(num: number | undefined): string {
  if (num === undefined) return '-'
  return num.toLocaleString()
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="响应体详情"
    width="900px"
    append-to=".app-main"
    align-center
    class="body-detail-dialog"
    destroy-on-close
  >
    <div class="body-detail-content">
      <!-- 使用量统计 -->
      <div v-if="usage" class="section usage-section">
        <div class="section-header">
          <el-icon class="section-icon"><Document /></el-icon>
          <span class="section-title">Token 使用量</span>
        </div>
        <div class="section-body">
          <div class="usage-grid">
            <div class="usage-item">
              <span class="usage-label">输入</span>
              <span class="usage-value">{{ formatNumber(usage.promptTokens) }}</span>
            </div>
            <div class="usage-item">
              <span class="usage-label">输出</span>
              <span class="usage-value">{{ formatNumber(usage.completionTokens) }}</span>
            </div>
            <div class="usage-item total">
              <span class="usage-label">总计</span>
              <span class="usage-value">{{ formatNumber(usage.totalTokens) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 思考过程 -->
      <div v-if="hasThinking" class="section thinking-section">
        <div class="section-header">
          <el-icon class="section-icon thinking-icon"><Cpu /></el-icon>
          <span class="section-title">思考过程</span>
        </div>
        <div class="section-body">
          <div class="thinking-block">
            <pre class="thinking-content">{{ thinkingContent }}</pre>
          </div>
        </div>
      </div>

      <!-- 响应内容 -->
      <div v-if="hasContent" class="section content-section">
        <div class="section-header">
          <el-icon class="section-icon"><ChatDotRound /></el-icon>
          <span class="section-title">响应内容</span>
        </div>
        <div class="section-body">
          <div class="content-block">
            <pre class="main-content">{{ mainContent }}</pre>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <el-empty v-if="!hasThinking && !hasContent && !usage" description="无法解析响应体" />
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

.section-body {
  padding: 16px;
}

/* 使用量网格 */
.usage-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.usage-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px;
  background: rgba(0, 212, 255, 0.08);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 10px;
}

.usage-item.total {
  background: rgba(0, 245, 160, 0.08);
  border-color: rgba(0, 245, 160, 0.2);
}

.usage-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.usage-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--app-color-primary);
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
}

.usage-item.total .usage-value {
  color: var(--app-color-success);
}

/* 思考区块 */
.thinking-section .section-header {
  background: linear-gradient(90deg, rgba(168, 85, 247, 0.15), transparent);
}

.thinking-section .section-icon {
  color: #a855f7;
}

.thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.1), rgba(139, 92, 246, 0.05));
  border: 1px solid rgba(168, 85, 247, 0.3);
  border-radius: 10px;
  overflow: hidden;
}

.thinking-content {
  margin: 0;
  padding: 16px;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.7;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-weight: 450;
  max-height: 300px;
  overflow-y: auto;
}

/* 内容区块 */
.content-block {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--app-border-default);
  border-radius: 10px;
  overflow: hidden;
}

.main-content {
  margin: 0;
  padding: 16px;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  line-height: 1.7;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-weight: 450;
  max-height: 400px;
  overflow-y: auto;
}

/* 滚动条 */
.body-detail-content::-webkit-scrollbar,
.thinking-content::-webkit-scrollbar,
.main-content::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.body-detail-content::-webkit-scrollbar-track,
.thinking-content::-webkit-scrollbar-track,
.main-content::-webkit-scrollbar-track {
  background: transparent;
}

.body-detail-content::-webkit-scrollbar-thumb,
.thinking-content::-webkit-scrollbar-thumb,
.main-content::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

.body-detail-content::-webkit-scrollbar-thumb:hover,
.thinking-content::-webkit-scrollbar-thumb:hover,
.main-content::-webkit-scrollbar-thumb:hover {
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

/* 暗色主题 */
html.dark .thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.12), rgba(139, 92, 246, 0.06));
  border-color: rgba(168, 85, 247, 0.35);
  box-shadow: 0 0 20px rgba(168, 85, 247, 0.1);
}

/* 明色主题 */
html.light:not(.cyberpunk):not(.dark) .thinking-block {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.08), rgba(139, 92, 246, 0.04));
  border-color: rgba(168, 85, 247, 0.25);
}

html.light:not(.cyberpunk):not(.dark) .content-block {
  background: #f9fafb;
}

/* 赛博朋克主题 */
html.cyberpunk .thinking-block {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.08), rgba(255, 0, 255, 0.04));
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .thinking-section .section-icon {
  color: #00ffff;
  filter: drop-shadow(0 0 5px rgba(0, 255, 255, 0.5));
}
</style>
