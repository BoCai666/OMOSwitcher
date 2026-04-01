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

// 定义 emit 事件
const emit = defineEmits<{
  (e: 'request-selected'): void
}>()

// 选中的请求 ID
const selectedId = ref<string | null>(null)

// 搜索关键字
const searchKeyword = ref('')

// 格式化时间戳
function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')
  const second = String(date.getSeconds()).padStart(2, '0')
  return `${year}/${month}/${day} ${hour}:${minute}:${second}`
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

// 过滤后的请求列表（按时间戳倒序排列，最近的请求在最前面）
const filteredRequests = computed(() => {
  const list = searchKeyword.value
    ? store.requests.filter((req: RequestListItem) => {
        const keyword = searchKeyword.value.toLowerCase()
        return req.provider.toLowerCase().includes(keyword) ||
          req.model.toLowerCase().includes(keyword) ||
          req.url.toLowerCase().includes(keyword)
      })
    : store.requests
  // 按 timestamp 倒序排列（最新的在前）
  return [...list].sort((a, b) => b.timestamp - a.timestamp)
})

// 获取今日日期字符串 (YYYY-MM-DD)
function getTodayStr(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}

// 计算今日请求总数（用于编号）
const todayRequestCount = computed(() => {
  const todayStr = getTodayStr()
  return store.requests.filter((req: RequestListItem) => {
    const reqDate = new Date(req.timestamp)
    const reqStr = `${reqDate.getFullYear()}-${String(reqDate.getMonth() + 1).padStart(2, '0')}-${String(reqDate.getDate()).padStart(2, '0')}`
    return reqStr === todayStr
  }).length
})

// 获取请求编号（按今日请求顺序）
function getRequestNumber(row: RequestListItem, index: number): number | string {
  const todayStr = getTodayStr()
  const reqDate = new Date(row.timestamp)
  const reqStr = `${reqDate.getFullYear()}-${String(reqDate.getMonth() + 1).padStart(2, '0')}-${String(reqDate.getDate()).padStart(2, '0')}`
  
  // 只给今日请求编号
  if (reqStr !== todayStr) {
    return '-'
  }
  
  // 由于列表是倒序排列的，编号 = 今日总数 - 当前索引
  // 但需要找到该请求在今日请求中的实际位置
  const todayRequests = filteredRequests.value.filter((req: RequestListItem) => {
    const d = new Date(req.timestamp)
    const s = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
    return s === todayStr
  })
  
  // 按时间正序排列今日请求，获取原始顺序
  const sortedTodayRequests = [...todayRequests].sort((a, b) => a.timestamp - b.timestamp)
  const originalIndex = sortedTodayRequests.findIndex(req => req.id === row.id)
  
  return originalIndex + 1
}

// 处理行点击事件
function handleRowClick(row: RequestListItem) {
  selectedId.value = row.id
  store.selectRequest(row.id)
  emit('request-selected')
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
      @row-click="handleRowClick"
      v-loading="store.loading"
      class="monitor-table"
      row-class-name="monitor-table-row"
      max-height="500"
    >
      <!-- 编号列 -->
      <el-table-column label="编号" width="70" align="center" fixed="left">
        <template #default="{ row, $index }">
          <span class="number-text">{{ getRequestNumber(row, $index) }}</span>
        </template>
      </el-table-column>

      <!-- 时间列 -->
      <el-table-column label="时间" width="170">
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
      <el-table-column label="URL" min-width="180" show-overflow-tooltip>
        <template #default="{ row }">
          <span class="url-text">{{ row.url }}</span>
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

/* loading 遮罩层样式 */
.monitor-table :deep(.el-loading-mask) {
  background-color: rgba(18, 18, 26, 0.8);
}

.monitor-table :deep(.el-loading-spinner .circular) {
  stroke: var(--app-color-primary);
}

.monitor-table :deep(.el-loading-spinner .el-loading-text) {
  color: var(--app-color-primary);
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

/* 条纹效果 */
.monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: var(--app-bg-hover);
}

/* 文本样式 */
.number-text {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--app-text-secondary);
  font-weight: 600;
  letter-spacing: 0.3px;
}

.time-text {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--app-text-secondary);
  letter-spacing: 0.3px;
  font-weight: 500;
}

.model-text {
  font-size: 13px;
  color: var(--app-text-primary);
  font-weight: 600;
  letter-spacing: 0.2px;
}

