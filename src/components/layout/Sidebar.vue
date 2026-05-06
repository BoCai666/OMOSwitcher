<script setup lang="ts">
// 侧边栏导航组件 - 霓虹科技风格
// 使用完全自定义样式，不使用 Element Plus 默认样式
import { useRoute, useRouter } from 'vue-router'
import { computed, ref, onMounted } from 'vue'
import { Sunny, Moon } from '@element-plus/icons-vue'
import { useTheme } from '@/composables/useTheme'
import { getVersion } from '@tauri-apps/api/app'

const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()

// 应用版本号
const appVersion = ref('')

// 控制入场动画是否已播放完成
const animationReady = ref(false)

// 组件挂载后标记动画已准备好，禁用后续的入场动画
onMounted(() => {
  // 获取应用版本号
  getVersion().then(v => { appVersion.value = v }).catch(() => {})
  // 等待入场动画完成后禁用动画
  setTimeout(() => {
    animationReady.value = true
  }, 300) // 最长动画延迟是 0.25s，加一点缓冲
})

// 导航菜单项配置
const menuItems = [
  { index: '/home', title: '主页', icon: 'House' },
  { index: '/config', title: '模型配置', icon: 'Setting' },
  { index: '/models', title: '模型管理', icon: 'Collection' },
  { index: '/presets', title: '预设管理', icon: 'Folder' },
  { index: '/quota', title: '额度管理', icon: 'Wallet' },
  { index: '/monitor', title: '监控', icon: 'Monitor' }
]

// 计算当前激活的菜单项
const activeIndex = computed(() => route.path)

// 处理菜单选择事件
const handleSelect = (index: string) => {
  router.push(index)
}
</script>

<template>
  <div class="sidebar">
    <!-- Logo 区域 -->
    <div class="sidebar-header">
      <div class="logo">
        <span class="logo-icon">⚡</span>
        <span class="logo-text">OMOSwitcher</span>
      </div>
    </div>

    <!-- 菜单区域 -->
    <nav class="sidebar-nav">
      <ul class="menu-list" :class="{ 'animation-done': animationReady }">
        <li
          v-for="item in menuItems"
          :key="item.index"
          class="menu-item"
          :class="{ 'is-active': activeIndex === item.index }"
          @click="handleSelect(item.index)"
        >
          <!-- 激活指示器 -->
          <div class="active-indicator"></div>
          
          <!-- 菜单内容 -->
          <div class="menu-content">
            <el-icon class="menu-icon" :size="18">
              <component :is="item.icon" />
            </el-icon>
            <span class="menu-title">{{ item.title }}</span>
          </div>
        </li>
      </ul>
    </nav>

    <!-- 底部状态区域 -->
    <div class="sidebar-footer">
      <div class="footer-row">
        <div class="theme-toggle-placeholder" @click="toggleTheme">
          <el-icon :size="16">
            <Sunny v-if="isDark" />
            <Moon v-else />
          </el-icon>
          <span>{{ isDark ? '切换明色' : '切换暗色' }}</span>
        </div>
        <span v-if="appVersion" class="version-text">v{{ appVersion }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ==================== 基础样式变量 ==================== */
.sidebar {
  --sidebar-width: var(--app-sidebar-width);
  --sidebar-transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

/* ==================== 侧边栏容器 - 基础结构 ==================== */
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: var(--sidebar-width);
  transition: var(--sidebar-transition);
}

/* ==================== Logo 区域 ==================== */
.sidebar-header {
  padding: var(--app-spacing-5) var(--app-spacing-4);
}

.logo {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-size: 18px;
  font-weight: 600;
  transition: var(--sidebar-transition);
}

.logo-icon {
  font-size: 24px;
  animation: pulse 2s ease-in-out infinite;
  transition: var(--sidebar-transition);
}

.logo-text {
  transition: var(--sidebar-transition);
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

/* ==================== 导航菜单 ==================== */
.sidebar-nav {
  flex: 1;
  padding: var(--app-spacing-3);
  overflow-y: auto;
}

.menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
}

/* ==================== 菜单项 - 基础结构 ==================== */
.menu-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: var(--app-spacing-3) var(--app-spacing-4);
  border-radius: var(--app-radius-md);
  cursor: pointer;
  overflow: hidden;
  transition: var(--sidebar-transition);
}

/* 激活指示器 - 左侧竖线 */
.active-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 4px;
  height: 60%;
  border-radius: 0 2px 2px 0;
  transition: var(--sidebar-transition);
}

/* 菜单内容 */
.menu-content {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
  flex: 1;
  z-index: 1;
}

.menu-icon {
  transition: var(--sidebar-transition);
}

.menu-title {
  font-size: 14px;
  font-weight: 500;
  transition: var(--sidebar-transition);
}

/* ==================== 底部区域 ==================== */
.sidebar-footer {
  padding: var(--app-spacing-2) var(--app-spacing-4);
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
  border-top: 1px solid var(--app-border-default);
}

