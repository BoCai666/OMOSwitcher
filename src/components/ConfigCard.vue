<script setup lang="ts">
/**
 * 配置卡片组件
 * 用于显示 Agent 或 Category 的配置信息
 * 点击卡片可查看详情
 */
import { computed } from 'vue'
import { ArrowRight } from '@element-plus/icons-vue'
import type { Model } from '@/types'

const props = defineProps<{
  name: string
  modelValue: string
  models: Model[]
  description?: string
  clickable?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'click': []
}>()

// 当前模型信息
const currentModelInfo = computed(() => {
  return props.models.find(m => m.id === props.modelValue)
})

// 供应商名称
const providerName = computed(() => {
  return currentModelInfo.value?.provider || props.modelValue.split('/')[0]
})

// 模型显示名称
const displayModel = computed(() => {
  return currentModelInfo.value?.name || props.modelValue.split('/').pop() || props.modelValue
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
      
      <!-- 当前模型显示 -->
      <div class="model-display">
        <div class="model-info">
          <span class="label">当前模型</span>
          <div class="model-details">
            <el-tag type="info" size="small" class="provider-tag">{{ providerName }}</el-tag>
            <span class="model-name">{{ displayModel }}</span>
          </div>
        </div>
      </div>
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
  padding: 12px 16px;
  background: linear-gradient(135deg, #f5f7fa 0%, #ecf5ff 100%);
  border-radius: 8px;
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.label {
  font-size: 12px;
  color: #909399;
}

.model-details {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.provider-tag {
  font-size: 11px;
}

.model-name {
  font-weight: 600;
  color: #409eff;
  font-size: 14px;
}
</style>
