<script setup lang="ts">
/**
 * 模型选择抽屉组件
 * 从右侧滑出，支持搜索和按供应商分组选择模型
 */
import { ref, computed, watch } from 'vue'
import { Search } from '@element-plus/icons-vue'
import type { Model } from '@/types'

const props = defineProps<{
  visible: boolean
  currentModel: string
  models: Model[]
  title?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'select': [modelId: string]
}>()

// 搜索关键词
const searchKeyword = ref('')

// 抽屉关闭时清空搜索
watch(() => props.visible, (newVal) => {
  if (!newVal) {
    searchKeyword.value = ''
  }
})

// 按供应商分组的模型（过滤后）
const groupedModels = computed(() => {
  const groups = new Map<string, Model[]>()
  
  for (const model of props.models) {
    // 搜索过滤
    if (searchKeyword.value) {
      const keyword = searchKeyword.value.toLowerCase()
      const nameMatch = model.name.toLowerCase().includes(keyword)
      const idMatch = model.id.toLowerCase().includes(keyword)
      if (!nameMatch && !idMatch) continue
    }
    
    const provider = model.provider
    if (!groups.has(provider)) {
      groups.set(provider, [])
    }
    groups.get(provider)!.push(model)
  }
  
  // 按供应商名称排序
  return new Map(Array.from(groups.entries()).sort((a, b) => a[0].localeCompare(b[0])))
})

// 当前选中的模型名称
const currentModelName = computed(() => {
  return props.models.find(m => m.id === props.currentModel)?.name || props.currentModel
})

// 关闭抽屉
function handleClose() {
  emit('update:visible', false)
}

// 选择模型
function handleSelectModel(modelId: string) {
  emit('select', modelId)
  handleClose()
}

// 判断是否是当前选中的模型
function isCurrentModel(modelId: string) {
  return modelId === props.currentModel
}
</script>

<template>
  <el-drawer
    :model-value="visible"
    :title="title || '选择模型'"
    direction="rtl"
    size="400px"
    :modal-class="'model-drawer-modal'"
    :drawer-class="'model-glass-drawer'"
    @update:model-value="emit('update:visible', $event)"
  >
    <div class="drawer-content">
      <!-- 当前选中的模型 -->
      <div class="current-model">
        <span class="label">当前模型:</span>
        <span class="value">{{ currentModelName }}</span>
      </div>
      
      <!-- 搜索框 -->
      <el-input
        v-model="searchKeyword"
        placeholder="搜索模型名称或 ID..."
        :prefix-icon="Search"
        clearable
        class="search-input"
      />
      
      <!-- 模型列表 -->
      <div class="model-list">
        <template v-if="groupedModels.size > 0">
          <div
            v-for="[provider, providerModels] in groupedModels"
            :key="provider"
            class="provider-group"
          >
            <div class="provider-header">
              <span class="provider-name">{{ provider }}</span>
              <span class="provider-count">{{ providerModels.length }}</span>
            </div>
            
            <div class="model-items">
              <div
                v-for="model in providerModels"
                :key="model.id"
                class="model-item"
                :class="{ 'is-current': isCurrentModel(model.id) }"
                @click="handleSelectModel(model.id)"
              >
                <div class="model-info">
                  <span class="model-name">{{ model.name }}</span>
                  <span class="model-id">{{ model.id }}</span>
                </div>
                <el-icon v-if="isCurrentModel(model.id)" class="check-icon">
                  <svg viewBox="0 0 1024 1024">
                    <path fill="currentColor" d="M406.656 706.944L195.84 496.256a32 32 0 10-45.248 45.248l256 256 512-512a32 32 0 00-45.248-45.248L406.592 706.944z"/>
                  </svg>
                </el-icon>
              </div>
            </div>
          </div>
        </template>
        
        <el-empty
          v-else
          description="没有找到匹配的模型"
          :image-size="80"
        />
      </div>
    </div>
  </el-drawer>
</template>

