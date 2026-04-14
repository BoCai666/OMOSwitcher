<script setup lang="ts">
/**
 * 用户抽屉组件
 * 从右侧滑出，提供 GitHub 登录/同步设置/用户信息
 */
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import { useSyncStore } from '@/stores/sync'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const syncStore = useSyncStore()

const visible = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val)
})

const isLoggedIn = computed(() => syncStore.isLoggedIn)
const currentUser = computed(() => syncStore.currentUser)
const isSyncing = computed(() => syncStore.isSyncing)
const lastSyncTime = computed(() => syncStore.lastSyncTime)
const lastError = computed(() => syncStore.lastError)

// 格式化同步时间
const formatSyncTime = (time: string | null | undefined): string => {
  if (!time) return '从未同步'
  const d = new Date(time)
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// OAuth 登录
async function handleOAuthLogin() {
  try {
    await syncStore.loginWithOAuth()
    ElMessage.success(`已登录为 ${currentUser.value?.login}`)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 同步操作
async function handleSync() {
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

// 上传
async function handleUpload() {
  try {
    const result = await syncStore.upload()
    if (result.type === 'Uploaded') {
      ElMessage.success(`已上传 ${result.count} 个预设`)
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 下载
async function handleDownload() {
  try {
    const result = await syncStore.download()
    if (result.type === 'Downloaded') {
      ElMessage.success(`已下载 ${result.count} 个预设`)
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 登出
async function handleLogout() {
  try {
    await syncStore.logout()
    ElMessage.success('已登出')
  } catch (e) {
    ElMessage.error(String(e))
  }
}
</script>

<template>
  <el-drawer
    v-model="visible"
    :show-close="true"
    :with-header="false"
    direction="rtl"
    size="380px"
    class="user-drawer"
    :append-to-body="true"
  >
    <div class="drawer-content">
      <!-- 未登录状态 -->
      <template v-if="!isLoggedIn">
        <div class="auth-section">
          <!-- 图标 -->
          <div class="auth-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
              <circle cx="12" cy="7" r="4" />
            </svg>
          </div>

          <h2 class="auth-title">登录 GitHub 同步预设</h2>
          <p class="auth-desc">登录后可将预设配置同步到 GitHub Gist，在多台设备间共享配置</p>

          <!-- OAuth 登录按钮 -->
          <button class="github-login-btn" @click="handleOAuthLogin">
            <svg viewBox="0 0 16 16" fill="currentColor" width="18" height="18">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            使用 GitHub 登录
          </button>

          <p class="auth-hint">也可以稍后在设置中登录</p>
        </div>
      </template>

      <!-- 已登录状态 -->
      <template v-else>
        <div class="logged-in-section">
          <!-- 用户信息 -->
          <div class="user-info">
            <img
              v-if="currentUser?.avatar_url"
              :src="currentUser.avatar_url"
              :alt="currentUser.login"
              class="user-avatar-lg"
            />
            <div class="user-avatar-lg-placeholder" v-else>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="8" r="4"/>
                <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/>
              </svg>
            </div>
            <div class="user-details">
              <h3 class="user-name">{{ currentUser?.name || currentUser?.login }}</h3>
              <span class="user-login">@{{ currentUser?.login }}</span>
            </div>
          </div>

          <!-- 同步状态 -->
          <div class="sync-status-card">
            <div class="sync-status-row">
              <span class="status-label">同步状态</span>
              <span class="status-value">
                <template v-if="isSyncing">
                  <svg class="sync-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 12a9 9 0 11-6.219-8.56"/>
                  </svg>
                  同步中...
                </template>
                <template v-else>{{ formatSyncTime(lastSyncTime) }}</template>
              </span>
            </div>
            <div v-if="lastError" class="sync-error">{{ lastError }}</div>
          </div>

          <!-- 操作按钮 -->
          <div class="sync-actions">
            <button class="action-btn action-btn-primary" :disabled="isSyncing" @click="handleSync">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12a9 9 0 11-6.219-8.56"/>
                <polyline points="21 3 21 9 15 9"/>
              </svg>
              同步
            </button>
            <button class="action-btn action-btn-upload" :disabled="isSyncing" @click="handleUpload">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              上传
            </button>
            <button class="action-btn action-btn-download" :disabled="isSyncing" @click="handleDownload">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
              下载
            </button>
          </div>

          <!-- 底部操作 -->
          <div class="drawer-footer">
            <button class="footer-btn footer-btn-logout" @click="handleLogout">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
                <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
                <polyline points="16 17 21 12 16 7"/>
                <line x1="21" y1="12" x2="9" y2="12"/>
              </svg>
              登出
            </button>
          </div>
        </div>
      </template>
    </div>
  </el-drawer>
</template>

<style scoped>
/* ==================== 抽屉内容 ==================== */
.drawer-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 20px;
  overflow-y: auto;
}

/* ==================== 未登录区域 ==================== */
.auth-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding-top: 32px;
}

.auth-icon {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--app-bg-hover);
  color: var(--app-color-primary);
}

.auth-icon svg {
  width: 28px;
  height: 28px;
}

.auth-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 0;
  text-align: center;
}

.auth-desc {
  font-size: 13px;
  color: var(--app-text-secondary);
  text-align: center;
  margin: 0;
  line-height: 1.6;
  max-width: 280px;
}

/* GitHub 登录按钮 */
.github-login-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  color: var(--app-text-primary);
}

.github-login-btn:hover {
  transform: translateY(-1px);
}

.auth-hint {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin: 8px 0 0;
}

/* ==================== 已登录区域 ==================== */
.logged-in-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 用户信息 */
.user-info {
  display: flex;
  align-items: center;
  gap: 14px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--app-border-default);
}

.user-avatar-lg {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--app-border-default);
}

.user-avatar-lg-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--app-bg-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--app-text-tertiary);
}

.user-avatar-lg-placeholder svg {
  width: 24px;
  height: 24px;
}

.user-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.user-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 0;
}

