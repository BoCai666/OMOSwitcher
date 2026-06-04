<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import ProgressRing from './ProgressRing.vue'
import type { BubbleQuotaItem } from '../composables/useBubbleQuota'

const props = defineProps<{
  quotas: BubbleQuotaItem[]
  isLoading: boolean
  error: string | null
}>()

const emit = defineEmits<{
  expand: []
  collapse: []
}>()

const localIndex = ref(0)
const isExpanded = ref(false)
const isHovering = ref(false)
let rotateTimer: ReturnType<typeof setInterval> | null = null
const ROTATE_INTERVAL = 4000

const currentQuota = computed(() => {
  if (props.quotas.length === 0) return null
  return props.quotas[localIndex.value] ?? props.quotas[0]
})

const hasMultiple = computed(() => props.quotas.length > 1)

function startRotate() {
  stopRotate()
  if (!hasMultiple.value) return
  rotateTimer = setInterval(() => {
    if (!isHovering.value) {
      localIndex.value = (localIndex.value + 1) % props.quotas.length
    }
  }, ROTATE_INTERVAL)
}

function stopRotate() {
  if (rotateTimer) {
    clearInterval(rotateTimer)
    rotateTimer = null
  }
}

function handleDblClick() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    emit('expand')
  } else {
    emit('collapse')
  }
}

function handleMouseEnter() { isHovering.value = true }
function handleMouseLeave() { isHovering.value = false }

watch(() => props.quotas, (newVal) => {
  if (newVal.length > 0) {
    localIndex.value = 0
    startRotate()
  }
}, { immediate: true, deep: true })

onUnmounted(() => {
  stopRotate()
})
</script>

<template>
  <div
    class="bubble-app"
    :class="{ 'is-expanded': isExpanded, 'is-loading': isLoading }"
    @dblclick="handleDblClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- Loading state -->
    <div v-if="isLoading" class="bubble-loading">
      <div class="loading-ring"></div>
      <span class="loading-text">加载中...</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="quotas.length === 0 && !isLoading" class="bubble-empty">
      <span class="empty-text">暂无数据</span>
    </div>

    <!-- Normal state -->
    <ProgressRing
      v-else
      :percentage="currentQuota!.remainingPercentage"
      :color="currentQuota!.color"
      :label="currentQuota!.label"
    />

    <!-- Error indicator -->
    <div v-if="error && !isLoading" class="error-indicator" title="刷新失败">
      ●
    </div>

    <!-- Expanded detail panel contents (rendered in App.vue) -->
  </div>
</template>

<style scoped>
.bubble-app {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  user-select: none;
  -webkit-app-region: no-drag;
  border-radius: 50%;
  overflow: visible;
  position: relative;
}

.bubble-app.is-expanded {
  border-radius: 12px;
}

.bubble-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.loading-ring {
  width: 36px;
  height: 36px;
  border: 3px solid var(--app-text-secondary, #666);
  border-top-color: var(--app-color-primary, #00ffff);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.bubble-empty {
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-secondary, #666);
}

.loading-text {
  font-size: 9px;
  color: var(--app-text-secondary, #999);
  margin-top: 4px;
}

.empty-text {
  font-size: 11px;
  color: var(--app-text-secondary, #999);
  text-align: center;
}

.error-indicator {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 8px;
  height: 8px;
  color: #e74c3c;
  font-size: 10px;
  line-height: 1;
}
</style>
