<script setup lang="ts">
/**
 * 同步冲突解决对话框
 * 当本地和远端预设配置都有修改时弹出，让用户选择保留哪个版本
 */
import { computed } from 'vue'
import { useSyncStore } from '@/stores/sync'
import type { ConflictResolution } from '@/services/syncApi'

const syncStore = useSyncStore()

/** 对话框是否可见 */
const visible = computed({
  get: () => syncStore.pendingConflict !== null,
  set: (val: boolean) => {
    if (!val) {
      syncStore.clearPendingConflict()
    }
  }
})

/** 冲突数据（仅 Conflict 类型） */
const conflict = computed(() => {
  const c = syncStore.pendingConflict
  if (c && c.type === 'Conflict') return c
  return null
})

/** 格式化时间显示 */
const formatTime = (isoStr: string): string => {
  const d = new Date(isoStr)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

/** 处理冲突解决 */
const handleResolve = async (resolution: ConflictResolution) => {
  await syncStore.resolveConflict(resolution)
}
</script>

<template>
  <el-dialog
    v-model="visible"
    title="同步冲突检测"
    width="620px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    class="sync-conflict-dialog"
    destroy-on-close
  >
    <!-- 说明文字 -->
    <p class="conflict-description">
      本地和远端预设配置都有修改，请选择保留哪个版本
    </p>

    <template v-if="conflict">
      <!-- 双卡片对比区域 -->
      <div class="conflict-cards">
        <!-- 本地版本 -->
        <div class="conflict-card conflict-card--local">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/>
            </svg>
          </div>
          <h4 class="card-title">本地版本</h4>
          <div class="card-meta">
            <div class="meta-row">
              <span class="meta-label">预设数量</span>
              <span class="meta-value">{{ conflict.local_count }} 个</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">最后修改</span>
              <span class="meta-value">{{ formatTime(conflict.local_updated_at) }}</span>
            </div>
          </div>
          <el-button
            type="primary"
            class="resolve-btn resolve-btn--local"
            @click="handleResolve('KeepLocal')"
          >
            保留本地
          </el-button>
        </div>

        <!-- 远端版本 -->
        <div class="conflict-card conflict-card--remote">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2C6.48 2 2 6.48 2 12C2 17.52 6.48 22 12 22C17.52 22 22 17.52 22 12C22 6.48 17.52 2 12 2Z"/>
              <path d="M2 12H22"/>
              <path d="M12 2C14.5 4.73 15.93 8.24 16 12C15.93 15.76 14.5 19.27 12 22C9.5 19.27 8.07 15.76 8 12C8.07 8.24 9.5 4.73 12 2Z"/>
            </svg>
          </div>
          <h4 class="card-title">远端版本</h4>
          <div class="card-meta">
            <div class="meta-row">
              <span class="meta-label">预设数量</span>
              <span class="meta-value">{{ conflict.remote_count }} 个</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">最后修改</span>
              <span class="meta-value">{{ formatTime(conflict.remote_updated_at) }}</span>
            </div>
          </div>
          <el-button
            type="primary"
            class="resolve-btn resolve-btn--remote"
            @click="handleResolve('KeepRemote')"
          >
            保留远端
          </el-button>
        </div>
      </div>

      <!-- 底部对比摘要 -->
      <div class="conflict-summary">
        <span class="summary-text">
          本地 {{ conflict.local_count }} 个预设 vs 远端 {{ conflict.remote_count }} 个预设
        </span>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
/* ============================================
   同步冲突对话框样式
   使用全局 CSS 变量，适配所有主题
   ============================================ */

.conflict-description {
  color: var(--app-text-secondary);
  font-size: 14px;
  line-height: 1.6;
  margin-bottom: var(--app-spacing-6);
  text-align: center;
}

/* 双卡片容器 */
.conflict-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--app-spacing-4);
  margin-bottom: var(--app-spacing-4);
}

/* 冲突卡片 */
.conflict-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--app-spacing-6) var(--app-spacing-4);
  border-radius: var(--app-radius-lg);
  border: 1px solid var(--app-border-default);
  background: var(--app-bg-elevated);
  transition: border-color var(--app-transition-fast), box-shadow var(--app-transition-fast);
}

.conflict-card:hover {
  border-color: var(--app-border-hover);
  box-shadow: var(--app-shadow-md);
}

/* 卡片图标 */
.card-icon {
  width: 40px;
  height: 40px;
  margin-bottom: var(--app-spacing-3);
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-icon svg {
  width: 100%;
  height: 100%;
}

.conflict-card--local .card-icon {
  color: var(--app-color-primary);
}

.conflict-card--remote .card-icon {
  color: var(--app-color-warning);
}

/* 卡片标题 */
.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin-bottom: var(--app-spacing-4);
  letter-spacing: 0.5px;
}

/* 元数据区域 */
.card-meta {
  width: 100%;
  margin-bottom: var(--app-spacing-5);
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--app-spacing-1) 0;
}

.meta-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.meta-value {
  font-size: 13px;
  color: var(--app-text-secondary);
  font-weight: 500;
}

/* 解决按钮 */
.resolve-btn {
  width: 100%;
  transition: all var(--app-transition-normal);
}

.resolve-btn--local {
  background: transparent !important;
  border: 1px solid var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
}

.resolve-btn--local:hover {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow: var(--app-shadow-glow-primary);
}

.resolve-btn--remote {
  background: transparent !important;
  border: 1px solid var(--app-color-warning) !important;
  color: var(--app-color-warning) !important;
}

.resolve-btn--remote:hover {
  background: rgba(245, 158, 11, 0.1) !important;
  box-shadow: 0 0 20px rgba(245, 158, 11, 0.3);
}

/* 底部对比摘要 */
.conflict-summary {
  text-align: center;
  padding: var(--app-spacing-3) 0 0;
  border-top: 1px solid var(--app-border-default);
}

.summary-text {
  font-size: 13px;
  color: var(--app-text-tertiary);
  letter-spacing: 0.3px;
}

/* ==================== Dialog 深度样式覆盖 ==================== */

:deep(.el-dialog) {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-xl) !important;
  box-shadow: var(--app-shadow-xl);
}

:deep(.el-dialog__header) {
  border-bottom: 1px solid var(--app-border-default);
  padding: var(--app-spacing-4) var(--app-spacing-6) !important;
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary) !important;
  font-weight: 600;
  font-size: 16px;
}

:deep(.el-dialog__body) {
  padding: var(--app-spacing-6) !important;
}

:deep(.el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-tertiary);
}

:deep(.el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-text-primary);
}

:deep(.el-overlay) {
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
}
</style>
