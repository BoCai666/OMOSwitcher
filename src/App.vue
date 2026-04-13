<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMonitorStore } from '@/stores/monitor'
import { listModels } from '@/services/modelStore'
import { listPresets } from '@/services/presetStore'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import AppLayout from '@/components/layout/AppLayout.vue'
import SyncConflictDialog from '@/components/SyncConflictDialog.vue'

const route = useRoute()
const monitorStore = useMonitorStore()
const startupAttempted = ref(false)

// 从路由 meta 获取页面标题
const pageTitle = computed(() => (route.meta.title as string) || 'OMOSwitcher')

// 应用启动时自动启动监控服务和预加载数据
onMounted(async () => {
  // 预热 Tauri IPC 调用（减少首次调用延迟）
  invoke('get_monitor_ports_config').catch(() => {})

  // 并行执行所有初始化操作（后台执行，不阻塞 UI 显示）
  Promise.allSettled([
    // 启动监控服务
    (async () => {
      try {
        startupAttempted.value = true
        await monitorStore.startMonitor()
        console.log('[App] Monitor service started successfully')
      } catch (error) {
        console.warn('[App] Failed to start monitor service:', error)
        ElMessage.warning({
          message: '监控服务启动失败，可在监控页面手动启动',
          duration: 5000,
        })
      }
    })(),
    // 预加载模型列表
    listModels().then(() => {
      console.log('[App] Models preloaded')
    }).catch((e) => {
      console.warn('[App] Failed to preload models:', e)
    }),
    // 预加载预设列表
    listPresets().then(() => {
      console.log('[App] Presets preloaded')
    }).catch((e) => {
      console.warn('[App] Failed to preload presets:', e)
    }),
  ])
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
  <AppLayout :title="pageTitle">
    <router-view v-slot="{ Component }">
      <transition name="page-fade" mode="out-in" :duration="100">
        <component :is="Component" />
      </transition>
    </router-view>
  </AppLayout>

  <!-- 全局同步冲突对话框 -->
  <SyncConflictDialog />
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

/* 页面切换过渡动画（快速版本） */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.08s ease;
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
}
</style>
