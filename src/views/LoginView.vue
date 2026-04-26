<script setup lang="ts">
/**
 * 登录页面
 * 应用启动时显示，登录或跳过后进入主应用
 */
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useSyncStore } from '@/stores/sync'

const router = useRouter()
const syncStore = useSyncStore()

// 本地存储键：是否跳过了登录
const SKIP_LOGIN_KEY = 'omo-skip-login'

// OAuth 登录中状态
const isLoggingIn = ref(false)
// 登录代次计数器：每次新登录/取消递增，用于丢弃过期请求的结果
const loginGeneration = ref(0)

// 窗口最大化状态
const isMaximized = ref(false)

// 动态导入 Tauri 窗口 API
let appWindow: Awaited<ReturnType<typeof import('@tauri-apps/api/window').getCurrentWindow>> | null = null

const initWindowApi = async () => {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  appWindow = getCurrentWindow()
  isMaximized.value = await appWindow.isMaximized()
}

const handleMinimize = async () => {
  if (!appWindow) return
  await appWindow.minimize()
}

const handleMaximize = async () => {
  if (!appWindow) return
  await appWindow.toggleMaximize()
  isMaximized.value = await appWindow.isMaximized()
}

const handleClose = async () => {
  if (!appWindow) return
  await appWindow.close()
}

onMounted(async () => {
  // 初始化窗口 API
  await initWindowApi()

  // 监听窗口大小变化
  if (appWindow) {
    appWindow.onResized(async () => {
      if (appWindow) {
        isMaximized.value = await appWindow.isMaximized()
      }
    })
  }

  // 已跳过登录则直接进入主页
  if (localStorage.getItem(SKIP_LOGIN_KEY) === 'true') {
    router.replace('/home')
  }
})

// OAuth 登录
async function handleOAuthLogin() {
  const thisGeneration = ++loginGeneration.value
  try {
    isLoggingIn.value = true
    await syncStore.loginWithOAuth()
    // 代次不匹配说明已被取消或被新请求取代
    if (loginGeneration.value !== thisGeneration) return
    ElMessage.success(`已登录为 ${syncStore.currentUser?.login}`)
    router.replace('/home')
  } catch (e) {
    if (loginGeneration.value === thisGeneration) {
      ElMessage.error(String(e))
    }
  } finally {
    if (loginGeneration.value === thisGeneration) {
      isLoggingIn.value = false
    }
  }
}

// 取消 OAuth 登录
async function handleCancelLogin() {
  // 递增代次，使正在等待的登录请求忽略结果
  loginGeneration.value++
  try {
    await syncStore.cancelOAuthLogin()
  } catch {
    // 取消失败静默处理
  } finally {
    isLoggingIn.value = false
  }
}

// 跳过登录
function handleSkip() {
  localStorage.setItem(SKIP_LOGIN_KEY, 'true')
  router.replace('/home')
}
</script>

