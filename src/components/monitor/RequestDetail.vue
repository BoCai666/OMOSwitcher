<script setup lang="ts">
/**
 * 请求详情组件
 * 显示选中的 LLM 请求的详细信息
 */
import { ref, watch } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import { Document, ChatDotRound, Tools } from '@element-plus/icons-vue'

// 使用状态管理
const store = useMonitorStore()

// 当前激活的标签页
const activeTab = ref('request')

// 格式化 JSON
function formatJSON(data: unknown): string {
  try {
    return JSON.stringify(data, null, 2)
  } catch {
    return String(data)
  }
}

// 格式化请求体（优先使用 parsedBody）
function formatRequestBody(request: { body: unknown; parsedBody?: unknown }): string {
  // 优先使用已解析的 body
  if (request.parsedBody) {
    return formatJSON(request.parsedBody)
  }
  
  // 如果 body 是 Buffer 对象（序列化后为 { type: "Buffer", data: [...] }）
  if (request.body && typeof request.body === 'object') {
    const body = request.body as { type?: string; data?: number[] }
    if (body.type === 'Buffer' && Array.isArray(body.data)) {
      try {
        // 将 Buffer 数组转换为字符串
        const str = String.fromCharCode.apply(null, body.data)
        // 尝试解析为 JSON
        try {
          return formatJSON(JSON.parse(str))
        } catch {
          return str
        }
      } catch {
        return formatJSON(request.body)
      }
    }
  }
  
  return formatJSON(request.body)
}

// 格式化响应体（优先使用 parsedBody）
function formatResponseBody(response: { body: unknown; parsedBody?: unknown }): string {
  // 优先使用已解析的 body
  if (response.parsedBody) {
    return formatJSON(response.parsedBody)
  }
  
  // 如果 body 是 Buffer 对象
  if (response.body && typeof response.body === 'object') {
    const body = response.body as { type?: string; data?: number[] }
    if (body.type === 'Buffer' && Array.isArray(body.data)) {
      try {
        const str = String.fromCharCode.apply(null, body.data)
        try {
          return formatJSON(JSON.parse(str))
        } catch {
          return str
        }
      } catch {
        return formatJSON(response.body)
      }
    }
  }
  
  return formatJSON(response.body)
}

// 格式化时间戳
function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString('zh-CN')
}

// 格式化持续时间
function formatDuration(duration: number | undefined): string {
  if (!duration) return '-'
  if (duration < 1000) {
    return `${duration}ms`
  }
  return `${(duration / 1000).toFixed(2)}s`
}

// 加载详情数据
async function loadDetails() {
  if (store.selectedRequestId) {
    await store.loadSelectedRequestDetails()
  }
}

// 监听选中请求变化
watch(() => store.selectedRequestId, () => {
  activeTab.value = 'request'
  loadDetails()
}, { immediate: true })
</script>