/* 明色主题下增强分隔线可见性 */
html.light .sidebar .sidebar-footer {
  border-top-color: var(--app-border-hover);
}

.footer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.theme-toggle-placeholder {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-size: 12px;
  cursor: pointer;
  transition: var(--sidebar-transition);
}

.version-text {
  font-size: 11px;
  color: var(--app-text-tertiary);
  transition: var(--sidebar-transition);
}

/* ==================== 滚动条样式 ==================== */
.sidebar-nav::-webkit-scrollbar {
  width: 4px;
}

.sidebar-nav::-webkit-scrollbar-track {
  background: transparent;
}

.sidebar-nav::-webkit-scrollbar-thumb {
  border-radius: 2px;
  transition: var(--sidebar-transition);
}

/* 菜单项进入动画 - 只在首次加载时播放 */
@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* 仅在动画未完成时应用入场动画 */
.menu-list:not(.animation-done) .menu-item {
  animation: slideIn 0.4s cubic-bezier(0.4, 0, 0.2, 1) backwards;
}

.menu-list:not(.animation-done) .menu-item:nth-child(1) { animation-delay: 0.05s; }
.menu-list:not(.animation-done) .menu-item:nth-child(2) { animation-delay: 0.1s; }
.menu-list:not(.animation-done) .menu-item:nth-child(3) { animation-delay: 0.15s; }
.menu-list:not(.animation-done) .menu-item:nth-child(4) { animation-delay: 0.2s; }
.menu-list:not(.animation-done) .menu-item:nth-child(5) { animation-delay: 0.25s; }
.menu-list:not(.animation-done) .menu-item:nth-child(6) { animation-delay: 0.3s; }
</style>

<style>
/* 
 * 主题样式 - 放在非 scoped 块中
 * 注意：Vue scoped CSS 编译器在处理 :global(html.xxx) .class 格式时存在 bug
 * 会导致样式被错误应用到 html 元素本身，因此移到这里使用普通选择器
 */

/* ==================== 赛博朋克主题 (html.cyberpunk) ==================== */
/* 霓虹边框 + 发光导航项 */

html.cyberpunk .sidebar {
  background: var(--app-bg-card);
  border-right: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 
    0 0 30px rgba(0, 255, 255, 0.1),
    inset -10px 0 30px rgba(0, 255, 255, 0.05);
}

html.cyberpunk .sidebar-header {
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk .logo {
  color: var(--app-text-primary);
}

html.cyberpunk .logo-icon {
  filter: drop-shadow(0 0 8px var(--app-accent-primary));
}

html.cyberpunk .logo-text {
  background: linear-gradient(135deg, var(--app-text-primary) 0%, var(--app-accent-primary) 50%, var(--app-accent-secondary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  text-shadow: 0 0 30px rgba(0, 255, 255, 0.5);
}

/* 赛博朋克菜单项 - 霓虹发光效果 */
html.cyberpunk .sidebar .menu-item {
  background: transparent;
  border: 1px solid transparent;
}

html.cyberpunk .sidebar .menu-item:hover {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.3);
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.1),
    inset 0 0 20px rgba(0, 255, 255, 0.05);
}

html.cyberpunk .sidebar .menu-item:hover .menu-icon {
  color: var(--app-accent-primary);
  filter: drop-shadow(0 0 8px var(--app-accent-primary));
}

html.cyberpunk .sidebar .menu-item:hover .menu-title {
  color: var(--app-text-primary);
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

/* 赛博朋克激活状态 - 强烈霓虹效果 */
html.cyberpunk .sidebar .menu-item.is-active {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.1) 0%, rgba(255, 0, 255, 0.05) 100%);
  border: 1px solid var(--app-accent-primary);
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.3),
    0 0 40px rgba(0, 255, 255, 0.1),
    inset 0 0 20px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .sidebar .menu-item.is-active .active-indicator {
  background: var(--app-accent-primary);
  transform: translateY(-50%) scaleY(1);
  box-shadow: 
    0 0 10px var(--app-accent-primary),
    0 0 20px var(--app-accent-primary),
    0 0 30px var(--app-accent-primary);
}

html.cyberpunk .sidebar .menu-item.is-active .menu-icon {
  color: var(--app-accent-primary);
  filter: drop-shadow(0 0 10px var(--app-accent-primary));
}

html.cyberpunk .sidebar .menu-item.is-active .menu-title {
  color: var(--app-text-primary);
  font-weight: 600;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

/* 赛博朋克指示器默认状态 */
html.cyberpunk .sidebar .active-indicator {
  background: var(--app-accent-primary);
  box-shadow: 0 0 10px var(--app-accent-primary);
}

html.cyberpunk .sidebar .menu-icon {
  color: var(--app-text-secondary);
}

html.cyberpunk .sidebar .menu-title {
  color: var(--app-text-secondary);
}

/* 赛博朋克底部区域 */
html.cyberpunk .sidebar-footer {
  border-top: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk .theme-toggle-placeholder {
  color: var(--app-text-tertiary);
}

html.cyberpunk .theme-toggle-placeholder:hover {
  color: var(--app-accent-primary);
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

/* 赛博朋克滚动条 */
html.cyberpunk .sidebar-nav::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.3);
}

html.cyberpunk .sidebar-nav::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.5);
}

