<script setup lang="ts">
// 主页仪表盘组件 - 显示配置概览、快速操作和最近预设
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { listPresets, loadPreset } from '@/services/presetStore'
import { getWorkingPath, setWorkingPath, getProxyConfig, setProxyConfig, getDefaultCaCertPath, checkCaCertExists, getMonitorPorts } from '@/services/settingsStore'
import { AGENT_NAMES, CATEGORY_NAMES, type OhMyOpenCodeConfig } from '@/types'
import { showSuccess, showError } from '@/utils/errorHandler'
import { useOpenCode } from '@/composables/useOpenCode'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const configStore = useConfigStore()

// OpenCode 启动功能
const { launchOpenCode, isLaunching, error } = useOpenCode()

// 工作路径输入
const workingPath = ref('')

// 代理配置
const proxyEnabled = ref(false)

// 证书是否存在
const certExists = ref<boolean | null>(null) // null 表示未检查

// 默认证书路径
const defaultCertPath = ref('')

// 代理端口
const proxyPort = ref(7101) // 默认端口

// 证书检查定时器
let certCheckTimer: ReturnType<typeof setInterval> | null = null

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

// 检查证书状态
async function checkCertStatus() {
  const exists = await checkCaCertExists()
  certExists.value = exists
  
  // 如果证书已存在，停止轮询
  if (exists && certCheckTimer) {
    clearInterval(certCheckTimer)
    certCheckTimer = null
  }
}

// 启动证书状态轮询
function startCertPolling() {
  // 如果已经在轮询，不重复启动
  if (certCheckTimer) return
  
  // 每 2 秒检查一次证书状态
  certCheckTimer = setInterval(async () => {
    await checkCertStatus()
  }, 2000)
}

// 停止证书状态轮询
function stopCertPolling() {
  if (certCheckTimer) {
    clearInterval(certCheckTimer)
    certCheckTimer = null
  }
}

// 加载代理配置
async function loadProxyConfig() {
  const config = await getProxyConfig()
  proxyEnabled.value = config.enabled
  // 获取默认证书路径
  defaultCertPath.value = await getDefaultCaCertPath()
  // 检查证书是否存在
  await checkCertStatus()
  // 获取代理端口配置
  const ports = await getMonitorPorts()
  proxyPort.value = ports.proxy
  
  // 如果启用了代理但证书不存在，启动轮询等待证书生成
  if (proxyEnabled.value && certExists.value === false) {
    startCertPolling()
  }
}

