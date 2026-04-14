<script setup lang="ts">
/**
 * 登录页面
 * 应用启动时显示，登录或跳过后进入主应用
 */
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useSyncStore } from '@/stores/sync'

const router = useRouter()
const syncStore = useSyncStore()

// 本地存储键：是否跳过了登录
const SKIP_LOGIN_KEY = 'omo-skip-login'

// 检查是否已跳过登录（纯本地判断，无网络请求）
onMounted(async () => {
  if (localStorage.getItem(SKIP_LOGIN_KEY) === 'true') {
    router.replace('/home')
  }
})

// OAuth 登录
async function handleOAuthLogin() {
  try {
    await syncStore.loginWithOAuth()
    ElMessage.success(`已登录为 ${syncStore.currentUser?.login}`)
    router.replace('/home')
  } catch (e) {
    ElMessage.error(String(e))
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
    <!-- 登录表单 -->
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

/* 登录卡片 */
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

/* GitHub 登录按钮 */
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
