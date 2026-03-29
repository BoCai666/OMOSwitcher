<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Preset } from '@/types'

const props = defineProps<{
  visible: boolean
  presets: Preset[]
  currentPreset?: string | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'save': [name: string, description?: string]
  'load': [preset: Preset]
  'delete': [name: string]
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

const newPresetName = ref('')
const newPresetDescription = ref('')

const handleSave = () => {
  if (!newPresetName.value.trim()) return
  emit('save', newPresetName.value.trim(), newPresetDescription.value.trim() || undefined)
  newPresetName.value = ''
  newPresetDescription.value = ''
}

const handleLoad = (preset: Preset) => {
  emit('load', preset)
  dialogVisible.value = false
}

const handleDelete = (name: string) => {
  emit('delete', name)
}

// 检查是否为当前预设
const isCurrentPreset = (name: string): boolean => {
  return props.currentPreset === name
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="预设管理"
    width="500px"
    class="glass-dialog"
    align-center
  >
    <!-- 保存新预设 -->
    <div class="save-section">
      <h4 class="section-title">保存当前配置</h4>
      <el-input
        v-model="newPresetName"
        placeholder="预设名称"
        class="preset-input"
      />
      <el-input
        v-model="newPresetDescription"
        placeholder="预设描述（可选）"
        class="preset-input"
      />
      <el-button type="primary" @click="handleSave" :disabled="!newPresetName.trim()">
        保存预设
      </el-button>
    </div>

    <!-- 预设列表 -->
    <div class="preset-list">
      <h4 class="section-title">已保存的预设</h4>
      <el-empty v-if="presets.length === 0" description="暂无预设" />
      <div v-else class="preset-items">
        <div
          v-for="preset in presets"
          :key="preset.name"
          class="preset-card"
          :class="{ 'current-preset': isCurrentPreset(preset.name) }"
        >
          <div class="preset-content">
            <div class="preset-info">
              <span class="preset-name">{{ preset.name }}</span>
              <span v-if="preset.description" class="preset-description">{{ preset.description }}</span>
              <span class="preset-time">{{ new Date(preset.updatedAt).toLocaleDateString() }}</span>
            </div>
            <div class="preset-actions">
              <el-button
                size="small"
                type="primary"
                class="load-btn"
                @click="handleLoad(preset)"
              >
                加载
              </el-button>
              <el-button
                size="small"
                class="delete-btn"
                @click="handleDelete(preset.name)"
              >
                删除
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
/* ==================== 玻璃效果对话框 ==================== */
:deep(.glass-dialog .el-dialog) {
  background: rgba(18, 18, 26, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
  box-shadow: var(--app-shadow-xl), var(--app-shadow-glow-primary);
}

:deep(.glass-dialog .el-dialog__header) {
  padding: var(--app-spacing-5) var(--app-spacing-6);
  border-bottom: 1px solid var(--app-border-default);
  margin-right: 0;
}

:deep(.glass-dialog .el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
}

:deep(.glass-dialog .el-dialog__body) {
  padding: var(--app-spacing-5) var(--app-spacing-6);
}

:deep(.glass-dialog .el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-tertiary);
  transition: color var(--app-transition-fast);
}

:deep(.glass-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
}

/* ==================== 区块标题 ==================== */
.section-title {
  margin: 0 0 var(--app-spacing-3) 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
}

/* ==================== 保存区域 ==================== */
.save-section {
  margin-bottom: var(--app-spacing-6);
  padding-bottom: var(--app-spacing-5);
  border-bottom: 1px solid var(--app-border-default);
}

.preset-input {
  margin-bottom: var(--app-spacing-3);
}

:deep(.preset-input .el-input__wrapper) {
  background: var(--app-bg-elevated);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  transition: all var(--app-transition-normal);
}

:deep(.preset-input .el-input__wrapper:hover) {
  border-color: var(--app-border-hover);
}

:deep(.preset-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 1px var(--app-color-primary), var(--app-shadow-glow-primary);
}

:deep(.preset-input .el-input__inner) {
  color: var(--app-text-primary);
  background: transparent;
}

/* ==================== 预设列表区域 ==================== */
.preset-list {
  padding-top: var(--app-spacing-2);
}

.preset-items {
  max-height: 300px;
  overflow-y: auto;
  padding-right: var(--app-spacing-2);
}

/* 自定义滚动条 */
.preset-items::-webkit-scrollbar {
  width: 4px;
}

.preset-items::-webkit-scrollbar-track {
  background: transparent;
}

.preset-items::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: var(--app-radius-full);
}

.preset-items::-webkit-scrollbar-thumb:hover {
  background: var(--app-border-hover);
}

/* ==================== 预设卡片样式 ==================== */
.preset-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  padding: var(--app-spacing-3) var(--app-spacing-4);
  margin-bottom: var(--app-spacing-3);
  transition: all var(--app-transition-normal);
  position: relative;
  overflow: hidden;
}

.preset-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--app-color-primary), transparent);
  opacity: 0;
  transition: opacity var(--app-transition-normal);
}

