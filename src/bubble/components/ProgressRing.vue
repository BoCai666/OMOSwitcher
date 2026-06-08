<script setup lang="ts">
import { computed } from 'vue'
import { useTweenedNumber } from '../composables/useTweenedNumber'

const props = withDefaults(defineProps<{
  percentage: number
  color: string
  label: string
}>(), {
  percentage: 0,
  color: '#4d6bfe',
  label: '--'
})

const clampedPercentage = computed(() => Math.max(0, Math.min(100, props.percentage)))

// 数字平滑过渡：刷新时数字从旧值滚动到新值
const displayPercentage = useTweenedNumber(() => clampedPercentage.value, { duration: 700 })

// 水波高度：百分比越高，水面越往上
const waterHeight = computed(() => clampedPercentage.value)

// 根据百分比返回水波颜色（主色+浅色）
const waterColor = computed(() => {
  const p = clampedPercentage.value
  if (p <= 15) return { main: '#c0392b', light: '#e74c3c', bg: '#1a0f0f' }
  if (p <= 30) return { main: '#e67e22', light: '#f39c12', bg: '#1a150f' }
  if (p <= 50) return { main: '#f1c40f', light: '#f39c12', bg: '#1a1a0f' }
  if (p <= 70) return { main: '#27ae60', light: '#2ecc71', bg: '#0f1a12' }
  if (p <= 85) return { main: '#2980b9', light: '#3498db', bg: '#0f141a' }
  return { 
    main: props.color, 
    light: lightenColor(props.color, 30), 
    bg: darkenColor(props.color, 70)
  }
})

// 文字颜色统一白色
const textColor = '#ffffff'

function lightenColor(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * percent)
  const R = Math.min(255, (num >> 16) + amt)
  const G = Math.min(255, ((num >> 8) & 0x00FF) + amt)
  const B = Math.min(255, (num & 0x0000FF) + amt)
  return '#' + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)
}

function darkenColor(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * percent)
  const R = Math.max(0, (num >> 16) - amt)
  const G = Math.max(0, ((num >> 8) & 0x00FF) - amt)
  const B = Math.max(0, (num & 0x0000FF) - amt)
  return '#' + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)
}
</script>

<template>
  <div class="water-ball-wrapper">
    <div 
      class="water-ball"
      :style="{
        '--water-height': waterHeight + '%',
        '--water-color': waterColor.main,
        '--water-color-light': waterColor.light,
        '--ball-bg': waterColor.bg
      }"
    >
      <!-- 水波容器 -->
      <div class="water-container">
        <div class="water">
          <div class="wave wave-1"></div>
          <div class="wave wave-2"></div>
          <div class="wave wave-3"></div>
        </div>
      </div>
      
      <!-- 球体边框/光泽效果 -->
      <div class="ball-shine"></div>
      
      <!-- 中心文字 -->
      <div class="ball-text">
        <span class="percentage" :style="{ color: textColor }">
          {{ Math.round(displayPercentage) }}%
        </span>
        <span class="label">{{ label }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.water-ball-wrapper {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.water-ball {
  position: relative;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: var(--ball-bg, #1a1a2e);
  overflow: hidden;
}

/* 水波容器 */
.water-container {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: var(--water-height);
  transition: height 0.8s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

/* 水体 */
.water {
  position: absolute;
  bottom: 0;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(180deg, var(--water-color-light) 0%, var(--water-color) 60%);
}

/* 波浪 */
.wave {
  position: absolute;
  top: -14px;
  left: 0;
  width: 200%;
  height: 28px;
  background: inherit;
  border-radius: 40%;
}

.wave-1 {
  opacity: 0.9;
  animation: wave-rotate 6s linear infinite;
  margin-left: -20%;
}

.wave-2 {
  opacity: 0.7;
  animation: wave-rotate 4s linear infinite reverse;
  margin-left: 10%;
  top: -12px;
  height: 24px;
}

.wave-3 {
  opacity: 0.5;
  animation: wave-rotate 8s linear infinite;
  margin-left: -40%;
  top: -10px;
  height: 20px;
}

@keyframes wave-rotate {
  0% {
    transform: translateX(0) rotate(0deg);
  }
  100% {
    transform: translateX(-25%) rotate(360deg);
  }
}

/* 球体光泽 */
.ball-shine {
  position: absolute;
  top: 8%;
  left: 15%;
  width: 25%;
  height: 15%;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.25) 0%, transparent 70%);
  pointer-events: none;
}

/* 文字 */
.ball-text {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 10;
  line-height: 1;
  text-align: center;
}

.percentage {
  font-size: 17px;
  font-weight: 800;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.6), 0 0 10px rgba(0, 0, 0, 0.3);
  transition: color 0.3s ease;
  letter-spacing: -0.5px;
}

.label {
  font-size: 8px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin-top: 2px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.6);
}
</style>