.url-text {
  font-size: 12px;
  color: #909399;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-weight: 450;
}

.tokens-text {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: #00F5A0;
  font-weight: 700;
  text-shadow: 0 0 10px rgba(0, 245, 160, 0.8), 0 0 20px rgba(0, 245, 160, 0.4);
}

.cost-text {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: #E6A23C;
  font-weight: 700;
}

.duration-text {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--app-text-secondary);
  font-weight: 500;
}

/* 标签样式 */
.provider-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 11px;
  font-weight: 600;
}

.status-tag {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-weight: 700;
  font-size: 12px;
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

html.cyberpunk .monitor-table :deep(.el-loading-mask) {
  background-color: rgba(10, 15, 30, 0.9);
}

html.cyberpunk .monitor-table :deep(.el-loading-spinner .circular) {
  stroke: var(--app-color-primary);
  filter: drop-shadow(0 0 10px rgba(0, 212, 255, 0.6));
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
  color: var(--app-color-primary) !important;
}

html.cyberpunk .refresh-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.35), rgba(255, 0, 128, 0.25));
  box-shadow:
    0 0 30px rgba(0, 212, 255, 0.4),
    0 0 60px rgba(255, 0, 128, 0.2);
  color: var(--app-color-primary) !important;
}

html.cyberpunk .refresh-btn .el-icon {
  color: var(--app-color-primary) !important;
}

html.cyberpunk .monitor-table :deep(.el-table__header th) {
  background: linear-gradient(180deg, rgba(0, 212, 255, 0.15), rgba(0, 0, 0, 0.3));
  border-bottom-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .monitor-table :deep(.el-table__row:hover) {
  background: rgba(0, 212, 255, 0.12) !important;
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
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid #d1d5db;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

html.glassmorphism .monitor-table :deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.8);
}

html.glassmorphism .monitor-table :deep(.el-loading-spinner .circular) {
  stroke: var(--app-color-primary);
}

html.glassmorphism .toolbar {
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .search-input :deep(.el-input__wrapper) {
  background: #ffffff;
  border: 1px solid #d1d5db;
}

html.glassmorphism .search-input :deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary);
}

html.glassmorphism .search-input :deep(.el-input__wrapper:focus-within) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .refresh-btn {
  background: var(--app-color-primary);
  border: none;
  color: #ffffff;
}

