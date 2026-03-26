<script setup lang="ts">
/**
 * 模型配置页面
 * 合并了 Agent 和 Category 的配置管理
 */
import { ref, onMounted, computed, watch } from 'vue'
import AppLayout from '@/components/layout/AppLayout.vue'
import ConfigCard from '@/components/ConfigCard.vue'
import ConfigDetailDialog from '@/components/ConfigDetailDialog.vue'
import ModelSelectDrawer from '@/components/ModelSelectDrawer.vue'
import { listModels } from '@/services/modelStore'
import { useConfigStore } from '@/stores/config'
import type { Model, AgentName, CategoryName, OhMyOpenCodeConfig } from '@/types'
import { AGENT_NAMES, AGENT_INFO, CATEGORY_NAMES, CATEGORY_INFO, createDefaultConfig } from '@/types'

// 页面标题
const pageTitle = '模型配置'

// 使用共享的配置 store
const configStore = useConfigStore()

// 模型列表
const models = ref<Model[]>([])

// 保存消息
const saveMessage = ref('')

// 当前激活的 Tab
const activeTab = ref('agents')

// 详情对话框
const detailDialogVisible = ref(false)
const selectedAgentName = ref<AgentName | null>(null)
const selectedCategoryName = ref<CategoryName | null>(null)

// 模型选择抽屉
const modelDrawerVisible = ref(false)
const modelDrawerType = ref<'agent' | 'category'>('agent')
const modelDrawerAgentName = ref<AgentName | null>(null)
const modelDrawerCategoryName = ref<CategoryName | null>(null)

// 配置数据（使用 store 中的配置，如果未加载则使用默认值）
const config = computed<OhMyOpenCodeConfig>({
  get: () => configStore.config || createDefaultConfig(),
  set: (value) => { configStore.config = value }
})

// 初始化数据
onMounted(async () => {
  // 加载模型列表
  models.value = await listModels()

  // 如果配置未加载，从文件加载
  if (!configStore.isLoaded) {
    await configStore.loadConfig()
  }
})

// ========== Agent 相关方法 ==========

function updateAgentModel(agentName: AgentName, modelId: string) {
  configStore.updateAgentModel(agentName, modelId)
}

function handleViewAgentDetail(agentName: AgentName) {
  selectedAgentName.value = agentName
  selectedCategoryName.value = null
  detailDialogVisible.value = true
}

// ========== Category 相关方法 ==========

function updateCategoryModel(categoryName: CategoryName, modelId: string) {
  configStore.updateCategoryModel(categoryName, modelId)
}

function handleViewCategoryDetail(categoryName: CategoryName) {
  selectedCategoryName.value = categoryName
  selectedAgentName.value = null
  detailDialogVisible.value = true
}

// ========== 模型选择相关 ==========

function handleChangeModelFromDetail() {
  // 从详情对话框触发模型选择
  if (selectedAgentName.value) {
    modelDrawerType.value = 'agent'
    modelDrawerAgentName.value = selectedAgentName.value
    modelDrawerCategoryName.value = null
  } else if (selectedCategoryName.value) {
    modelDrawerType.value = 'category'
    modelDrawerCategoryName.value = selectedCategoryName.value
    modelDrawerAgentName.value = null
  }
  modelDrawerVisible.value = true
}

function handleSelectModel(modelId: string) {
  if (modelDrawerType.value === 'agent' && modelDrawerAgentName.value) {
    updateAgentModel(modelDrawerAgentName.value, modelId)
  } else if (modelDrawerType.value === 'category' && modelDrawerCategoryName.value) {
    updateCategoryModel(modelDrawerCategoryName.value, modelId)
  }
}

// ========== 保存 ==========

async function handleSave() {
  saveMessage.value = ''
  
  try {
    await configStore.saveConfig()
    saveMessage.value = '保存成功'
    setTimeout(() => {
      saveMessage.value = ''
    }, 3000)
  } catch (error) {
    saveMessage.value = '保存失败: ' + (error as Error).message
  }
}

// 模型选择抽屉标题
const modelDrawerTitle = computed(() => {
  if (modelDrawerType.value === 'agent' && modelDrawerAgentName.value) {
    return `选择模型 - ${AGENT_INFO[modelDrawerAgentName.value].displayName}`
  } else if (modelDrawerType.value === 'category' && modelDrawerCategoryName.value) {
    return `选择模型 - ${CATEGORY_INFO[modelDrawerCategoryName.value].displayName}`
  }
  return '选择模型'
})

