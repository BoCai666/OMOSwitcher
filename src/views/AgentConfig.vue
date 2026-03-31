<script setup lang="ts">
/**
 * Agent 配置页面
 * 为每个 Agent 配置模型
 */
import { ref, onMounted, computed } from 'vue'
import { useConfigStore } from '@/stores/config'
import { listModels } from '@/services/modelStore'
import { showSuccess, showError } from '@/utils/errorHandler'
import type { AgentName, Model } from '@/types'
import { AGENT_NAMES, AGENT_INFO } from '@/types/config'
import { AGENT_DETAILS } from '@/data/agentDetails'

const configStore = useConfigStore()

// 加载状态
const loading = ref(true)

// 模型列表
const models = ref<Model[]>([])

// 搜索关键词
const searchKeyword = ref('')

// 过滤后的 Agent 列表
const filteredAgents = computed(() => {
  if (!searchKeyword.value) {
    return AGENT_NAMES
  }
  const keyword = searchKeyword.value.toLowerCase()
  return AGENT_NAMES.filter(name => {
    const info = AGENT_INFO[name]
    const details = AGENT_DETAILS[name]
    return (
      name.toLowerCase().includes(keyword) ||
      info.displayName.toLowerCase().includes(keyword) ||
      info.description.toLowerCase().includes(keyword) ||
      (details?.triggers?.some(t => t.toLowerCase().includes(keyword))) ||
      (details?.useWhen?.some(u => u.toLowerCase().includes(keyword))) ||
      (details?.avoidWhen?.some(a => a.toLowerCase().includes(keyword)))
    )
  })
})

// 获取 Agent 当前配置的模型
function getAgentModel(name: AgentName): string {
  return configStore.config?.agents?.[name]?.model || ''
}

// 获取 Agent 详情
function getAgentDetail(name: AgentName) {
  return AGENT_DETAILS[name]
}

// 更新 Agent 模型
function updateAgentModel(name: AgentName, model: string) {
  configStore.updateAgentModel(name, model)
}

// 保存配置
async function saveConfig() {
  try {
    await configStore.saveConfig()
    showSuccess('配置已保存')
  } catch (error) {
    showError('保存配置失败')
  }
}

// 加载数据
onMounted(async () => {
  loading.value = true
  try {
    // 加载配置
    if (!configStore.isLoaded) {
      await configStore.loadConfig()
    }
    // 加载模型列表
    models.value = await listModels()
  } catch (error) {
    showError('加载数据失败')
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="agent-config">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">Agent 配置</h1>
      <p class="page-desc">为每个 Agent 配置使用的模型</p>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索 Agent..."
        clearable
        class="search-input"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      
      <el-button 
        type="primary" 
        :disabled="!configStore.hasUnsavedChanges"
        @click="saveConfig"
      >
        <el-icon><DocumentChecked /></el-icon>
        保存配置
      </el-button>
    </div>

    <!-- 加载状态 -->
    <el-skeleton v-if="loading" :rows="10" animated />

    <!-- Agent 列表 -->
    <div v-else class="agent-list">
      <el-card 
        v-for="name in filteredAgents" 
        :key="name" 
        class="agent-card"
        shadow="hover"
      >
        <div class="agent-header">
          <div class="agent-info">
            <h3 class="agent-name">{{ AGENT_INFO[name].displayName }}</h3>
            <el-tag size="small" type="info">{{ name }}</el-tag>
          </div>
          <div class="agent-model-select">
            <el-select
              :model-value="getAgentModel(name)"
              placeholder="选择模型"
              filterable
              @change="(model: string) => updateAgentModel(name, model)"
            >
              <el-option
                v-for="model in models"
                :key="model.id"
                :label="model.name"
                :value="model.id"
              />
            </el-select>
          </div>
        </div>
        
        <p class="agent-desc">{{ AGENT_INFO[name].description }}</p>
        
        <!-- Agent 详细信息 -->
        <el-collapse class="agent-details">
          <el-collapse-item title="详细信息">
            <div v-if="getAgentDetail(name)" class="detail-content">
              <div class="detail-section">
                <h4>触发条件</h4>
                <ul>
                  <li v-for="(trigger, idx) in getAgentDetail(name).triggers" :key="idx">
                    {{ trigger }}
                  </li>
                </ul>
              </div>
              <div class="detail-section">
                <h4>适用场景</h4>
                <ul>
                  <li v-for="(item, idx) in getAgentDetail(name).useWhen" :key="idx">
                    {{ item }}
                  </li>
                </ul>
              </div>
              <div class="detail-section">
                <h4>避免场景</h4>
                <ul>
                  <li v-for="(item, idx) in getAgentDetail(name).avoidWhen" :key="idx">
                    {{ item }}
                  </li>
                </ul>
              </div>
            </div>
          </el-collapse-item>
        </el-collapse>
      </el-card>
    </div>

    <!-- 空状态 -->
    <el-empty v-if="!loading && filteredAgents.length === 0" description="没有找到匹配的 Agent" />
  </div>
</template>

<style scoped>
.agent-config {
  padding: var(--app-spacing-6);
}

.page-header {
  margin-bottom: var(--app-spacing-6);
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0 0 var(--app-spacing-2) 0;
  color: var(--app-text-primary);
}

.page-desc {
  margin: 0;
  color: var(--app-text-tertiary);
}

.toolbar {
  display: flex;
  gap: var(--app-spacing-3);
  margin-bottom: var(--app-spacing-6);
}

.search-input {
  max-width: 300px;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-4);
}

.agent-card {
  transition: all 0.3s ease;
}

.agent-card:hover {
  transform: translateY(-2px);
}

.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-3);
}

.agent-info {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.agent-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.agent-model-select {
  min-width: 200px;
}

.agent-desc {
  margin: 0 0 var(--app-spacing-3) 0;
  color: var(--app-text-secondary);
  line-height: 1.6;
}

.agent-details {
  border: none;
}

.detail-content {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: var(--app-spacing-4);
}

.detail-section h4 {
  margin: 0 0 var(--app-spacing-2) 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.detail-section ul {
  margin: 0;
  padding-left: var(--app-spacing-4);
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.8;
}
</style>
