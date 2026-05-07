<script setup lang="ts">
/**
 * 更新确认对话框（美化版）
 * 当检测到有新版本时弹出，显示版本信息、更新说明和下载进度
 */
import { computed, ref, onMounted } from 'vue'
import { useUpdateStore } from '@/stores/update'
import { getVersion } from '@tauri-apps/api/app'
import { parse } from 'marked'
import { Download, CircleCheck, Clock, Top, Close } from '@element-plus/icons-vue'

const updateStore = useUpdateStore()

/** 当前应用版本 */
const currentVersion = ref('')

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

/** 渲染 Markdown 为 HTML */
const renderedBody = computed(() => {
  if (!updateStore.updateInfo?.body) return ''
  return parse(updateStore.updateInfo.body, { async: false }) as string
})

/** 是否有更新内容 */
const hasBody = computed(() => !!updateStore.updateInfo?.body)

// 组件挂载时获取当前版本号
onMounted(() => {
  getVersion().then(v => { currentVersion.value = v }).catch(() => {})
})
</script>

<template>
  <el-dialog
    v-model="visible"
    :show-close="false"
    width="520px"
    :close-on-click-modal="!updateStore.isDownloading"
    :close-on-press-escape="!updateStore.isDownloading"
    class="update-dialog"
    destroy-on-close
    align-center
  >
    <!-- 自定义头部 -->
    <template #header>
      <div class="update-dialog-header">
        <div class="header-icon-wrapper">
          <el-icon class="header-icon" :size="28"><Top /></el-icon>
        </div>
        <div class="header-content">
          <h3 class="header-title">发现新版本</h3>
          <p class="header-subtitle">新版本已就绪，是否立即更新？</p>
        </div>
        <el-button
          v-if="!updateStore.isDownloading"
          class="close-btn"
          circle
          text
          @click="updateStore.dismiss()"
        >
          <el-icon><Close /></el-icon>
        </el-button>
      </div>
    </template>

    <!-- 版本对比卡片 -->
    <div class="version-compare">
      <div class="version-item current">
        <span class="version-label">当前版本</span>
        <span class="version-number">v{{ currentVersion }}</span>
      </div>
      <div class="version-arrow">
        <el-icon><Top /></el-icon>
      </div>
      <div class="version-item latest">
        <span class="version-label">新版本</span>
        <span class="version-number">v{{ updateStore.updateInfo?.version }}</span>
      </div>
    </div>

    <!-- 更新说明区 -->
    <template v-if="!updateStore.isDownloading">
      <!-- 更新日志 -->
      <div v-if="hasBody" class="update-changelog">
        <div class="changelog-header">
          <el-icon><CircleCheck /></el-icon>
          <span>更新内容</span>
        </div>
        <div class="changelog-body markdown-body" v-html="renderedBody" />
      </div>

      <!-- 发布日期 -->
      <div v-if="updateStore.updateInfo?.date" class="update-meta">
        <el-icon><Clock /></el-icon>
        <span>发布于 {{ formatDate(updateStore.updateInfo.date) }}</span>
      </div>
    </template>

    <!-- 下载进度区 -->
    <div
      v-if="updateStore.isDownloading"
      class="download-progress"
    >
      <div class="progress-header">
        <el-icon class="download-icon is-loading"><Download /></el-icon>
        <span class="progress-title">正在下载更新...</span>
      </div>
      <div class="progress-bar-wrapper">
        <div class="progress-track">
          <div
            class="progress-fill"
            :style="{ width: updateStore.downloadProgress + '%' }"
          />
        </div>
        <div class="progress-info">
          <span class="progress-percent">{{ updateStore.downloadProgress }}%</span>
          <span v-if="updateStore.downloadProgress < 100" class="progress-hint">下载完成后将自动安装</span>
          <span v-else class="progress-hint complete">下载完成，准备安装...</span>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div
      v-if="updateStore.error"
      class="update-error"
    >
      <el-icon><Close /></el-icon>
      <span>{{ updateStore.error }}</span>
    </div>

    <!-- 底部按钮区 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button
          v-if="!updateStore.isDownloading"
          class="btn-later"
          @click="updateStore.dismiss()"
        >
          稍后提醒
        </el-button>
        <el-button
          v-if="!updateStore.isDownloading"
          type="primary"
          class="btn-update"
          @click="updateStore.install()"
        >
          <el-icon><Download /></el-icon>
          立即更新
        </el-button>
        <el-button
          v-else
          type="primary"
          class="btn-updating"
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
/* ==================== 对话框头部 ==================== */
.update-dialog-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.header-icon-wrapper {
  width: 56px;
  height: 56px;
  border-radius: var(--app-radius-lg);
  background: linear-gradient(135deg, var(--app-color-primary) 0%, var(--app-color-success) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: var(--app-shadow-glow-primary);
}

.header-icon {
  color: var(--app-text-inverse);
  font-weight: bold;
}

.header-content {
  flex: 1;
  min-width: 0;
}

.header-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
  line-height: 1.4;
}

