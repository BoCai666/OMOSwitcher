<script setup lang="ts">
/**
 * 模型配置页面
 * 合并了 Agent 和 Category 的配置管理
 */
import { ref, onMounted, computed, watch } from 'vue'
import ConfigCard from '@/components/ConfigCard.vue'
import ConfigDetailDialog from '@/components/ConfigDetailDialog.vue'
import ModelSelectDrawer from '@/components/ModelSelectDrawer.vue'
import { listModels } from '@/services/modelStore'
import { useConfigStore } from '@/stores/config'
import type { Model, AgentName, CategoryName, OhMyOpenCodeConfig } from '@/types'
import { AGENT_NAMES, AGENT_INFO, CATEGORY_NAMES, CATEGORY_INFO, createDefaultConfig } from '@/types'

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

// 直接打开 Agent 的模型选择抽屉
function handleChangeAgentModel(agentName: AgentName) {
  modelDrawerType.value = 'agent'
  modelDrawerAgentName.value = agentName
  modelDrawerCategoryName.value = null
  modelDrawerVisible.value = true
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

// 直接打开 Category 的模型选择抽屉
function handleChangeCategoryModel(categoryName: CategoryName) {
  modelDrawerType.value = 'category'
  modelDrawerCategoryName.value = categoryName
  modelDrawerAgentName.value = null
  modelDrawerVisible.value = true
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

      <!-- 热重载状态提示 -->
      <el-alert
        v-if="configStore.hotReloadStatus"
        :title="configStore.hotReloadStatus.message"
        :type="configStore.hotReloadStatus.success ? 'success' : configStore.hotReloadStatus.skipped ? 'info' : 'warning'"
        show-icon
        class="save-alert"
        closable
        @close="configStore.hotReloadStatus = null"
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
              @click-model="handleChangeAgentModel(agentName as AgentName)"
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
              @click-model="handleChangeCategoryModel(name)"
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
</template>

<style scoped>
/* ==================== 基础布局 ==================== */
.model-config {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--app-spacing-4);
}

/* ==================== 页面头部 ==================== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-5);
  padding: var(--app-spacing-4) 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

/* ==================== 赛博朋克主题 - 未保存标签 ==================== */
html.cyberpunk .page-header :deep(.el-tag--warning) {
  background: rgba(255, 170, 0, 0.15);
  border: 1px solid rgba(255, 170, 0, 0.4);
  color: var(--app-color-warning);
  backdrop-filter: blur(4px);
  box-shadow: 0 0 10px rgba(255, 170, 0, 0.2);
  transition: all 0.3s ease;
}

html.cyberpunk .page-header :deep(.el-tag--warning:hover) {
  box-shadow: 0 0 16px rgba(255, 170, 0, 0.4);
  border-color: rgba(255, 170, 0, 0.6);
}

/* ==================== 玻璃拟态主题 - 未保存标签 ==================== */
html.glassmorphism .page-header :deep(.el-tag--warning) {
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: var(--app-color-warning);
}

/* ==================== 赛博朋克主题 - 保存按钮 ==================== */
html.cyberpunk .page-header :deep(.el-button--primary) {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 0%, rgba(255, 0, 255, 0.15) 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  color: var(--app-color-primary);
  font-weight: 600;
  padding: 10px 24px;
  border-radius: var(--app-radius-md);
  box-shadow:
    0 0 15px rgba(0, 255, 255, 0.3),
    inset 0 0 10px rgba(0, 255, 255, 0.05);
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

html.cyberpunk .page-header :deep(.el-button--primary::before) {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
  transition: left 0.5s;
}

html.cyberpunk .page-header :deep(.el-button--primary:hover) {
  box-shadow:
    0 0 25px rgba(0, 255, 255, 0.5),
    0 0 50px rgba(0, 255, 255, 0.2),
    inset 0 0 15px rgba(0, 255, 255, 0.1);
  transform: translateY(-1px);
  border-color: rgba(0, 255, 255, 0.6);
}

html.cyberpunk .page-header :deep(.el-button--primary:hover::before) {
  left: 100%;
}

html.cyberpunk .page-header :deep(.el-button--primary:active) {
  transform: translateY(0);
  box-shadow:
    0 0 10px rgba(0, 255, 255, 0.3),
    inset 0 0 10px rgba(0, 255, 255, 0.1);
}

/* ==================== 玻璃拟态主题 - 保存按钮 ==================== */
html.glassmorphism .page-header :deep(.el-button--primary) {
  background: linear-gradient(135deg, var(--app-color-primary) 0%, var(--app-color-secondary) 100%);
  border: none;
  color: white;
  font-weight: 600;
  padding: 10px 24px;
  border-radius: var(--app-radius-md);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.25);
  transition: all 0.3s ease;
}

html.glassmorphism .page-header :deep(.el-button--primary:hover) {
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.35);
  transform: translateY(-1px);
}