/* 赛博朋克特殊动画效果 */
@keyframes neonFlicker {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.95; }
}

html.cyberpunk .sidebar .menu-item.is-active {
  animation: neonFlicker 3s ease-in-out infinite;
}

/* ==================== 玻璃拟态主题 (html.glassmorphism) ==================== */
/* 玻璃背景 + 柔和阴影 */

html.glassmorphism .sidebar {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-right: 1px solid rgba(255, 255, 255, 0.5);
  box-shadow: 
    4px 0 24px rgba(0, 0, 0, 0.08),
    inset -1px 0 0 rgba(255, 255, 255, 0.8);
}

html.glassmorphism .sidebar-header {
  border-bottom: 1px solid rgba(255, 255, 255, 0.6);
}

html.glassmorphism .logo {
  color: var(--app-text-primary);
}

html.glassmorphism .logo-icon {
  filter: drop-shadow(0 2px 4px rgba(37, 99, 235, 0.3));
}

html.glassmorphism .logo-text {
  background: linear-gradient(135deg, #1e293b 0%, #2563eb 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 玻璃拟态菜单项 - 柔和效果 */
html.glassmorphism .sidebar .menu-item {
  background: rgba(255, 255, 255, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: var(--app-radius-lg);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

html.glassmorphism .sidebar .menu-item:hover {
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(37, 99, 235, 0.3);
  box-shadow: 
    0 4px 16px rgba(0, 0, 0, 0.08),
    0 0 0 1px rgba(37, 99, 235, 0.1);
  transform: translateX(4px);
}

html.glassmorphism .sidebar .menu-item:hover .menu-icon {
  color: var(--app-color-primary);
}

html.glassmorphism .sidebar .menu-item:hover .menu-title {
  color: var(--app-text-primary);
}

/* 玻璃拟态激活状态 - 柔和高亮 */
html.glassmorphism .sidebar .menu-item.is-active {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: 
    0 4px 20px rgba(37, 99, 235, 0.15),
    0 0 0 1px rgba(37, 99, 235, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

html.glassmorphism .sidebar .menu-item.is-active .active-indicator {
  background: var(--app-color-primary);
  transform: translateY(-50%) scaleY(1);
  box-shadow: 0 0 12px rgba(37, 99, 235, 0.4);
}

html.glassmorphism .sidebar .menu-item.is-active .menu-icon {
  color: var(--app-color-primary);
}

html.glassmorphism .sidebar .menu-item.is-active .menu-title {
  color: var(--app-text-primary);
  font-weight: 600;
}

/* 玻璃拟态指示器默认状态 */
html.glassmorphism .sidebar .active-indicator {
  background: var(--app-color-primary);
  box-shadow: 0 0 8px rgba(37, 99, 235, 0.3);
}

html.glassmorphism .sidebar .menu-icon {
  color: var(--app-text-secondary);
}

html.glassmorphism .sidebar .menu-title {
  color: var(--app-text-secondary);
}

/* 玻璃拟态底部区域 */
html.glassmorphism .sidebar-footer {
  border-top: 1px solid rgba(255, 255, 255, 0.6);
}

html.glassmorphism .theme-toggle-placeholder {
  color: var(--app-text-tertiary);
}

html.glassmorphism .theme-toggle-placeholder:hover {
  color: var(--app-color-primary);
}

/* 玻璃拟态滚动条 */
html.glassmorphism .sidebar-nav::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.4);
}

html.glassmorphism .sidebar-nav::-webkit-scrollbar-thumb:hover {
  background: rgba(100, 116, 139, 0.5);
}

/* 玻璃拟态微光效果 */
@keyframes glassShimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

html.glassmorphism .sidebar .menu-item.is-active::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(255, 255, 255, 0.3) 50%,
    transparent 100%
  );
  background-size: 200% 100%;
  animation: glassShimmer 3s ease-in-out infinite;
  border-radius: inherit;
  pointer-events: none;
}

/* ==================== 主题切换过渡动画 ==================== */
/* 全局主题切换时的平滑过渡 */

html.theme-transitioning .sidebar,
html.theme-transitioning .sidebar-header,
html.theme-transitioning .sidebar-footer,
html.theme-transitioning .menu-item,
html.theme-transitioning .active-indicator,
html.theme-transitioning .menu-icon,
html.theme-transitioning .menu-title,
html.theme-transitioning .theme-toggle-placeholder,
html.theme-transitioning .version-text,
html.theme-transitioning .logo-text,
html.theme-transitioning .logo-icon {
  transition: all 0.6s cubic-bezier(0.4, 0, 0.2, 1) !important;
}
</style>