<style scoped>
/* 抽屉玻璃效果 - 穿透到外层 */
:deep(.model-glass-drawer) {
  background: var(--app-glass-bg) !important;
  backdrop-filter: blur(12px) !important;
  -webkit-backdrop-filter: blur(12px) !important;
  border-left: 1px solid var(--app-border-default);
}

:deep(.model-glass-drawer .el-drawer__header) {
  color: var(--app-text-primary);
  border-bottom: 1px solid var(--app-border-default);
  padding: 16px 20px;
  margin-bottom: 0;
}

:deep(.model-glass-drawer .el-drawer__title) {
  color: var(--app-text-primary);
  font-size: 16px;
  font-weight: 600;
}

:deep(.model-glass-drawer .el-drawer__close-btn) {
  color: var(--app-text-secondary);
  transition: all 0.3s ease;
}

:deep(.model-glass-drawer .el-drawer__close-btn:hover) {
  color: var(--app-color-primary);
  transform: rotate(90deg);
}

.drawer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
}

/* 当前选中模型 */
.current-model {
  padding: 12px 16px;
  background: rgba(42, 42, 58, 0.6);
  border-radius: 8px;
  margin-bottom: 16px;
  border: 1px solid var(--app-border-default);
}

.current-model .label {
  color: var(--app-text-secondary);
  font-size: 13px;
}

.current-model .value {
  color: var(--app-color-primary);
  font-weight: 500;
  margin-left: 8px;
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.4);
}

/* 搜索框 - 霓虹边框聚焦效果 */
:deep(.search-input .el-input__wrapper) {
  background: rgba(42, 42, 58, 0.6);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
  transition: all 0.3s ease;
}

:deep(.search-input .el-input__inner) {
  color: var(--app-text-primary);
  background: transparent;
}

:deep(.search-input .el-input__inner::placeholder) {
  color: var(--app-text-secondary);
}

:deep(.search-input .el-input__icon) {
  color: var(--app-text-secondary);
}

:deep(.search-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: var(--app-glow-primary), inset 0 0 10px rgba(0, 212, 255, 0.1);
}

.search-input {
  margin-bottom: 16px;
}

/* 模型列表 */
.model-list {
  flex: 1;
  overflow-y: auto;
}

.provider-group {
  margin-bottom: 20px;
}

/* 供应商分组标题 - 渐变背景 */
.provider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.15) 0%, rgba(0, 212, 255, 0.05) 100%);
  border-radius: 8px;
  margin-bottom: 10px;
  border: 1px solid rgba(0, 212, 255, 0.2);
}

.provider-name {
  font-weight: 600;
  color: var(--app-color-primary);
  font-size: 14px;
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
}

.provider-count {
  font-size: 11px;
  color: var(--app-text-secondary);
  background: rgba(42, 42, 58, 0.8);
  padding: 3px 10px;
  border-radius: 12px;
  border: 1px solid var(--app-border-default);
}

.model-items {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* 模型列表项 - 悬停高亮 + 霓虹边框 */
.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  border: 1px solid transparent;
  background: rgba(42, 42, 58, 0.3);
}

.model-item:hover {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2), inset 0 0 10px rgba(0, 212, 255, 0.05);
  transform: translateX(4px);
}

/* 选中状态 - 霓虹边框 + 发光 */
.model-item.is-current {
  background: rgba(0, 212, 255, 0.12);
  border-color: var(--app-color-primary);
  box-shadow: var(--app-glow-strong), inset 0 0 15px rgba(0, 212, 255, 0.1);
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.model-name {
  font-weight: 500;
  color: var(--app-text-primary);
  font-size: 14px;
}

.model-id {
  font-size: 11px;
  color: var(--app-text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
}

.model-item:hover .model-name {
  color: var(--app-color-primary);
}

.check-icon {
  color: var(--app-color-primary);
  font-size: 18px;
  filter: drop-shadow(0 0 6px rgba(0, 212, 255, 0.6));
}

/* 空状态 */
:deep(.el-empty__description) {
  color: var(--app-text-secondary);
}

/* 滚动条样式 - 深色主题 */
.model-list::-webkit-scrollbar {
  width: 6px;
}

.model-list::-webkit-scrollbar-track {
  background: rgba(42, 42, 58, 0.3);
  border-radius: 3px;
}

.model-list::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.4);
  border-radius: 3px;
}

