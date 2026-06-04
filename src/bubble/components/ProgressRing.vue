<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  percentage: number
  color: string
  label: string
}>(), {
  percentage: 0,
  color: '#4d6bfe',
  label: '--'
})

const RADIUS = 32
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

const dashOffset = computed(() => {
  const clamped = Math.max(0, Math.min(100, props.percentage))
  return CIRCUMFERENCE * (1 - clamped / 100)
})

const textColor = computed(() => {
  // 低百分比显示警告色，否则用传入的品牌色
  if (props.percentage <= 20) return '#e74c3c'
  if (props.percentage <= 40) return '#f39c12'
  return props.color
})
</script>

<template>
  <div class="progress-ring-wrapper">
    <svg viewBox="0 0 80 80" class="progress-ring">
      <!-- 背景环 -->
      <circle
        cx="40" cy="40"
        :r="RADIUS"
        fill="none"
        stroke="currentColor"
        stroke-width="4"
        opacity="0.15"
      />
      <!-- 前景进度环 -->
      <circle
        cx="40" cy="40"
        :r="RADIUS"
        fill="none"
        :stroke="textColor"
        stroke-width="4"
        stroke-linecap="round"
        :stroke-dasharray="CIRCUMFERENCE"
        :stroke-dashoffset="dashOffset"
        transform="rotate(-90 40 40)"
        class="progress-ring-fg"
      />
    </svg>
    <!-- 中心文字 -->
    <div class="ring-text">
      <span class="percentage" :style="{ color: textColor }">
        {{ Math.round(percentage) }}%
      </span>
      <span class="label">{{ label }}</span>
    </div>
  </div>
</template>

<style scoped>
.progress-ring-wrapper {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-ring {
  position: absolute;
  width: 76px;
  height: 76px;
  color: var(--app-text-secondary, #888);
}

.progress-ring-fg {
  transition: stroke-dashoffset 0.6s ease, stroke 0.3s ease;
}

.ring-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1;
  line-height: 1;
}

.percentage {
  font-size: 16px;
  font-weight: 700;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  transition: color 0.3s ease;
}

.label {
  font-size: 9px;
  font-weight: 500;
  color: var(--app-text-secondary, #999);
  margin-top: 2px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