/* ==================== Alert 提示 ==================== */
.save-alert {
  margin-bottom: var(--app-spacing-5);
  border-radius: var(--app-radius-lg);
  backdrop-filter: blur(12px);
  transition: all 0.3s ease;
}

/* 赛博朋克主题 - Alert */
html.cyberpunk .save-alert {
  background: rgba(0, 255, 136, 0.1) !important;
  border: 1px solid rgba(0, 255, 136, 0.3);
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 0 20px rgba(0, 255, 136, 0.1);
}

html.cyberpunk .save-alert:deep(.el-alert__title) {
  color: var(--app-color-success);
  font-weight: 500;
}

html.cyberpunk .save-alert:deep(.el-alert__icon) {
  color: var(--app-color-success);
}

html.cyberpunk .save-alert:deep(.el-alert--error) {
  background: rgba(255, 51, 102, 0.1) !important;
  border-color: rgba(255, 51, 102, 0.3);
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.3),
    0 0 20px rgba(255, 51, 102, 0.1);
}

html.cyberpunk .save-alert:deep(.el-alert--error .el-alert__title),
html.cyberpunk .save-alert:deep(.el-alert--error .el-alert__icon) {
  color: var(--app-color-danger);
}

/* 玻璃拟态主题 - Alert */
html.glassmorphism .save-alert {
  background: rgba(16, 185, 129, 0.08) !important;
  border: 1px solid rgba(16, 185, 129, 0.25);
}

html.glassmorphism .save-alert:deep(.el-alert__title) {
  color: var(--app-color-success);
}

html.glassmorphism .save-alert:deep(.el-alert--error) {
  background: rgba(239, 68, 68, 0.08) !important;
  border-color: rgba(239, 68, 68, 0.25);
}

/* ==================== Tab 切换 ==================== */
.config-tabs {
  margin-bottom: var(--app-spacing-5);
}

.config-tabs :deep(.el-tabs__header) {
  margin-bottom: var(--app-spacing-5);
  border-bottom: 1px solid var(--app-border-default);
}

.config-tabs :deep(.el-tabs__nav-wrap::after) {
  background: transparent;
}

/* Tab 项基础样式 */
.config-tabs :deep(.el-tabs__item) {
  color: var(--app-text-tertiary);
  font-size: 15px;
  font-weight: 500;
  padding: 0 var(--app-spacing-6);
  height: 48px;
  line-height: 48px;
  transition: all 0.3s ease;
  position: relative;
}

/* 赛博朋克主题 - Tab */
html.cyberpunk .config-tabs :deep(.el-tabs__item:hover) {
  color: var(--app-text-secondary);
  background: linear-gradient(180deg, transparent 0%, rgba(0, 255, 255, 0.05) 100%);
}

html.cyberpunk .config-tabs :deep(.el-tabs__item.is-active) {
  color: var(--app-color-primary);
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .config-tabs :deep(.el-tabs__active-bar) {
  background: linear-gradient(90deg, var(--app-color-primary) 0%, var(--app-color-secondary) 100%);
  height: 3px;
  border-radius: 2px;
  box-shadow:
    0 0 10px var(--app-color-primary),
    0 0 20px rgba(0, 255, 255, 0.5);
}

/* 玻璃拟态主题 - Tab */
html.glassmorphism .config-tabs :deep(.el-tabs__item:hover) {
  color: var(--app-text-secondary);
}

html.glassmorphism .config-tabs :deep(.el-tabs__item.is-active) {
  color: var(--app-color-primary);
}

html.glassmorphism .config-tabs :deep(.el-tabs__active-bar) {
  background: var(--app-color-primary);
  height: 3px;
  border-radius: 2px;
}

/* ==================== Tab 内容区域 ==================== */
.tab-header {
  margin-bottom: var(--app-spacing-5);
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background: var(--app-bg-card);
  border-radius: var(--app-radius-md);
  border: 1px solid var(--app-border-default);
  transition: all 0.3s ease;
}

/* 赛博朋克主题 - Tab 头部 */
html.cyberpunk .tab-header {
  background: rgba(26, 26, 46, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.1);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.05);
}

/* 玻璃拟态主题 - Tab 头部 */
html.glassmorphism .tab-header {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(8px);
}

.subtitle {
  color: var(--app-text-tertiary);
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.3px;
}

/* ==================== 配置网格 ==================== */
.config-grid {
  display: grid;
  gap: var(--app-spacing-5);
  padding: var(--app-spacing-2);
}

/* 大屏幕显示 3 列 */
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
    gap: var(--app-spacing-4);
  }
}

/* 超大屏幕优化 */
@media (min-width: 1400px) {
  .config-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: var(--app-spacing-6);
  }
}
</style>