<template>
  <div class="login-view">
    <!-- 简化标题栏：拖拽区域 + 窗口控制 -->
    <div class="login-title-bar" data-tauri-drag-region @dblclick="handleMaximize">
      <div class="login-title-bar-left" data-tauri-drag-region>
        <svg class="login-title-logo" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <circle cx="12" cy="12" r="4" fill="currentColor"/>
          <circle cx="12" cy="5" r="1.5" fill="currentColor"/>
          <circle cx="17" cy="15" r="1.5" fill="currentColor"/>
          <circle cx="7" cy="15" r="1.5" fill="currentColor"/>
        </svg>
        <span class="login-title-text">OMOSwitcher</span>
      </div>
      <div class="login-title-bar-center" data-tauri-drag-region></div>
      <div class="login-title-bar-right">
        <button class="login-window-btn" type="button" @click="handleMinimize" title="最小化">
          <svg viewBox="0 0 24 24" fill="none"><rect x="4" y="11" width="16" height="2" fill="currentColor"/></svg>
        </button>
        <button class="login-window-btn" type="button" @click="handleMaximize" :title="isMaximized ? '还原' : '最大化'">
          <svg v-if="!isMaximized" viewBox="0 0 24 24" fill="none"><rect x="4" y="4" width="16" height="16" stroke="currentColor" stroke-width="2"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none"><path d="M4 8H8V4H20V16H16V20H4V8Z" stroke="currentColor" stroke-width="2"/><rect x="8" y="8" width="12" height="12" stroke="currentColor" stroke-width="2"/></svg>
        </button>
        <button class="login-window-btn login-close-btn" type="button" @click="handleClose" title="关闭">
          <svg viewBox="0 0 24 24" fill="none"><path d="M6 6L18 18M6 18L18 6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
        </button>
      </div>
    </div>

    <!-- 登录卡片 -->
    <div class="login-card">
      <!-- Logo -->
      <div class="login-logo">
        <svg class="logo-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <circle cx="12" cy="12" r="4" fill="currentColor"/>
          <circle cx="12" cy="5" r="1.5" fill="currentColor"/>
          <circle cx="17" cy="15" r="1.5" fill="currentColor"/>
          <circle cx="7" cy="15" r="1.5" fill="currentColor"/>
        </svg>
        <h1 class="logo-text">OMOSwitcher</h1>
      </div>

      <p class="login-subtitle">OhMyOpenCode 模型配置管理工具</p>

      <!-- OAuth 登录中状态 -->
      <div v-if="isLoggingIn" class="login-loading">
        <div class="loading-spinner"></div>
        <p class="loading-text">{{ syncStore.lastError || '正在等待 GitHub 授权...' }}</p>
        <p v-if="!syncStore.lastError" class="loading-hint">请在浏览器中完成授权</p>
        <button class="cancel-login-btn" @click="handleCancelLogin">取消登录</button>
      </div>

      <!-- 正常登录 UI -->
      <template v-else>
        <!-- 登录提示 -->
        <div class="login-hint">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
            <path d="M12 16v-4"/>
            <path d="M12 8h.01"/>
          </svg>
          <span>登录 GitHub 后可同步预设配置到云端，在多台设备间共享</span>
        </div>

        <!-- GitHub 登录按钮 -->
        <button class="github-login-btn" @click="handleOAuthLogin">
          <svg viewBox="0 0 16 16" fill="currentColor" width="20" height="20">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
          </svg>
          使用 GitHub 登录
        </button>

        <!-- 跳过登录 -->
        <button class="skip-login-btn" @click="handleSkip">
          跳过，稍后再登录
        </button>
      </template>
    </div>

    <!-- 版本信息 -->
    <div class="login-footer">
      <span>OMOSwitcher v2.0</span>
    </div>
  </div>
</template>

<style scoped>
.login-view {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  position: relative;
  overflow: hidden;
}

/* ==================== 简化标题栏 ==================== */
.login-title-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 36px;
  display: flex;
  align-items: center;
  user-select: none;
  -webkit-user-select: none;
  z-index: 10;
}

.login-title-bar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  flex-shrink: 0;
}

.login-title-logo {
  width: 18px;
  height: 18px;
  color: var(--app-color-primary);
}

.login-title-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-tertiary);
}

.login-title-bar-center {
  flex: 1;
  height: 100%;
}

.login-title-bar-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  padding-right: 4px;
}

.login-window-btn {
  width: 46px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--app-text-tertiary);
  transition: all 0.2s ease;
}

.login-window-btn svg {
  width: 12px;
  height: 12px;
}

.login-window-btn:hover {
  color: var(--app-text-primary);
  background: var(--app-bg-hover);
}

.login-close-btn:hover {
  background: #ef4444;
  color: #ffffff;
}

/* ==================== 登录卡片 ==================== */
.login-card {
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 40px 32px;
  border-radius: 16px;
  border: 1px solid var(--app-border-default);
  background: var(--app-bg-card);
  box-shadow: var(--app-shadow-xl);
}

/* Logo */
.login-logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 40px;
  height: 40px;
  color: var(--app-color-primary);
}

.logo-text {
  font-size: 24px;
  font-weight: 700;
  color: var(--app-text-primary);
  margin: 0;
  letter-spacing: -0.5px;
}

.login-subtitle {
  font-size: 14px;
  color: var(--app-text-secondary);
  margin: -8px 0 0;
}

/* 登录提示 */
.login-hint {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 12px 14px;
  background: var(--app-bg-hover);
  border-radius: 8px;
  font-size: 12px;
  color: var(--app-text-secondary);
  line-height: 1.5;
  text-align: left;
}

.login-hint svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--app-color-primary);
}

/* ==================== OAuth 登录中状态 ==================== */
.login-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 20px 0;
  width: 100%;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--app-border-default);
  border-top-color: var(--app-color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--app-text-primary);
  margin: 0;
}

.loading-hint {
  font-size: 13px;
  color: var(--app-text-tertiary);
  margin: 0;
}

.cancel-login-btn {
  margin-top: 8px;
  padding: 8px 24px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: transparent;
  border: 1px solid var(--app-border-default);
  color: var(--app-text-secondary);
}

.cancel-login-btn:hover {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
  background: var(--app-bg-hover);
}

/* ==================== GitHub 登录按钮 ==================== */
.github-login-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 12px 20px;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  color: var(--app-text-primary);
}

