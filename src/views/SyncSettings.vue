<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Connection, Upload, Download, Refresh, SwitchButton } from '@element-plus/icons-vue'
import { useSyncStore } from '@/stores/sync'

const syncStore = useSyncStore()

// PAT 输入
const patInput = ref('')

// 当前认证状态
const authState = computed(() => syncStore.authState)
const currentUser = computed(() => syncStore.currentUser)
const isSyncing = computed(() => syncStore.isSyncing)
const lastSyncTime = computed(() => syncStore.lastSyncTime)
const lastError = computed(() => syncStore.lastError)

// OAuth Web Flow 登录（推荐）
const handleOAuthLogin = async () => {
  try {
    await syncStore.loginWithOAuth()
    ElMessage.success(`已登录为 ${currentUser.value?.login}`)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// PAT 登录
const handlePatLogin = async () => {
  const pat = patInput.value.trim()
  if (!pat) {
    ElMessage.warning('请输入 PAT')
    return
  }
  try {
    await syncStore.loginWithPat(pat)
    patInput.value = ''
    ElMessage.success(`已登录为 ${currentUser.value?.login}`)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 登出
const handleLogout = async () => {
  try {
    await syncStore.logout()
    ElMessage.success('已登出')
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 手动同步
const handleSync = async () => {
  try {
    const result = await syncStore.sync()
    if (result.type === 'UpToDate') {
      ElMessage.success('配置已是最新')
    } else if (result.type === 'Uploaded') {
      ElMessage.success(`已上传 ${result.count} 个预设`)
    } else if (result.type === 'Downloaded') {
      ElMessage.success(`已下载 ${result.count} 个预设`)
    } else if (result.type === 'Conflict') {
      ElMessage.warning('检测到同步冲突，请在弹出的对话框中选择')
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 手动上传
const handleUpload = async () => {
  try {
    const result = await syncStore.upload()
    if (result.type === 'Uploaded') {
      ElMessage.success(`已上传 ${result.count} 个预设`)
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 手动下载
const handleDownload = async () => {
  try {
    const result = await syncStore.download()
    if (result.type === 'Downloaded') {
      ElMessage.success(`已下载 ${result.count} 个预设`)
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 格式化同步时间
const formatSyncTime = (time: string | null | undefined) => {
  if (!time) return '从未同步'
  const d = new Date(time)
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <div class="sync-settings">
    <div class="page-header">
      <div class="header-left">
        <span class="subtitle">GitHub 同步设置 — 跨设备同步预设配置</span>
      </div>
    </div>

    <!-- 未登录状态 -->
    <el-card v-if="authState.type === 'LoggedOut'" class="auth-card">
      <div class="auth-section">
        <div class="auth-icon">
          <el-icon :size="48" color="var(--app-color-primary)"><Connection /></el-icon>
        </div>
        <h2 class="auth-title">登录 GitHub</h2>
        <p class="auth-desc">登录后可将预设配置同步到 GitHub Gist，在多台设备间共享配置</p>

        <!-- OAuth Web Flow 登录 -->
        <el-button
          type="primary"
          class="neon-button-primary"
          @click="handleOAuthLogin"
        >
          <el-icon><Connection /></el-icon>
          使用 GitHub 登录
        </el-button>

        <div class="auth-divider">
          <span>或</span>
        </div>

        <!-- PAT 登录 -->
        <div class="pat-section">
          <el-input
            v-model="patInput"
            placeholder="请输入 GitHub Classic PAT (需要 gist 权限)"
            type="password"
            show-password
            @keyup.enter="handlePatLogin"
          />
          <el-button type="primary" class="neon-button-primary" @click="handlePatLogin">
            使用 PAT 登录
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- OAuth 登录进行中 -->
    <el-card v-else-if="authState.type === 'OAuthLoggingIn'" class="auth-card">
      <div class="auth-section">
        <div class="device-flow-pending">
          <el-icon :size="48" color="var(--app-color-primary)" class="is-loading"><Refresh /></el-icon>
          <h2 class="auth-title">等待 GitHub 授权</h2>
          <p class="auth-desc">浏览器已打开，请在 GitHub 页面点击授权按钮</p>
        </div>
      </div>
    </el-card>

    <!-- 已登录状态 -->
    <el-card v-else class="auth-card">
      <div class="logged-in-section">
        <!-- 用户信息 -->
        <div class="user-info">
          <el-avatar :size="64" :src="currentUser?.avatar_url" />
          <div class="user-details">
            <h2 class="user-name">{{ currentUser?.name || currentUser?.login }}</h2>
            <span class="user-login">@{{ currentUser?.login }}</span>
          </div>
        </div>

        <!-- 同步状态 -->
        <div class="sync-status">
          <div class="sync-status-row">
            <span class="status-label">同步状态</span>
            <span class="status-value">
              <template v-if="isSyncing">
                <el-icon class="is-loading"><Refresh /></el-icon>
                同步中...
              </template>
              <template v-else>
                {{ formatSyncTime(lastSyncTime) }}
              </template>
            </span>
          </div>
          <div v-if="lastError" class="sync-error">
            <span>{{ lastError }}</span>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="sync-actions">
          <el-button
            class="neon-button-primary"
            :loading="isSyncing"
            @click="handleSync"
          >
            <el-icon><Refresh /></el-icon>
            同步
          </el-button>
          <el-button
            class="neon-button-upload"
            :loading="isSyncing"
            @click="handleUpload"
          >
            <el-icon><Upload /></el-icon>
            上传
          </el-button>
          <el-button
            class="neon-button-download"
            :loading="isSyncing"
            @click="handleDownload"
          >
            <el-icon><Download /></el-icon>
            下载
          </el-button>
          <el-button
            class="neon-button-logout"
            @click="handleLogout"
          >
            <el-icon><SwitchButton /></el-icon>
            登出
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<style scoped>
.sync-settings {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 0 8px;
}

.subtitle {
  color: var(--app-text-secondary);
  font-size: 14px;
  letter-spacing: 0.5px;
}

/* 认证卡片 */
.auth-card {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
  border-radius: 12px !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4) !important;
}

:deep(.el-card__body) {
  padding: 32px !important;
}

.auth-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.auth-icon {
  margin-bottom: 4px;
}

.auth-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 0;
}

.auth-desc {
  color: var(--app-text-secondary);
  font-size: 14px;
  text-align: center;
  margin: 0;
  max-width: 400px;
}

/* 分隔线 */
.auth-divider {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 100%;
  max-width: 400px;
  color: var(--app-text-tertiary);
  font-size: 13px;
}

.auth-divider::before,
.auth-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--app-border-default);
}

/* PAT 输入区 */
.pat-section {
  display: flex;
  gap: 12px;
  width: 100%;
  max-width: 500px;
}

.pat-section .el-input {
  flex: 1;
}

/* Device Flow 进行中 */
.device-flow-pending {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.user-code-block {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

/* 已登录区域 */
.logged-in-section {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.user-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 0;
}

.user-login {
  font-size: 14px;
  color: var(--app-text-secondary);
}

/* 同步状态 */
.sync-status {
  padding: 16px;
  background: var(--app-bg-base);
  border-radius: 8px;
  border: 1px solid var(--app-border-default);
}

.sync-status-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-label {
  font-size: 14px;
  color: var(--app-text-secondary);
}

.status-value {
  font-size: 14px;
  color: var(--app-text-primary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.sync-error {
  margin-top: 8px;
  color: var(--app-color-danger);
  font-size: 13px;
}

/* 操作按钮 */
.sync-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

/* 霓虹按钮 */
.neon-button-primary {
  background: transparent !important;
  border: 1px solid var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.3) !important;
  transition: all 0.3s ease !important;
}

.neon-button-primary:hover {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.5) !important;
}

.neon-button-upload {
  background: transparent !important;
  border: 1px solid var(--app-color-success) !important;
  color: var(--app-color-success) !important;
  transition: all 0.3s ease !important;
}

.neon-button-upload:hover {
  background: rgba(0, 255, 157, 0.1) !important;
  box-shadow: 0 0 15px rgba(0, 255, 157, 0.4) !important;
}

.neon-button-download {
  background: transparent !important;
  border: 1px solid var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  transition: all 0.3s ease !important;
}

.neon-button-download:hover {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.4) !important;
}

.neon-button-logout {
  background: transparent !important;
  border: 1px solid var(--app-color-danger) !important;
  color: var(--app-color-danger) !important;
  transition: all 0.3s ease !important;
}

.neon-button-logout:hover {
  background: rgba(255, 71, 87, 0.1) !important;
  box-shadow: 0 0 15px rgba(255, 71, 87, 0.4) !important;
}

/* Input 样式 */
:deep(.el-input__wrapper) {
  background: var(--app-bg-base) !important;
  border: 1px solid var(--app-border-default) !important;
  box-shadow: none !important;
}

:deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary) !important;
}

:deep(.el-input__inner) {
  color: var(--app-text-primary) !important;
}

:deep(.el-input__inner::placeholder) {
  color: var(--app-text-secondary) !important;
}

/* Element Plus 按钮 override */
:deep(.el-button) {
  --el-button-bg-color: transparent;
  --el-button-border-color: var(--app-border-default);
  --el-button-text-color: var(--app-text-primary);
  --el-button-hover-bg-color: var(--app-bg-hover);
}

/* Loading 动画 */
.is-loading {
  animation: rotating 1.5s linear infinite;
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk .auth-card {
  background: rgba(26, 26, 46, 0.9) !important;
  border: 1px solid rgba(0, 255, 255, 0.2) !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5), 0 0 40px rgba(0, 255, 255, 0.1) !important;
}

html.cyberpunk .auth-title {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .sync-status {
  background: rgba(0, 255, 255, 0.03);
  border-color: rgba(0, 255, 255, 0.15);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism .auth-card {
  background: rgba(255, 255, 255, 0.7) !important;
  border: 1px solid rgba(255, 255, 255, 0.9) !important;
  backdrop-filter: blur(16px) !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1) !important;
}

html.glassmorphism .sync-status {
  background: rgba(255, 255, 255, 0.5);
  border-color: rgba(0, 0, 0, 0.08);
}

html.glassmorphism .neon-button-primary {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary)) !important;
  border: none !important;
  color: white !important;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.3) !important;
}

html.glassmorphism .neon-button-upload {
  background: rgba(16, 185, 129, 0.15) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
  box-shadow: none !important;
}

html.glassmorphism .neon-button-download {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.3) !important;
  box-shadow: none !important;
}

html.glassmorphism .neon-button-logout {
  background: rgba(239, 68, 68, 0.1) !important;
  border: 1px solid rgba(239, 68, 68, 0.3) !important;
  color: var(--app-color-danger) !important;
  box-shadow: none !important;
}

html.glassmorphism :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.8) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
}
</style>
