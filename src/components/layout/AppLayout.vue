<script setup lang="ts">
// 应用布局组件 - 使用 Element Plus Container 布局
import Sidebar from './Sidebar.vue'
import SaveIndicator from '@/components/SaveIndicator.vue'

// 定义组件属性
interface Props {
  title?: string // 页面标题
}

defineProps<Props>()
</script>

<template>
  <el-container class="app-layout">
    <!-- 左侧侧边栏 - 固定 200px 宽度 -->
    <el-aside width="200px" class="app-aside">
      <Sidebar />
    </el-aside>
    
    <!-- 右侧主内容区域 -->
    <el-container class="app-main-container">
      <!-- 顶部标题栏 -->
      <el-header class="app-header">
        <h1 class="app-title">{{ title || 'OMOSwitcher' }}</h1>
        <SaveIndicator class="header-save-indicator" />
      </el-header>
      
      <!-- 内容区域 -->
      <el-main class="app-main">
        <slot />
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped>
/* 应用布局容器 - 占满整个视口 */
.app-layout {
  height: 100vh;
  width: 100%;
}

/* 侧边栏样式 */
.app-aside {
  background-color: #304156;
  flex-shrink: 0;
}

/* 主内容区域容器 */
.app-main-container {
  flex: 1;
  overflow: hidden;
}

/* 顶部标题栏样式 */
.app-header {
  position: relative;
  display: flex;
  align-items: center;
  background-color: #fff;
  border-bottom: 1px solid #dcdfe6;
  flex-shrink: 0;
  height: 60px;
  padding: 0 20px;
}

.app-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.header-save-indicator {
  position: absolute;
  right: 20px;
}

/* 内容区域样式 */
.app-main {
  background-color: #f0f2f5;
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}
</style>
