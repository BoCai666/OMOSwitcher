<script setup lang="ts">
/**
 * 配置详情对话框组件
 * 用于显示 Agent 或 Category 的详细信息，包括系统提示词
 * 点击当前模型可切换模型
 */
import { ref, computed, watch } from 'vue'
import { Edit } from '@element-plus/icons-vue'
import type { AgentName, CategoryName, Model } from '@/types'
import type { FallbackModels } from '@/types/config'
import { AGENT_DETAILS } from '@/data/agentDetails'
import { CATEGORY_DETAILS } from '@/data/categoryDetails'
import { AGENT_INFO, CATEGORY_INFO } from '@/types/config'
import { loadSystemPrompt } from '@/services/promptLoader'
import type { PromptLang } from '@/services/promptLoader'
import FallbackChainEditor from './FallbackChainEditor.vue'

const props = defineProps<{
  visible: boolean
  type: 'agent' | 'category'
  name: AgentName | CategoryName
  currentModel: string
  models: Model[]
  fallbackModels: FallbackModels | undefined
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'change-model': []
  'update:fallback-models': [value: FallbackModels | undefined]
  'add-fallback-model': []
}>()

// 对话框可见性（必须最先声明，因为后面的 watch 会引用它）
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 语言切换
const promptLang = ref<PromptLang>('zh')

// 系统提示词内容（同步加载）
const promptContent = ref<string>('')

// 加载系统提示词（同步，已预加载）
function loadPrompt() {
  if (!props.name) return
  promptContent.value = loadSystemPrompt(props.type, props.name, promptLang.value)
}

// 监听名称、类型和语言变化，重新加载提示词
watch([() => props.name, () => props.type, promptLang], () => {
  if (dialogVisible.value) {
    loadPrompt()
  }
}, { immediate: true })

// 内置回退链折叠状态（默认展开）
const builtinChainExpanded = ref(true)

// 获取详情数据
const detail = computed(() => {
  if (props.type === 'agent') {
    return AGENT_DETAILS[props.name as AgentName]
  } else {
    return CATEGORY_DETAILS[props.name as CategoryName]
  }
})

// 中文描述（用于详情页显示）
const descriptionZh = computed(() => {
  if (props.type === 'agent') {
    return AGENT_INFO[props.name as AgentName]?.description || detail.value?.description
  } else {
    return CATEGORY_INFO[props.name as CategoryName]?.description || detail.value?.description
  }
})

// 当前模型信息
const currentModelInfo = computed(() => {
  return props.models.find(m => m.id === props.currentModel)
})

// 供应商名称
const providerName = computed(() => {
  return currentModelInfo.value?.provider || props.currentModel.split('/')[0]
})

// 模型显示名称
const modelDisplayName = computed(() => {
  return currentModelInfo.value?.name || props.currentModel.split('/').pop() || props.currentModel
})

// 当前显示的系统提示词（从文件动态加载）

// 格式化回退链显示
function formatFallbackModel(fallback: { model: string; variant?: string; providers: string[] }): { 
  label: string
  isCurrent: boolean 
} {
  const modelId = fallback.providers[0] + '/' + fallback.model
  return {
    label: `${fallback.model}${fallback.variant ? ` (${fallback.variant})` : ''}`,
    isCurrent: modelId === props.currentModel || fallback.model === props.currentModel.split('/').pop()
  }
}

// 点击切换模型
function handleChangeModel() {
  emit('change-model')
}

// 回退模型 v-model 包装
const fallbackModelsValue = computed({
  get: () => props.fallbackModels,
  set: (value) => emit('update:fallback-models', value)
})

// 添加回退模型 - 冒泡给父组件
function handleAddFallbackModel() {
  emit('add-fallback-model')
}

