<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import BubbleApp from './components/BubbleApp.vue'
import { useBubbleQuota } from './composables/useBubbleQuota'

const appWindow = getCurrentWindow()
const isExpanded = ref(false)
const { quotas, isLoading, error } = useBubbleQuota()

const ITEM_HEIGHT = 44
const PANEL_PADDING = 12

// 实际文字测量（不依赖 DOM 当前宽度）
function measureTextWidth(text: string, fontSize: number, fontWeight: string): number {
  // 创建一个隐藏的 span 来精确测量文字宽度
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

async function measureAndResize() {
  await nextTick()
  if (quotas.value.length === 0) return

  // 测量百分比和倒计时的自然宽度
  // 名称有 max-width 截断（140px），不需要测量
  let maxPercentWidth = 0
  let maxResetWidth = 0

  for (const q of quotas.value) {
    const percentText = `${Math.round(q.remainingPercentage)}%`
    const percentW = measureTextWidth(percentText, 11, '700')
    const resetW = q.resetTimeText ? measureTextWidth(q.resetTimeText, 9, '400') : 0
    if (percentW > maxPercentWidth) maxPercentWidth = percentW
    if (resetW > maxResetWidth) maxResetWidth = resetW
  }

  // 布局：圆点 + gap + 名称(140px) + gap + 百分比 + gap + 倒计时 + item padding
  const DOT_WIDTH = 6
  const NAME_MAX_WIDTH = 140
  const GAP = 6
  const GAPS = GAP * 3
  const ITEM_PADDING = 8 * 2
  const SHELL_PADDING = 8 * 2
  const SAFETY_MARGIN = 12

  const contentWidth = DOT_WIDTH + GAPS + NAME_MAX_WIDTH + maxPercentWidth + maxResetWidth + ITEM_PADDING
  const windowWidth = Math.ceil(contentWidth + SHELL_PADDING + SAFETY_MARGIN)

  const count = quotas.value.length
  const height = Math.max(80, PANEL_PADDING + count * ITEM_HEIGHT)

  await appWindow.setSize(new LogicalSize(windowWidth, height))
}

async function toggleExpand() {
  if (!isExpanded.value) {
    // 展开：先解除圆形裁剪并扩大窗口，最后才显示面板
    // 避免面板在 80x80 圆形窗口内瞬间出现被裁剪
    document.documentElement.classList.add('bubble-expanded')
    await measureAndResize()
    isExpanded.value = true
  } else {
    // 收起：先隐藏面板，再恢复圆形裁剪，最后缩小窗口
    isExpanded.value = false
    await nextTick() // 确保 Vue 已更新 DOM，面板变为 hidden
    document.documentElement.classList.remove('bubble-expanded')
    await appWindow.setSize(new LogicalSize(80, 80))
  }
}

// 数据加载完成或变化时重新测量（如果当前展开）
watch(quotas, async () => {
  if (isExpanded.value) {
    await measureAndResize()
  }
}, { deep: true })

// 拖拽相关状态
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
    // 不是拖拽 → 切换展开/收起
    toggleExpand()
  }
}
</script>

<template>
  <div class="bubble-shell" :class="{ expanded: isExpanded }" @mousedown="handleMouseDown">
    <!-- 悬浮球内容：绝对定位叠加 -->
    <div class="content-wrapper" :class="{ active: !isExpanded }">
      <BubbleApp
        :quotas="quotas"
        :is-loading="isLoading"
        :error="error"
      />
    </div>

    <!-- 面板内容：绝对定位叠加 -->
    <div class="detail-panel" :class="{ active: isExpanded }">
      <div class="detail-list">
        <div
          v-for="q in quotas"
          :key="q.providerId"
          class="detail-item"
        >
          <div class="item-info">
            <span class="dot" :style="{ background: q.color }"></span>
            <span class="name">{{ q.providerName }}</span>
            <span class="percent" :style="{ color: q.remainingPercentage <= 20 ? '#e74c3c' : '#4caf50' }">
              {{ Math.round(q.remainingPercentage) }}%
            </span>
            <span class="reset-time">{{ q.resetTimeText }}</span>
          </div>
          <div class="item-bar">
            <div
              class="item-bar-fill"
              :style="{
                width: Math.round(q.remainingPercentage) + '%',
                background: q.remainingPercentage <= 20 ? '#e74c3c' : q.remainingPercentage <= 40 ? '#f39c12' : q.color
              }"
            ></div>
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
  overflow: hidden !important;
  border-radius: 50%;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  position: relative;
  transition: border-radius 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.bubble-shell.expanded {
  border-radius: 12px;
  background: #1a1a2e !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
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
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  padding: 10px;
  visibility: hidden;
  pointer-events: none;
}

.content-wrapper.active,
.detail-panel.active {
  visibility: visible;
  pointer-events: auto;
}

.detail-list {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 6px;
  width: 100%;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 4px 10px;
  border-radius: 6px;
  background: rgba(255,255,255,0.08);
  border: 1px solid rgba(255,255,255,0.15);
  font-size: 11px;
  color: var(--app-text-primary, #fff);
  width: 100%;
  box-sizing: border-box;
}

.item-info {
  display: flex;
  align-items: center;
  gap: 6px;
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
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
  flex: 0 0 auto;
}

.percent {
  font-weight: 700;
  flex: 0 0 auto;
}

.reset-time {
  font-size: 9px;
  color: var(--app-text-secondary, #999);
  white-space: nowrap;
  flex: 0 0 auto;
}

.item-bar {
  width: 100%;
  height: 3px;
  border-radius: 2px;
  background: rgba(255,255,255,0.1);
  overflow: hidden;
}

.item-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.4s ease;
}

</style>
