<script setup lang="ts">
// 主页仪表盘组件 - 显示配置概览、快速操作和最近预设
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { listPresets, loadPreset } from '@/services/presetStore'
import { getWorkingPath, setWorkingPath } from '@/services/settingsStore'
import { AGENT_NAMES, CATEGORY_NAMES, type OhMyOpenCodeConfig } from '@/types'
import { showSuccess, showError } from '@/utils/errorHandler'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useOpenCode } from '@/composables/useOpenCode'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const configStore = useConfigStore()

// OpenCode 启动功能
const { launchOpenCode, isLaunching, error } = useOpenCode()

// 工作路径输入
const workingPath = ref('')

// 加载保存的路径
async function loadSavedPath() {
  const saved = await getWorkingPath()
  if (saved) {
    workingPath.value = saved
  }
}

// 保存路径
async function savePath() {
  if (workingPath.value) {
    await setWorkingPath(workingPath.value)
  } else {
    await setWorkingPath('')
  }
}

// 打开文件夹选择对话框
async function browseFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择工作目录'
    })
    if (selected) {
      workingPath.value = selected
    }
  } catch (e) {
    // 用户取消选择，不做处理
  }
}

// 启动 OpenCode（带路径）
async function handleLaunchOpenCode() {
  await savePath()
  launchOpenCode(workingPath.value)
}

// 监听错误并显示提示
watch(error, (newError) => {
  if (newError) {
    showError(newError)
  }
})

// 预设列表
const recentPresets = ref<PresetInfo[]>([])

// 当前激活的预设名称
const activePresetName = ref<string>('')

// 预设信息类型
interface PresetInfo {
  name: string
  description?: string
  updatedAt: string
}

// 获取配置数据
const configData = computed<OhMyOpenCodeConfig | null>(() => {
  return configStore.config
})

// 统计信息
const stats = computed(() => {
  const config = configData.value
  return {
    agentCount: config?.agents ? Object.keys(config.agents).length : 0,
    categoryCount: config?.categories ? Object.keys(config.categories).length : 0,
    presetCount: recentPresets.value.length,
    totalAgents: AGENT_NAMES.length,
    totalCategories: CATEGORY_NAMES.length
  }
})

// 加载最近预设列表（最多显示5个）
async function loadRecentPresets() {
  const presets = (await listPresets())
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    .slice(0, 5)
    .map(p => ({
      name: p.name,
      description: p.description,
      updatedAt: p.updatedAt
    }))
  recentPresets.value = presets
}

// 格式化日期
function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 应用预设
async function applyPreset(name: string) {
  const preset = await loadPreset(name)
  if (preset) {
    configStore.applyPreset(preset.config)
    activePresetName.value = name
    // 显示成功提示
    showSuccess(`已应用预设: ${name}`)
  }
}

// 跳转到 Agent 配置页面
function goToAgents() {
  router.push('/agents')
}

// 跳转到 Category 配置页面
function goToCategories() {
  router.push('/categories')
}

// 跳转到预设管理页面
function goToPresets() {
  router.push('/presets')
}

// 保存当前配置
async function saveConfig() {
  await configStore.saveConfig()
  showSuccess('配置已保存')
}

// 快速创建新预设
function createPreset() {
  router.push('/presets')
}

onMounted(() => {
  loadRecentPresets()
  // 尝试加载配置
  configStore.loadConfig()
  // 加载保存的工作路径
  loadSavedPath()
})
</script>

