<script setup lang="ts">
/**
 * 更新日志对话框
 * 更新完成后首次启动时显示，展示新版本更新内容
 */
import { ref, onMounted, computed } from 'vue'
import { getPendingChangelog, clearPendingChangelog } from '@/services/settingsStore'
import { ElButton } from 'element-plus'
import { parse } from 'marked'

/** 对话框可见性 */
const visible = ref(false)

/** 更新日志信息 */
const changelog = ref<{
  version: string
  date?: string
  body?: string
} | null>(null)

/** 格式化发布日期 */
const formatDate = (dateStr: string | undefined): string => {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

/** 渲染 Markdown 为 HTML */
const renderedBody = computed(() => {
  if (!changelog.value?.body) return ''
  return parse(changelog.value.body, { async: false }) as string
})

/** 关闭对话框并清除记录 */
const handleClose = async () => {
  visible.value = false
  await clearPendingChangelog()
}

// 启动时检测是否有待显示的更新日志
onMounted(async () => {
  try {
    const pending = await getPendingChangelog()
    if (pending) {
      changelog.value = pending
      visible.value = true
    }
  } catch {
    // 静默处理，不影响启动
  }
})
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="'更新完成'"
    width="520px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
    class="changelog-dialog"
    destroy-on-close
  >
    <!-- 版本信息头部 -->
    <div class="changelog-header">
      <div class="version-badge">
        <span class="version-label">新版本</span>
        <span class="version-number">v{{ changelog?.version }}</span>
      </div>
      <span v-if="changelog?.date" class="release-date">
        发布于 {{ formatDate(changelog.date) }}
      </span>
    </div>

    <!-- 更新内容 -->
    <div class="changelog-body">
      <div class="changelog-section-title">更新内容</div>
      <div v-if="renderedBody" class="markdown-body" v-html="renderedBody" />
      <div v-else class="changelog-empty">
        本次更新包含性能优化和问题修复，建议立即体验。
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <ElButton type="primary" @click="handleClose">
          开始使用
        </ElButton>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
/* 版本信息头部 */
.changelog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--app-border-default);
}

.version-badge {
  display: flex;
  align-items: center;
  gap: 8px;
}

.version-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
  background: var(--app-bg-hover);
  padding: 2px 8px;
  border-radius: var(--app-radius-sm);
}

.version-number {
  font-size: 18px;
  font-weight: 700;
  color: var(--app-color-primary);
}

.release-date {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

/* 更新内容区域 */
.changelog-body {
  min-height: 120px;
}

.changelog-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin-bottom: 12px;
}

.changelog-content {
  color: var(--app-text-secondary);
  font-size: 14px;
  line-height: 1.8;
  white-space: pre-wrap;
}

.changelog-empty {
  color: var(--app-text-tertiary);
  font-size: 14px;
  line-height: 1.6;
  font-style: italic;
}

/* ==================== Markdown 渲染样式 ==================== */
.markdown-body {
  color: var(--app-text-secondary);
  font-size: 14px;
  line-height: 1.8;
}

.markdown-body :deep(h2) {
  font-size: 16px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 16px 0 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--app-border-default);
}

.markdown-body :deep(h3) {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 14px 0 6px;
}

.markdown-body :deep(p) {
  margin: 8px 0;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 8px 0;
  padding-left: 20px;
}

.markdown-body :deep(li) {
  margin: 4px 0;
}

.markdown-body :deep(a) {
  color: var(--app-color-primary);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(code) {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  background: var(--app-bg-hover);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--app-color-primary);
}

.markdown-body :deep(pre) {
  background: var(--app-bg-hover);
  padding: 12px;
  border-radius: var(--app-radius-md);
  overflow-x: auto;
  margin: 10px 0;
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
  color: var(--app-text-secondary);
}

.markdown-body :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 10px 0;
  font-size: 13px;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  padding: 8px 12px;
  border: 1px solid var(--app-border-default);
  text-align: left;
}

.markdown-body :deep(th) {
  background: var(--app-bg-hover);
  font-weight: 600;
  color: var(--app-text-primary);
}

.markdown-body :deep(tr:nth-child(even)) {
  background: var(--app-bg-hover);
}

.markdown-body :deep(blockquote) {
  margin: 10px 0;
  padding: 8px 12px;
  border-left: 3px solid var(--app-color-primary);
  background: var(--app-bg-hover);
  color: var(--app-text-tertiary);
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--app-border-default);
  margin: 16px 0;
}

/* 底部按钮 */
.dialog-footer {
  display: flex;
  justify-content: center;
  padding-top: 8px;
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
  margin-right: 0 !important;
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary) !important;
  font-weight: 600;
  font-size: 16px;
}

:deep(.el-dialog__body) {
  padding: var(--app-spacing-6) !important;
}

:deep(.el-dialog__footer) {
  border-top: 1px solid var(--app-border-default);
  padding: var(--app-spacing-4) var(--app-spacing-6) !important;
}

:deep(.el-overlay) {
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
}
</style>