// 保存代理配置
async function saveProxyConfig() {
  await setProxyConfig({
    enabled: proxyEnabled.value,
    // 使用默认证书路径
    caCertPath: defaultCertPath.value || undefined
  })
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

// 启动 OpenCode（带路径和代理配置）
async function handleLaunchOpenCode() {
  await savePath()
  await saveProxyConfig()
  launchOpenCode(workingPath.value, proxyEnabled.value, defaultCertPath.value)
  
  // 如果启用了代理，启动证书状态轮询
  if (proxyEnabled.value) {
    // 延迟 1 秒后开始检查，给证书生成留出时间
    setTimeout(() => {
      checkCertStatus()
      if (certExists.value === false) {
        startCertPolling()
      }
    }, 1000)
  }
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

// 跳转到模型配置页面
function goToConfig() {
  router.push('/config')
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

// 监听代理开关变化
watch(proxyEnabled, (enabled) => {
  // 启用代理时，检查证书状态
  if (enabled && certExists.value === false) {
    startCertPolling()
  } else if (!enabled) {
    // 关闭代理时，停止轮询
    stopCertPolling()
  }
})

onMounted(() => {
  loadRecentPresets()
  // 尝试加载配置
  configStore.loadConfig()
  // 加载保存的工作路径
  loadSavedPath()
  // 加载代理配置
  loadProxyConfig()
})

onUnmounted(() => {
  // 清理证书检查定时器
  stopCertPolling()
})
</script>

<template>
  <div class="home-dashboard">
      <!-- 欢迎区域 -->
      <el-row :gutter="20" class="welcome-section">
        <el-col :span="24">
          <div class="welcome-card glass-card neon-border">
            <div class="welcome-content">
              <h1 class="welcome-title">欢迎使用 OMOSwitcher</h1>
              <p class="welcome-subtitle">OhMyOpenCode 模型配置管理工具</p>
              <div class="welcome-status">
                <el-tag :type="!configStore.hasUnsavedChanges ? 'success' : 'warning'" effect="dark" size="large" class="status-tag">
                  <el-icon v-if="!configStore.hasUnsavedChanges"><Check /></el-icon>
                  <el-icon v-else><Warning /></el-icon>
                  {{ !configStore.hasUnsavedChanges ? '配置已保存' : '配置未保存' }}
                </el-tag>
                <span v-if="activePresetName" class="active-preset">
                  <span class="preset-label">当前预设:</span>
                  <strong class="preset-name neon-text">{{ activePresetName }}</strong>
                </span>
              </div>
            </div>
            <div class="welcome-icon">
              <el-icon :size="64" class="setting-icon"><Setting /></el-icon>
            </div>
          </div>
        </el-col>
      </el-row>

      <!-- 统计概览卡片 -->
      <el-row :gutter="20" class="stats-section">
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card hover-lift" shadow="never">
            <div class="stat-content">
              <div class="stat-icon agent-icon">
                <el-icon :size="32"><User /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value neon-text">{{ stats.agentCount }}/{{ stats.totalAgents }}</div>
                <div class="stat-label">Agent 配置</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card hover-lift" shadow="never">
            <div class="stat-content">
              <div class="stat-icon category-icon">
                <el-icon :size="32"><Folder /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value neon-text-success">{{ stats.categoryCount }}/{{ stats.totalCategories }}</div>
                <div class="stat-label">Category 配置</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :xs="24" :sm="12" :md="8">
          <el-card class="stat-card hover-lift" shadow="never">
            <div class="stat-content">
              <div class="stat-icon preset-icon">
                <el-icon :size="32"><Collection /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value neon-text-warning">{{ stats.presetCount }}</div>
                <div class="stat-label">已保存预设</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 快速操作区域 -->
      <el-row :gutter="20" class="actions-section">
        <el-col :span="24">
          <el-card class="actions-card" shadow="never">
            <template #header>
              <div class="section-header">
                <span class="section-title neon-text-subtle">快速操作</span>
              </div>
            </template>
            <div class="quick-actions">
               <!-- 工作路径输入 -->
               <div class="path-input-wrapper">
                 <el-input
                   v-model="workingPath"
                   placeholder="输入工作目录路径（留空使用用户主目录）"
                   clearable
                   class="path-input neon-input"
                 >
                   <template #prepend>
                     <el-icon><Folder /></el-icon>
                     <span>工作路径</span>
                   </template>
                   <template #append>
                     <el-button class="neon-btn-secondary" @click="browseFolder">
                       <el-icon><FolderOpened /></el-icon>
                       浏览
                     </el-button>
                   </template>
                  </el-input>
                </div>
                
                <!-- 代理配置区域 -->
                <div class="proxy-config-wrapper glass-card-overlay">
                  <div class="proxy-switch-row">
                    <el-switch
                      v-model="proxyEnabled"
                      active-text="启用监控代理"
                      inactive-text="直连模式"
                      class="glass-switch"
                    />
                    <el-tag v-if="proxyEnabled" :type="certExists === true ? 'success' : 'info'" size="small" effect="dark">
                      {{ certExists === true ? '证书已就绪' : (certExists === false ? '证书生成中...' : '检查中...') }}
                    </el-tag>
                  </div>
                  
                  <el-collapse-transition>
                    <div v-if="proxyEnabled" class="proxy-cert-info">
                      <!-- 证书路径显示 -->
                      <div class="cert-path-display">
                        <el-icon><Key /></el-icon>
                        <span class="cert-label">CA 证书路径：</span>
                        <code class="cert-path">{{ defaultCertPath }}</code>
                      </div>
                      
                      <!-- 证书不存在提示 -->
                      <el-alert
                        v-if="certExists === false"
                        title="正在生成证书"
                        type="info"
                        :closable="false"
                        show-icon
                      >
                        <template #default>
                          首次启用监控代理时会自动生成 CA 证书，请稍候片刻。证书生成完成后即可正常使用监控功能。
                        </template>
                      </el-alert>
                      
                      <div class="proxy-info">
                        <el-icon><InfoFilled /></el-icon>
                        <span>启用后，流量将通过监控代理 (localhost:{{ proxyPort }})，可监控 LLM API 调用。</span>
                      </div>
                    </div>
                  </el-collapse-transition>
                </div>
               
               <el-button 
                 class="action-btn neon-btn-primary" 
                 size="large" 
                 :loading="isLaunching"
                 @click="handleLaunchOpenCode"
               >
                 <el-icon><VideoPlay /></el-icon>
                 启动 OpenCode
               </el-button>
                <el-button class="action-btn neon-btn-secondary" size="large" @click="goToConfig">
                  <el-icon><Setting /></el-icon>
                  模型配置
                </el-button>
               <el-button class="action-btn neon-btn-info" size="large" @click="goToPresets">
                 <el-icon><Collection /></el-icon>
                 管理预设
               </el-button>
               <el-button 
                 class="action-btn neon-btn-warning" 
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
          <el-card class="presets-card" shadow="never">
            <template #header>
              <div class="section-header">
                <span class="section-title neon-text-subtle">最近使用的预设</span>
                <el-button link class="view-all-btn" @click="goToPresets">
                  查看全部
                  <el-icon class="el-icon--right"><ArrowRight /></el-icon>
                </el-button>
              </div>
            </template>
            
            <div v-if="recentPresets.length > 0" class="preset-list">
              <div
                v-for="(preset, index) in recentPresets"
                :key="preset.name"
                class="preset-item glass-card-overlay"
                :class="{ active: preset.name === activePresetName }"
                :style="{ '--stagger-delay': `${index * 50}ms` }"
                @click="applyPreset(preset.name)"
              >
                <div class="preset-info">
                  <div class="preset-name">
                    <el-icon class="preset-icon"><Document /></el-icon>
                    <span class="preset-name-text">{{ preset.name }}</span>
                    <el-tag v-if="preset.name === activePresetName" size="small" effect="dark" class="active-badge">
                      当前
                    </el-tag>
                  </div>
                  <div v-if="preset.description" class="preset-desc">
                    {{ preset.description }}
                  </div>
                </div>
                <div class="preset-meta">
                  <span class="preset-date">{{ formatDate(preset.updatedAt) }}</span>
                  <el-button 
                    class="apply-btn neon-btn-primary" 
                    link 
                    size="small"
                    @click.stop="applyPreset(preset.name)"
                  >
                    应用
                  </el-button>
                </div>
              </div>
            </div>
            
            <el-empty v-else description="暂无预设" class="empty-presets">
              <el-button class="neon-btn-primary" @click="createPreset">
                创建第一个预设
              </el-button>
            </el-empty>
          </el-card>
        </el-col>
      </el-row>
    </div>
</template>

<style scoped>
/* ==================== 基础样式 ==================== */
.home-dashboard {
  padding: 0;
}

/* 区块间距 */
.welcome-section,
.stats-section,
.actions-section,
.presets-section {
  margin-bottom: var(--app-spacing-5);
}

/* ==================== 赛博朋克主题 - 欢迎卡片 ==================== */
html.cyberpunk .welcome-card {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.1) 0%, rgba(255, 0, 255, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

/* 赛博朋克欢迎卡片文字 - 使用浅色确保对比度 */
html.cyberpunk .welcome-title {
  color: #ffffff;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .welcome-subtitle {
  color: #e0e0ff;
}

html.cyberpunk .welcome-status,
html.cyberpunk .active-preset,
html.cyberpunk .active-preset strong {
  color: #ffffff;
}

/* ==================== 玻璃拟态主题 - 欢迎卡片 ==================== */
html.glassmorphism .welcome-card {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15) 0%, rgba(14, 165, 233, 0.1) 100%);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

/* 玻璃拟态欢迎卡片文字 - 使用深色确保对比度 */
html.glassmorphism .welcome-title {
  color: #1e293b;
}

html.glassmorphism .welcome-subtitle {
  color: #475569;
}

html.glassmorphism .welcome-status,
html.glassmorphism .active-preset,
html.glassmorphism .active-preset strong {
  color: #1e293b;
}

/* 欢迎区域 */
.welcome-card {
  background: linear-gradient(135deg, var(--app-color-primary) 0%, var(--app-color-success) 100%);
  border-radius: var(--app-radius-lg);
  padding: var(--app-spacing-8);
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--app-text-inverse);
  box-shadow: var(--app-shadow-lg);
  transition: all 0.3s ease;
}

.welcome-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 var(--app-spacing-2) 0;
  color: var(--app-text-inverse);
}

.welcome-subtitle {
  font-size: 16px;
  margin: 0 0 var(--app-spacing-5) 0;
  opacity: 0.9;
  color: var(--app-text-inverse);
}

.welcome-status {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-4);
}

