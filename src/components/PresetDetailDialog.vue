<script setup lang="ts">
/**
 * 预设详情对话框组件
 * 用于展示预设的完整配置信息
 */
import { computed, ref } from 'vue'
import type { Preset } from '@/types'
import { AGENT_NAMES, CATEGORY_NAMES } from '@/types'

// Props 定义
const props = defineProps<{
  visible: boolean
  preset: Preset | null
}>()

// Emits 定义
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

// 对话框可见性控制
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// JSON 预览显示控制
const showJsonPreview = ref(false)

// 格式化日期显示
const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// Agent 配置列表（用于表格展示）
const agentConfigs = computed(() => {
  if (!props.preset?.config?.agents) return []
  return AGENT_NAMES.map((name) => ({
    name,
    model: props.preset!.config.agents[name]?.model || '-'
  }))
})

// Category 配置列表（用于表格展示）
const categoryConfigs = computed(() => {
  if (!props.preset?.config?.categories) return []
  return CATEGORY_NAMES.map((name) => ({
    name,
    model: props.preset!.config.categories[name]?.model || '-'
  }))
})

// 格式化 JSON 带语法高亮
const formattedJson = computed(() => {
  if (!props.preset?.config) return '{}'
  try {
    return JSON.stringify(props.preset.config, null, 2)
  } catch {
    return '{}'
  }
})

// 语法高亮处理
const highlightJson = (json: string): string => {
  return json
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/(".*?"):/g, '<span class="json-key">$1</span>:')
    .replace(/: "(.*?)"/g, ': <span class="json-string">"$1"</span>')
    .replace(/: (\d+)/g, ': <span class="json-number">$1</span>')
    .replace(/: (true|false)/g, ': <span class="json-boolean">$1</span>')
    .replace(/: (null)/g, ': <span class="json-null">$1</span>')
    .replace(/([{}\[\]])/g, '<span class="json-bracket">$1</span>')
}

// 高亮后的 JSON
const highlightedJson = computed(() => highlightJson(formattedJson.value))

// 关闭对话框
const handleClose = () => {
  dialogVisible.value = false
}

// 切换 JSON 预览
const toggleJsonPreview = () => {
  showJsonPreview.value = !showJsonPreview.value
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="预设详情"
    width="700px"
    :close-on-click-modal="true"
    class="glass-detail-dialog"
    align-center
  >
    <div v-if="preset" class="preset-detail">
      <!-- 基本信息卡片 -->
      <div class="info-section">
        <h4 class="section-title">基本信息</h4>
        <div class="info-card">
          <div class="info-item">
            <span class="info-label">预设名称</span>
            <span class="info-value">{{ preset.name }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">创建时间</span>
            <span class="info-value">{{ formatDate(preset.createdAt) }}</span>
          </div>
          <div class="info-item full-width">
            <span class="info-label">描述</span>
            <span class="info-value description">{{ preset.description || '无描述' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">更新时间</span>
            <span class="info-value">{{ formatDate(preset.updatedAt) }}</span>
          </div>
        </div>
      </div>

      <!-- Agent 配置 -->
      <div class="config-section">
        <div class="config-header">
          <h4 class="config-title">Agent 模型配置</h4>
          <span class="config-badge">{{ agentConfigs.length }}</span>
        </div>
        <el-table
          :data="agentConfigs"
          size="small"
          :border="false"
          max-height="200"
        >
          <el-table-column prop="name" label="Agent 名称" width="180">
            <template #default="{ row }">
              <span class="agent-name">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="model" label="模型">
            <template #default="{ row }">
              <span :class="['model-name', { 'model-empty': row.model === '-' }]">
                {{ row.model === '-' ? '未配置' : row.model }}
              </span>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- Category 配置 -->
      <div class="config-section">
        <div class="config-header">
          <h4 class="config-title">Category 模型配置</h4>
          <span class="config-badge">{{ categoryConfigs.length }}</span>
        </div>
        <el-table
          :data="categoryConfigs"
          size="small"
          :border="false"
          max-height="200"
        >
          <el-table-column prop="name" label="Category 名称" width="180">
            <template #default="{ row }">
              <span class="category-name">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="model" label="模型">
            <template #default="{ row }">
              <span :class="['model-name', { 'model-empty': row.model === '-' }]">
                {{ row.model === '-' ? '未配置' : row.model }}
              </span>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- JSON 配置预览 -->
      <div class="json-section">
        <div class="json-header">
          <h4 class="section-title" style="margin: 0;">原始配置</h4>
          <span class="json-toggle" @click="toggleJsonPreview">
            <el-icon>
              <component :is="showJsonPreview ? 'ArrowUp' : 'ArrowDown'" />
            </el-icon>
            {{ showJsonPreview ? '收起' : '展开' }} JSON
          </span>
        </div>
        <transition name="json-fade">
          <div v-show="showJsonPreview" class="json-code-block">
            <div class="json-code-header">
              <div class="json-code-dots">
                <span class="json-code-dot red"></span>
                <span class="json-code-dot yellow"></span>
                <span class="json-code-dot green"></span>
              </div>
              <span class="json-code-label">config.json</span>
            </div>
            <pre class="json-code-content" v-html="highlightedJson"></pre>
          </div>
        </transition>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
/* ==================== 玻璃效果对话框 ==================== */
:deep(.glass-detail-dialog .el-dialog) {
  background: rgba(18, 18, 26, 0.9);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
  box-shadow: var(--app-shadow-xl), var(--app-shadow-glow-primary);
  overflow: hidden;
}

:deep(.glass-detail-dialog .el-dialog__header) {
  padding: var(--app-spacing-5) var(--app-spacing-6);
  border-bottom: 1px solid var(--app-border-default);
  margin-right: 0;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, transparent 100%);
}

:deep(.glass-detail-dialog .el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
  letter-spacing: 0.5px;
}

:deep(.glass-detail-dialog .el-dialog__body) {
  padding: var(--app-spacing-5) var(--app-spacing-6);
  background: transparent;
}

:deep(.glass-detail-dialog .el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-tertiary);
  transition: all var(--app-transition-fast);
}

:deep(.glass-detail-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  transform: rotate(90deg);
}

:deep(.glass-detail-dialog .el-dialog__footer) {
  padding: var(--app-spacing-4) var(--app-spacing-6);
  border-top: 1px solid var(--app-border-default);
  background: rgba(0, 0, 0, 0.2);
}

/* ==================== 详情容器 ==================== */
.preset-detail {
  padding: 0;
}

/* ==================== 区块标题 ==================== */
.section-title {
  margin: 0 0 var(--app-spacing-3) 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.section-title::before {
  content: '';
  width: 3px;
  height: 16px;
  background: linear-gradient(180deg, var(--app-color-primary), var(--app-color-secondary));
  border-radius: var(--app-radius-full);
}

/* ==================== 信息卡片 ==================== */
.info-section {
  margin-bottom: var(--app-spacing-6);
}

.info-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  padding: var(--app-spacing-4);
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--app-spacing-4);
  position: relative;
  overflow: hidden;
}

.info-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--app-color-primary), transparent);
  opacity: 0.5;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-1);
}

