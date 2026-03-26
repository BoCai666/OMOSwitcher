<script setup lang="ts">
/**
 * 配置详情对话框组件
 * 用于显示 Agent 或 Category 的详细信息，包括系统提示词
 */
import { ref, computed, watch } from 'vue'
import type { AgentName, CategoryName, Model } from '@/types'
import { AGENT_DETAILS } from '@/data/agentDetails'
import { CATEGORY_DETAILS } from '@/data/categoryDetails'

const props = defineProps<{
  visible: boolean
  type: 'agent' | 'category'
  name: AgentName | CategoryName
  currentModel: string
  models: Model[]
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

// 语言切换：'zh' | 'en'
const promptLang = ref<'zh' | 'en'>('zh')

// 对话框可见性
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

// 获取详情数据
const detail = computed(() => {
  if (props.type === 'agent') {
    return AGENT_DETAILS[props.name as AgentName]
  } else {
    return CATEGORY_DETAILS[props.name as CategoryName]
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

// 当前显示的系统提示词
const currentPrompt = computed(() => {
  if (!detail.value) return ''
  return promptLang.value === 'zh' 
    ? detail.value.systemPrompt.zh 
    : detail.value.systemPrompt.en
})

// 关闭对话框时重置语言
watch(dialogVisible, (val) => {
  if (!val) {
    promptLang.value = 'zh'
  }
})
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :title="detail?.displayName || name"
    width="800px"
    top="5vh"
    class="config-detail-dialog"
    destroy-on-close
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
        <el-descriptions-item label="当前模型" :span="2">
          <div class="model-info">
            <el-tag type="info" size="small">{{ providerName }}</el-tag>
            <span class="model-name">{{ modelDisplayName }}</span>
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">
          <span class="description-text">{{ detail.description }}</span>
        </el-descriptions-item>
        <!-- Agent 特有信息 -->
        <template v-if="type === 'agent'">
          <el-descriptions-item label="计算成本">
            <el-tag 
              :type="(detail as any).cost === 'FREE' ? 'success' : (detail as any).cost === 'CHEAP' ? 'warning' : 'danger'" 
              size="small"
            >
              {{ (detail as any).cost }}
            </el-tag>
          </el-descriptions-item>
        </template>
        <!-- Category 特有信息 -->
        <template v-if="type === 'category'">
          <el-descriptions-item label="默认模型">
            <el-tag type="info" size="small">{{ (detail as any).model }}</el-tag>
          </el-descriptions-item>
        </template>
      </el-descriptions>

      <!-- 使用场景 -->
      <div v-if="type === 'agent' && (detail as any).useWhen" class="section">
        <h4 class="section-title">适用场景</h4>
        <ul class="use-when-list">
          <li v-for="(item, index) in (detail as any).useWhen" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>

      <!-- 触发条件 -->
      <div v-if="type === 'agent' && (detail as any).triggers?.length" class="section">
        <h4 class="section-title">触发条件</h4>
        <ul class="trigger-list">
          <li v-for="(item, index) in (detail as any).triggers" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>

      <!-- 避免场景 -->
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
          <pre>{{ currentPrompt }}</pre>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.config-detail-dialog :deep(.el-dialog__body) {
  max-height: 70vh;
  overflow-y: auto;
  padding: 20px;
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.basic-info {
  margin-bottom: 0;
}

.info-value {
  font-weight: 500;
  color: #303133;
}

.model-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-name {
  font-weight: 500;
  color: #409eff;
}

.description-text {
  color: #606266;
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
  color: #303133;
  margin: 0 0 10px 0;
  padding-left: 10px;
  border-left: 3px solid #409eff;
}

.section-header .section-title {
  margin: 0;
}

.use-when-list,
.trigger-list,
.avoid-list {
  margin: 0;
  padding-left: 20px;
  color: #606266;
  line-height: 1.8;
}

.use-when-list li,
.trigger-list li,
.avoid-list li {
  margin-bottom: 6px;
}

.trigger-list {
  background-color: #f0f9eb;
  padding: 12px 12px 12px 32px;
  border-radius: 4px;
}

.avoid-list {
  background-color: #fef0f0;
  padding: 12px 12px 12px 32px;
  border-radius: 4px;
}

.prompt-content {
  background-color: #f5f7fa;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
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
  line-height: 1.6;
  color: #303133;
}
</style>