// 模型选择抽屉当前模型
const modelDrawerCurrentModel = computed(() => {
  if (modelDrawerType.value === 'agent' && modelDrawerAgentName.value) {
    return config.value.agents[modelDrawerAgentName.value].model
  } else if (modelDrawerType.value === 'category' && modelDrawerCategoryName.value) {
    return config.value.categories[modelDrawerCategoryName.value].model
  }
  return ''
})

// 监听 store 中的错误
watch(() => configStore.error, (newError) => {
  if (newError) {
    saveMessage.value = '错误: ' + newError
  }
})
</script>

<template>
  <AppLayout :title="pageTitle">
    <div class="model-config">
      <!-- 页面头部 -->
      <div class="page-header">
        <div class="header-left">
          <el-tag v-if="configStore.hasUnsavedChanges" type="warning" size="small">
            未保存
          </el-tag>
        </div>
        <div class="header-right">
          <el-button
            type="primary"
            :loading="configStore.isSaving"
            @click="handleSave"
          >
            保存配置
          </el-button>
        </div>
      </div>

      <!-- 保存状态提示 -->
      <el-alert
        v-if="saveMessage"
        :title="saveMessage"
        :type="saveMessage.includes('失败') || saveMessage.includes('错误') ? 'error' : 'success'"
        show-icon
        class="save-alert"
        closable
        @close="saveMessage = ''"
      />

      <!-- Tab 切换 -->
      <el-tabs v-model="activeTab" class="config-tabs">
        <!-- Agent 配置 -->
        <el-tab-pane label="Agent 配置" name="agents">
          <div class="tab-header">
            <span class="subtitle">共 {{ AGENT_NAMES.length }} 个 Agent</span>
          </div>
          <div class="config-grid">
            <ConfigCard
              v-for="agentName in AGENT_NAMES"
              :key="agentName"
              :name="AGENT_INFO[agentName as AgentName].displayName"
              :description="AGENT_INFO[agentName as AgentName].description"
              v-model="config.agents[agentName as AgentName].model"
              :models="models"
              :clickable="true"
              @click="handleViewAgentDetail(agentName as AgentName)"
            />
          </div>
        </el-tab-pane>

        <!-- Category 配置 -->
        <el-tab-pane label="Category 配置" name="categories">
          <div class="tab-header">
            <span class="subtitle">共 {{ CATEGORY_NAMES.length }} 个 Category</span>
          </div>
          <div class="config-grid">
            <ConfigCard
              v-for="name in CATEGORY_NAMES"
              :key="name"
              :name="CATEGORY_INFO[name].displayName"
              :description="CATEGORY_INFO[name].description"
              v-model="config.categories[name].model"
              :models="models"
              :clickable="true"
              @click="handleViewCategoryDetail(name)"
            />
          </div>
        </el-tab-pane>
      </el-tabs>

      <!-- Agent 详情对话框 -->
      <ConfigDetailDialog
        v-if="selectedAgentName"
        v-model:visible="detailDialogVisible"
        type="agent"
        :name="selectedAgentName"
        :current-model="config.agents[selectedAgentName].model"
        :models="models"
        @change-model="handleChangeModelFromDetail"
      />

      <!-- Category 详情对话框 -->
      <ConfigDetailDialog
        v-if="selectedCategoryName"
        v-model:visible="detailDialogVisible"
        type="category"
        :name="selectedCategoryName"
        :current-model="config.categories[selectedCategoryName].model"
        :models="models"
        @change-model="handleChangeModelFromDetail"
      />

      <!-- 模型选择抽屉 -->
      <ModelSelectDrawer
        v-model:visible="modelDrawerVisible"
        :current-model="modelDrawerCurrentModel"
        :models="models"
        :title="modelDrawerTitle"
        @select="handleSelectModel"
      />
    </div>
  </AppLayout>
</template>

<style scoped>
.model-config {
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.save-alert {
  margin-bottom: 20px;
}

.config-tabs {
  margin-bottom: 20px;
}

.tab-header {
  margin-bottom: 16px;
}

.subtitle {
  color: #909399;
  font-size: 14px;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

/* 响应式布局：大屏幕显示 3 列 */
@media (min-width: 992px) {
  .config-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

/* 中等屏幕显示 2 列 */
@media (min-width: 768px) and (max-width: 991px) {
  .config-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* 小屏幕显示 1 列 */
@media (max-width: 767px) {
  .config-grid {
    grid-template-columns: 1fr;
  }
}
</style>
