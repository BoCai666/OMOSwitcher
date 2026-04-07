<script setup lang="ts">
/**
 * 模型选择弹窗组件
 * 双栏布局：左侧供应商列表 + 右侧模型网格
 * 数据来源：模型管理页的可用模型列表 (getAvailableModels)
 */
import { ref, computed, watch } from 'vue'
import { Search, Check } from '@element-plus/icons-vue'
import { getAvailableModels, type AvailableModel } from '@/services/opencodeModels'

const props = defineProps<{
  visible: boolean
  currentModel: string
  title?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'select': [modelId: string]
}>()

// 对话框可见性（双向绑定)
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 可用模型列表（从模型管理页获取）
const availableModels = ref<AvailableModel[]>([])
const loading = ref(false)

// 搜索关键词
const searchKeyword = ref('')

// 当前选中的供应商
const selectedProvider = ref<string | null>(null)

// 能力筛选
const capabilityFilter = ref<'all' | 'tool' | 'reasoning' | 'attachment'>('all')

// 加载可用模型
async function loadAvailableModels() {
  loading.value = true
  try {
    availableModels.value = await getAvailableModels()
    // 默认选中第一个供应商
    if (availableModels.value.length > 0 && !selectedProvider.value) {
      selectedProvider.value = availableModels.value[0].provider
    }
  } catch (error) {
    console.error('加载可用模型失败:', error)
    availableModels.value = []
  } finally {
    loading.value = false
  }
}

// 弹窗打开时加载数据
watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadAvailableModels()
    searchKeyword.value = ''
    capabilityFilter.value = 'all'
  }
})

// 按供应商分组的模型
const providerStats = computed(() => {
  const stats = new Map<string, AvailableModel[]>()
  
  for (const model of availableModels.value) {
    const provider = model.provider
    if (!stats.has(provider)) {
      stats.set(provider, [])
    }
    stats.get(provider)!.push(model)
  }
  
  // 按模型数量排序
  return new Map(Array.from(stats.entries()).sort((a, b) => b[1].length - a[1].length))
})

// 供应商列表（过滤搜索）
const filteredProviders = computed(() => {
  if (!searchKeyword.value) {
    return Array.from(providerStats.value.keys())
  }
  const keyword = searchKeyword.value.toLowerCase()
  return Array.from(providerStats.value.keys()).filter(provider => 
    provider.toLowerCase().includes(keyword) ||
    providerStats.value.get(provider)!.some(m => 
      m.name.toLowerCase().includes(keyword) || m.id.toLowerCase().includes(keyword)
    )
  )
})

// 当前供应商的模型（过滤搜索 + 能力筛选）
const currentProviderModels = computed(() => {
  if (!selectedProvider.value) return []
  
  let models = providerStats.value.get(selectedProvider.value) || []
  
  // 搜索过滤
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    models = models.filter(m => 
      m.name.toLowerCase().includes(keyword) || m.id.toLowerCase().includes(keyword)
    )
  }
  
  // 能力筛选
  if (capabilityFilter.value !== 'all') {
    switch (capabilityFilter.value) {
      case 'tool':
        models = models.filter(m => m.tool_call)
        break
      case 'reasoning':
        models = models.filter(m => m.reasoning)
        break
      case 'attachment':
        models = models.filter(m => m.attachment)
        break
    }
  }
  
  return models
})

// 当前模型 ID（数据源原始格式）
const currentModelId = computed(() => props.currentModel)

