<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import BubbleApp from './components/BubbleApp.vue'
import { useBubbleQuota } from './composables/useBubbleQuota'

const appWindow = getCurrentWindow()
const isExpanded = ref(false)
const { quotas, isLoading, error } = useBubbleQuota()

const ITEM_HEIGHT = 24
const PANEL_PADDING = 12
const GAP_HEIGHT = 4

function measureTextWidth(text: string, fontSize: number, fontWeight: string): number {
  const span = document.createElement('span')
  span.style.visibility = 'hidden'
  span.style.position = 'absolute'
  span.style.whiteSpace = 'nowrap'
  span.style.fontSize = `${fontSize}px`
  span.style.fontWeight = fontWeight
  span.style.fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
  span.textContent = text
  document.body.appendChild(span)
  const width = span.getBoundingClientRect().width
  document.body.removeChild(span)
  return width
}

// 填充色按额度状态显示：鲜艳配色（主色+浅色用于渐变）
function getFillColor(percentage: number): { main: string; light: string } {
  if (percentage <= 10) return { main: '#e74c3c', light: '#ff6b6b' } // 严重不足
  if (percentage <= 20) return { main: '#f39c12', light: '#ffd93d' } // 危险
  if (percentage <= 50) return { main: '#f1c40f', light: '#f9ca24' } // 警告
  return { main: '#2ecc71', light: '#6dd5b8' } // 充足
}

async function measureAndResize() {
  await nextTick()
  if (quotas.value.length === 0) return

  let maxContentWidth = 0

  for (const q of quotas.value) {
    // 测量每项实际文本宽度，不设上限，确保窗口足够容纳完整内容
    const nameW = measureTextWidth(q.providerName, 11, '500')
    const percentText = `${Math.round(q.remainingPercentage)}%`
    const percentW = measureTextWidth(percentText, 11, '700')
    const resetW = q.resetTimeText ? measureTextWidth(q.resetTimeText, 9, '400') : 0

    // 计算该行的总内容宽度：dot + 4个gap + name + spacer(min 8px) + percent + reset + item padding
    const rowWidth = 6 + (6 * 4) + nameW + 8 + percentW + resetW + (10 * 2)
    if (rowWidth > maxContentWidth) maxContentWidth = rowWidth
  }

  // detail-panel padding 是 12px，左右共 24px
  const SHELL_PADDING = 12 * 2
  const SAFETY_MARGIN = 20

  const windowWidth = Math.ceil(maxContentWidth + SHELL_PADDING + SAFETY_MARGIN)

  const count = quotas.value.length
  const gapHeight = count > 1 ? (count - 1) * GAP_HEIGHT : 0
  const height = Math.max(80, PANEL_PADDING + count * ITEM_HEIGHT + gapHeight)

  await appWindow.setSize(new LogicalSize(windowWidth, height))
}

async function toggleExpand() {
  if (!isExpanded.value) {
    document.documentElement.classList.add('bubble-expanded')
    await measureAndResize()
    isExpanded.value = true
  } else {
    isExpanded.value = false
    await nextTick()
    document.documentElement.classList.remove('bubble-expanded')
    await appWindow.setSize(new LogicalSize(80, 80))
  }
}

watch(quotas, async () => {
  if (isExpanded.value) {
    await measureAndResize()
  }
}, { deep: true })

const DRAG_THRESHOLD = 5
let isDragging = false
let dragStartX = 0
let dragStartY = 0
let windowStartX = 0
let windowStartY = 0

async function handleMouseDown(e: MouseEvent) {
  const pos = await appWindow.outerPosition()
  windowStartX = pos.x
  windowStartY = pos.y
  dragStartX = e.screenX
  dragStartY = e.screenY
  isDragging = false
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

function handleMouseMove(e: MouseEvent) {
  const dx = Math.abs(e.screenX - dragStartX)
  const dy = Math.abs(e.screenY - dragStartY)
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    isDragging = true
    appWindow.setPosition(
      new PhysicalPosition(
        windowStartX + (e.screenX - dragStartX),
        windowStartY + (e.screenY - dragStartY)
      )
    )
  }
}

function handleMouseUp() {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
  if (isDragging) {
    appWindow.outerPosition().then(pos => {
      invoke('save_bubble_position', { x: pos.x, y: pos.y })
    })
  } else {
    toggleExpand()
  }
}
</script>

<template>
  <div class="bubble-shell" :class="{ expanded: isExpanded }" @mousedown="handleMouseDown">
    <!-- 悬浮球内容 -->
    <div class="content-wrapper" :class="{ active: !isExpanded }">
      <BubbleApp :quotas="quotas" :is-loading="isLoading" :error="error" />
    </div>

    <!-- 详情面板 -->
    <div class="detail-panel" :class="{ active: isExpanded }">
      <div class="detail-list">
        <div
          v-for="q in quotas"
          :key="q.providerId"
          class="detail-item"
          :style="{
            '--fill-width': Math.round(q.remainingPercentage) + '%',
            '--fill-color': getFillColor(q.remainingPercentage).main,
            '--fill-color-light': getFillColor(q.remainingPercentage).light
          }"
        >
          <div class="item-fill"></div>
          <div class="item-content">
            <span class="dot" :style="{ background: q.color }"></span>
            <span class="name">{{ q.providerName }}</span>
            <span class="spacer"></span>
            <span 
              class="percent"
            >
              {{ Math.round(q.remainingPercentage) }}%
            </span>
            <span v-if="q.resetTimeText" class="reset-time">{{ q.resetTimeText }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
.bubble-shell {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  position: relative;
}

/* ============ 展开面板样式 ============ */
.bubble-shell.expanded {
  background: rgba(30, 30, 32, 0.95) !important;
  backdrop-filter: blur(16px) saturate(150%);
  -webkit-backdrop-filter: blur(16px) saturate(150%);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

.content-wrapper {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  padding: 8px;
  overflow: hidden !important;
  visibility: hidden;
  pointer-events: none;
}

.detail-panel {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 100%;
  height: 100%;
  padding: 12px;
  box-sizing: border-box;
  visibility: hidden;
  pointer-events: none;
}

.content-wrapper.active,
.detail-panel.active {
  visibility: visible;
  pointer-events: auto;
}

/* 列表容器 - 无滚动条 */
.detail-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* 单个 provider 行 - 整行作为进度条 */
.detail-item {
  position: relative;
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-sizing: border-box;
  overflow: hidden;
  cursor: default;
}

/* 填充层 - 渐变+光泽效果 */
.item-fill {
  position: absolute;
  top: 0;
  left: 0;
  width: var(--fill-width);
  height: 100%;
  background: linear-gradient(90deg, var(--fill-color) 0%, var(--fill-color-light) 100%);
  opacity: 0.3;
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 5px 0 0 5px;
}

/* 内容层 - 在填充色上方 */
.item-content {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  flex-wrap: nowrap;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.name {
  font-weight: 500;
  font-size: 11px;
  white-space: nowrap;
  color: rgba(255, 255, 255, 0.85);
}

.spacer {
  flex: 1;
  min-width: 8px;
}

.percent {
  font-weight: 700;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.95);
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.reset-time {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.3);
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