.active-preset {
  font-size: 14px;
  color: var(--app-text-inverse);
}

.active-preset strong {
  color: var(--app-text-inverse);
  font-weight: 600;
}

.welcome-icon {
  opacity: 0.3;
}

/* ==================== 霓虹文字效果 - 赛博朋克 ==================== */
html.cyberpunk .neon-text {
  color: var(--app-color-primary);
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .neon-text-success {
  color: var(--app-color-success);
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

html.cyberpunk .neon-text-warning {
  color: var(--app-color-warning);
  text-shadow: 0 0 10px rgba(255, 170, 0, 0.5);
}

html.cyberpunk .neon-text-subtle {
  color: var(--app-color-primary);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

/* ==================== 霓虹文字效果 - 玻璃拟态 ==================== */
html.glassmorphism .neon-text {
  color: var(--app-color-primary);
}

html.glassmorphism .neon-text-success {
  color: var(--app-color-success);
}

html.glassmorphism .neon-text-warning {
  color: var(--app-color-warning);
}

html.glassmorphism .neon-text-subtle {
  color: var(--app-color-primary);
}

/* ==================== 统计卡片 ==================== */
.stat-card {
  transition: transform var(--app-transition-normal), box-shadow 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
}

/* 赛博朋克主题 - 统计卡片悬停 */
html.cyberpunk .stat-card:hover {
  box-shadow: 0 0 30px rgba(0, 255, 255, 0.2);
}

/* 玻璃拟态主题 - 统计卡片悬停 */
html.glassmorphism .stat-card:hover {
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.15);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-4);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--app-radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--app-text-inverse);
}

