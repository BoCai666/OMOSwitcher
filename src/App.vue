<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import { ElMessage } from 'element-plus'

const monitorStore = useMonitorStore()
const startupAttempted = ref(false)

// 应用启动时自动启动监控服务
onMounted(async () => {
  try {
    startupAttempted.value = true
    await monitorStore.startMonitor()
    console.log('[App] Monitor service started successfully')
  } catch (error) {
    // 启动失败不阻塞应用，用户可手动重试
    console.warn('[App] Failed to start monitor service:', error)
    ElMessage.warning({
      message: '监控服务启动失败，可在监控页面手动启动',
      duration: 5000,
    })
  }
})

// 应用关闭时停止监控服务
onUnmounted(async () => {
  try {
    await monitorStore.stopMonitor()
    console.log('[App] Monitor service stopped')
  } catch (error) {
    console.warn('[App] Failed to stop monitor service:', error)
  }
})
</script>

<template>
  <div class="app-container">
    <router-view />
  </div>
</template>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

#app {
  width: 100%;
  height: 100%;
}

.app-container {
  width: 100%;
  height: 100%;
}
</style>