.github-login-btn:hover {
  transform: translateY(-2px);
}

/* 跳过登录 */
.skip-login-btn {
  background: transparent;
  border: none;
  font-size: 13px;
  color: var(--app-text-tertiary);
  cursor: pointer;
  padding: 8px 16px;
  border-radius: 6px;
  transition: all 0.2s;
  margin-top: 4px;
}

.skip-login-btn:hover {
  color: var(--app-text-secondary);
  background: var(--app-bg-hover);
}

/* 底部版本 */
.login-footer {
  position: absolute;
  bottom: 24px;
  font-size: 12px;
  color: var(--app-text-tertiary);
}

/* ==================== Cyberpunk 主题 ==================== */
html.cyberpunk .login-view {
  background: linear-gradient(135deg, #0a0a0f 0%, #12121a 50%, #0d0d14 100%);
}

html.cyberpunk .login-title-bar {
  background: rgba(10, 10, 20, 0.8);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .login-title-logo {
  color: #00ffff;
  filter: drop-shadow(0 0 6px rgba(0, 255, 255, 0.5));
}

html.cyberpunk .login-title-text {
  color: rgba(0, 255, 255, 0.7);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .login-window-btn {
  color: rgba(180, 180, 210, 0.7);
}

html.cyberpunk .login-window-btn:hover {
  color: #ffffff;
  background: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .login-close-btn:hover {
  background: rgba(255, 51, 102, 0.2);
  color: #ff3366;
}

html.cyberpunk .login-card {
  background: rgba(18, 18, 30, 0.9);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: 
    0 0 40px rgba(0, 255, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.cyberpunk .logo-icon {
  color: #00ffff;
  filter: drop-shadow(0 0 10px rgba(0, 255, 255, 0.6));
}

html.cyberpunk .logo-text {
  color: #ffffff;
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .login-hint {
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .loading-spinner {
  border-color: rgba(0, 255, 255, 0.2);
  border-top-color: #00ffff;
}

html.cyberpunk .loading-text {
  color: #ffffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .cancel-login-btn {
  border-color: rgba(0, 255, 255, 0.3);
  color: rgba(0, 255, 255, 0.7);
}

html.cyberpunk .cancel-login-btn:hover {
  border-color: rgba(0, 255, 255, 0.6);
  color: #00ffff;
  background: rgba(0, 255, 255, 0.08);
}

html.cyberpunk .github-login-btn {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.12), rgba(0, 255, 255, 0.05));
  border-color: rgba(0, 255, 255, 0.4);
  color: #00ffff;
}

html.cyberpunk .github-login-btn:hover {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(0, 255, 255, 0.1));
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.4),
    0 0 40px rgba(0, 255, 255, 0.2);
}

/* ==================== Glassmorphism 主题 ==================== */
html.glassmorphism .login-view {
  background: linear-gradient(135deg, #e0e7ff 0%, #f0f9ff 50%, #fae8ff 100%);
}

html.glassmorphism .login-title-bar {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.5);
}

html.glassmorphism .login-title-logo {
  color: #2563eb;
}

html.glassmorphism .login-title-text {
  color: #64748b;
}

html.glassmorphism .login-window-btn {
  color: #94a3b8;
}

html.glassmorphism .login-window-btn:hover {
  color: #475569;
  background: rgba(0, 0, 0, 0.04);
}

html.glassmorphism .login-close-btn:hover {
  background: #ef4444;
  color: #ffffff;
}

html.glassmorphism .login-card {
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

html.glassmorphism .logo-icon {
  color: #2563eb;
}

html.glassmorphism .logo-text {
  color: #1e293b;
}

html.glassmorphism .login-hint {
  background: rgba(37, 99, 235, 0.06);
  border: 1px solid rgba(37, 99, 235, 0.15);
}

html.glassmorphism .login-hint svg {
  color: #2563eb;
}

html.glassmorphism .loading-spinner {
  border-color: rgba(37, 99, 235, 0.2);
  border-top-color: #2563eb;
}

html.glassmorphism .cancel-login-btn {
  border-color: rgba(37, 99, 235, 0.3);
  color: #2563eb;
}

html.glassmorphism .cancel-login-btn:hover {
  border-color: rgba(37, 99, 235, 0.5);
  background: rgba(37, 99, 235, 0.08);
}

html.glassmorphism .github-login-btn {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.12), rgba(37, 99, 235, 0.05));
  border-color: rgba(37, 99, 235, 0.3);
  color: #2563eb;
}

html.glassmorphism .github-login-btn:hover {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.2), rgba(37, 99, 235, 0.1));
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.2);
}
</style>