// 关闭对话框时重置语言和折叠状态
watch(dialogVisible, (val) => {
  if (!val) {
    promptLang.value = 'zh'
    builtinChainExpanded.value = true
  }
})
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :title="detail?.displayName || name"
    width="800px"
    class="config-detail-dialog"
    destroy-on-close
    append-to=".app-main"
    align-center
  >
    <div v-if="detail" class="detail-content">
      <!-- 基本信息 -->
      <el-descriptions :column="2" border class="basic-info">
        <el-descriptions-item label="名称">
          <span class="info-value">{{ detail.displayName }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="类型">
          <el-tag :type="type === 'agent' ? 'primary' : 'success'" size="small">
            {{ type === 'agent' ? 'Agent' : 'Category' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="当前模型">
          <div class="model-info clickable" @click="handleChangeModel">
            <el-tag type="info" size="small">{{ providerName }}</el-tag>
            <span class="model-name">{{ modelDisplayName }}</span>
            <el-icon class="edit-icon"><Edit /></el-icon>
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="推荐模型">
          <el-tag type="success" size="small">{{ detail.recommendedModel }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">
          <span class="description-text">{{ descriptionZh }}</span>
        </el-descriptions-item>
      </el-descriptions>

      <!-- 自定义回退链 -->
      <div class="section">
        <h4 class="section-title">自定义回退链</h4>
        <FallbackChainEditor
          v-model:fallback-models="fallbackModelsValue"
          :current-model="currentModel"
          @add-model="handleAddFallbackModel"
        />
      </div>

      <!-- 内置回退链（参考） -->
      <div v-if="detail.fallbackChain?.length" class="section builtin-chain-section">
        <div class="builtin-chain-toggle" @click="builtinChainExpanded = !builtinChainExpanded">
          <el-icon class="toggle-arrow" :class="{ expanded: builtinChainExpanded }">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M9.5 7l5 5-5 5z"/>
            </svg>
          </el-icon>
          <span class="toggle-text">内置推荐回退链</span>
          <span class="toggle-count">{{ detail.fallbackChain.length }} 个模型</span>
        </div>
        <el-collapse-transition>
          <div v-show="builtinChainExpanded" class="builtin-chain-content">
            <div class="fallback-chain">
              <div 
                v-for="(fallback, index) in detail.fallbackChain" 
                :key="index" 
                class="fallback-item builtin"
                :class="{ current: formatFallbackModel(fallback).isCurrent }"
              >
                <span class="fallback-index">{{ index + 1 }}</span>
                <span class="fallback-model">{{ fallback.model }}</span>
                <span v-if="fallback.variant" class="variant-badge">{{ fallback.variant }}</span>
                <span class="providers">{{ fallback.providers.slice(0, 3).join(', ') }}{{ fallback.providers.length > 3 ? '...' : '' }}</span>
              </div>
            </div>
          </div>
        </el-collapse-transition>
      </div>

      <!-- 使用场景 (Agent 特有) -->
      <div v-if="type === 'agent' && (detail as any).useWhen" class="section">
        <h4 class="section-title">适用场景</h4>
        <ul class="use-when-list">
          <li v-for="(item, index) in (detail as any).useWhen" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>

      <!-- 触发条件 (Agent 特有) -->
      <div v-if="type === 'agent' && (detail as any).triggers?.length" class="section">
        <h4 class="section-title">触发条件</h4>
        <ul class="trigger-list">
          <li v-for="(item, index) in (detail as any).triggers" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>

      <!-- 避免场景 (Agent 特有) -->
      <div v-if="type === 'agent' && (detail as any).avoidWhen?.length" class="section">
        <h4 class="section-title">避免场景</h4>
        <ul class="avoid-list">
          <li v-for="(item, index) in (detail as any).avoidWhen" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>

      <!-- 系统提示词 -->
      <div class="section">
        <div class="section-header">
          <h4 class="section-title">系统提示词</h4>
          <el-radio-group v-model="promptLang" size="small">
            <el-radio-button value="zh">中文</el-radio-button>
            <el-radio-button value="en">English</el-radio-button>
          </el-radio-group>
        </div>
        <div class="prompt-content">
          <pre>{{ promptContent }}</pre>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
/* 对话框基础样式 - 背景由各主题样式接管，此处只设置通用属性 */
/* 注意：config-detail-dialog 是添加到 el-dialog 上的 class，它们是同一个元素 */
:deep(.el-dialog.config-detail-dialog) {
  background-color: transparent;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border-default);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  max-width: 90vw;
}

/* 对话框标题霓虹下划线 */
:deep(.el-dialog.config-detail-dialog .el-dialog__header) {
  border-bottom: 2px solid var(--app-color-primary, #00d4ff);
  box-shadow: 0 2px 10px rgba(0, 212, 255, 0.3);
  margin-right: 0;
  padding: 20px;
}

:deep(.el-dialog.config-detail-dialog .el-dialog__title) {
  color: var(--app-text-primary, #e5eaf3);
  font-weight: 600;
}

/* 关闭按钮样式 */
:deep(.el-dialog.config-detail-dialog .el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-tertiary);
  transition: all 0.3s;
}

:deep(.el-dialog.config-detail-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary, #00d4ff);
}

/* 对话框内容区域 - 修复双滚动条问题 */
:deep(.el-dialog.config-detail-dialog .el-dialog__body) {
  padding: 20px;
  background: transparent;
  overflow: visible;
  max-height: none;
}

/* 底部按钮区域 */
:deep(.el-dialog.config-detail-dialog .el-dialog__footer) {
  border-top: 1px solid var(--app-border-default, #2a2a3a);
  padding: 15px 20px;
}

/* 禁用 el-overlay 的滚动条，避免双滚动条 */
:deep(.el-overlay),
:deep(.el-overlay-dialog) {
  overflow: hidden !important;
}

/* 内容容器负责滚动 - 只允许垂直滚动 */
.detail-content {
  max-height: 70vh;
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 基本信息区域样式 */
.basic-info {
  margin-bottom: 0;
}

.basic-info :deep(.el-descriptions__body) {
  background-color: var(--app-bg-card, #12121a);
}

.basic-info :deep(.el-descriptions__label) {
  background-color: rgba(0, 212, 255, 0.05);
  color: var(--app-text-tertiary);
  white-space: nowrap;
}

.basic-info :deep(.el-descriptions__content) {
  background-color: var(--app-bg-card, #12121a);
  color: var(--app-text-primary, #e5eaf3);
}

.info-value {
  font-weight: 500;
  color: var(--app-text-primary, #e5eaf3);
}

/* 模型信息 - 霓虹边框 + 悬停发光 */
.model-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-info.clickable {
  cursor: pointer;
  padding: 2px 6px;
  margin: 0;
  border-radius: 6px;
  border: 1px solid transparent;
  background: linear-gradient(var(--app-bg-card, #12121a), var(--app-bg-card, #12121a)) padding-box,
              linear-gradient(135deg, var(--app-color-primary, #00d4ff), #00a8ff) border-box;
  transition: all 0.3s ease;
  display: inline-flex;
  width: fit-content;
}

.model-info.clickable:hover {
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.4),
              inset 0 0 10px rgba(0, 212, 255, 0.1);
  transform: translateY(-1px);
}

.model-info.clickable:hover .edit-icon {
  opacity: 1;
  color: var(--app-color-primary, #00d4ff);
}

.model-name {
  font-weight: 500;
  color: var(--app-color-primary, #00d4ff);
}

.edit-icon {
  color: var(--app-text-tertiary);
  font-size: 14px;
  opacity: 0;
  transition: all 0.3s;
}

.description-text {
  color: var(--app-text-secondary);
  line-height: 1.6;
}

.section {
  margin-top: 8px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-primary, #e5eaf3);
  margin: 0 0 10px 0;
  padding-left: 10px;
  border-left: 3px solid var(--app-color-primary, #00d4ff);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.2);
}

.section-header .section-title {
  margin: 0;
}

/* 回退模型链样式 */
.fallback-chain {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.fallback-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background-color: var(--app-bg-card, #12121a);
  border: 1px solid var(--app-border-default, #2a2a3a);
  border-radius: 6px;
  transition: all 0.3s;
}

.fallback-item:hover {
  border-color: rgba(0, 212, 255, 0.3);
  box-shadow: 0 2px 8px rgba(0, 212, 255, 0.1);
}

.fallback-item.current {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.1), rgba(0, 168, 255, 0.05));
  border-color: var(--app-color-primary, #00d4ff);
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.2);
}

.fallback-item.current .fallback-index {
  background: linear-gradient(135deg, var(--app-color-primary, #00d4ff), #00a8ff);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.4);
}

.fallback-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 24px;
  background: linear-gradient(135deg, #606266, var(--app-text-tertiary));
  color: white;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.3s;
}

.fallback-model {
  font-weight: 500;
  color: var(--app-text-primary, #e5eaf3);
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  font-size: 13px;
}

.variant-badge {
  font-size: 11px;
  color: var(--app-color-success);
  background-color: rgba(103, 194, 58, 0.1);
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid rgba(103, 194, 58, 0.3);
}

.providers {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin-left: auto;
}

/* 内置回退链折叠区域 */
.builtin-chain-section {
  opacity: 0.85;
}

.builtin-chain-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 6px;
  background-color: var(--app-bg-card, #12121a);
  border: 1px solid var(--app-color-primary, #00d4ff);
  transition: all 0.3s;
  user-select: none;
}

.builtin-chain-toggle:hover {
  border-color: var(--app-color-primary, #00d4ff);
  opacity: 1;
  background-color: var(--app-color-primary-light, rgba(0, 212, 255, 0.1));
}

.toggle-arrow {
  color: var(--app-color-primary, #00d4ff);
  transition: transform 0.3s;
  flex-shrink: 0;
}

.toggle-arrow.expanded {
  transform: rotate(90deg);
}

.toggle-text {
  font-size: 13px;
  color: var(--app-text-secondary);
}

.toggle-count {
  font-size: 11px;
  color: var(--app-text-tertiary);
  margin-left: auto;
}

.builtin-chain-content {
  margin-top: 8px;
}

.fallback-item.builtin {
  opacity: 0.6;
}

.use-when-list,
.trigger-list,
.avoid-list {
  margin: 0;
  padding-left: 20px;
  color: var(--app-text-secondary);
  line-height: 1.8;
}

.use-when-list li,
.trigger-list li,
.avoid-list li {
  margin-bottom: 6px;
}

.use-when-list li::marker,
.trigger-list li::marker {
  color: var(--app-color-primary, #00d4ff);
}

.avoid-list li::marker {
  color: var(--app-color-danger);
}

.trigger-list {
  background: linear-gradient(135deg, rgba(103, 194, 58, 0.05), rgba(103, 194, 58, 0.02));
  padding: 14px 14px 14px 32px;
  border-radius: 6px;
  border: 1px solid rgba(103, 194, 58, 0.2);
}

.avoid-list {
  background: linear-gradient(135deg, rgba(245, 108, 108, 0.05), rgba(245, 108, 108, 0.02));
  padding: 14px 14px 14px 32px;
  border-radius: 6px;
  border: 1px solid rgba(245, 108, 108, 0.2);
}

/* 系统提示词区域 - 代码块样式 + 霓虹边框 */
.prompt-content {
  background-color: var(--app-bg-card, #12121a);
  border: 1px solid var(--app-color-primary, #00d4ff);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.15),
              inset 0 0 30px rgba(0, 212, 255, 0.03);
  border-radius: 8px;
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.prompt-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'SF Mono', 'Monaco', 'Consolas', 'Liberation Mono', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.7;
  color: #c0c0d0;
}

/* 语言切换按钮组样式 */
:deep(.el-radio-button__inner) {
  background-color: var(--app-bg-card, #12121a);
  border-color: var(--app-border-default, #2a2a3a);
  color: var(--app-text-tertiary);
}

:deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 168, 255, 0.1));
  border-color: var(--app-color-primary, #00d4ff);
  color: var(--app-color-primary, #00d4ff);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

/* 关闭按钮样式 */
:deep(.el-button) {
  background-color: transparent;
  border-color: var(--app-border-default, #2a2a3a);
  color: var(--app-text-tertiary);
}

:deep(.el-button:hover) {
  border-color: var(--app-color-primary, #00d4ff);
  color: var(--app-color-primary, #00d4ff);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.2);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk :deep(.el-dialog.config-detail-dialog) {
  background-color: var(--app-glass-bg);
  backdrop-filter: var(--app-glass-blur);
  -webkit-backdrop-filter: var(--app-glass-blur);
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow:
    0 25px 50px rgba(0, 0, 0, 0.6),
    0 0 40px rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.el-dialog.config-detail-dialog .el-dialog__header) {
  border-bottom: 2px solid rgba(0, 255, 255, 0.4);
  box-shadow: 0 4px 20px rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.el-dialog.config-detail-dialog .el-dialog__title) {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk :deep(.el-dialog.config-detail-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 8px rgba(0, 255, 255, 0.6));
}

html.cyberpunk :deep(.el-dialog.config-detail-dialog .el-dialog__footer) {
  border-top: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk .basic-info :deep(.el-descriptions__body) {
  background-color: rgba(26, 26, 46, 0.8) !important;
}

html.cyberpunk .basic-info :deep(.el-descriptions__table) {
  background-color: rgba(26, 26, 46, 0.8) !important;
}

html.cyberpunk .basic-info :deep(.el-descriptions__label) {
  background-color: rgba(0, 255, 255, 0.1) !important;
  color: var(--app-text-tertiary) !important;
  border-bottom: 1px solid rgba(0, 255, 255, 0.15) !important;
  border-color: rgba(0, 255, 255, 0.15) !important;
}

html.cyberpunk .basic-info :deep(.el-descriptions__content) {
  background-color: rgba(26, 26, 46, 0.6) !important;
  color: var(--app-text-primary) !important;
  border-bottom: 1px solid rgba(0, 255, 255, 0.1) !important;
  border-color: rgba(0, 255, 255, 0.1) !important;
}

html.cyberpunk .basic-info :deep(.el-descriptions__cell) {
  border-color: rgba(0, 255, 255, 0.1) !important;
  background-color: transparent !important;
}

html.cyberpunk .model-info.clickable {
  background: linear-gradient(var(--app-bg-card, #1a1a2e), var(--app-bg-card, #1a1a2e)) padding-box,
              linear-gradient(135deg, var(--app-color-primary, #00ffff), #ff00ff) border-box;
}

html.cyberpunk .model-info.clickable:hover {
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.5),
              inset 0 0 15px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .model-name {
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .section-title {
  border-left: 3px solid var(--app-color-primary, #00ffff);
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .fallback-item {
  background-color: rgba(26, 26, 46, 0.7) !important;
  border: 1px solid rgba(0, 255, 255, 0.15) !important;
}

html.cyberpunk .fallback-item:hover {
  border-color: rgba(0, 255, 255, 0.4) !important;
  box-shadow: 0 4px 16px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .fallback-item.current {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15), rgba(255, 0, 255, 0.08)) !important;
  border-color: var(--app-color-primary) !important;
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .fallback-item.current .fallback-index {
  background: linear-gradient(135deg, var(--app-color-primary, #00ffff), #00ccff);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .fallback-model {
  color: var(--app-text-primary) !important;
}

html.cyberpunk .providers {
  color: var(--app-text-tertiary) !important;
}

html.cyberpunk .trigger-list {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.1), rgba(0, 255, 136, 0.03));
  border: 1px solid rgba(0, 255, 136, 0.25);
}

html.cyberpunk .avoid-list {
  background: linear-gradient(135deg, rgba(255, 51, 102, 0.1), rgba(255, 51, 102, 0.03));
  border: 1px solid rgba(255, 51, 102, 0.25);
}

html.cyberpunk .prompt-content {
  background-color: rgba(26, 26, 46, 0.9) !important;
  border: 1px solid rgba(0, 255, 255, 0.3) !important;
  box-shadow:
    0 0 25px rgba(0, 255, 255, 0.2),
    inset 0 0 40px rgba(0, 255, 255, 0.05);
}

html.cyberpunk .prompt-content pre {
  color: var(--app-text-secondary) !important;
}

/* 赛博朋克 - 当前模型标签样式 */
html.cyberpunk .model-info.clickable {
  background: linear-gradient(rgba(26, 26, 46, 0.9), rgba(26, 26, 46, 0.9)) padding-box,
              linear-gradient(135deg, var(--app-color-primary, #00ffff), #ff00ff) border-box !important;
}

html.cyberpunk .model-info.clickable .el-tag {
  background-color: rgba(0, 255, 255, 0.15) !important;
  border-color: rgba(0, 255, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.cyberpunk .model-name {
  color: var(--app-color-primary) !important;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.5);
}

html.cyberpunk :deep(.el-radio-button__inner) {
  background-color: rgba(26, 26, 46, 0.8);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.25), rgba(255, 0, 255, 0.15));
  border-color: var(--app-color-primary);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.4);
}

html.cyberpunk :deep(.el-button:hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism :deep(.el-dialog.config-detail-dialog) {
  background-color: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.95);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.1);
}

html.glassmorphism :deep(.el-dialog.config-detail-dialog .el-dialog__header) {
  border-bottom: 2px solid rgba(37, 99, 235, 0.2);
  box-shadow: none;
}

html.glassmorphism :deep(.el-dialog.config-detail-dialog .el-dialog__title) {
  text-shadow: none;
}

html.glassmorphism :deep(.el-dialog.config-detail-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: none;
}

html.glassmorphism :deep(.el-dialog.config-detail-dialog .el-dialog__footer) {
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .basic-info :deep(.el-descriptions__body) {
  background-color: rgba(248, 250, 252, 0.8);
}

html.glassmorphism .basic-info :deep(.el-descriptions__label) {
  background-color: rgba(37, 99, 235, 0.05);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .basic-info :deep(.el-descriptions__content) {
  background-color: rgba(255, 255, 255, 0.6);
  border-bottom: 1px solid rgba(0, 0, 0, 0.03);
}

html.glassmorphism .model-info.clickable {
  background: linear-gradient(rgba(255, 255, 255, 0.8), rgba(255, 255, 255, 0.8)) padding-box,
              linear-gradient(135deg, var(--app-color-primary, #2563eb), #8b5cf6) border-box;
}

html.glassmorphism .model-info.clickable:hover {
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.15);
}

html.glassmorphism .model-name {
  text-shadow: none;
}

html.glassmorphism .section-title {
  border-left: 3px solid var(--app-color-primary, #2563eb);
  text-shadow: none;
}

html.glassmorphism .fallback-item {
  background-color: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .fallback-item:hover {
  border-color: rgba(37, 99, 235, 0.2);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

html.glassmorphism .fallback-item.current {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.08), rgba(139, 92, 246, 0.04));
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .fallback-item.current .fallback-index {
  background: linear-gradient(135deg, var(--app-color-primary, #2563eb), #3b82f6);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

html.glassmorphism .trigger-list {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.06), rgba(16, 185, 129, 0.02));
  border: 1px solid rgba(16, 185, 129, 0.15);
}

html.glassmorphism .avoid-list {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.06), rgba(239, 68, 68, 0.02));
  border: 1px solid rgba(239, 68, 68, 0.15);
}

html.glassmorphism .prompt-content {
  background-color: rgba(248, 250, 252, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.08);
  box-shadow: none;
}

html.glassmorphism .prompt-content pre {
  color: #374151;
}

html.glassmorphism :deep(.el-radio-button__inner) {
  background-color: rgba(255, 255, 255, 0.8);
  border-color: rgba(0, 0, 0, 0.08);
}

html.glassmorphism :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15), rgba(139, 92, 246, 0.1));
  border-color: var(--app-color-primary);
  box-shadow: none;
}

html.glassmorphism :deep(.el-button:hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.15);
}

/* ==================== 明色主题 (html.light - 非玻璃拟态) ==================== */
html.light :deep(.el-dialog.config-detail-dialog) {
  background-color: rgba(255, 255, 255, 0.98);
  border: 1px solid var(--app-border-default);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.15);
}

html.light :deep(.el-dialog.config-detail-dialog .el-dialog__header) {
  border-bottom: 2px solid var(--app-color-primary);
  box-shadow: none;
}

html.light :deep(.el-dialog.config-detail-dialog .el-dialog__title) {
  color: var(--app-text-primary);
  text-shadow: none;
}

html.light :deep(.el-dialog.config-detail-dialog .el-dialog__footer) {
  border-top: 1px solid var(--app-border-default);
}

html.light .basic-info :deep(.el-descriptions__body) {
  background-color: rgba(248, 250, 252, 0.9);
}

html.light .basic-info :deep(.el-descriptions__label) {
  background-color: rgba(0, 168, 232, 0.05);
  border-bottom: 1px solid var(--app-border-default);
}

html.light .basic-info :deep(.el-descriptions__content) {
  background-color: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid var(--app-border-default);
}

html.light .model-info.clickable {
  background: linear-gradient(rgba(255, 255, 255, 0.9), rgba(255, 255, 255, 0.9)) padding-box,
              linear-gradient(135deg, var(--app-color-primary), #3b82f6) border-box;
}

html.light .model-info.clickable:hover {
  box-shadow: 0 4px 16px rgba(0, 168, 232, 0.2);
}

html.light .model-name {
  color: var(--app-color-primary);
  text-shadow: none;
}

html.light .section-title {
  border-left: 3px solid var(--app-color-primary);
  color: var(--app-text-primary);
  text-shadow: none;
}

html.light .fallback-item {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid var(--app-border-default);
}

html.light .fallback-item:hover {
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

html.light .fallback-item.current {
  background: linear-gradient(135deg, rgba(0, 168, 232, 0.1), rgba(59, 130, 246, 0.05));
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 16px rgba(0, 168, 232, 0.15);
}

html.light .fallback-item.current .fallback-index {
  background: linear-gradient(135deg, var(--app-color-primary), #3b82f6);
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.3);
}

html.light .trigger-list {
  background: linear-gradient(135deg, rgba(103, 194, 58, 0.08), rgba(103, 194, 58, 0.03));
  border: 1px solid rgba(103, 194, 58, 0.2);
}

html.light .avoid-list {
  background: linear-gradient(135deg, rgba(245, 108, 108, 0.08), rgba(245, 108, 108, 0.03));
  border: 1px solid rgba(245, 108, 108, 0.2);
}

html.light .prompt-content {
  background-color: rgba(248, 250, 252, 0.95);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
}

html.light .prompt-content pre {
  color: #374151;
}

html.light :deep(.el-radio-button__inner) {
  background-color: rgba(255, 255, 255, 0.9);
  border-color: var(--app-border-default);
}

html.light :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, rgba(0, 168, 232, 0.15), rgba(59, 130, 246, 0.1));
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: none;
}

html.light :deep(.el-button:hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.15);
}

/* ==================== 暗色主题 (html.dark) ==================== */
html.dark :deep(.el-dialog.config-detail-dialog) {
  background-color: rgba(18, 18, 26, 0.95);
  border: 1px solid var(--app-border-default);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5), 0 0 30px rgba(0, 212, 255, 0.08);
}

html.dark :deep(.el-dialog.config-detail-dialog .el-dialog__header) {
  border-bottom: 2px solid var(--app-color-primary);
  box-shadow: 0 2px 10px rgba(0, 212, 255, 0.2);
}

html.dark :deep(.el-dialog.config-detail-dialog .el-dialog__title) {
  color: var(--app-text-primary);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

html.dark :deep(.el-dialog.config-detail-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 212, 255, 0.6));
}

html.dark :deep(.el-dialog.config-detail-dialog .el-dialog__footer) {
  border-top: 1px solid var(--app-border-default);
}

/* 暗色主题 - 弹窗内容区域背景（参考明色模式逻辑） */
html.dark :deep(.el-dialog.config-detail-dialog .el-dialog__body) {
  background-color: transparent;
}

/* 暗色主题 - 滚动内容容器不设置独立背景，由 el-dialog 统一处理 */
html.dark .detail-content {
  background-color: transparent;
}

/* 暗色主题 - 描述列表样式 */
html.dark .basic-info :deep(.el-descriptions__body) {
  background-color: var(--app-bg-card) !important;
}

html.dark .basic-info :deep(.el-descriptions__table) {
  background-color: var(--app-bg-card) !important;
}

html.dark .basic-info :deep(.el-descriptions__label) {
  background-color: rgba(0, 212, 255, 0.08) !important;
  color: var(--app-text-tertiary) !important;
  border-bottom: 1px solid var(--app-border-default) !important;
  border-color: var(--app-border-default) !important;
}

html.dark .basic-info :deep(.el-descriptions__content) {
  background-color: var(--app-bg-card) !important;
  color: var(--app-text-primary) !important;
  border-bottom: 1px solid var(--app-border-default) !important;
  border-color: var(--app-border-default) !important;
}

html.dark .basic-info :deep(.el-descriptions__cell) {
  border-color: var(--app-border-default) !important;
  background-color: transparent !important;
}

/* 暗色主题 - 标签样式 */
html.dark .basic-info :deep(.el-tag) {
  background-color: rgba(0, 212, 255, 0.15) !important;
  border-color: rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.dark .basic-info :deep(.el-tag--primary) {
  background-color: rgba(0, 212, 255, 0.15) !important;
  border-color: rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.dark .basic-info :deep(.el-tag--success) {
  background-color: rgba(16, 185, 129, 0.15) !important;
  border-color: rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

html.dark .basic-info :deep(.el-tag--info) {
  background-color: rgba(0, 212, 255, 0.1) !important;
  border-color: rgba(0, 212, 255, 0.25) !important;
  color: var(--app-color-primary) !important;
}

/* 暗色主题 - 模型信息样式 */
html.dark .model-info.clickable {
  background: linear-gradient(var(--app-bg-card), var(--app-bg-card)) padding-box,
              linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary)) border-box;
}

html.dark .model-info.clickable:hover {
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.4),
              inset 0 0 12px rgba(0, 212, 255, 0.1);
}

html.dark .model-name {
  color: var(--app-color-primary);
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
}

/* 暗色主题 - 区块标题 */
html.dark .section-title {
  border-left: 3px solid var(--app-color-primary);
  color: var(--app-text-primary);
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.2);
}

/* 暗色主题 - 回退模型链 */
html.dark .fallback-item {
  background-color: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
}

html.dark .fallback-item:hover {
  border-color: rgba(0, 212, 255, 0.3) !important;
  box-shadow: 0 2px 12px rgba(0, 212, 255, 0.1);
}

html.dark .fallback-item.current {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.12), rgba(0, 168, 255, 0.06)) !important;
  border-color: var(--app-color-primary) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

html.dark .fallback-item.current .fallback-index {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.4);
}

html.dark .fallback-model {
  color: var(--app-text-primary) !important;
}

html.dark .variant-badge {
  background-color: rgba(16, 185, 129, 0.15) !important;
  border-color: rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

html.dark .providers {
  color: var(--app-text-tertiary) !important;
}

/* 暗色主题 - 触发条件列表 */
html.dark .trigger-list {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.08), rgba(16, 185, 129, 0.03));
  border: 1px solid rgba(16, 185, 129, 0.2);
}

html.dark .trigger-list li::marker {
  color: var(--app-color-success);
}

/* 暗色主题 - 避免场景列表 */
html.dark .avoid-list {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.08), rgba(239, 68, 68, 0.03));
  border: 1px solid rgba(239, 68, 68, 0.2);
}

html.dark .avoid-list li::marker {
  color: var(--app-color-danger);
}

/* 暗色主题 - 系统提示词区域 */
html.dark .prompt-content {
  background-color: var(--app-bg-card) !important;
  border: 1px solid var(--app-color-primary) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.12),
              inset 0 0 30px rgba(0, 212, 255, 0.03);
}

html.dark .prompt-content pre {
  color: var(--app-text-secondary) !important;
}

/* 暗色主题 - 当前模型标签样式 */
html.dark .model-info.clickable {
  background: linear-gradient(var(--app-bg-card), var(--app-bg-card)) padding-box,
              linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary)) border-box !important;
}

html.dark .model-info.clickable .el-tag {
  background-color: rgba(0, 212, 255, 0.15) !important;
  border-color: rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.dark .model-name {
  color: var(--app-color-primary) !important;
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
}

/* 暗色主题 - 语言切换按钮 */
html.dark :deep(.el-radio-button__inner) {
  background-color: var(--app-bg-card);
  border-color: var(--app-border-default);
  color: var(--app-text-tertiary);
}

html.dark :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 168, 255, 0.12));
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.25);
}

/* 暗色主题 - 关闭按钮 */
html.dark :deep(.el-button) {
  background-color: transparent;
  border-color: var(--app-border-default);
  color: var(--app-text-secondary);
}

html.dark :deep(.el-button:hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.2);
}

/* 暗色主题 - 滚动条样式 */
html.dark .detail-content::-webkit-scrollbar {
  width: 6px;
}

html.dark .detail-content::-webkit-scrollbar-track {
  background: var(--app-bg-base);
  border-radius: 3px;
}

html.dark .detail-content::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

html.dark .detail-content::-webkit-scrollbar-thumb:hover {
  background: var(--app-border-hover);
}
</style>
