<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import BubbleApp from './components/BubbleApp.vue'
import { useBubbleQuota } from './composables/useBubbleQuota'

const appWindow = getCurrentWindow()
const isExpanded = ref(false)
const { quotas, isLoading, error } = useBubbleQuota()

const ITEM_HEIGHT = 32
const PANEL_PADDING = 12
const PANEL_WIDTH = 200  // 固定宽度足够显示 "ProviderName 100% 3天后"

async function toggleExpand() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    const count = quotas.value.length
    const height = Math.max(80, PANEL_PADDING + count * ITEM_HEIGHT)
    await appWindow.setSize(new LogicalSize(PANEL_WIDTH, height))
  } else {
    await appWindow.setSize(new LogicalSize(80, 80))
  }
}

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
    <BubbleApp
      v-if="!isExpanded"
      :quotas="quotas"
      :is-loading="isLoading"
      :error="error"
    />
    
    <!-- 展开详情面板 -->
    <div v-if="isExpanded" class="detail-panel">
      <div class="detail-list">
        <div
          v-for="q in quotas"
          :key="q.providerId"
          class="detail-item"
        >
          <span class="dot" :style="{ background: q.color }"></span>
          <span class="name">{{ q.providerName }}</span>
          <span class="percent" :style="{ color: q.remainingPercentage <= 20 ? '#e74c3c' : '#4caf50' }">
            {{ Math.round(q.remainingPercentage) }}%
          </span>
          <span class="reset-time">{{ q.resetTimeText }}</span>
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
  overflow: hidden !important;  /* 彻底禁止滚动条 */
  transition: all 0.3s ease;
  position: relative;
}

.bubble-shell.expanded {
  padding: 6px 8px;
}

.detail-panel {
  flex: 1;
  padding: 4px;
  min-height: 0;
  overflow: hidden !important;  /* 彻底禁止滚动条 */
}

.detail-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  background: rgba(255,255,255,0.05);
  font-size: 11px;
  color: var(--app-text-primary, #fff);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.name {
  flex: 1;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.percent {
  font-weight: 700;
  min-width: 28px;
  text-align: right;
}

.reset-time {
  font-size: 9px;
  color: var(--app-text-secondary, #999);
  min-width: 50px;
  text-align: right;
}
</style>