.header-subtitle {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--app-text-tertiary);
  line-height: 1.4;
}

.close-btn {
  flex-shrink: 0;
  color: var(--app-text-tertiary);
}

.close-btn:hover {
  color: var(--app-text-primary);
}

/* ==================== 版本对比 ==================== */
.version-compare {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 20px;
  background: var(--app-bg-hover);
  border-radius: var(--app-radius-lg);
  margin: 16px 0;
}

.version-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.version-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.version-number {
  font-size: 18px;
  font-weight: 700;
  color: var(--app-text-primary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.version-item.latest .version-number {
  color: var(--app-color-primary);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

.version-arrow {
  width: 32px;
  height: 32px;
  border-radius: var(--app-radius-full);
  background: var(--app-bg-active);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--app-color-primary);
}

.version-arrow .el-icon {
  transform: rotate(90deg);
  font-size: 16px;
}

/* ==================== 更新日志 ==================== */
.update-changelog {
  margin: 16px 0;
}

.changelog-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.changelog-header .el-icon {
  color: var(--app-color-success);
}

.changelog-body {
  max-height: 240px;
  overflow-y: auto;
  padding: 12px;
  background: var(--app-bg-hover);
  border-radius: var(--app-radius-md);
  border: 1px solid var(--app-border-default);
}

/* 自定义滚动条 */
.changelog-body::-webkit-scrollbar {
  width: 4px;
}

.changelog-body::-webkit-scrollbar-track {
  background: transparent;
}

.changelog-body::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 2px;
}

.changelog-body::-webkit-scrollbar-thumb:hover {
  background: var(--app-border-hover);
}

/* ==================== Markdown 渲染样式 ==================== */
.markdown-body {
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.7;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 12px 0 6px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--app-border-default);
}

.markdown-body :deep(p) {
  margin: 6px 0;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 6px 0;
  padding-left: 18px;
}

.markdown-body :deep(li) {
  margin: 3px 0;
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
  font-size: 12px;
  background: var(--app-bg-active);
  padding: 2px 5px;
  border-radius: 3px;
  color: var(--app-color-primary);
}

.markdown-body :deep(pre) {
  background: var(--app-bg-active);
  padding: 10px;
  border-radius: var(--app-radius-sm);
  overflow-x: auto;
  margin: 8px 0;
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
  color: var(--app-text-secondary);
}

/* ==================== 发布日期 ==================== */
.update-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.update-meta .el-icon {
  font-size: 14px;
}

/* ==================== 下载进度 ==================== */
.download-progress {
  margin: 20px 0;
  padding: 20px;
  background: var(--app-bg-hover);
  border-radius: var(--app-radius-lg);
  border: 1px solid var(--app-border-default);
}

.progress-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.download-icon {
  color: var(--app-color-primary);
  font-size: 20px;
}