.preset-card:hover {
  border-color: var(--app-color-primary);
  transform: translateY(-2px);
  box-shadow: var(--app-shadow-md), var(--app-shadow-glow-primary);
}

.preset-card:hover::before {
  opacity: 0.6;
}

/* ==================== 当前预设霓虹高亮 ==================== */
.preset-card.current-preset {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 1px var(--app-color-primary), var(--app-shadow-glow-intense);
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, var(--app-bg-card) 100%);
}

.preset-card.current-preset::before {
  opacity: 1;
}

.preset-card.current-preset::after {
  content: '当前';
  position: absolute;
  top: var(--app-spacing-1);
  right: var(--app-spacing-2);
  font-size: 10px;
  font-weight: 600;
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.15);
  padding: 2px 8px;
  border-radius: var(--app-radius-full);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.preset-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--app-spacing-4);
}

.preset-info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.preset-name {
  font-weight: 500;
  color: var(--app-text-primary);
  font-size: 14px;
  margin-bottom: 2px;
}

.preset-description {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preset-time {
  font-size: 11px;
  color: var(--app-text-disabled);
}

.preset-actions {
  display: flex;
  gap: var(--app-spacing-2);
  flex-shrink: 0;
}

/* ==================== 按钮样式 ==================== */
:deep(.load-btn) {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  border: none;
  color: var(--app-text-inverse);
  font-weight: 500;
  transition: all var(--app-transition-normal);
}

:deep(.load-btn:hover) {
  transform: scale(1.05);
  box-shadow: var(--app-shadow-glow-primary);
}

:deep(.delete-btn) {
  background: transparent;
  border: 1px solid var(--app-border-default);
  color: var(--app-text-tertiary);
  font-weight: 500;
  transition: all var(--app-transition-normal);
}

:deep(.delete-btn:hover) {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--app-color-danger);
  color: var(--app-color-danger);
  transform: scale(1.05);
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.3);
}

/* ==================== 空状态 ==================== */
:deep(.el-empty .el-empty__description) {
  color: var(--app-text-tertiary);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk :deep(.glass-dialog .el-dialog) {
  background: rgba(26, 26, 46, 0.95);
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow:
    0 25px 50px rgba(0, 0, 0, 0.6),
    0 0 40px rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.glass-dialog .el-dialog__header) {
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.glass-dialog .el-dialog__title) {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk :deep(.glass-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 8px rgba(0, 255, 255, 0.6));
}

html.cyberpunk .section-title {
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .save-section {
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.preset-input .el-input__wrapper) {
  background: rgba(26, 26, 46, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.preset-input .el-input__wrapper:hover) {
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk :deep(.preset-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3), inset 0 0 15px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .preset-card {
  background: rgba(26, 26, 46, 0.7);
  border: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .preset-card::before {
  background: linear-gradient(90deg, transparent, var(--app-color-primary), transparent);
}

html.cyberpunk .preset-card:hover {
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .preset-card.current-preset {
  border-color: var(--app-color-primary);
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.1) 0%, rgba(26, 26, 46, 0.7) 100%);
  box-shadow: 0 0 30px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .preset-card.current-preset::after {
  background: rgba(0, 255, 255, 0.2);
  color: var(--app-color-primary);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.6);
}

html.cyberpunk :deep(.load-btn) {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(255, 0, 255, 0.15));
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.load-btn:hover) {
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.4);
}

html.cyberpunk :deep(.delete-btn:hover) {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--app-color-danger);
  box-shadow: 0 0 15px rgba(255, 51, 102, 0.4);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism :deep(.glass-dialog .el-dialog) {
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.95);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.1);
}

html.glassmorphism :deep(.glass-dialog .el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism :deep(.glass-dialog .el-dialog__title) {
  text-shadow: none;
}

html.glassmorphism :deep(.glass-dialog .el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
}

html.glassmorphism .section-title {
  text-shadow: none;
}

html.glassmorphism .save-section {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism :deep(.preset-input .el-input__wrapper) {
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.08);
}

html.glassmorphism :deep(.preset-input .el-input__wrapper:hover) {
  border-color: rgba(0, 0, 0, 0.15);
}

html.glassmorphism :deep(.preset-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 1px var(--app-color-primary);
}

html.glassmorphism .preset-card {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .preset-card::before {
  background: linear-gradient(90deg, transparent, var(--app-color-primary), transparent);
}

html.glassmorphism .preset-card:hover {
  border-color: rgba(37, 99, 235, 0.2);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
}

html.glassmorphism .preset-card.current-preset {
  border-color: var(--app-color-primary);
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.08) 0%, rgba(255, 255, 255, 0.7) 100%);
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .preset-card.current-preset::after {
  background: rgba(37, 99, 235, 0.1);
  color: var(--app-color-primary);
}

html.glassmorphism :deep(.load-btn) {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  border: none;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.25);
}

html.glassmorphism :deep(.load-btn:hover) {
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.35);
}

html.glassmorphism :deep(.delete-btn) {
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

html.glassmorphism :deep(.delete-btn:hover) {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--app-color-danger);
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.15);
}

html.glassmorphism .preset-items::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
}

html.glassmorphism .preset-items::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
}
</style>
