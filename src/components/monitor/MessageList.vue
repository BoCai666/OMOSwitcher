<script setup lang="ts">
/**
 * 消息列表组件
 * 渲染大模型请求中的消息列表，支持 thinking block、text content 等多种内容类型
 */
import { ref } from 'vue'
import { ArrowRight, Cpu } from '@element-plus/icons-vue'
import { formatTokens, getRoleIcon, getRoleTagType, getRoleDisplayName } from '@/composables/useRequestParser'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
defineProps<{
  messages: any[]
}>()

// 当前展开的消息
const expandedMessages = ref<number[]>([])

// 区块折叠状态（默认折叠）
const messagesCollapsed = ref(true)

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
  <div class="section messages-section collapsible-section">
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
</template>

<style scoped>
/* ========== 区块样式（与 RequestBodyDetailDialog 保持一致） ========== */
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

.section-body {
  padding: 16px;
}

/* ========== 消息列表 ========== */
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

/* 滚动条 */
.message-content pre::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.message-content pre::-webkit-scrollbar-track {
  background: transparent;
}

.message-content pre::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

.message-content pre::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
}

/* 赛博朋克主题 */
html.cyberpunk .section {
  border-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .section-header {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.1), transparent);
  border-bottom-color: rgba(0, 212, 255, 0.2);
}

html.cyberpunk .section-badge {
  background: rgba(0, 255, 255, 0.15);
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .message-item:hover {
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .expand-icon.expanded {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 255, 255, 0.5));
}

html.cyberpunk .message-tokens {
  color: var(--app-color-success);
  background: rgba(0, 255, 136, 0.15);
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

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

/* 玻璃拟态主题 */
html.glassmorphism .section {
  background: rgba(255, 255, 255, 0.9);
  border-color: #e5e7eb;
}

html.glassmorphism .section-header {
  background: #f9fafb;
  border-bottom-color: #e5e7eb;
}

html.glassmorphism .section-badge {
  background: rgba(37, 99, 235, 0.1);
  border-color: rgba(37, 99, 235, 0.3);
  color: var(--app-color-primary);
}

html.glassmorphism .message-item {
  background: #ffffff;
}

html.glassmorphism .message-tokens {
  color: #059669;
  background: rgba(16, 185, 129, 0.1);
}

html.glassmorphism .message-content pre {
  background: #f9fafb;
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

/* 明色主题 */
html.light:not(.cyberpunk):not(.dark) .section {
  background: #ffffff;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .section-header {
  background: #f9fafb;
  border-bottom-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .section-badge {
  background: rgba(0, 168, 232, 0.1);
  border-color: rgba(0, 168, 232, 0.3);
  color: var(--app-color-primary);
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

html.light:not(.cyberpunk):not(.dark) .message-content pre {
  background: #f9fafb;
  border-color: #e5e7eb;
}

html.light:not(.cyberpunk):not(.dark) .expand-icon.expanded {
  color: var(--app-color-primary);
}
</style>
