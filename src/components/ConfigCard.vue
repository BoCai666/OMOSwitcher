<script setup lang="ts">
// 配置卡片组件 - 用于显示 Agent 或 Category 的配置信息
import { computed } from 'vue'
import { ArrowRight } from '@element-plus/icons-vue'
import type { Model } from '@/types'

const props = defineProps<{
  name: string
  modelValue: string
  models: Model[]
  editable?: boolean
  description?: string
  clickable?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'click': []
}>()

// 使用计算属性实现 v-model 双向绑定
const currentModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 计算当前模型的显示名称
const displayModel = computed(() => {
  return props.models.find(m => m.id === props.modelValue)?.name || props.modelValue
})

// 按供应商分组的模型
const groupedModels = computed(() => {
  const groups = new Map<string, Model[]>()
  
  for (const model of props.models) {
    const provider = model.provider
    if (!groups.has(provider)) {
      groups.set(provider, [])
    }
    groups.get(provider)!.push(model)
  }
  
  // 按供应商名称排序
  return new Map(Array.from(groups.entries()).sort((a, b) => a[0].localeCompare(b[0])))
})
</script>

<template>
  <el-card 
    class="config-card" 
    shadow="hover"
    :class="{ 'clickable': clickable }"
    @click="clickable && emit('click')"
  >
    <template #header>
      <div class="card-header">
        <span class="name">{{ name }}</span>
        <el-icon v-if="clickable" class="click-hint"><ArrowRight /></el-icon>
      </div>
    </template>
    
    <div class="card-content">
      <!-- 描述信息 -->
      <div v-if="description" class="description">
        {{ description }}
      </div>
      
      <div class="model-display">
        <span class="label">当前模型:</span>
        <span class="value">{{ displayModel }}</span>
      </div>
      
      <el-select
        v-if="editable !== false"
        v-model="currentModel"
        placeholder="选择模型"
        class="model-select"
        filterable
      >
        <el-option-group
          v-for="[provider, providerModels] in groupedModels"
          :key="provider"
          :label="provider"
        >
          <el-option
            v-for="model in providerModels"
            :key="model.id"
            :label="model.name"
            :value="model.id"
          >
            <div class="model-option">
              <span class="model-name">{{ model.name }}</span>
              <span class="model-id-small">{{ model.id }}</span>
            </div>
          </el-option>
        </el-option-group>
      </el-select>
    </div>
  </el-card>
</template>

<style scoped>
.config-card {
  margin-bottom: 16px;
}

.config-card.clickable {
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.config-card.clickable:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.name {
  font-weight: 600;
  color: #303133;
}

.click-hint {
  color: #909399;
  font-size: 14px;
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.description {
  font-size: 13px;
  color: #606266;
  line-height: 1.5;
  padding: 8px 12px;
  background-color: #f5f7fa;
  border-radius: 4px;
  border-left: 3px solid #409eff;
}

.model-display {
  display: flex;
  gap: 8px;
}

.label {
  color: #909399;
}

.value {
  color: #409eff;
  font-weight: 500;
}

.model-select {
  width: 100%;
}

.model-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.model-name {
  font-weight: 500;
}

.model-id-small {
  font-size: 12px;
  color: #909399;
  font-family: 'Courier New', monospace;
}
</style>
