<script setup lang="ts">
/**
 * 预设详情对话框组件
 * 用于展示预设的完整配置信息
 */
import { computed } from 'vue'
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

// 关闭对话框
const handleClose = () => {
  dialogVisible.value = false
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="预设详情"
    width="700px"
    :close-on-click-modal="true"
    @close="handleClose"
  >
    <div v-if="preset" class="preset-detail">
      <!-- 基本信息 -->
      <div class="info-section">
        <h4 class="section-title">基本信息</h4>
        <el-descriptions :column="2" border>
          <el-descriptions-item label="预设名称">
            {{ preset.name }}
          </el-descriptions-item>
          <el-descriptions-item label="描述">
            {{ preset.description || '无描述' }}
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatDate(preset.createdAt) }}
          </el-descriptions-item>
          <el-descriptions-item label="更新时间">
            {{ formatDate(preset.updatedAt) }}
          </el-descriptions-item>
        </el-descriptions>
      </div>

      <!-- Agent 配置 -->
      <div class="config-section">
        <h4 class="section-title">Agent 模型配置</h4>
        <el-table
          :data="agentConfigs"
          size="small"
          border
          max-height="200"
        >
          <el-table-column prop="name" label="Agent 名称" width="180" />
          <el-table-column prop="model" label="模型" />
        </el-table>
      </div>

      <!-- Category 配置 -->
      <div class="config-section">
        <h4 class="section-title">Category 模型配置</h4>
        <el-table
          :data="categoryConfigs"
          size="small"
          border
          max-height="200"
        >
          <el-table-column prop="name" label="Category 名称" width="180" />
          <el-table-column prop="model" label="模型" />
        </el-table>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.preset-detail {
  padding: 0 8px;
}

.info-section {
  margin-bottom: 24px;
}

.config-section {
  margin-bottom: 20px;
}

.config-section:last-child {
  margin-bottom: 0;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}
</style>