// 关闭弹窗
function handleClose() {
  dialogVisible.value = false
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

// 选择供应商
function selectProvider(provider: string) {
  selectedProvider.value = provider
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :title="title || '选择模型'"
    width="850px"
    class="model-select-dialog"
    destroy-on-close
    append-to=".app-main"
    align-center
  >
    <div class="dialog-content">
      <!-- 顶部：搜索 + 当前模型 -->
      <div class="dialog-header">
        <div class="search-wrapper">
          <el-input
            v-model="searchKeyword"
            placeholder="搜索供应商或模型..."
            :prefix-icon="Search"
            clearable
            class="search-input"
          />
        </div>
        <div class="current-model-display">
          <span class="label">当前:</span>
          <span class="model-id-text">{{ currentModelId }}</span>
        </div>
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="loading-container">
        <el-icon class="is-loading" :size="32"><Loading /></el-icon>
        <span>加载模型列表...</span>
      </div>

      <!-- 双栏布局 -->
      <div v-else class="dual-panel">
        <!-- 左侧：供应商列表 -->
        <div class="provider-panel">
          <div class="panel-header">
            <div class="header-title">
              <span>供应商</span>
              <span class="count-badge">{{ filteredProviders.length }}</span>
            </div>
          </div>
          <div class="provider-list">
            <div
              v-for="provider in filteredProviders"
              :key="provider"
              class="provider-item"
              :class="{ active: provider === selectedProvider }"
              @click="selectProvider(provider)"
            >
              <span class="provider-name">{{ provider }}</span>
            </div>
          </div>
        </div>

        <!-- 右侧：模型网格 -->
        <div class="model-panel">
          <div class="panel-header">
            <div class="header-left">
              <div class="header-title">
                <span>{{ selectedProvider || '选择供应商' }}</span>
              </div>
            </div>
            <!-- 能力筛选 -->
            <div class="capability-filters">
              <button
                class="filter-btn"
                :class="{ active: capabilityFilter === 'all' }"
                @click="capabilityFilter = 'all'"
              >全部</button>
              <button
                class="filter-btn tool"
                :class="{ active: capabilityFilter === 'tool' }"
                @click="capabilityFilter = 'tool'"
              >
                <span class="filter-dot tool" />
                工具
              </button>
              <button
                class="filter-btn reasoning"
                :class="{ active: capabilityFilter === 'reasoning' }"
                @click="capabilityFilter = 'reasoning'"
              >
                <span class="filter-dot reasoning" />
                推理
              </button>
              <button
                class="filter-btn attachment"
                :class="{ active: capabilityFilter === 'attachment' }"
                @click="capabilityFilter = 'attachment'"
              >
                <span class="filter-dot attachment" />
                图片
              </button>
            </div>
          </div>
          
          <div class="model-grid">
            <div
              v-for="model in currentProviderModels"
              :key="model.id"
              class="model-card"
              :class="{ active: isCurrentModel(model.id) }"
              @click="handleSelectModel(model.id)"
            >
              <div class="model-card-header">
                <span class="model-name">{{ model.name }}</span>
                <div v-if="isCurrentModel(model.id)" class="current-badge">
                  <el-icon><Check /></el-icon>
                </div>
              </div>
              <div class="model-card-id">{{ model.id.split('/')[1] }}</div>
              <div class="model-card-tags">
                <span v-if="model.tool_call" class="capability-tag tool">
                  <span class="tag-dot" />
                  工具
                </span>
                <span v-if="model.reasoning" class="capability-tag reasoning">
                  <span class="tag-dot" />
                  推理
                </span>
                <span v-if="model.attachment" class="capability-tag attachment">
                  <span class="tag-dot" />
                  图片
                </span>
              </div>
            </div>
            
            <el-empty
              v-if="currentProviderModels.length === 0 && selectedProvider"
              description="没有匹配的模型"
              :image-size="80"
            />
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
/* 弹窗样式 */
:deep(.el-dialog) {
  background: var(--app-bg-card);
  border-radius: var(--app-radius-lg);
  border: 1px solid var(--app-border-default);
}

:deep(.el-dialog__header) {
  padding: 16px 24px;
  border-bottom: 1px solid var(--app-border-default);
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
  font-size: 16px;
}

:deep(.el-dialog__body) {
  padding: 0;
}

:deep(.el-dialog__close) {
  color: var(--app-text-secondary);
}

:deep(.el-dialog__close:hover) {
  color: var(--app-color-primary);
}

.dialog-content {
  display: flex;
  flex-direction: column;
  height: 520px;
}

/* 顶部区域 */
.dialog-header {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--app-border-default);
  background: var(--app-bg-elevated);
}

.search-wrapper {
  flex: 1;
  max-width: 280px;
}

.search-input :deep(.el-input__wrapper) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
}

.search-input :deep(.el-input__inner) {
  color: var(--app-text-primary);
}

.current-model-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--app-bg-card);
  border-radius: var(--app-radius-md);
  border: 1px solid var(--app-border-default);
}

.current-model-display .label {
  font-size: 13px;
  color: var(--app-text-tertiary);
  flex-shrink: 0;
}

.current-model-display .model-id-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-color-primary);
  font-family: 'Cascadia Code', 'SF Mono', monospace;
}

/* 加载状态 */
.loading-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--app-text-secondary);
}

.loading-container .is-loading {
  animation: spin 1s linear infinite;
  color: var(--app-color-primary);
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 双栏布局 */
.dual-panel {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 左侧供应商面板 */
.provider-panel {
  width: 220px;
  border-right: 1px solid var(--app-border-default);
  display: flex;
  flex-direction: column;
  background: var(--app-bg-elevated);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--app-border-default);
  background: var(--app-bg-card);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-secondary);
}

.count-badge {
  background: var(--app-bg-active);
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  color: var(--app-text-tertiary);
  font-weight: 500;
}

.provider-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.provider-item {
  padding: 10px 16px;
  margin-bottom: 4px;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: var(--app-radius-md);
  border: 1px solid transparent;
}

.provider-item:hover {
  background: var(--app-bg-hover);
  border-color: var(--app-border-default);
}

.provider-item.active {
  background: var(--app-bg-active);
  border-color: var(--app-color-primary);
}

.provider-item .provider-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--app-text-primary);
}

.provider-item.active .provider-name {
  color: var(--app-color-primary);
  font-weight: 600;
}