.user-login {
  font-size: 13px;
  color: var(--app-text-secondary);
}

/* 同步状态卡片 */
.sync-status-card {
  padding: 14px 16px;
  background: var(--app-bg-hover);
  border-radius: 8px;
  border: 1px solid var(--app-border-default);
}

.sync-status-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-label {
  font-size: 13px;
  color: var(--app-text-secondary);
}

.status-value {
  font-size: 13px;
  color: var(--app-text-primary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.sync-spin {
  width: 14px;
  height: 14px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.sync-error {
  margin-top: 8px;
  font-size: 12px;
  color: var(--app-color-danger);
}

/* 操作按钮 */
.sync-actions {
  display: flex;
  gap: 10px;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 8px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: transparent;
  border: 1px solid var(--app-border-default);
  color: var(--app-text-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn svg {
  width: 14px;
  height: 14px;
}

.action-btn-primary {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
}

.action-btn-primary:not(:disabled):hover {
  background: var(--app-color-primary);
  color: #fff;
}

.action-btn-upload {
  border-color: var(--app-color-success);
  color: var(--app-color-success);
}

.action-btn-upload:not(:disabled):hover {
  background: var(--app-color-success);
  color: #fff;
}

.action-btn-download {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
}

.action-btn-download:not(:disabled):hover {
  background: var(--app-color-primary);
  color: #fff;
}

/* 底部操作 */
.drawer-footer {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid var(--app-border-default);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  background: transparent;
  border: none;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.2s;
}

.footer-btn-logout {
  color: var(--app-color-danger);
}

.footer-btn-logout:hover {
  background: rgba(239, 68, 68, 0.08);
}

/* ==================== Cyberpunk 主题 ==================== */
html.cyberpunk .github-login-btn {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.3);
  color: #00ffff;
}

html.cyberpunk .github-login-btn:hover {
  background: rgba(0, 255, 255, 0.15);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .auth-icon {
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
}

html.cyberpunk .user-avatar-lg {
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .sync-status-card {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.15);
}

html.cyberpunk .action-btn-primary {
  border-color: rgba(0, 255, 255, 0.4);
  color: #00ffff;
}

html.cyberpunk .action-btn-primary:not(:disabled):hover {
  background: rgba(0, 255, 255, 0.2);
  color: #00ffff;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .action-btn-upload {
  border-color: rgba(0, 255, 136, 0.4);
  color: #00ff88;
}

html.cyberpunk .action-btn-upload:not(:disabled):hover {
  background: rgba(0, 255, 136, 0.2);
  color: #00ff88;
  box-shadow: 0 0 15px rgba(0, 255, 136, 0.3);
}

html.cyberpunk .action-btn-download {
  border-color: rgba(0, 255, 255, 0.4);
  color: #00ffff;
}

html.cyberpunk .action-btn-download:not(:disabled):hover {
  background: rgba(0, 255, 255, 0.2);
  color: #00ffff;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

/* ==================== Glassmorphism 主题 ==================== */
html.glassmorphism .github-login-btn {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.12), rgba(37, 99, 235, 0.05));
  border-color: rgba(37, 99, 235, 0.3);
  color: #2563eb;
}

html.glassmorphism .github-login-btn:hover {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.2), rgba(37, 99, 235, 0.1));
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2);
}

html.glassmorphism .auth-icon {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.2);
  color: #2563eb;
}

html.glassmorphism .user-avatar-lg {
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .sync-status-card {
  background: rgba(255, 255, 255, 0.5);
  border-color: rgba(37, 99, 235, 0.15);
  backdrop-filter: blur(8px);
}

html.glassmorphism .action-btn-primary:not(:disabled):hover {
  background: #2563eb;
  color: #fff;
}

html.glassmorphism .action-btn-upload:not(:disabled):hover {
  background: #10b981;
  color: #fff;
}

html.glassmorphism .action-btn-download:not(:disabled):hover {
  background: #2563eb;
  color: #fff;
}

/* ==================== Drawer 深度样式覆盖 ==================== */
:deep(.el-drawer) {
  background: var(--app-bg-card) !important;
  border-left: 1px solid var(--app-border-default);
}

:deep(.el-drawer__body) {
  padding: 0 !important;
}

:deep(.el-drawer__close-btn) {
  color: var(--app-text-tertiary);
}

:deep(.el-drawer__close-btn:hover) {
  color: var(--app-text-primary);
}

:deep(.el-overlay) {
  background-color: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(2px);
}

/* Cyberpunk Drawer */
html.cyberpunk :deep(.el-drawer) {
  background: rgba(18, 18, 30, 0.97) !important;
  border-left: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.5), -5px 0 20px rgba(0, 255, 255, 0.1);
}

/* Glassmorphism Drawer */
html.glassmorphism :deep(.el-drawer) {
  background: rgba(255, 255, 255, 0.92) !important;
  border-left: 1px solid rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px);
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.08);
}

/* Input 样式覆盖 */
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
</style>
