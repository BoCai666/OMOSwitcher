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
.drawer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.current-model {
  padding: 12px 16px;
  background: #f5f7fa;
  border-radius: 8px;
  margin-bottom: 16px;
}

.current-model .label {
  color: #909399;
  font-size: 13px;
}

.current-model .value {
  color: #409eff;
  font-weight: 500;
  margin-left: 8px;
}

.search-input {
  margin-bottom: 16px;
}

.model-list {
  flex: 1;
  overflow-y: auto;
}

.provider-group {
  margin-bottom: 16px;
}

.provider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 6px;
  margin-bottom: 8px;
}

.provider-name {
  font-weight: 600;
  color: #303133;
  font-size: 14px;
}

.provider-count {
  font-size: 12px;
  color: #909399;
  background: #e4e7ed;
  padding: 2px 8px;
  border-radius: 10px;
}

.model-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.model-item:hover {
  background: #ecf5ff;
  border-color: #409eff;
}

.model-item.is-current {
  background: #ecf5ff;
  border-color: #409eff;
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.model-name {
  font-weight: 500;
  color: #303133;
  font-size: 14px;
}

.model-id {
  font-size: 12px;
  color: #909399;
  font-family: 'Courier New', monospace;
}

.check-icon {
  color: #409eff;
  font-size: 18px;
}

/* 滚动条样式 */
.model-list::-webkit-scrollbar {
  width: 6px;
}

.model-list::-webkit-scrollbar-track {
  background: #f5f7fa;
  border-radius: 3px;
}

.model-list::-webkit-scrollbar-thumb {
  background: #c0c4cc;
  border-radius: 3px;
}

.model-list::-webkit-scrollbar-thumb:hover {
  background: #909399;
}
</style>
