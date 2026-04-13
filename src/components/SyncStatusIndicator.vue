<script setup lang="ts">
/**
 * 同步状态指示器
 * 显示 GitHub 配置同步的实时状态：同步中 / 已同步 / 失败 / 离线 / 冲突
 */
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { Loading, CircleCheck, CircleClose, Warning, RefreshRight } from '@element-plus/icons-vue'
import { useSyncStore } from '@/stores/sync'

const syncStore = useSyncStore()
const router = useRouter()

/** 状态枚举 */
type SyncStatus = 'syncing' | 'synced' | 'error' | 'offline' | 'conflict'

/** 当前同步状态 */
const status = computed<SyncStatus>(() => {
  if (syncStore.isSyncing) return 'syncing'
  if (!syncStore.isLoggedIn) return 'offline'
  if (syncStore.pendingConflict) return 'conflict'
  if (syncStore.lastError) return 'error'
  if (syncStore.lastSyncTime) return 'synced'
  return 'offline'
})

/** 状态显示文案 */
const statusLabel = computed(() => {
  switch (status.value) {
    case 'syncing': return '同步中...'
    case 'synced': return '已同步'
    case 'error': return '同步失败'
    case 'offline': return '未同步'
    case 'conflict': return '同步冲突'
  }
})

/** tooltip 内容：成功时显示最后同步时间，失败时显示错误信息 */
const tooltipContent = computed(() => {
  if (status.value === 'synced' && syncStore.lastSyncTime) {
    return `最后同步：${formatTime(syncStore.lastSyncTime)}`
  }
  if (status.value === 'error' && syncStore.lastError) {
    return syncStore.lastError
  }
  return statusLabel.value
})

/** 格式化 ISO 时间为可读格式 */
function formatTime(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return iso
  }
}

/** 重试同步 */
function handleRetry() {
  syncStore.sync()
}

/** 跳转到同步设置页 */
function handleConflictClick() {
  router.push('/sync')
}
</script>

<template>
  <div class="sync-status" :class="[`sync-status--${status}`]">
    <el-tooltip
      :content="tooltipContent"
      placement="top"
      :show-after="400"
      :hide-after="0"
    >
      <div class="sync-status__body">
        <!-- 图标 -->
        <el-icon class="sync-status__icon" :size="14">
          <Loading v-if="status === 'syncing'" class="is-loading" />
          <CircleCheck v-else-if="status === 'synced'" />
          <CircleClose v-else-if="status === 'error'" />
          <Warning v-else-if="status === 'conflict'" />
          <CircleClose v-else />
        </el-icon>

        <!-- 文案 -->
        <span class="sync-status__label">{{ statusLabel }}</span>

        <!-- 重试按钮（仅失败态） -->
        <el-icon
          v-if="status === 'error'"
          class="sync-status__action"
          :size="13"
          @click.stop="handleRetry"
        >
          <RefreshRight />
        </el-icon>
      </div>
    </el-tooltip>

    <!-- 冲突态：整行可点击跳转 -->
    <div
      v-if="status === 'conflict'"
      class="sync-status__clickable-overlay"
      @click="handleConflictClick"
    />
  </div>
</template>

<style scoped>
/* ==================== 容器 ==================== */
.sync-status {
  position: relative;
  padding: var(--app-spacing-1) var(--app-spacing-2);
  border-radius: var(--app-radius-md);
  font-size: 12px;
  line-height: 1;
  transition: all 0.3s var(--app-easing-smooth, cubic-bezier(0.4, 0, 0.2, 1));
}

.sync-status__body {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-1);
}

.sync-status__icon {
  flex-shrink: 0;
  transition: color 0.3s;
}

.sync-status__label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.3s;
}

.sync-status__action {
  flex-shrink: 0;
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.2s, color 0.2s;
}

.sync-status__action:hover {
  opacity: 1;
}

/* 冲突态可点击覆盖层 */
.sync-status__clickable-overlay {
  position: absolute;
  inset: 0;
  cursor: pointer;
  border-radius: inherit;
}

/* ==================== 状态颜色 ==================== */

/* 同步中 - 主色调 */
.sync-status--syncing {
  color: var(--app-color-primary);
}
.sync-status--syncing .sync-status__icon {
  color: var(--app-color-primary);
}

/* 已同步 - 成功绿 */
.sync-status--synced {
  color: var(--app-color-success);
}
.sync-status--synced .sync-status__icon {
  color: var(--app-color-success);
}

/* 失败 - 危险红 */
.sync-status--error {
  color: var(--app-color-danger);
}
.sync-status--error .sync-status__icon {
  color: var(--app-color-danger);
}
.sync-status--error .sync-status__action {
  color: var(--app-color-danger);
}

/* 离线 - 灰色 */
.sync-status--offline {
  color: var(--app-text-disabled);
}
.sync-status--offline .sync-status__icon {
  color: var(--app-text-disabled);
}

/* 冲突 - 警告黄 */
.sync-status--conflict {
  color: var(--app-color-warning);
  cursor: pointer;
}
.sync-status--conflict .sync-status__icon {
  color: var(--app-color-warning);
}

/* ==================== 旋转动画（同步中） ==================== */
:deep(.is-loading) {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