<template>
  <div class="request-detail">
    <!-- 未选中请求时的空状态 -->
    <el-empty
      v-if="!store.selectedRequestId"
      description="点击左侧请求查看详情"
      :image-size="120"
    >
      <template #image>
        <el-icon :size="60" color="#dcdfe6"><Document /></el-icon>
      </template>
    </el-empty>

    <!-- 请求详情内容 -->
    <div v-else class="detail-content">
      <!-- 基本信息 -->
      <el-descriptions :column="2" border size="small" class="basic-info">
        <el-descriptions-item label="请求 ID" :span="2">
          <code class="id-code">{{ store.selectedRequestId }}</code>
        </el-descriptions-item>
        <el-descriptions-item label="时间">
          {{ store.selectedRequest ? formatTime(store.selectedRequest.timestamp) : '-' }}
        </el-descriptions-item>
        <el-descriptions-item label="Provider">
          <el-tag size="small">{{ store.selectedRequest?.provider || '-' }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="模型" :span="2">
          {{ store.selectedRequest?.model || '-' }}
        </el-descriptions-item>
        <el-descriptions-item label="方法">
          <el-tag
            :type="store.selectedRequest?.method === 'POST' ? 'primary' : 'info'"
            size="small"
          >
            {{ store.selectedRequest?.method || '-' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="耗时">
          {{ formatDuration(store.selectedMetrics?.duration) }}
        </el-descriptions-item>
        <el-descriptions-item label="Tokens" v-if="store.selectedMetrics">
          {{ store.selectedMetrics.totalTokens.toLocaleString() }}
        </el-descriptions-item>
        <el-descriptions-item label="费用" v-if="store.selectedMetrics">
          ${{ store.selectedMetrics.estimatedCost.toFixed(4) }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 标签页内容 -->
      <el-tabs v-model="activeTab" class="detail-tabs" type="border-card">
        <!-- 请求体 -->
        <el-tab-pane label="请求体" name="request">
          <template #label>
            <span class="tab-label">
              <el-icon><Document /></el-icon>
              请求体
            </span>
          </template>
          <div class="json-content">
            <pre v-if="store.selectedRequest"><code>{{ formatRequestBody(store.selectedRequest) }}</code></pre>
            <el-empty v-else description="暂无请求数据" />
          </div>
        </el-tab-pane>

        <!-- 响应体 -->
        <el-tab-pane label="响应体" name="response">
          <template #label>
            <span class="tab-label">
              <el-icon><ChatDotRound /></el-icon>
              响应体
            </span>
          </template>
          <div class="json-content">
            <pre v-if="store.selectedResponse"><code>{{ formatResponseBody(store.selectedResponse) }}</code></pre>
            <el-empty v-else description="暂无响应数据" />
          </div>
        </el-tab-pane>

        <!-- MCP 调用 -->
        <el-tab-pane label="MCP 调用" name="mcp">
          <template #label>
            <span class="tab-label">
              <el-icon><Tools /></el-icon>
              MCP 调用
              <el-tag
                v-if="store.selectedMcpCalls.length > 0"
                size="small"
                type="primary"
                class="tab-badge"
              >
                {{ store.selectedMcpCalls.length }}
              </el-tag>
            </span>
          </template>
          <div class="mcp-content">
            <div v-if="store.selectedMcpCalls.length > 0" class="mcp-list">
              <el-collapse>
                <el-collapse-item
                  v-for="(call, index) in store.selectedMcpCalls"
                  :key="call.id"
                  :title="`${index + 1}. ${call.toolName}`"
                >
                  <el-descriptions :column="1" size="small" border>
                    <el-descriptions-item label="工具名称">
                      {{ call.toolName }}
                    </el-descriptions-item>
                    <el-descriptions-item label="工具标题">
                      {{ call.toolTitle || '-' }}
                    </el-descriptions-item>
                    <el-descriptions-item label="Server">
                      {{ call.serverName || '-' }}
                    </el-descriptions-item>
                    <el-descriptions-item label="Transport">
                      <el-tag size="small">{{ call.transportType || 'stdio' }}</el-tag>
                    </el-descriptions-item>
                    <el-descriptions-item label="参数">
                      <pre class="inline-json">{{ formatJSON(call.arguments) }}</pre>
                    </el-descriptions-item>
                    <el-descriptions-item label="结果">
                      <pre class="inline-json">{{ formatJSON(call.resultContent) }}</pre>
                    </el-descriptions-item>
                    <el-descriptions-item label="执行耗时">
                      {{ formatDuration(call.executionDuration) }}
                    </el-descriptions-item>
                  </el-descriptions>
                </el-collapse-item>
              </el-collapse>
            </div>
            <el-empty v-else description="该请求没有 MCP 调用" />
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped>
.request-detail {
  height: 100%;
  padding: 16px;
  overflow-y: auto;
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.basic-info {
  background-color: #fff;
}

.id-code {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #409eff;
}

.detail-tabs {
  flex: 1;
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 4px;
}

.tab-badge {
  margin-left: 4px;
}

.json-content {
  background-color: #f5f7fa;
  border-radius: 4px;
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.json-content pre {
  margin: 0;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: #303133;
  white-space: pre-wrap;
  word-break: break-all;
}

.mcp-content {
  max-height: 400px;
  overflow-y: auto;
}

.mcp-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.inline-json {
  margin: 0;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  line-height: 1.4;
  color: #606266;
  background-color: #f5f7fa;
  padding: 8px;
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

:deep(.el-descriptions__cell) {
  padding: 8px 12px;
}

:deep(.el-collapse-item__header) {
  font-weight: 500;
}

:deep(.el-empty) {
  padding: 40px 0;
}
</style>
