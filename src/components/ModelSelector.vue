<script setup lang="ts">
import { computed } from 'vue'
import type { Model } from '@/types'

const props = withDefaults(defineProps<{
  modelValue: string
  models: Model[]
  placeholder?: string
  disabled?: boolean
}>(), {
  placeholder: '请选择模型',
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// v-model 绑定
const currentModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})
</script>

<template>
  <el-select
    v-model="currentModel"
    :placeholder="placeholder"
    :disabled="disabled"
    filterable
    clearable
    class="model-selector"
  >
    <el-option
      v-for="model in models"
      :key="model.id"
      :label="model.name"
      :value="model.id"
    >
      <div class="model-option">
        <span class="model-name">{{ model.name }}</span>
        <span class="model-id">{{ model.id }}</span>
      </div>
    </el-option>
  </el-select>
</template>

<style scoped>
.model-selector {
  width: 100%;
}

.model-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.model-name {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model-id {
  font-size: 12px;
  color: #909399;
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
