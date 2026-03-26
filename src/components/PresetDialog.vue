<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Preset } from '@/types'

const props = defineProps<{
  visible: boolean
  presets: Preset[]
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
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="预设管理"
    width="500px"
  >
    <!-- 保存新预设 -->
    <div class="save-section">
      <h4>保存当前配置</h4>
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
      <h4>已保存的预设</h4>
      <el-empty v-if="presets.length === 0" description="暂无预设" />
      <div v-else class="preset-items">
        <div v-for="preset in presets" :key="preset.name" class="preset-item">
          <div class="preset-info">
            <span class="preset-name">{{ preset.name }}</span>
            <span class="preset-time">{{ new Date(preset.updatedAt).toLocaleDateString() }}</span>
          </div>
          <div class="preset-actions">
            <el-button size="small" type="primary" @click="handleLoad(preset)">
              加载
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(preset.name)">
              删除
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
.save-section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #ebeef5;
}

.save-section h4,
.preset-list h4 {
  margin: 0 0 12px;
  color: #303133;
}

.preset-input {
  margin-bottom: 12px;
}

.preset-items {
  max-height: 300px;
  overflow-y: auto;
}

.preset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  margin-bottom: 8px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.preset-info {
  display: flex;
  flex-direction: column;
}

.preset-name {
  font-weight: 500;
  color: #303133;
}

.preset-time {
  font-size: 12px;
  color: #909399;
}

.preset-actions {
  display: flex;
  gap: 8px;
}
</style>