.agent-icon {
  background: linear-gradient(135deg, var(--app-color-primary) 0%, color-mix(in srgb, var(--app-color-primary) 70%, white) 100%);
}

.category-icon {
  background: linear-gradient(135deg, var(--app-color-success) 0%, color-mix(in srgb, var(--app-color-success) 70%, white) 100%);
}

.preset-icon {
  background: linear-gradient(135deg, var(--app-color-warning) 0%, color-mix(in srgb, var(--app-color-warning) 70%, white) 100%);
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--app-text-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 14px;
  color: var(--app-text-tertiary);
  margin-top: var(--app-spacing-1);
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
  color: var(--app-text-primary);
}

/* ==================== 快速操作 ==================== */
.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--app-spacing-3);
}

.path-input-wrapper {
  width: 100%;
  margin-bottom: var(--app-spacing-2);
}

.path-input {
  width: 100%;
}

.path-input :deep(.el-input-group__prepend) {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

/* ==================== 代理配置区域 ==================== */
.proxy-config-wrapper {
  width: 100%;
  margin-bottom: var(--app-spacing-3);
  padding: var(--app-spacing-3);
  background-color: var(--app-bg-hover);
  border-radius: var(--app-radius-md);
  transition: all 0.3s ease;
}

/* 赛博朋克主题 - 代理配置区域 */
html.cyberpunk .proxy-config-wrapper {
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.15);
}

/* 玻璃拟态主题 - 代理配置区域 */
html.glassmorphism .proxy-config-wrapper {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(37, 99, 235, 0.25);
  backdrop-filter: blur(8px);
}

.proxy-switch-row {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
  margin-bottom: var(--app-spacing-2);
}

.proxy-cert-config {
  margin-top: var(--app-spacing-3);
}

.proxy-cert-info {
  margin-top: var(--app-spacing-3);
}

.cert-path-display {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  padding: var(--app-spacing-2) var(--app-spacing-3);
  background-color: var(--app-bg-hover);
  border-radius: var(--app-radius-sm);
  margin-bottom: var(--app-spacing-2);
}

