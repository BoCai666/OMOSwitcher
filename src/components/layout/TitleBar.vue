<script setup lang="ts">
/**
 * 自定义标题栏组件
 * 提供窗口控制、拖拽移动、双击最大化/还原功能
 * 支持主题特定视觉效果和用户头像入口
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { useSyncStore } from '@/stores/sync'
import UserDrawer from '@/components/UserDrawer.vue'

// 窗口最大化状态
const isMaximized = ref(false)

// 主题（仅用于 class 绑定，切换功能在 Sidebar）
const { isCyberpunk, isGlassmorphism } = useTheme()

// 同步状态
const syncStore = useSyncStore()
const showUserDrawer = ref(false)

// 动态导入 Tauri API（避免 SSR 问题）
let appWindow: Awaited<ReturnType<typeof import('@tauri-apps/api/window').getCurrentWindow>> | null = null

// 初始化窗口 API
const initWindowApi = async () => {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  appWindow = getCurrentWindow()
  // 初始化时检查窗口状态
  isMaximized.value = await appWindow.isMaximized()
}

// 窗口控制函数
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

// 双击标题栏最大化/还原
const handleDoubleClick = async () => {
  await handleMaximize()
}

// 监听窗口状态变化
const unlistenResize = ref<(() => void) | null>(null)

onMounted(async () => {
  await initWindowApi()
  
  // 监听窗口大小变化事件
  if (appWindow) {
    unlistenResize.value = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow!.isMaximized()
    })
  }
})

onUnmounted(() => {
  // 清理事件监听
  if (unlistenResize.value) {
    unlistenResize.value()
  }
})
</script>

<template>
  <div 
    class="title-bar"
    :class="{ 'cyberpunk-mode': isCyberpunk, 'glassmorphism-mode': isGlassmorphism }"
    data-tauri-drag-region
    @dblclick="handleDoubleClick"
  >
    <!-- 左侧：Logo 区域 -->
    <div class="title-bar-left">
      <div class="logo">
        <svg class="logo-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <circle cx="12" cy="12" r="4" fill="currentColor"/>
          <circle cx="12" cy="5" r="1.5" fill="currentColor"/>
          <circle cx="17" cy="15" r="1.5" fill="currentColor"/>
          <circle cx="7" cy="15" r="1.5" fill="currentColor"/>
        </svg>
        <span class="logo-text" :class="{ 'glitch-text': isCyberpunk }" data-text="OMO">OMO</span>
      </div>
    </div>

    <!-- 中间：拖拽区域（占位） -->
    <div class="title-bar-center" data-tauri-drag-region></div>

    <!-- 右侧：用户头像 + 窗口控制按钮 -->
    <div class="title-bar-right">
      <!-- 用户头像按钮 -->
      <button 
        class="user-avatar-btn"
        type="button"
        :title="syncStore.isLoggedIn ? '用户设置' : '登录'"
        @click.stop="showUserDrawer = true"
      >
        <!-- 已登录：显示 GitHub 头像 -->
        <img 
          v-if="syncStore.isLoggedIn && syncStore.currentUser?.avatar_url" 
          :src="syncStore.currentUser.avatar_url" 
          :alt="syncStore.currentUser.login"
          class="user-avatar-img"
        />
        <!-- 未登录：空心人形图标占位 -->
        <span v-else class="user-avatar-placeholder">
          <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
            <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </span>
      </button>

      <!-- 最小化按钮 -->
      <button 
        class="window-btn minimize-btn"
        type="button"
        @click="handleMinimize"
        title="最小化"
      >
        <svg class="window-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="4" y="11" width="16" height="2" fill="currentColor"/>
        </svg>
      </button>

      <!-- 最大化/还原按钮 -->
      <button 
        class="window-btn maximize-btn"
        type="button"
        @click="handleMaximize"
        :title="isMaximized ? '还原' : '最大化'"
      >
        <!-- 最大化图标 -->
        <svg 
          v-if="!isMaximized"
          class="window-icon" 
          viewBox="0 0 24 24" 
          fill="none" 
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect x="4" y="4" width="16" height="16" stroke="currentColor" stroke-width="2"/>
        </svg>
        <!-- 还原图标 -->
        <svg 
          v-else
          class="window-icon" 
          viewBox="0 0 24 24" 
          fill="none" 
          xmlns="http://www.w3.org/2000/svg"
        >
          <path d="M4 8H8V4H20V16H16V20H4V8Z" stroke="currentColor" stroke-width="2"/>
          <rect x="8" y="8" width="12" height="12" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>

      <!-- 关闭按钮 -->
      <button 
        class="window-btn close-btn"
        type="button"
        @click="handleClose"
        title="关闭"
      >
        <svg class="window-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M6 6L18 18M6 18L18 6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- 用户抽屉 -->
    <UserDrawer v-model="showUserDrawer" />
  </div>
</template>

<style scoped>
/* ==================== 基础样式 ==================== */
.title-bar {
  display: flex;
  align-items: center;
  height: 36px;
  user-select: none;
  -webkit-user-select: none;
  position: relative;
  overflow: hidden;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 基础背景（无主题时） */
.title-bar::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

/* ==================== Cyberpunk 主题 - 霓虹故障风格 ==================== */
html.cyberpunk .title-bar::before {
  background: linear-gradient(
    180deg,
    #0a0a1a 0%,
    #12121f 50%,
    #0a0a1a 100%
  );
  border-bottom: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.2),
    0 0 40px rgba(0, 255, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.cyberpunk .title-bar::after {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  background: 
    linear-gradient(
      90deg,
      transparent 0%,
      rgba(0, 255, 255, 0.03) 50%,
      transparent 100%
    );
  animation: cyberpunk-scan 4s linear infinite;
  pointer-events: none;
}

@keyframes cyberpunk-scan {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

/* Cyberpunk Logo 样式 */
html.cyberpunk .logo-icon {
  width: 20px;
  height: 20px;
  color: #00ffff;
  filter: drop-shadow(0 0 8px rgba(0, 255, 255, 0.8));
  animation: logo-pulse 2s ease-in-out infinite;
}

@keyframes logo-pulse {
  0%, 100% { filter: drop-shadow(0 0 8px rgba(0, 255, 255, 0.8)); }
  50% { filter: drop-shadow(0 0 15px rgba(0, 255, 255, 1)); }
}

/* 故障效果文本 */
.glitch-text {
  position: relative;
  font-size: 15px;
  font-weight: 700;
  color: #e0e0ff;
  letter-spacing: 2px;
  text-transform: uppercase;
}

.glitch-text::before,
.glitch-text::after {
  content: attr(data-text);
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0.8;
}

.glitch-text::before {
  color: #ff00ff;
  animation: glitch-1 2s infinite linear alternate-reverse;
  clip-path: polygon(0 0, 100% 0, 100% 35%, 0 35%);
}

.glitch-text::after {
  color: #00ffff;
  animation: glitch-2 2s infinite linear alternate-reverse;
  clip-path: polygon(0 65%, 100% 65%, 100% 100%, 0 100%);
}

@keyframes glitch-1 {
  0%, 90%, 100% { transform: translate(0); }
  92% { transform: translate(-2px, 1px); }
  94% { transform: translate(2px, -1px); }
  96% { transform: translate(-1px, 2px); }
  98% { transform: translate(1px, -2px); }
}

@keyframes glitch-2 {
  0%, 90%, 100% { transform: translate(0); }
  91% { transform: translate(2px, -1px); }
  93% { transform: translate(-2px, 1px); }
  95% { transform: translate(1px, 2px); }
  97% { transform: translate(-1px, -2px); }
}

/* Cyberpunk 用户头像按钮样式 */
html.cyberpunk .user-avatar-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 5px;
  transition: all 0.3s ease;
  position: relative;
}

html.cyberpunk .user-avatar-btn:hover {
  background: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .user-avatar-img {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  object-fit: cover;
  border: 1.5px solid rgba(0, 255, 255, 0.6);
  box-shadow: 0 0 8px rgba(0, 255, 255, 0.5);
  transition: all 0.3s ease;
}

html.cyberpunk .user-avatar-btn:hover .user-avatar-img {
  border-color: rgba(0, 255, 255, 0.9);
  box-shadow: 0 0 16px rgba(0, 255, 255, 0.8);
}

html.cyberpunk .user-avatar-placeholder {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1.5px dashed rgba(0, 255, 255, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(0, 255, 255, 0.5);
  transition: all 0.3s ease;
}

html.cyberpunk .user-avatar-placeholder svg {
  width: 14px;
  height: 14px;
}

html.cyberpunk .user-avatar-btn:hover .user-avatar-placeholder {
  border-color: rgba(0, 255, 255, 0.7);
  color: rgba(0, 255, 255, 0.8);
  box-shadow: 0 0 12px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .window-btn {
  width: 46px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: #b0b0d0;
  transition: all 0.3s ease;
  position: relative;
}

html.cyberpunk .window-btn::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 50%;
  width: 0;
  height: 2px;
  background: #00ffff;
  transition: all 0.3s ease;
  transform: translateX(-50%);
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.8);
}

html.cyberpunk .window-btn:hover {
  color: #ffffff;
  background: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .window-btn:hover::after {
  width: 60%;
}

html.cyberpunk .close-btn:hover {
  background: rgba(255, 51, 102, 0.2);
  color: #ff3366;
}

html.cyberpunk .close-btn:hover::after {
  background: #ff3366;
  box-shadow: 0 0 10px rgba(255, 51, 102, 0.8);
}

/* ==================== Glassmorphism 主题 - 专业玻璃风格 ==================== */
html.glassmorphism .title-bar::before {
  background: linear-gradient(
    180deg,
    rgba(255, 255, 255, 0.95) 0%,
    rgba(255, 255, 255, 0.85) 50%,
    rgba(248, 250, 252, 0.9) 100%
  );
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 
    0 1px 3px rgba(0, 0, 0, 0.05),
    0 4px 20px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

html.glassmorphism .logo-icon {
  width: 18px;
  height: 18px;
  color: #2563eb;
  transition: all 0.3s ease;
}

html.glassmorphism .logo-text {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  letter-spacing: 1px;
  transition: all 0.3s ease;
}

html.glassmorphism .title-bar-left:hover .logo-icon {
  color: #1d4ed8;
  transform: scale(1.05);
}

/* Glassmorphism 用户头像按钮样式 */
html.glassmorphism .user-avatar-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 5px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 50%;
  margin: 0 2px;
}

html.glassmorphism .user-avatar-btn:hover {
  background: rgba(37, 99, 235, 0.08);
}

html.glassmorphism .user-avatar-img {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  object-fit: cover;
  border: 1.5px solid rgba(37, 99, 235, 0.3);
  transition: all 0.3s ease;
}

html.glassmorphism .user-avatar-btn:hover .user-avatar-img {
  border-color: rgba(37, 99, 235, 0.6);
  box-shadow: 0 0 8px rgba(37, 99, 235, 0.2);
}

html.glassmorphism .user-avatar-placeholder {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1.5px dashed rgba(100, 116, 139, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(100, 116, 139, 0.5);
  transition: all 0.3s ease;
}

html.glassmorphism .user-avatar-placeholder svg {
  width: 14px;
  height: 14px;
}

html.glassmorphism .user-avatar-btn:hover .user-avatar-placeholder {
  border-color: rgba(37, 99, 235, 0.5);
  color: rgba(37, 99, 235, 0.7);
}

html.glassmorphism .window-btn {
  width: 46px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: #64748b;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

html.glassmorphism .window-btn:hover {
  color: #475569;
  background: rgba(0, 0, 0, 0.04);
}

html.glassmorphism .close-btn:hover {
  background: #ef4444;
  color: #ffffff;
}

/* ==================== 公共布局样式 ==================== */
.title-bar-left {
  display: flex;
  align-items: center;
  padding: 0 12px;
  flex-shrink: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-bar-center {
  flex: 1;
  height: 100%;
}

.title-bar-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  padding-right: 4px;
}

.window-icon {
  width: 12px;
  height: 12px;
}

/* 用户头像按钮基础样式 */
.user-avatar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 5px;
  transition: all 0.3s ease;
  outline: none;
}

.user-avatar-img {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  object-fit: cover;
  transition: all 0.3s ease;
}

.user-avatar-placeholder {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1.5px dashed rgba(128, 128, 128, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(128, 128, 128, 0.5);
  transition: all 0.3s ease;
}

.user-avatar-placeholder svg {
  width: 14px;
  height: 14px;
}

/* ==================== 主题切换过渡动画 ==================== */
html:not(.reduce-motion) .title-bar {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

html:not(.reduce-motion) .logo-icon,
html:not(.reduce-motion) .logo-text,
html:not(.reduce-motion) .window-btn,
html:not(.reduce-motion) .user-avatar-btn {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 主题切换时的闪烁效果 */
html:not(.reduce-motion) .title-bar::before {
  transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

/* ==================== 减少动画偏好 ==================== */
html.reduce-motion .glitch-text::before,
html.reduce-motion .glitch-text::after {
  animation: none;
}

html.reduce-motion .logo-icon {
  animation: none;
}

html.reduce-motion .title-bar::after {
  animation: none;
}
</style>