html.glassmorphism .refresh-btn:hover {
  background: var(--app-color-primary-hover);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

html.glassmorphism .monitor-table {
  background: transparent;
}

html.glassmorphism .monitor-table :deep(.el-table__header-wrapper) {
  background: #f9fafb;
}

html.glassmorphism .monitor-table :deep(.el-table__header th) {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  color: var(--app-text-primary);
}

html.glassmorphism .monitor-table :deep(.el-table__row td) {
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: #f9fafb;
}

html.glassmorphism .monitor-table :deep(.el-table__row:hover) {
  background: rgba(37, 99, 235, 0.1) !important;
}

html.glassmorphism .tokens-text,
html.glassmorphism .cost-text {
  /* 保持发光效果 */
}

html.glassmorphism .provider-tag {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.glassmorphism .status-tag.el-tag--success {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

html.glassmorphism .status-tag.el-tag--danger {
  background: rgba(239, 68, 68, 0.1) !important;
  border: 1px solid rgba(239, 68, 68, 0.3) !important;
  color: var(--app-color-danger) !important;
}

html.glassmorphism .status-tag.el-tag--warning {
  background: rgba(245, 158, 11, 0.1) !important;
  border: 1px solid rgba(245, 158, 11, 0.3) !important;
  color: var(--app-color-warning) !important;
}

html.glassmorphism .status-tag.el-tag--info {
  background: #f3f4f6 !important;
  border: 1px solid #d1d5db !important;
  color: var(--app-text-secondary) !important;
}

/* ========== 明色主题 (html.light - 非玻璃拟态/非暗色) ========== */
html.light:not(.cyberpunk):not(.dark) .request-list {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.8);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-loading-spinner .circular) {
  stroke: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .toolbar {
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .search-input :deep(.el-input__wrapper) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .search-input :deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .search-input :deep(.el-input__wrapper:focus-within) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 0 2px rgba(0, 168, 232, 0.1);
}

html.light:not(.cyberpunk):not(.dark) .refresh-btn {
  background: var(--app-color-primary);
  border: none;
  color: #ffffff;
}

html.light:not(.cyberpunk):not(.dark) .refresh-btn:hover {
  background: var(--app-color-primary-hover);
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table {
  background: transparent;
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-table__header-wrapper) {
  background: var(--app-bg-elevated);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-table__header th) {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
  color: var(--app-text-primary);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-table__row td) {
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: var(--app-bg-hover);
}

html.light:not(.cyberpunk):not(.dark) .monitor-table :deep(.el-table__row:hover) {
  background: #93c5fd !important;
}

html.light:not(.cyberpunk):not(.dark) .tokens-text {
  color: var(--app-color-success);
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .cost-text {
  /* 保持发光效果 */
}

html.light:not(.cyberpunk):not(.dark) .provider-tag {
  background: rgba(0, 168, 232, 0.1) !important;
  border: 1px solid rgba(0, 168, 232, 0.5) !important;
  color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .status-tag.el-tag--success {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.5) !important;
  color: var(--app-color-success) !important;
}

html.light:not(.cyberpunk):not(.dark) .status-tag.el-tag--danger {
  background: rgba(239, 68, 68, 0.1) !important;
  border: 1px solid rgba(239, 68, 68, 0.5) !important;
  color: var(--app-color-danger) !important;
}

html.light:not(.cyberpunk):not(.dark) .status-tag.el-tag--warning {
  background: rgba(245, 158, 11, 0.1) !important;
  border: 1px solid rgba(245, 158, 11, 0.5) !important;
  color: var(--app-color-warning) !important;
}

html.light:not(.cyberpunk):not(.dark) .status-tag.el-tag--info {
  background: var(--app-bg-hover) !important;
  border: 1px solid var(--app-border-default) !important;
  color: var(--app-text-secondary) !important;
}

/* ========== 暗色主题 (html.dark) ========== */
html.dark .refresh-btn {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.1));
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: var(--app-color-primary) !important;
  border-radius: 8px;
  transition: all 0.3s ease;
}

html.dark .refresh-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.3), rgba(0, 212, 255, 0.15));
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
  color: var(--app-color-primary) !important;
}

html.dark .refresh-btn .el-icon {
  color: var(--app-color-primary) !important;
}

html.dark .request-list {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
}

html.dark .monitor-table :deep(.el-loading-mask) {
  background-color: rgba(18, 18, 26, 0.85);
}

html.dark .monitor-table :deep(.el-loading-spinner .circular) {
  stroke: var(--app-color-primary);
}

html.dark .monitor-table :deep(.el-loading-spinner .el-loading-text) {
  color: var(--app-color-primary);
}

html.dark .toolbar {
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .search-input :deep(.el-input__wrapper) {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
}

html.dark .search-input :deep(.el-input__wrapper:focus-within) {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

html.dark .monitor-table :deep(.el-table__header th) {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
  color: var(--app-text-secondary);
}

html.dark .monitor-table :deep(.el-table__row td) {
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .monitor-table :deep(.el-table__row.el-table__row--striped) {
  background: var(--app-bg-hover);
}

html.dark .monitor-table :deep(.el-table__row:hover) {
  background: rgba(0, 212, 255, 0.08) !important;
}

html.dark .tokens-text,
html.dark .cost-text {
  /* 保持发光效果 */
}

html.dark .provider-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.dark .status-tag.el-tag--success {
  background: rgba(16, 185, 129, 0.15) !important;
  border: 1px solid rgba(16, 185, 129, 0.4) !important;
  color: var(--app-color-success) !important;
}

html.dark .status-tag.el-tag--danger {
  background: rgba(239, 68, 68, 0.15) !important;
  border: 1px solid rgba(239, 68, 68, 0.4) !important;
  color: var(--app-color-danger) !important;
}

html.dark .status-tag.el-tag--warning {
  background: rgba(245, 158, 11, 0.15) !important;
  border: 1px solid rgba(245, 158, 11, 0.4) !important;
  color: var(--app-color-warning) !important;
}

html.dark .status-tag.el-tag--info {
  background: rgba(136, 136, 136, 0.15) !important;
  border: 1px solid rgba(136, 136, 136, 0.4) !important;
  color: var(--app-text-secondary) !important;
}
</style>