.cert-label {
  color: var(--app-text-tertiary);
  font-size: 13px;
}

.cert-path {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--app-color-primary);
  background-color: color-mix(in srgb, var(--app-color-primary) 10%, transparent);
  padding: 2px 6px;
  border-radius: var(--app-radius-sm);
  word-break: break-all;
}

.proxy-info {
  display: flex;
  align-items: flex-start;
  gap: var(--app-spacing-2);
  margin-top: var(--app-spacing-2);
  padding: var(--app-spacing-2) var(--app-spacing-3);
  background-color: color-mix(in srgb, var(--app-color-primary) 10%, transparent);
  border-radius: var(--app-radius-sm);
  font-size: 12px;
  color: var(--app-color-primary);
  line-height: 1.5;
  transition: all 0.3s ease;
}

/* 赛博朋克主题 - 代理信息 */
html.cyberpunk .proxy-info {
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.2);
}

/* 玻璃拟态主题 - 代理信息 */
html.glassmorphism .proxy-info {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.2);
}

.proxy-info .el-icon {
  margin-top: 2px;
  flex-shrink: 0;
}

/* ==================== 霓虹按钮样式 ==================== */
/* 赛博朋克主题 - 主按钮 */
html.cyberpunk .neon-btn-primary {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15), rgba(0, 255, 255, 0.05));
  border: 1px solid rgba(0, 255, 255, 0.4);
  color: var(--app-color-primary);
  box-shadow: 
    0 0 10px rgba(0, 255, 255, 0.2),
    inset 0 0 10px rgba(0, 255, 255, 0.05);
  transition: all 0.3s ease;
}

html.cyberpunk .neon-btn-primary:hover {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.25), rgba(0, 255, 255, 0.1));
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.4),
    0 0 40px rgba(0, 255, 255, 0.2),
    inset 0 0 15px rgba(0, 255, 255, 0.1);
  transform: translateY(-1px);
}

html.cyberpunk .neon-btn-secondary {
  background: linear-gradient(135deg, rgba(255, 0, 255, 0.15), rgba(255, 0, 255, 0.05));
  border: 1px solid rgba(255, 0, 255, 0.4);
  color: var(--app-color-secondary);
  box-shadow: 
    0 0 10px rgba(255, 0, 255, 0.2),
    inset 0 0 10px rgba(255, 0, 255, 0.05);
  transition: all 0.3s ease;
}

html.cyberpunk .neon-btn-secondary:hover {
  background: linear-gradient(135deg, rgba(255, 0, 255, 0.25), rgba(255, 0, 255, 0.1));
  box-shadow: 
    0 0 20px rgba(255, 0, 255, 0.4),
    0 0 40px rgba(255, 0, 255, 0.2),
    inset 0 0 15px rgba(255, 0, 255, 0.1);
  transform: translateY(-1px);
}

html.cyberpunk .neon-btn-success {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.15), rgba(0, 255, 136, 0.05));
  border: 1px solid rgba(0, 255, 136, 0.4);
  color: var(--app-color-success);
  box-shadow: 
    0 0 10px rgba(0, 255, 136, 0.2),
    inset 0 0 10px rgba(0, 255, 136, 0.05);
  transition: all 0.3s ease;
}

html.cyberpunk .neon-btn-success:hover {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.25), rgba(0, 255, 136, 0.1));
  box-shadow: 
    0 0 20px rgba(0, 255, 136, 0.4),
    0 0 40px rgba(0, 255, 136, 0.2),
    inset 0 0 15px rgba(0, 255, 136, 0.1);
  transform: translateY(-1px);
}

html.cyberpunk .neon-btn-info {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.15), rgba(251, 191, 36, 0.05));
  border: 1px solid rgba(251, 191, 36, 0.4);
  color: var(--app-color-accent);
  box-shadow: 
    0 0 10px rgba(251, 191, 36, 0.2),
    inset 0 0 10px rgba(251, 191, 36, 0.05);
  transition: all 0.3s ease;
}

html.cyberpunk .neon-btn-info:hover {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.25), rgba(251, 191, 36, 0.1));
  box-shadow: 
    0 0 20px rgba(251, 191, 36, 0.4),
    0 0 40px rgba(251, 191, 36, 0.2),
    inset 0 0 15px rgba(251, 191, 36, 0.1);
  transform: translateY(-1px);
}