<template>
  <AppLayout title="仪表盘">
    <div class="home-dashboard">
      <!-- 欢迎区域 -->
      <el-row :gutter="20" class="welcome-section">
        <el-col :span="24">
          <div class="welcome-card">
            <div class="welcome-content">
              <h1 class="welcome-title">欢迎使用 OMOSwitcher</h1>
              <p class="welcome-subtitle">OhMyOpenCode 模型配置管理工具</p>
              <div class="welcome-status">
                <el-tag :type="!configStore.hasUnsavedChanges ? 'success' : 'warning'" effect="dark" size="large">
                  <el-icon v-if="!configStore.hasUnsavedChanges"><Check /></el-icon>
                  <el-icon v-else><Warning /></el-icon>
                  {{ !configStore.hasUnsavedChanges ? '配置已保存' : '配置未保存' }}
                </el-tag>
                <span v-if="activePresetName" class="active-preset">
                  当前预设: <strong>{{ activePresetName }}</strong>
                </span>
              </div>
            </div>
            <div class="welcome-icon">
              <el-icon :size="64" color="#409eff"><Setting /></el-icon>
            </div>
          </div>
        </el-col>
      </el-row>

      <!-- 统计概览卡片 -->
      <el-row :gutter="20" class="stats-section">
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card" shadow="hover">
            <div class="stat-content">
              <div class="stat-icon agent-icon">
                <el-icon :size="32"><User /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ stats.agentCount }}/{{ stats.totalAgents }}</div>
                <div class="stat-label">Agent 配置</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card" shadow="hover">
            <div class="stat-content">
              <div class="stat-icon category-icon">
                <el-icon :size="32"><Folder /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ stats.categoryCount }}/{{ stats.totalCategories }}</div>
                <div class="stat-label">Category 配置</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card" shadow="hover">
            <div class="stat-content">
              <div class="stat-icon preset-icon">
                <el-icon :size="32"><Collection /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ stats.presetCount }}</div>
                <div class="stat-label">已保存预设</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 快速操作区域 -->
      <el-row :gutter="20" class="actions-section">
        <el-col :span="24">
          <el-card shadow="never">
            <template #header>
              <div class="section-header">
                <span class="section-title">快速操作</span>
              </div>
            </template>
            <div class="quick-actions">
               <!-- 工作路径输入 -->
               <div class="path-input-wrapper">
                 <el-input
                   v-model="workingPath"
                   placeholder="输入工作目录路径（留空使用用户主目录）"
                   clearable
                   class="path-input"
                 >
                   <template #prepend>
                     <el-icon><Folder /></el-icon>
                     <span>工作路径</span>
                   </template>
                   <template #append>
                     <el-button @click="browseFolder">
                       <el-icon><FolderOpened /></el-icon>
                       浏览
                     </el-button>
                   </template>
                 </el-input>
               </div>
              <el-button 
                type="primary" 
                size="large" 
                :loading="isLaunching"
                @click="handleLaunchOpenCode"
              >
                <el-icon><VideoPlay /></el-icon>
                启动 OpenCode
              </el-button>
              <el-tag type="success" size="small" class="proxy-hint">
                自动设置监控代理 (localhost:8080)
              </el-tag>
              <el-button type="primary" size="large" @click="goToAgents">
                <el-icon><User /></el-icon>
                配置 Agents
              </el-button>
              <el-button type="success" size="large" @click="goToCategories">
                <el-icon><Folder /></el-icon>
                配置 Categories
              </el-button>
              <el-button type="info" size="large" @click="goToPresets">
                <el-icon><Collection /></el-icon>
                管理预设
              </el-button>
              <el-button 
                type="warning" 
                size="large" 
                :disabled="!configStore.hasUnsavedChanges"
                @click="saveConfig"
              >
                <el-icon><DocumentChecked /></el-icon>
                保存配置
              </el-button>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 最近预设区域 -->
      <el-row :gutter="20" class="presets-section">
        <el-col :span="24">
          <el-card shadow="never">
            <template #header>
              <div class="section-header">
                <span class="section-title">最近使用的预设</span>
                <el-button link type="primary" @click="goToPresets">
                  查看全部
                  <el-icon class="el-icon--right"><ArrowRight /></el-icon>
                </el-button>
              </div>
            </template>
            
            <div v-if="recentPresets.length > 0" class="preset-list">
              <div
                v-for="preset in recentPresets"
                :key="preset.name"
                class="preset-item"
                :class="{ active: preset.name === activePresetName }"
                @click="applyPreset(preset.name)"
              >
                <div class="preset-info">
                  <div class="preset-name">
                    <el-icon><Document /></el-icon>
                    {{ preset.name }}
                  </div>
                  <div v-if="preset.description" class="preset-desc">
                    {{ preset.description }}
                  </div>
                </div>
                <div class="preset-meta">
                  <span class="preset-date">{{ formatDate(preset.updatedAt) }}</span>
                  <el-button 
                    type="primary" 
                    link 
                    size="small"
                    @click.stop="applyPreset(preset.name)"
                  >
                    应用
                  </el-button>
                </div>
              </div>
            </div>
            
            <el-empty v-else description="暂无预设">
              <el-button type="primary" @click="createPreset">
                创建第一个预设
              </el-button>
            </el-empty>
          </el-card>
        </el-col>
      </el-row>
    </div>
  </AppLayout>
</template>

<style scoped>
/* 仪表盘容器 */
.home-dashboard {
  padding: 0;
}

/* 区块间距 */
.welcome-section,
.stats-section,
.actions-section,
.presets-section {
  margin-bottom: 20px;
}

/* 欢迎区域 */
.welcome-card {
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  border-radius: 12px;
  padding: 32px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: white;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
}

.welcome-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: white;
}

.welcome-subtitle {
  font-size: 16px;
  margin: 0 0 20px 0;
  opacity: 0.9;
  color: rgba(255, 255, 255, 0.9);
}

.welcome-status {
  display: flex;
  align-items: center;
  gap: 16px;
}

.active-preset {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

.active-preset strong {
  color: white;
  font-weight: 600;
}

.welcome-icon {
  opacity: 0.3;
}

/* 统计卡片 */
.stat-card {
  transition: transform 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.agent-icon {
  background: linear-gradient(135deg, #409eff 0%, #79bbff 100%);
}

.category-icon {
  background: linear-gradient(135deg, #67c23a 0%, #95d475 100%);
}

.preset-icon {
  background: linear-gradient(135deg, #e6a23c 0%, #f3d19e 100%);
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  line-height: 1.2;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

/* 区块标题 */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

/* 快速操作 */
.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.path-input-wrapper {
  width: 100%;
  margin-bottom: 8px;
}

.path-input {
  width: 100%;
}

.path-input :deep(.el-input-group__prepend) {
  display: flex;
  align-items: center;
  gap: 8px;
}

.quick-actions .el-button {
  display: flex;
  align-items: center;
  gap: 8px;
}

.proxy-hint {
  margin-left: 8px;
  vertical-align: middle;
}

/* 预设列表 */
.preset-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: #f5f7fa;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;
}

.preset-item:hover {
  background-color: #ecf5ff;
  border-color: #409eff;
}

.preset-item.active {
  background-color: #ecf5ff;
  border-color: #409eff;
}

.preset-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preset-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #303133;
}

.preset-desc {
  font-size: 12px;
  color: #909399;
}

.preset-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preset-date {
  font-size: 12px;
  color: #c0c4cc;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .welcome-card {
    flex-direction: column;
    text-align: center;
    gap: 20px;
  }

  .welcome-icon {
    display: none;
  }

  .stat-card {
    margin-bottom: 12px;
  }

  .quick-actions {
    justify-content: center;
  }

  .preset-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .preset-meta {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
