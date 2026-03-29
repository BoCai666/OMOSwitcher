<script setup lang="ts">
/**
 * 请求列表组件
 * 使用 el-table 显示 LLM 请求记录
 */
import { ref, computed, onMounted } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import type { RequestListItem } from '@/types/monitor'
import { Refresh, Search } from '@element-plus/icons-vue'

// 使用状态管理
const store = useMonitorStore()

// 选中的请求 ID
const selectedId = ref<string | null>(null)

// 搜索关键字
const searchKeyword = ref('')

// 格式化时间戳
function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 格式化持续时间
function formatDuration(duration: number | undefined): string {
  if (!duration) return '-'
  if (duration < 1000) {
    return `${duration}ms`
  }
  return `${(duration / 1000).toFixed(2)}s`
}

// 格式化费用
function formatCost(cost: number | undefined): string {
  if (cost === undefined || cost === null) return '-'
  return `$${cost.toFixed(4)}`
}

// 格式化 Token 数
function formatTokens(tokens: number | undefined): string {
  if (tokens === undefined || tokens === null) return '-'
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}K`
  }
  return tokens.toString()
}

// 获取状态码标签类型
function getStatusType(statusCode: number | undefined): string {
  if (!statusCode) return 'info'
  if (statusCode >= 200 && statusCode < 300) return 'success'
  if (statusCode >= 400) return 'danger'
  return 'warning'
}

// 过滤后的请求列表
const filteredRequests = computed(() => {
  if (!searchKeyword.value) return store.requests
  const keyword = searchKeyword.value.toLowerCase()
  return store.requests.filter((req: RequestListItem) =>
    req.provider.toLowerCase().includes(keyword) ||
    req.model.toLowerCase().includes(keyword) ||
    req.url.toLowerCase().includes(keyword)
  )
})

// 处理行点击事件
function handleRowClick(row: RequestListItem) {
  selectedId.value = row.id
  store.selectRequest(row.id)
}

// 处理刷新
async function handleRefresh() {
  await store.fetchRequests()
}

// 组件挂载时加载数据
onMounted(() => {
  store.fetchRequests()
})
</script>

<template>
  <div class="request-list">
    <!-- 工具栏 -->
    <div class="toolbar">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索 Provider、模型或 URL"
        clearable
        class="search-input"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <el-button type="primary" @click="handleRefresh" :loading="store.loading" class="refresh-btn">
        <el-icon><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <!-- 请求表格 -->
    <el-table
      :data="filteredRequests"
      stripe
      size="small"
      highlight-current-row
      @row-click="handleRowClick"
      v-loading="store.loading"
      class="monitor-table"
      row-class-name="monitor-table-row"
    >
      <!-- 时间列 -->
      <el-table-column label="时间" width="140" fixed="left">
        <template #default="{ row }">
          <span class="time-text">{{ formatTime(row.timestamp) }}</span>
        </template>
      </el-table-column>

      <!-- Provider 列 -->
      <el-table-column label="Provider" width="100">
        <template #default="{ row }">
          <el-tag size="small" effect="dark" class="provider-tag">{{ row.provider }}</el-tag>
        </template>
      </el-table-column>

      <!-- 模型列 -->
      <el-table-column label="模型" width="140" show-overflow-tooltip>
        <template #default="{ row }">
          <span class="model-text">{{ row.model }}</span>
        </template>
      </el-table-column>

      <!-- URL 列 -->
      <el-table-column label="URL" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">
          <span class="url-text">{{ row.url }}</span>
        </template>
      </el-table-column>

      <!-- Tokens 列 -->
      <el-table-column label="Tokens" width="90" align="right">
        <template #default="{ row }">
          <span class="tokens-text">{{ formatTokens(row.tokens) }}</span>
        </template>
      </el-table-column>

      <!-- 费用列 -->
      <el-table-column label="费用" width="90" align="right">
        <template #default="{ row }">
          <span class="cost-text">{{ formatCost(row.cost) }}</span>
        </template>
      </el-table-column>

      <!-- 状态码列 -->
      <el-table-column label="状态" width="80" align="center">
        <template #default="{ row }">
          <el-tag
            :type="getStatusType(row.statusCode)"
            size="small"
            effect="dark"
            class="status-tag"
          >
            {{ row.statusCode || '-' }}
          </el-tag>
        </template>
      </el-table-column>

      <!-- 耗时列 -->
      <el-table-column label="耗时" width="90" align="right">
        <template #default="{ row }">
          <span class="duration-text">{{ formatDuration(row.duration) }}</span>
        </template>
      </el-table-column>
    </el-table>

    <!-- 空状态 -->
    <el-empty
      v-if="filteredRequests.length === 0 && !store.loading"
      description="暂无请求数据"
      :image-size="100"
      class="monitor-empty"
    />
  </div>
</template>

<style scoped>
.request-list {
  padding: 20px;
  background: var(--app-bg-card);
  border-radius: 16px;
  border: 1px solid var(--app-border-default);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--app-border-default);
}

.search-input {
  width: 320px;
}

.search-input :deep(.el-input__wrapper) {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
  border-radius: 8px;
}

.search-input :deep(.el-input__inner) {
  color: var(--app-text-primary);
}

.search-input :deep(.el-input__inner::placeholder) {
  color: var(--app-text-tertiary);
}

.refresh-btn {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.1));
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: var(--app-color-primary);
  border-radius: 8px;
  transition: all 0.3s ease;
}

.refresh-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.3), rgba(0, 212, 255, 0.15));
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
  transform: translateY(-1px);
}

/* 表格样式 */
.monitor-table {
  background: transparent;
  border-radius: 12px;
  overflow: hidden;
}

.monitor-table :deep(.el-table__header-wrapper) {
  background: rgba(0, 0, 0, 0.2);
}

.monitor-table :deep(.el-table__header) {
  background: transparent;
}

.monitor-table :deep(.el-table__header th) {
  background: rgba(0, 0, 0, 0.3);
  color: var(--app-text-secondary);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--app-border-default);
  padding: 12px 8px;
}

.monitor-table :deep(.el-table__body) {
  background: transparent;
}

.monitor-table :deep(.el-table__row) {
  background: transparent;
  transition: all 0.3s ease;
  cursor: pointer;
}

.monitor-table :deep(.el-table__row td) {
  background: transparent;
  border-bottom: 1px solid var(--app-border-default);
  padding: 12px 8px;
  transition: all 0.3s ease;
}

/* 悬停高亮效果 */
.monitor-table :deep(.el-table__row:hover) {
  background: rgba(0, 212, 255, 0.08) !important;
}

.monitor-table :deep(.el-table__row:hover td) {
  background: transparent !important;
}

/* 选中状态霓虹边框 */
.monitor-table :deep(.el-table__row.current-row) {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow:
    inset 3px 0 0 var(--app-color-primary),
    0 0 20px rgba(0, 212, 255, 0.2);
}

.monitor-table :deep(.el-table__row.current-row td) {
  background: transparent !important;
}

/* 条纹效果 */
.monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: var(--app-bg-hover);
}

/* 文本样式 */
.time-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 11px;
  color: var(--app-text-secondary);
  letter-spacing: 0.3px;
}

.model-text {
  font-size: 13px;
  color: var(--app-text-secondary);
  font-weight: 500;
}

.url-text {
  font-size: 11px;
  color: var(--app-text-tertiary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
}

.tokens-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 12px;
  color: var(--app-color-success);
  font-weight: 600;
  text-shadow: 0 0 10px rgba(0, 245, 160, 0.4);
}

.cost-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 12px;
  color: var(--app-color-warning);
  font-weight: 600;
  text-shadow: 0 0 10px rgba(255, 215, 0, 0.4);
}

.duration-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 11px;
  color: var(--app-text-secondary);
}

/* 标签样式 */
.provider-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
}

.status-tag {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 600;
}

.status-tag.el-tag--success {
  background: rgba(0, 245, 160, 0.15) !important;
  border: 1px solid rgba(0, 245, 160, 0.4) !important;
  color: var(--app-color-success) !important;
}

.status-tag.el-tag--danger {
  background: rgba(255, 77, 77, 0.15) !important;
  border: 1px solid rgba(255, 77, 77, 0.4) !important;
  color: var(--app-color-danger) !important;
}

.status-tag.el-tag--warning {
  background: rgba(255, 170, 0, 0.15) !important;
  border: 1px solid rgba(255, 170, 0, 0.4) !important;
  color: var(--app-color-warning) !important;
}

.status-tag.el-tag--info {
  background: rgba(136, 136, 136, 0.15) !important;
  border: 1px solid rgba(136, 136, 136, 0.4) !important;
  color: var(--app-text-secondary) !important;
}

/* 空状态 */
.monitor-empty :deep(.el-empty__description) {
  color: var(--app-text-tertiary);
}

.monitor-empty :deep(.el-icon) {
  color: var(--app-text-tertiary);
}

/* ========== Cyberpunk 主题 ========== */
html.cyberpunk .request-list {
  background: linear-gradient(135deg, rgba(10, 15, 30, 0.95), rgba(5, 10, 25, 0.98));
  border: 1px solid rgba(0, 212, 255, 0.4);
  box-shadow: 0 0 30px rgba(0, 212, 255, 0.15);
}

html.cyberpunk .search-input :deep(.el-input__wrapper) {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .search-input :deep(.el-input__wrapper:focus-within) {
  border-color: rgba(0, 212, 255, 0.7);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.25);
}

html.cyberpunk .refresh-btn {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.25), rgba(255, 0, 128, 0.15));
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .refresh-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.35), rgba(255, 0, 128, 0.25));
  box-shadow:
    0 0 30px rgba(0, 212, 255, 0.4),
    0 0 60px rgba(255, 0, 128, 0.2);
}

html.cyberpunk .monitor-table :deep(.el-table__header th) {
  background: linear-gradient(180deg, rgba(0, 212, 255, 0.15), rgba(0, 0, 0, 0.3));
  border-bottom-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .monitor-table :deep(.el-table__row:hover) {
  background: rgba(0, 212, 255, 0.12) !important;
}

html.cyberpunk .monitor-table :deep(.el-table__row.current-row) {
  background: rgba(0, 212, 255, 0.18) !important;
  box-shadow:
    inset 4px 0 0 var(--app-color-primary),
    inset -4px 0 0 rgba(255, 0, 128, 0.5),
    0 0 30px rgba(0, 212, 255, 0.25);
}

html.cyberpunk .provider-tag {
  background: rgba(0, 212, 255, 0.2) !important;
  border-color: rgba(0, 212, 255, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .tokens-text {
  text-shadow:
    0 0 10px rgba(0, 245, 160, 0.8),
    0 0 20px rgba(0, 245, 160, 0.4);
}

html.cyberpunk .cost-text {
  text-shadow:
    0 0 10px rgba(255, 215, 0, 0.8),
    0 0 20px rgba(255, 215, 0, 0.4);
}

html.cyberpunk .status-tag.el-tag--success {
  background: rgba(0, 245, 160, 0.2) !important;
  border-color: rgba(0, 245, 160, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 245, 160, 0.3);
}

html.cyberpunk .status-tag.el-tag--danger {
  background: rgba(255, 77, 77, 0.2) !important;
  border-color: rgba(255, 77, 77, 0.5) !important;
  box-shadow: 0 0 15px rgba(255, 77, 77, 0.3);
}

/* ========== Glassmorphism 主题 ========== */
html.glassmorphism .request-list {
  background: var(--app-glass-bg, rgba(255, 255, 255, 0.25));
  border: 1px solid rgba(255, 255, 255, 0.3);
  backdrop-filter: blur(30px);
  -webkit-backdrop-filter: blur(30px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

html.glassmorphism .toolbar {
  border-bottom-color: rgba(255, 255, 255, 0.2);
}

html.glassmorphism .search-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.3);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

html.glassmorphism .search-input :deep(.el-input__wrapper:focus-within) {
  border-color: rgba(255, 255, 255, 0.5);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

html.glassmorphism .refresh-btn {
  background: rgba(255, 255, 255, 0.25);
  border-color: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

html.glassmorphism .refresh-btn:hover {
  background: rgba(255, 255, 255, 0.35);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.12);
}

html.glassmorphism .monitor-table {
  background: transparent;
}

html.glassmorphism .monitor-table :deep(.el-table__header th) {
  background: rgba(255, 255, 255, 0.15);
  border-bottom-color: rgba(255, 255, 255, 0.2);
}

html.glassmorphism .monitor-table :deep(.el-table__row) {
  background: transparent;
}

html.glassmorphism .monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: rgba(255, 255, 255, 0.08);
}

html.glassmorphism .monitor-table :deep(.el-table__row:hover) {
  background: rgba(255, 255, 255, 0.15) !important;
}

html.glassmorphism .monitor-table :deep(.el-table__row.current-row) {
  background: rgba(255, 255, 255, 0.22) !important;
  box-shadow: inset 3px 0 0 var(--app-color-primary);
}

html.glassmorphism .monitor-table :deep(.el-table__row td) {
  border-bottom-color: rgba(255, 255, 255, 0.15);
}

html.glassmorphism .tokens-text,
html.glassmorphism .cost-text {
  text-shadow: none;
}

html.glassmorphism .provider-tag {
  background: rgba(255, 255, 255, 0.25) !important;
  border-color: rgba(255, 255, 255, 0.4) !important;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

html.glassmorphism .status-tag {
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

html.glassmorphism .status-tag.el-tag--success {
  background: rgba(0, 245, 160, 0.2) !important;
  border-color: rgba(0, 245, 160, 0.4) !important;
}

html.glassmorphism .status-tag.el-tag--danger {
  background: rgba(255, 77, 77, 0.2) !important;
  border: 1px solid rgba(255, 77, 77, 0.4) !important;
}

/* 玻璃拟态主题 - 刷新按钮增强对比度 */
html.glassmorphism .refresh-btn {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.2), rgba(37, 99, 235, 0.1)) !important;
  border: 1px solid rgba(37, 99, 235, 0.4) !important;
  color: #1d4ed8 !important;
  font-weight: 500;
}

html.glassmorphism .refresh-btn:hover {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.3), rgba(37, 99, 235, 0.15)) !important;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2) !important;
}
</style>
