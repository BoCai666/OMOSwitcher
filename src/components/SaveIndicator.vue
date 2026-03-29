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

// 状态颜色 - 使用 CSS 变量
const statusColor = computed(() => {
  switch (configStore.saveStatus) {
    case 'saving':
      return 'var(--app-color-primary)'
    case 'saved':
      return 'var(--app-color-success)'
    case 'error':
      return 'var(--app-color-danger)'
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
        :class="[
          'status-icon',
          `is-${configStore.saveStatus}`
        ]"
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
  gap: 8px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  font-size: 13px;
  backdrop-filter: blur(8px);
}

.status-icon {
  font-size: 16px;
  transition: all var(--app-transition-normal);
}

/* 保存中状态 - 霓虹旋转动画 */
.status-icon.is-saving {
  animation: neon-spin 0.8s linear infinite;
  filter: drop-shadow(0 0 4px var(--app-color-primary))
          drop-shadow(0 0 8px var(--app-color-primary))
          drop-shadow(0 0 12px var(--app-color-primary));
}

/* 成功状态 - 绿色发光 */
.status-icon.is-saved {
  animation: success-pulse 0.6s ease-out;
  filter: drop-shadow(0 0 6px var(--app-color-success))
          drop-shadow(0 0 12px var(--app-color-success));
}

/* 错误状态 - 红色发光 */
.status-icon.is-error {
  animation: error-shake 0.5s ease-in-out;
  filter: drop-shadow(0 0 6px var(--app-color-danger))
          drop-shadow(0 0 12px var(--app-color-danger));
}

.status-text {
  font-weight: 500;
  transition: all var(--app-transition-normal);
  text-shadow: 0 0 8px currentColor;
}

/* 淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

/* 成功脉冲动画 */
@keyframes success-pulse {
  0% {
    transform: scale(0.8);
    filter: drop-shadow(0 0 2px var(--app-color-success));
  }
  50% {
    transform: scale(1.1);
    filter: drop-shadow(0 0 10px var(--app-color-success))
            drop-shadow(0 0 20px rgba(16, 185, 129, 0.6));
  }
  100% {
    transform: scale(1);
    filter: drop-shadow(0 0 6px var(--app-color-success))
            drop-shadow(0 0 12px rgba(16, 185, 129, 0.4));
  }
}

/* 错误抖动动画 */
@keyframes error-shake {
  0%, 100% {
    transform: translateX(0);
  }
  20% {
    transform: translateX(-4px) rotate(-5deg);
  }
  40% {
    transform: translateX(4px) rotate(5deg);
  }
  60% {
    transform: translateX(-3px) rotate(-3deg);
  }
  80% {
    transform: translateX(3px) rotate(3deg);
  }
}

/* 状态文字发光效果 */
.status-text {
  animation: text-glow 1.5s ease-in-out infinite alternate;
}

@keyframes text-glow {
  from {
    text-shadow: 0 0 4px currentColor;
  }
  to {
    text-shadow: 0 0 8px currentColor,
                 0 0 16px currentColor;
  }
}
</style>
