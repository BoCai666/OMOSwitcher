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
        style="width: 300px"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <el-button type="primary" @click="handleRefresh" :loading="store.loading">
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
      style="width: 100%"
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
          <el-tag size="small" effect="plain">{{ row.provider }}</el-tag>
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
            effect="light"
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
    />
  </div>
</template>

<style scoped>
.request-list {
  padding: 16px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.time-text {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #606266;
}

.model-text {
  font-size: 13px;
  color: #303133;
}

.url-text {
  font-size: 12px;
  color: #606266;
  font-family: monospace;
}

.tokens-text {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #67c23a;
  font-weight: 500;
}

.cost-text {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #e6a23c;
  font-weight: 500;
}

.duration-text {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #909399;
}

/* 选中行样式 */
:deep(.el-table__row.current-row) {
  background-color: #ecf5ff !important;
}

:deep(.el-table__row:hover) {
  cursor: pointer;
}
</style>