.info-item.full-width {
  grid-column: 1 / -1;
}

.info-label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.info-value {
  font-size: 14px;
  color: var(--app-text-primary);
  font-weight: 500;
}

.info-value.description {
  color: var(--app-text-secondary);
  font-style: italic;
}

/* ==================== 配置区块 ==================== */
.config-section {
  margin-bottom: var(--app-spacing-5);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  overflow: hidden;
}

.config-section:last-child {
  margin-bottom: 0;
}

.config-header {
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.03) 0%, transparent 100%);
  border-bottom: 1px solid var(--app-border-default);
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.config-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 0;
}

.config-badge {
  font-size: 10px;
  padding: 2px 8px;
  background: rgba(0, 212, 255, 0.1);
  color: var(--app-color-primary);
  border-radius: var(--app-radius-full);
  font-weight: 500;
}

/* ==================== 表格样式 ==================== */
:deep(.config-section .el-table) {
  background: transparent;
  --el-table-border-color: var(--app-border-default);
  --el-table-header-bg-color: rgba(0, 212, 255, 0.05);
  --el-table-row-hover-bg-color: rgba(0, 212, 255, 0.03);
}

:deep(.config-section .el-table__header-wrapper th) {
  background: transparent;
  border-bottom: 1px solid var(--app-border-default);
  color: var(--app-text-secondary);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: var(--app-spacing-3) var(--app-spacing-4);
}

:deep(.config-section .el-table__row) {
  background: transparent;
}

:deep(.config-section .el-table__row td) {
  border-bottom: 1px solid rgba(42, 42, 58, 0.5);
  padding: var(--app-spacing-3) var(--app-spacing-4);
  color: var(--app-text-primary);
  font-size: 13px;
}

:deep(.config-section .el-table__row:last-child td) {
  border-bottom: none;
}

:deep(.config-section .el-table__row:hover td) {
  color: var(--app-color-primary);
}

/* 模型名称特殊样式 */
.model-name {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  color: var(--app-color-secondary);
  background: rgba(0, 255, 213, 0.08);
  padding: 2px 8px;
  border-radius: var(--app-radius-sm);
  display: inline-block;
}

.model-empty {
  color: var(--app-text-disabled);
  font-style: italic;
}

/* ==================== JSON 预览区块 ==================== */
.json-section {
  margin-top: var(--app-spacing-5);
}

.json-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--app-spacing-3);
}

.json-toggle {
  font-size: 12px;
  color: var(--app-color-primary);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--app-spacing-1);
  transition: all var(--app-transition-fast);
}

.json-toggle:hover {
  color: var(--app-color-secondary);
}

.json-code-block {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  overflow: hidden;
  position: relative;
}

.json-code-block::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, var(--app-color-primary), var(--app-color-secondary), transparent);
}

.json-code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--app-spacing-2) var(--app-spacing-3);
  background: rgba(0, 212, 255, 0.03);
  border-bottom: 1px solid var(--app-border-default);
}

.json-code-dots {
  display: flex;
  gap: 6px;
}

.json-code-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.json-code-dot.red {
  background: #ff5f56;
}

.json-code-dot.yellow {
  background: #ffbd2e;
}

.json-code-dot.green {
  background: #27ca40;
}

.json-code-label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  font-family: 'JetBrains Mono', monospace;
}

.json-code-content {
  padding: var(--app-spacing-4);
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: var(--app-text-secondary);
  overflow-x: auto;
  max-height: 300px;
  overflow-y: auto;
}

/* JSON 语法高亮 */
.json-key {
  color: #9cdcfe;
}

.json-string {
  color: #ce9178;
}

.json-number {
  color: #b5cea8;
}

.json-boolean {
  color: #569cd6;
}

.json-null {
  color: #569cd6;
}

.json-bracket {
  color: #ffd700;
}

/* 滚动条样式 */
.json-code-content::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.json-code-content::-webkit-scrollbar-track {
  background: transparent;
}

.json-code-content::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: var(--app-radius-full);
}

.json-code-content::-webkit-scrollbar-thumb:hover {
  background: var(--app-border-hover);
}

/* ==================== 底部按钮 ==================== */
:deep(.el-button) {
  transition: all var(--app-transition-normal);
}

:deep(.el-button:hover) {
  transform: translateY(-1px);
}
</style>