.progress-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.progress-bar-wrapper {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.progress-track {
  width: 100%;
  height: 8px;
  background: var(--app-bg-active);
  border-radius: var(--app-radius-full);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--app-color-primary) 0%, var(--app-color-success) 100%);
  border-radius: var(--app-radius-full);
  transition: width 0.3s ease;
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 20px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3));
  border-radius: 0 var(--app-radius-full) var(--app-radius-full) 0;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-percent {
  font-size: 16px;
  font-weight: 700;
  color: var(--app-color-primary);
  font-family: 'JetBrains Mono', monospace;
}

.progress-hint {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.progress-hint.complete {
  color: var(--app-color-success);
}

/* 下载图标旋转动画 */
.download-icon.is-loading {
  animation: download-bounce 1s ease-in-out infinite;
}

@keyframes download-bounce {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-4px);
  }
}

/* ==================== 错误提示 ==================== */
.update-error {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--app-radius-md);
  color: var(--app-color-danger);
  font-size: 13px;
}

.update-error .el-icon {
  flex-shrink: 0;
  font-size: 16px;
}

/* ==================== 底部按钮 ==================== */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 8px;
}

.btn-later {
  background: transparent;
  border: 1px solid var(--app-border-default);
  color: var(--app-text-secondary);
}

.btn-later:hover {
  border-color: var(--app-border-hover);
  color: var(--app-text-primary);
}

.btn-update {
  background: linear-gradient(135deg, var(--app-color-primary) 0%, var(--app-color-success) 100%);
  border: none;
  color: var(--app-text-inverse);
  font-weight: 600;
  padding: 8px 20px;
}

.btn-update:hover {
  background: linear-gradient(135deg, var(--app-color-primary) 0%, var(--app-color-success) 100%);
  opacity: 0.9;
  transform: translateY(-1px);
  box-shadow: var(--app-shadow-glow-primary);
}

.btn-update .el-icon {
  margin-right: 4px;
}

.btn-updating {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  color: var(--app-text-tertiary);
}

/* ==================== Dialog 深度样式覆盖 ==================== */
:deep(.el-dialog) {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-xl) !important;
  box-shadow: var(--app-shadow-xl);
  overflow: hidden;
}

:deep(.el-dialog__header) {
  padding: 0 var(--app-spacing-6) !important;
  margin: 0 !important;
  border-bottom: 1px solid var(--app-border-default);
}

:deep(.el-dialog__body) {
  padding: var(--app-spacing-4) var(--app-spacing-6) !important;
}

:deep(.el-dialog__footer) {
  padding: var(--app-spacing-4) var(--app-spacing-6) !important;
  border-top: 1px solid var(--app-border-default);
}

:deep(.el-overlay) {
  background-color: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
}

/* ==================== 赛博朋克主题适配 ==================== */
html.cyberpunk .header-icon-wrapper {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 0%, rgba(0, 255, 255, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .header-icon {
  color: #00ffff;
}

html.cyberpunk .version-item.latest .version-number {
  color: #00ffff;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .version-arrow {
  background: rgba(0, 255, 255, 0.1);
  color: #00ffff;
}

html.cyberpunk .progress-fill {
  background: linear-gradient(90deg, #00ffff 0%, #00ff88 100%);
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .btn-update {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 255, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  color: #00ffff;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .btn-update:hover {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.25) 0%, rgba(0, 255, 255, 0.1) 100%);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.4);
}

/* ==================== 玻璃拟态主题适配 ==================== */
html.glassmorphism .header-icon-wrapper {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(14, 165, 233, 0.1) 100%);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2);
}

html.glassmorphism .version-item.latest .version-number {
  color: #2563eb;
}

html.glassmorphism .progress-fill {
  background: linear-gradient(90deg, #2563eb 0%, #10b981 100%);
}

html.glassmorphism .btn-update {
  background: linear-gradient(135deg, #2563eb 0%, #10b981 100%);
  color: #ffffff;
}

html.glassmorphism .btn-update:hover {
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.3);
}
</style>
