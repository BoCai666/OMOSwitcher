<script setup lang="ts">
// 保存状态指示器组件
import { computed } from 'vue'
import { useConfigStore, type SaveStatus } from '@/stores/config'
import { ElIcon } from 'element-plus'
import { Loading, CircleCheck, Warning } from '@element-plus/icons-vue'

const configStore = useConfigStore()

// 状态文本映射
const statusText: Record<SaveStatus, string> = {
  idle: '',
  saving: '保存中...',
  saved: '已保存',
  error: '保存失败'
}

// 状态图标映射
const statusIcon = computed(() => {
  switch (configStore.saveStatus) {
    case 'saving':
      return Loading
    case 'saved':
      return CircleCheck
    case 'error':
      return Warning
    default:
      return null
  }
})

// 状态颜色
const statusColor = computed(() => {
  switch (configStore.saveStatus) {
    case 'saving':
      return '#409eff'
    case 'saved':
      return '#67c23a'
    case 'error':
      return '#f56c6c'
    default:
      return 'transparent'
  }
})

// 是否显示
const visible = computed(() => configStore.saveStatus !== 'idle')
</script>

<template>
  <Transition name="fade">
    <div v-if="visible" class="save-indicator">
      <el-icon 
        :class="['status-icon', { 'is-loading': configStore.saveStatus === 'saving' }]"
        :style="{ color: statusColor }"
      >
        <component :is="statusIcon" />
      </el-icon>
      <span class="status-text" :style="{ color: statusColor }">
        {{ statusText[configStore.saveStatus] }}
      </span>
    </div>
  </Transition>
</template>

<style scoped>
.save-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  font-size: 13px;
}

.status-icon {
  font-size: 14px;
}

.status-icon.is-loading {
  animation: rotate 1s linear infinite;
}

.status-text {
  font-weight: 500;
}

/* 淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 旋转动画 */
@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