html.cyberpunk .neon-btn-warning {
  background: linear-gradient(135deg, rgba(255, 170, 0, 0.15), rgba(255, 170, 0, 0.05));
  border: 1px solid rgba(255, 170, 0, 0.4);
  color: var(--app-color-warning);
  box-shadow: 
    0 0 10px rgba(255, 170, 0, 0.2),
    inset 0 0 10px rgba(255, 170, 0, 0.05);
  transition: all 0.3s ease;
}

html.cyberpunk .neon-btn-warning:hover {
  background: linear-gradient(135deg, rgba(255, 170, 0, 0.25), rgba(255, 170, 0, 0.1));
  box-shadow: 
    0 0 20px rgba(255, 170, 0, 0.4),
    0 0 40px rgba(255, 170, 0, 0.2),
    inset 0 0 15px rgba(255, 170, 0, 0.1);
  transform: translateY(-1px);
}

/* 玻璃拟态主题 - 按钮 */
html.glassmorphism .neon-btn-primary {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15), rgba(37, 99, 235, 0.05));
  border: 1px solid rgba(37, 99, 235, 0.4);
  color: var(--app-color-primary);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.15);
  transition: all 0.3s ease;
}

html.glassmorphism .neon-btn-primary:hover {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.25), rgba(37, 99, 235, 0.1));
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.25);
  transform: translateY(-1px);
}

html.glassmorphism .neon-btn-secondary,
html.glassmorphism .neon-btn-success,
html.glassmorphism .neon-btn-info,
html.glassmorphism .neon-btn-warning {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(37, 99, 235, 0.3);
  backdrop-filter: blur(8px);
  transition: all 0.3s ease;
}

/* ==================== 预设列表 ==================== */
.preset-list {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
}

.preset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background-color: var(--app-bg-hover);
  border-radius: var(--app-radius-md);
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;
}

/* 赛博朋克主题 - 预设项 */
html.cyberpunk .preset-item {
  background: rgba(26, 26, 46, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.1);
}

html.cyberpunk .preset-item:hover {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.15);
  transform: translateX(4px);
}

html.cyberpunk .preset-item.active {
  background: rgba(0, 255, 255, 0.12);
  border-color: var(--app-color-primary);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.2);
}

/* 玻璃拟态主题 - 预设项 */
html.glassmorphism .preset-item {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(37, 99, 235, 0.2);
  backdrop-filter: blur(8px);
}

html.glassmorphism .preset-item:hover {
  background: rgba(255, 255, 255, 0.7);
  border-color: rgba(37, 99, 235, 0.4);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  transform: translateX(4px);
}

html.glassmorphism .preset-item.active {
  background: rgba(37, 99, 235, 0.1);
  border-color: var(--app-color-primary);
}

.preset-item:hover {
  background-color: color-mix(in srgb, var(--app-color-primary) 8%, transparent);
  border-color: var(--app-color-primary);
}

.preset-item.active {
  background-color: color-mix(in srgb, var(--app-color-primary) 8%, transparent);
  border-color: var(--app-color-primary);
}

.preset-info {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-1);
}

.preset-name {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-weight: 500;
  color: var(--app-text-primary);
}

.preset-desc {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.preset-meta {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.preset-date {
  font-size: 12px;
  color: var(--app-text-disabled);
}

/* 当前标签 */
html.cyberpunk .active-badge {
  background: rgba(0, 255, 136, 0.2) !important;
  border: 1px solid rgba(0, 255, 136, 0.4) !important;
  color: var(--app-color-success) !important;
  box-shadow: 0 0 10px rgba(0, 255, 136, 0.3);
}

html.glassmorphism .active-badge {
  background: rgba(16, 185, 129, 0.15) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

/* 空状态 */
.empty-presets {
  padding: var(--app-spacing-8) 0;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .welcome-card {
    flex-direction: column;
    text-align: center;
    gap: var(--app-spacing-5);
  }

  .welcome-icon {
    display: none;
  }

  .stat-card {
    margin-bottom: var(--app-spacing-3);
  }

  .quick-actions {
    justify-content: center;
  }

  .preset-item {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--app-spacing-2);
  }

  .preset-meta {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