.model-list::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk :deep(.model-glass-drawer) {
  background: rgba(26, 26, 46, 0.95) !important;
  border-left: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: -10px 0 40px rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.model-glass-drawer .el-drawer__header) {
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.model-glass-drawer .el-drawer__title) {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk :deep(.model-glass-drawer .el-drawer__close-btn:hover) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 8px rgba(0, 255, 255, 0.6));
}

html.cyberpunk .current-model {
  background: rgba(0, 255, 255, 0.08);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .current-model .value {
  text-shadow: 0 0 12px rgba(0, 255, 255, 0.6);
}

html.cyberpunk :deep(.search-input .el-input__wrapper) {
  background: rgba(26, 26, 46, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.search-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3), inset 0 0 15px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .provider-header {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 0%, rgba(0, 255, 255, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .provider-name {
  text-shadow: 0 0 12px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .provider-count {
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.25);
}

html.cyberpunk .model-item {
  background: rgba(26, 26, 46, 0.5);
  border: 1px solid transparent;
}

html.cyberpunk .model-item:hover {
  background: rgba(0, 255, 255, 0.1);
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.25), inset 0 0 15px rgba(0, 255, 255, 0.08);
}

html.cyberpunk .model-item.is-current {
  background: rgba(0, 255, 255, 0.15);
  border-color: var(--app-color-primary);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.4), inset 0 0 20px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .model-item:hover .model-name {
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .check-icon {
  filter: drop-shadow(0 0 10px rgba(0, 255, 255, 0.8));
}

html.cyberpunk .model-list::-webkit-scrollbar-track {
  background: rgba(0, 255, 255, 0.05);
}

html.cyberpunk .model-list::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.3);
}

html.cyberpunk .model-list::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism :deep(.model-glass-drawer) {
  background: rgba(255, 255, 255, 0.9) !important;
  border-left: 1px solid rgba(255, 255, 255, 0.95);
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.08);
}

html.glassmorphism :deep(.model-glass-drawer .el-drawer__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism :deep(.model-glass-drawer .el-drawer__title) {
  text-shadow: none;
}

html.glassmorphism :deep(.model-glass-drawer .el-drawer__close-btn:hover) {
  color: var(--app-color-primary);
  filter: none;
}

html.glassmorphism .current-model {
  background: rgba(37, 99, 235, 0.08);
  border: 1px solid rgba(37, 99, 235, 0.15);
  box-shadow: none;
}

html.glassmorphism .current-model .value {
  text-shadow: none;
}

html.glassmorphism :deep(.search-input .el-input__wrapper) {
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.08);
}

html.glassmorphism :deep(.search-input .el-input__wrapper.is-focus) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 1px var(--app-color-primary);
}

html.glassmorphism .provider-header {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.1) 0%, rgba(37, 99, 235, 0.03) 100%);
  border: 1px solid rgba(37, 99, 235, 0.15);
  box-shadow: none;
}

html.glassmorphism .provider-name {
  text-shadow: none;
}

html.glassmorphism .provider-count {
  background: rgba(37, 99, 235, 0.08);
  border: 1px solid rgba(37, 99, 235, 0.15);
}

html.glassmorphism .model-item {
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .model-item:hover {
  background: rgba(37, 99, 235, 0.08);
  border-color: rgba(37, 99, 235, 0.25);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .model-item.is-current {
  background: rgba(37, 99, 235, 0.1);
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.15);
}

html.glassmorphism .model-item:hover .model-name {
  text-shadow: none;
}

html.glassmorphism .check-icon {
  filter: none;
}

html.glassmorphism .model-list::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.02);
}

html.glassmorphism .model-list::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
}

html.glassmorphism .model-list::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
}
</style>
