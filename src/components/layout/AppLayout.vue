<script setup lang="ts">
// 应用布局组件 - 使用 Element Plus Container 布局
import { computed } from 'vue'
import Sidebar from './Sidebar.vue'
import TitleBar from './TitleBar.vue'
import ParticleField from '@/components/ParticleField.vue'

// 定义组件属性
interface Props {
  title?: string // 页面标题
}

const props = defineProps<Props>()

// 检测当前是否为暗色/赛博朋克主题
const isDark = computed(() => {
  const html = document.documentElement
  return html.classList.contains('cyberpunk') || 
         (html.classList.contains('dark') && !html.classList.contains('glassmorphism'))
})
</script>

<template>
  <div class="app-layout-wrapper">
    <!-- 赛博朋克暗色主题特效 -->
    <template v-if="isDark">
      <!-- 粒子场背景 -->
      <ParticleField 
        :enabled="true" 
        :count="80" 
        color="#00ffff" 
      />
      <!-- 网格背景层 -->
      <div class="grid-background-layer"></div>
      <!-- 扫描线效果 -->
      <div class="scanlines-overlay"></div>
    </template>
    
    <!-- 顶部标题栏 - 32px 高度 -->
    <TitleBar />
    
    <el-container class="app-layout">
      <!-- 左侧侧边栏 - 固定 200px 宽度 -->
      <el-aside width="220px" class="app-aside">
        <Sidebar />
      </el-aside>
      
      <!-- 右侧主内容区域 -->
      <el-container class="app-main-container">
        <!-- 页面标题区 -->
        <el-header class="app-header">
          <h1 class="app-title">{{ props.title || 'OMOSwitcher' }}</h1>
        </el-header>
        
        <!-- 内容区域 -->
        <el-main class="app-main">
          <!-- 滚动容器分离出来，遮罩层能覆盖整个 app-main -->
          <div class="app-main-scroll">
            <div class="app-main-content">
              <slot />
            </div>
          </div>
        </el-main>
      </el-container>
    </el-container>
  </div>
</template>

<style scoped>
/* 应用布局容器 - 占满整个视口 */
.app-layout-wrapper {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  background-color: var(--app-bg-base);
  /* 主题切换过渡动画 */
  transition: background-color 0.5s var(--app-easing-smooth);
}

/* 网格背景层 - 赛博朋克主题 */
.grid-background-layer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
  background-image: 
    linear-gradient(rgba(0, 255, 255, 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 255, 255, 0.05) 1px, transparent 1px);
  background-size: 40px 40px;
  background-position: center center;
  opacity: 0.6;
}

/* 扫描线覆盖层 - 赛博朋克主题 */
.scanlines-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 255, 255, 0.03) 2px,
    rgba(0, 255, 255, 0.03) 4px
  );
  animation: scanlines-move 8s linear infinite;
}

@keyframes scanlines-move {
  0% { background-position: 0 0; }
  100% { background-position: 0 100%; }
}

/* ==================== 基础布局样式 ==================== */

.app-layout {
  flex: 1;
  height: calc(100vh - 36px);
  width: 100%;
  overflow: hidden;
  position: relative;
  z-index: 2;
}

/* 侧边栏样式 */
.app-aside {
  flex-shrink: 0;
  overflow: hidden;
  transition: all 0.5s var(--app-easing-smooth);
}

/* 主内容区域容器 */
.app-main-container {
  flex: 1;
  overflow: hidden;
  background-color: var(--app-bg-base);
  transition: background-color 0.5s var(--app-easing-smooth);
}

/* 顶部标题栏样式 */
.app-header {
  position: relative;
  display: flex;
  align-items: center;
  background-color: var(--app-bg-base);
  border-bottom: 1px solid var(--app-border-default);
  flex-shrink: 0;
  height: 60px;
  padding: 0 20px;
  transition: all 0.5s var(--app-easing-smooth);
}

.app-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
  transition: color 0.5s var(--app-easing-smooth);
}

/* 内容区域样式 - 不滚动，作为遮罩层的定位基准 */
.app-main {
  position: relative;
  background-color: var(--app-bg-base);
  overflow: visible;
  flex: 1;
  height: 0; /* flexbox 中让 flex: 1 生效 */
  transition: background-color 0.5s var(--app-easing-smooth);
}

/* 滚动容器 - 绝对定位铺满 app-main */
.app-main-scroll {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  overflow-y: auto;
  padding: 20px;
}

.app-main-content {
  min-height: 100%;
  transition: all 0.5s var(--app-easing-smooth);
}

/* ==================== 赛博朋克暗色主题 (html.cyberpunk) ==================== */

