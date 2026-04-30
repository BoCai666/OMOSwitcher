<script setup lang="ts">
/**
 * 更新确认对话框
 * 当检测到有新版本时弹出，显示版本信息、更新说明和下载进度
 */
import { computed } from 'vue'
import { useUpdateStore } from '@/stores/update'

const updateStore = useUpdateStore()

/** 对话框是否可见 */
const visible = computed({
  get: () => updateStore.hasUpdate && !updateStore.isChecking,
  set: (val: boolean) => {
    if (!val) {
      updateStore.dismiss()
    }
  }
})

/** 格式化发布日期显示 */
const formatDate = (dateStr: string | undefined): string => {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="'发现新版本 v' + (updateStore.updateInfo?.version ?? '')"
    width="480px"
    :close-on-click-modal="false"
    class="update-dialog"
    destroy-on-close
  >
    <!-- 更新说明区 -->
    <template v-if="!updateStore.isDownloading">
      <p
        v-if="updateStore.updateInfo?.body"
        class="update-body"
      >
        {{ updateStore.updateInfo.body }}
      </p>
      <p
        v-if="updateStore.updateInfo?.date"
        class="update-date"
      >
        发布于 {{ formatDate(updateStore.updateInfo.date) }}
      </p>
    </template>

    <!-- 下载进度区 -->
    <div
      v-if="updateStore.isDownloading"
      class="update-progress"
    >
      <p class="progress-text">
        正在下载更新...
      </p>
      <el-progress
        :percentage="updateStore.downloadProgress"
        :stroke-width="8"
        status=""
      />
      <p class="progress-percent">
        {{ updateStore.downloadProgress }}%
      </p>
    </div>

    <!-- 错误提示 -->
    <p
      v-if="updateStore.error"
      class="update-error"
    >
      {{ updateStore.error }}
    </p>

    <!-- 底部按钮区 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button
          :disabled="updateStore.isDownloading"
          @click="updateStore.dismiss()"
        >
          稍后提醒
        </el-button>
        <el-button
          v-if="!updateStore.isDownloading"
          type="primary"
          @click="updateStore.install()"
        >
          立即更新
        </el-button>
        <el-button
          v-else
          type="primary"
          loading
          disabled
        >
          下载中...
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
/* 更新说明文本 */
.update-body {
  color: var(--app-text-secondary);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
}

/* 发布日期 */
.update-date {
  color: var(--app-text-tertiary);
  font-size: 12px;
  margin-top: 8px;
}

/* 下载进度区域 */
.update-progress {
  margin: 16px 0;
}

.progress-text {
  color: var(--app-text-secondary);
  font-size: 14px;
  margin-bottom: 12px;
}

.progress-percent {
  color: var(--app-text-tertiary);
  font-size: 12px;
  margin-top: 8px;
  text-align: right;
}

/* 错误提示 */
.update-error {
  color: var(--app-color-danger);
  font-size: 13px;
  margin-top: 12px;
}

/* 底部按钮 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
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