/* 右侧模型面板 */
.model-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.model-panel .panel-header {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

/* 能力筛选按钮 */
.capability-filters {
  display: flex;
  gap: 6px;
}

.filter-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  background: var(--app-bg-card);
  color: var(--app-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.filter-btn:hover {
  border-color: var(--app-border-hover);
  color: var(--app-text-primary);
}

.filter-btn.active {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.1);
}

.filter-btn.tool.active {
  border-color: #10b981;
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.filter-btn.reasoning.active {
  border-color: #a855f7;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.1);
}

.filter-btn.attachment.active {
  border-color: #f59e0b;
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.filter-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.filter-dot.tool {
  background: #10b981;
}

.filter-dot.reasoning {
  background: #a855f7;
}

.filter-dot.attachment {
  background: #f59e0b;
}

/* 模型网格 */
.model-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
  padding: 16px;
  overflow-y: auto;
  align-content: start;
}

/* 模型卡片 */
.model-card {
  padding: 14px;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.model-card:hover {
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.model-card.active {
  border-color: var(--app-color-primary);
  background: var(--app-bg-active);
  box-shadow: 0 4px 16px rgba(0, 212, 255, 0.15);
}

.model-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.model-card .model-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--app-text-primary);
  line-height: 1.3;
  word-break: break-word;
}

.model-card.active .model-name {
  color: var(--app-color-primary);
}

.current-badge {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--app-color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
}

.model-card-id {
  font-size: 11px;
  color: var(--app-text-tertiary);
  margin-top: 6px;
  font-family: 'Cascadia Code', 'SF Mono', monospace;
}

.model-card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}

.capability-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  padding: 3px 8px;
  border-radius: var(--app-radius-sm);
  font-weight: 500;
}

.capability-tag .tag-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
}

.capability-tag.tool {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.capability-tag.tool .tag-dot {
  background: #10b981;
}

.capability-tag.reasoning {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
  border: 1px solid rgba(168, 85, 247, 0.2);
}

.capability-tag.reasoning .tag-dot {
  background: #a855f7;
}

.capability-tag.attachment {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.2);
}

.capability-tag.attachment .tag-dot {
  background: #f59e0b;
}

/* 滚动条 */
.provider-list::-webkit-scrollbar,
.model-grid::-webkit-scrollbar {
  width: 6px;
}

.provider-list::-webkit-scrollbar-track,
.model-grid::-webkit-scrollbar-track {
  background: transparent;
}

.provider-list::-webkit-scrollbar-thumb,
.model-grid::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

.provider-list::-webkit-scrollbar-thumb:hover,
.model-grid::-webkit-scrollbar-thumb:hover {
  background: var(--app-text-tertiary);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk :deep(.el-dialog) {
  background: rgba(26, 26, 46, 0.98);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: 0 0 40px rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk :deep(.el-dialog__title) {
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .dialog-header {
  background: rgba(0, 255, 255, 0.03);
  border-bottom-color: rgba(0, 255, 255, 0.15);
}

html.cyberpunk .search-input :deep(.el-input__wrapper) {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk .current-model-display {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk .current-model-display .model-id-text {
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .provider-panel {
  border-right-color: rgba(0, 255, 255, 0.15);
  background: rgba(0, 255, 255, 0.02);
}

html.cyberpunk .panel-header {
  background: rgba(0, 255, 255, 0.05);
  border-bottom-color: rgba(0, 255, 255, 0.15);
}

html.cyberpunk .provider-item:hover {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk .provider-item.active {
  background: rgba(0, 255, 255, 0.12);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .model-card:hover {
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .model-card.active {
  border-color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .current-badge {
  background: #00ffff;
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .filter-btn.active {
  border-color: #00ffff;
  color: #00ffff;
  background: rgba(0, 255, 255, 0.15);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism :deep(.el-dialog) {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

html.glassmorphism :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

html.glassmorphism .dialog-header {
  background: rgba(37, 99, 235, 0.03);
  border-bottom-color: rgba(0, 0, 0, 0.05);
}

html.glassmorphism .search-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.8);
}

html.glassmorphism .current-model-display {
  background: rgba(37, 99, 235, 0.05);
  border-color: rgba(37, 99, 235, 0.15);
}

html.glassmorphism .current-model-display .model-id-text {
  color: #2563eb;
}

html.glassmorphism .provider-panel {
  background: rgba(37, 99, 235, 0.02);
}

html.glassmorphism .provider-item:hover {
  background: rgba(37, 99, 235, 0.05);
}

html.glassmorphism .provider-item.active {
  background: rgba(37, 99, 235, 0.1);
}

html.glassmorphism .model-card:hover {
  border-color: #2563eb;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.15);
}

html.glassmorphism .model-card.active {
  border-color: #2563eb;
  background: rgba(37, 99, 235, 0.08);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2);
}

html.glassmorphism .current-badge {
  background: #2563eb;
}

html.glassmorphism .filter-btn.active {
  border-color: #2563eb;
  color: #2563eb;
  background: rgba(37, 99, 235, 0.1);
}
</style>