html.cyberpunk .app-layout-wrapper {
  background: linear-gradient(135deg, #0a0a0f 0%, #12121a 50%, #0d0d14 100%);
}

html.cyberpunk .app-aside {
  background: rgba(10, 10, 15, 0.85);
  backdrop-filter: blur(10px);
  border-right: 1px solid rgba(0, 212, 255, 0.2);
  box-shadow: 
    inset -1px 0 0 rgba(0, 212, 255, 0.1),
    5px 0 20px rgba(0, 0, 0, 0.5);
}

html.cyberpunk .app-header {
  background: rgba(10, 10, 15, 0.9);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(0, 212, 255, 0.3);
  box-shadow: 
    0 2px 10px rgba(0, 212, 255, 0.1),
    inset 0 -1px 0 rgba(0, 212, 255, 0.1);
}

html.cyberpunk .app-title {
  color: var(--app-color-neon-cyan);
  text-shadow: 
    0 0 10px rgba(0, 255, 255, 0.5),
    0 0 20px rgba(0, 255, 255, 0.3);
  font-weight: 700;
  letter-spacing: 1px;
}

html.cyberpunk .app-main {
  background: rgba(18, 18, 26, 0.7);
  backdrop-filter: blur(5px);
  position: relative;
}

html.cyberpunk .app-main-content {
  position: relative;
  background: transparent;
  border: none;
  border-radius: 0;
  padding: 24px;
  box-shadow: none;
}

/* 霓虹边框效果已禁用 - 背景改为铺满整个区域 */

/* ==================== 玻璃拟态明色主题 (html.glassmorphism) ==================== */

html.glassmorphism .app-layout-wrapper {
  background: linear-gradient(135deg, #e0e7ff 0%, #f0f9ff 50%, #fae8ff 100%);
}

html.glassmorphism .app-aside {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: var(--app-effect-glass-blur);
  border-right: 1px solid rgba(255, 255, 255, 0.5);
  box-shadow: 
    5px 0 20px rgba(0, 0, 0, 0.1),
    inset -1px 0 0 rgba(255, 255, 255, 0.8);
}

html.glassmorphism .app-header {
  background: rgba(255, 255, 255, 0.65);
  backdrop-filter: var(--app-effect-glass-blur);
  border-bottom: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 
    0 4px 20px rgba(0, 0, 0, 0.05),
    inset 0 -1px 0 rgba(255, 255, 255, 0.8);
}

html.glassmorphism .app-title {
  color: #4a5568;
  font-weight: 600;
  letter-spacing: 0.5px;
}

html.glassmorphism .app-main {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: var(--app-effect-glass-blur);
}

html.glassmorphism .app-main-content {
  position: relative;
  background: transparent;
  border: none;
  border-radius: 0;
  padding: 24px;
  box-shadow: none;
}

/* 柔光效果已禁用 - 背景改为铺满整个区域 */

/* ==================== 深色主题 (html.dark - 非赛博朋克) ==================== */

html.dark .app-aside {
  background: var(--app-bg-card);
  border-right: 1px solid var(--app-border-default);
}

html.dark .app-header {
  background: var(--app-bg-card);
  border-bottom: 1px solid var(--app-border-default);
}

/* ==================== 明色主题 (html.light - 非玻璃拟态/非暗色) ==================== */

html.light:not(.cyberpunk):not(.dark) .app-layout-wrapper {
  background-color: var(--app-bg-base);
  background-image: none;
}

html.light:not(.cyberpunk):not(.dark) .app-aside {
  background: var(--app-bg-card);
  border-right: 1px solid var(--app-border-default);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

html.light:not(.cyberpunk):not(.dark) .app-header {
  background: var(--app-bg-card);
  border-bottom: 1px solid var(--app-border-default);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

html.light:not(.cyberpunk):not(.dark) .app-main {
  background-color: var(--app-bg-base);
  /* 使用 backdrop-filter 创建 containing block，限制 Drawer 遮罩层范围 */
  /* blur(0px) 没有实际模糊效果，但能创建与暗色模式相同的 containing block 行为 */
  backdrop-filter: blur(0px);
  -webkit-backdrop-filter: blur(0px);
}

html.light:not(.cyberpunk):not(.dark) .app-main-content {
  background: transparent;
}

/* 明色模式 - 创建独立的层叠上下文，使侧边栏不受遮罩层影响 */
html.light:not(.cyberpunk):not(.dark) .app-layout-wrapper {
  isolation: isolate;
  position: relative;
}

html.light:not(.cyberpunk):not(.dark) .app-aside {
  position: relative;
  z-index: 2100;
}

/* ==================== 主题切换动画优化 ==================== */

/* 所有需要过渡的元素 */
.app-layout-wrapper,
.app-aside,
.app-header,
.app-title,
.app-main,
.app-main-content {
  will-change: background-color, border-color, box-shadow, color;
}

/* 平滑滚动 */
.app-main {
  scroll-behavior: smooth;
}

/* 响应式优化 */
@media (prefers-reduced-motion: reduce) {
  .app-layout-wrapper,
  .app-aside,
  .app-header,
  .app-title,
  .app-main,
  .app-main-content,
  .scanlines-overlay {
    animation: none;
    transition: none;
  }
}
</style>
