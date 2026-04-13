<script setup lang="ts">
/**
 * 请求详情组件
 * 显示选中的 LLM 请求的详细信息
 */
import { ref, watch } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import { Document, ChatDotRound, Tools, View } from '@element-plus/icons-vue'
import RequestBodyDetailDialog from './RequestBodyDetailDialog.vue'
import ResponseBodyDetailDialog from './ResponseBodyDetailDialog.vue'

// 使用状态管理
const store = useMonitorStore()

// 当前激活的标签页
const activeTab = ref('request')

// MCP 折叠面板激活项
const activeMcpNames = ref<string[]>([])

// 请求体详情弹窗
const bodyDetailVisible = ref(false)

// 响应体详情弹窗
const responseDetailVisible = ref(false)

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
  activeMcpNames.value = []
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
      class="detail-empty"
    >
      <template #image>
        <div class="empty-icon-wrapper">
          <el-icon :size="48" class="empty-icon"><Document /></el-icon>
        </div>
      </template>
    </el-empty>

    <!-- 请求详情内容 -->
    <div v-else class="detail-content">
      <!-- 基本信息 -->
      <div class="basic-info-panel">
        <div class="info-header">
          <span class="info-title">请求详情</span>
          <el-tag size="small" effect="dark" class="id-tag">{{ store.selectedRequestId?.slice(0, 8) }}</el-tag>
        </div>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">时间</span>
            <span class="info-value time-value">{{ store.selectedRequest ? formatTime(store.selectedRequest.timestamp) : '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Provider</span>
            <el-tag size="small" effect="dark" class="provider-tag">{{ store.selectedRequest?.provider || '-' }}</el-tag>
          </div>
          <div class="info-item span-2">
            <span class="info-label">模型</span>
            <span class="info-value model-value">{{ store.selectedRequest?.model || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">方法</span>
            <el-tag
              :type="store.selectedRequest?.method === 'POST' ? 'primary' : 'info'"
              size="small"
              effect="dark"
              class="method-tag"
            >
              {{ store.selectedRequest?.method || '-' }}
            </el-tag>
          </div>
          <div class="info-item">
            <span class="info-label">耗时</span>
            <span class="info-value duration-value">{{ formatDuration(store.selectedMetrics?.duration) }}</span>
          </div>
          <div class="info-item" v-if="store.selectedMetrics">
            <span class="info-label">Tokens</span>
            <span class="info-value tokens-value">{{ store.selectedMetrics.totalTokens.toLocaleString() }}</span>
          </div>
          <div class="info-item" v-if="store.selectedMetrics">
            <span class="info-label">费用</span>
            <span class="info-value cost-value">${{ store.selectedMetrics.estimatedCost.toFixed(4) }}</span>
          </div>
        </div>
      </div>

      <!-- 标签页内容 -->
      <el-tabs v-model="activeTab" class="detail-tabs" type="border-card">
        <!-- 请求体 -->
        <el-tab-pane label="请求体" name="request">
          <template #label>
            <span class="tab-label">
              <el-icon><Document /></el-icon>
              <span>请求体</span>
            </span>
          </template>
          <div class="code-block">
            <div class="code-header">
              <span class="code-title">Request Body</span>
              <div class="code-actions">
                <el-button
                  type="primary"
                  size="small"
                  :icon="View"
                  @click="bodyDetailVisible = true"
                  class="detail-btn"
                >
                  详情
                </el-button>
                <div class="code-dots">
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
              </div>
            </div>
            <div class="code-content">
              <pre v-if="store.selectedRequest"><code class="json-code">{{ formatRequestBody(store.selectedRequest) }}</code></pre>
              <el-empty v-else description="暂无请求数据" class="code-empty" />
            </div>
          </div>
        </el-tab-pane>

        <!-- 响应体 -->
        <el-tab-pane label="响应体" name="response">
          <template #label>
            <span class="tab-label">
              <el-icon><ChatDotRound /></el-icon>
              <span>响应体</span>
            </span>
          </template>
          <div class="code-block">
            <div class="code-header">
              <span class="code-title">Response Body</span>
              <div class="code-actions">
                <el-button
                  type="primary"
                  size="small"
                  :icon="View"
                  @click="responseDetailVisible = true"
                  class="detail-btn"
                >
                  详情
                </el-button>
                <div class="code-dots">
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
              </div>
            </div>
            <div class="code-content">
              <pre v-if="store.selectedResponse"><code class="json-code">{{ formatResponseBody(store.selectedResponse) }}</code></pre>
              <el-empty v-else description="暂无响应数据" class="code-empty" />
            </div>
          </div>
        </el-tab-pane>

        <!-- MCP 调用 -->
        <el-tab-pane label="MCP 调用" name="mcp">
          <template #label>
            <span class="tab-label">
              <el-icon><Tools /></el-icon>
              <span>MCP 调用</span>
              <el-tag
                v-if="store.selectedMcpCalls.length > 0"
                size="small"
                type="primary"
                effect="dark"
                class="tab-badge"
              >
                {{ store.selectedMcpCalls.length }}
              </el-tag>
            </span>
          </template>
          <div class="mcp-content">
            <div v-if="store.selectedMcpCalls.length > 0" class="mcp-list">
              <el-collapse v-model="activeMcpNames" class="mcp-collapse">
                <el-collapse-item
                  v-for="(call, index) in store.selectedMcpCalls"
                  :key="call.id"
                  :name="call.id"
                  class="mcp-item"
                >
                  <template #title>
                    <div class="mcp-title">
                      <span class="mcp-index">{{ index + 1 }}</span>
                      <span class="mcp-tool-name">{{ call.toolName }}</span>
                      <el-tag size="small" effect="dark" class="mcp-server-tag" v-if="call.serverName">
                        {{ call.serverName }}
                      </el-tag>
                    </div>
                  </template>
                  <div class="mcp-details">
                    <div class="mcp-detail-item" v-if="call.toolTitle">
                      <span class="detail-label">工具标题</span>
                      <span class="detail-value">{{ call.toolTitle }}</span>
                    </div>
                    <div class="mcp-detail-item">
                      <span class="detail-label">Transport</span>
                      <el-tag size="small" effect="dark">{{ call.transportType || 'stdio' }}</el-tag>
                    </div>
                    <div class="mcp-detail-item">
                      <span class="detail-label">参数</span>
                      <div class="code-snippet">
                        <pre><code>{{ formatJSON(call.arguments) }}</code></pre>
                      </div>
                    </div>
                    <div class="mcp-detail-item">
                      <span class="detail-label">结果</span>
                      <div class="code-snippet">
                        <pre><code>{{ formatJSON(call.resultContent) }}</code></pre>
                      </div>
                    </div>
                    <div class="mcp-detail-item">
                      <span class="detail-label">执行耗时</span>
                      <span class="detail-value duration">{{ formatDuration(call.executionDuration) }}</span>
                    </div>
                  </div>
                </el-collapse-item>
              </el-collapse>
            </div>
            <el-empty v-else description="该请求没有 MCP 调用" class="mcp-empty" />
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 请求体详情弹窗 -->
    <RequestBodyDetailDialog
      v-model:visible="bodyDetailVisible"
      :request-body="store.selectedRequest?.parsedBody || store.selectedRequest?.body || {}"
      :actual-tokens="store.selectedMetrics?.totalTokens"
    />

    <!-- 响应体详情弹窗 -->
    <ResponseBodyDetailDialog
      v-model:visible="responseDetailVisible"
      :response-body="store.selectedResponse?.body"
      :parsed-body="store.selectedResponse?.parsedBody"
    />
  </div>
</template>

<style scoped>
.request-detail {
  height: auto;
}

/* 空状态 */
.detail-empty :deep(.el-empty__description) {
  color: var(--app-text-tertiary);
  margin-top: 16px;
}

.empty-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100px;
  height: 100px;
  background: var(--app-bg-hover);
  border: 2px dashed var(--app-border-default);
  border-radius: 20px;
}

.empty-icon {
  color: var(--app-text-tertiary);
}

/* 详情内容 */
.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 基本信息面板 */
.basic-info-panel {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--app-border-default);
  border-radius: 12px;
  overflow: hidden;
}

.info-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid var(--app-border-default);
}

.info-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  letter-spacing: 0.5px;
}

.id-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-weight: 600;
  font-size: 12px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1px;
  background: var(--app-border-default);
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 18px;
  background: rgba(0, 0, 0, 0.2);
}

.info-item.span-2 {
  grid-column: span 2;
}

.info-label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.info-value {
  font-size: 14px;
  color: var(--app-text-primary);
  font-weight: 500;
}

.time-value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-text-secondary);
  font-weight: 500;
}

.model-value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-color-primary);
  font-weight: 600;
}

.duration-value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-color-warning);
  font-weight: 600;
}

.tokens-value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-color-success);
  font-weight: 700;
}

.cost-value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-color-warning);
  font-weight: 700;
}

.provider-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.3) !important;
  color: var(--app-color-primary) !important;
  width: fit-content;
}

.method-tag {
  width: fit-content;
}

/* 标签页 */
.detail-tabs {
  border: 1px solid var(--app-border-default);
  border-radius: 12px;
  overflow: hidden;
}

.detail-tabs :deep(.el-tabs__header) {
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--app-border-default);
  margin: 0;
}

.detail-tabs :deep(.el-tabs__nav) {
  border: none !important;
}

.detail-tabs :deep(.el-tabs__item) {
  color: var(--app-text-tertiary);
  border: none !important;
  padding: 0 20px !important;
  height: 44px;
  line-height: 44px;
  transition: all 0.3s ease;
}

.detail-tabs :deep(.el-tabs__item:hover) {
  color: var(--app-text-tertiary);
}

.detail-tabs :deep(.el-tabs__item.is-active) {
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.1);
}

.detail-tabs :deep(.el-tabs__active-bar) {
  background: var(--app-color-primary);
  height: 2px;
  box-shadow: 0 0 10px var(--app-color-primary);
}

.detail-tabs :deep(.el-tabs__content) {
  padding: 16px;
  background: rgba(0, 0, 0, 0.1);
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-badge {
  margin-left: 4px;
  background: rgba(0, 212, 255, 0.2) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

/* 代码块 */
.code-block {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 12px;
  overflow: hidden;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--app-bg-hover);
  border-bottom: 1px solid var(--app-border-default);
}

.code-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.detail-btn {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.15), rgba(0, 212, 255, 0.05));
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: var(--app-color-primary);
  font-weight: 500;
}

.detail-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.25), rgba(0, 212, 255, 0.1));
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

.code-title {
  font-size: 11px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

.code-dots {
  display: flex;
  gap: 6px;
}

.code-dots span {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.code-dots span:nth-child(1) {
  background: #ff5f56;
}

.code-dots span:nth-child(2) {
  background: #ffbd2e;
}

.code-dots span:nth-child(3) {
  background: #27c93f;
}

.code-content {
  max-height: 400px;
  overflow-y: auto;
  padding: 16px;
}

.code-content::-webkit-scrollbar {
  width: 8px;
}

.code-content::-webkit-scrollbar-track {
  background: var(--app-bg-hover);
}

.code-content::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 4px;
}

.code-content::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 212, 255, 0.3);
}

.json-code {
  margin: 0;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  font-weight: 450;
}

/* JSON 语法高亮模拟 */
.json-code :deep(*) {
  color: inherit;
}

.code-empty :deep(.el-empty__description) {
  color: var(--app-text-tertiary);
}

/* MCP 内容 */
.mcp-content {
  max-height: 500px;
  overflow-y: auto;
}

.mcp-content::-webkit-scrollbar {
  width: 8px;
}

.mcp-content::-webkit-scrollbar-track {
  background: var(--app-bg-hover);
}

.mcp-content::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 4px;
}

.mcp-empty :deep(.el-empty__description) {
  color: var(--app-text-tertiary);
}

/* MCP 折叠面板 */
.mcp-collapse {
  border: none;
  background: transparent;
}

.mcp-collapse :deep(.el-collapse-item) {
  margin-bottom: 12px;
  border: 1px solid var(--app-border-default);
  border-radius: 10px;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
}

.mcp-collapse :deep(.el-collapse-item:hover) {
  border-color: rgba(0, 212, 255, 0.3);
}

.mcp-collapse :deep(.el-collapse-item__header) {
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid var(--app-border-default);
  padding: 14px 16px;
  height: auto;
  line-height: 1.5;
  transition: all 0.3s ease;
}

.mcp-collapse :deep(.el-collapse-item__header:hover) {
  background: rgba(0, 212, 255, 0.05);
}

.mcp-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-text-tertiary);
  transition: all 0.3s ease;
}

.mcp-collapse :deep(.el-collapse-item.is-active .el-collapse-item__arrow) {
  color: var(--app-color-primary);
}

.mcp-collapse :deep(.el-collapse-item__wrap) {
  background: transparent;
  border: none;
}

.mcp-collapse :deep(.el-collapse-item__content) {
  padding: 0;
}

.mcp-title {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.mcp-index {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--app-color-primary);
}

.mcp-tool-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-primary);
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
}

.mcp-server-tag {
  background: rgba(0, 245, 160, 0.15) !important;
  border: 1px solid rgba(0, 245, 160, 0.3) !important;
  color: var(--app-color-success) !important;
  margin-left: auto;
}

.mcp-details {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.mcp-detail-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-value {
  font-size: 14px;
  color: var(--app-text-primary);
  font-weight: 500;
}

.detail-value.duration {
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  color: var(--app-color-warning);
  font-weight: 600;
}

.code-snippet {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 8px;
  padding: 12px;
  overflow-x: auto;
}

.code-snippet pre {
  margin: 0;
  font-family: 'SF Mono', 'Menlo', 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--app-text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  font-weight: 450;
}

/* ========== Cyberpunk 主题 ========== */
html.cyberpunk .request-detail {
  /* 弹窗内容样式 */
}

html.cyberpunk .empty-icon-wrapper {
  background: rgba(0, 212, 255, 0.1);
  border-color: rgba(0, 212, 255, 0.4);
  box-shadow: 0 0 25px rgba(0, 212, 255, 0.2);
}

html.cyberpunk .empty-icon {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 10px rgba(0, 212, 255, 0.5));
}

html.cyberpunk .basic-info-panel {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.1), rgba(255, 0, 128, 0.05));
  border-color: rgba(0, 212, 255, 0.35);
}

html.cyberpunk .info-header {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.15), transparent);
  border-bottom-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .id-tag {
  background: rgba(0, 212, 255, 0.2) !important;
  border-color: rgba(0, 212, 255, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .info-item {
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(0, 212, 255, 0.15);
}

html.cyberpunk .provider-tag {
  background: rgba(0, 212, 255, 0.2) !important;
  border-color: rgba(0, 212, 255, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .model-value,
html.cyberpunk .tokens-value,
html.cyberpunk .cost-value,
html.cyberpunk .duration-value {
  text-shadow: 0 0 10px currentColor;
}

html.cyberpunk .detail-tabs :deep(.el-tabs__header) {
  background: linear-gradient(180deg, rgba(0, 212, 255, 0.12), rgba(0, 0, 0, 0.2));
  border-bottom-color: rgba(0, 212, 255, 0.3);
}

html.cyberpunk .detail-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(0, 212, 255, 0.15);
  color: var(--app-color-primary);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);
}

html.cyberpunk .detail-tabs :deep(.el-tabs__active-bar) {
  box-shadow: 0 0 20px var(--app-color-primary), 0 0 40px rgba(255, 0, 128, 0.3);
}

html.cyberpunk .detail-tabs :deep(.el-tabs__content) {
  background: rgba(0, 0, 0, 0.3);
}

html.cyberpunk .tab-badge {
  background: rgba(0, 212, 255, 0.25) !important;
  border-color: rgba(0, 212, 255, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .code-block {
  background: linear-gradient(135deg, rgba(10, 15, 30, 0.9), rgba(5, 10, 25, 0.95));
  border-color: rgba(0, 212, 255, 0.35);
  box-shadow: inset 0 0 30px rgba(0, 212, 255, 0.05);
}

html.cyberpunk .code-header {
  background: rgba(0, 212, 255, 0.1);
  border-bottom-color: rgba(0, 212, 255, 0.25);
}

html.cyberpunk .detail-btn {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(0, 255, 255, 0.08));
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .detail-btn:hover {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.3), rgba(0, 255, 255, 0.12));
  border-color: rgba(0, 255, 255, 0.6);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .mcp-collapse :deep(.el-collapse-item) {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.08), rgba(255, 0, 128, 0.04));
  border-color: rgba(0, 212, 255, 0.35);
}

html.cyberpunk .mcp-collapse :deep(.el-collapse-item:hover) {
  border-color: rgba(0, 212, 255, 0.6);
  box-shadow: 0 0 25px rgba(0, 212, 255, 0.2);
}

html.cyberpunk .mcp-collapse :deep(.el-collapse-item__header) {
  background: rgba(0, 212, 255, 0.1);
  border-bottom-color: rgba(0, 212, 255, 0.25);
}

html.cyberpunk .mcp-index {
  background: rgba(0, 212, 255, 0.2);
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

html.cyberpunk .mcp-server-tag {
  background: rgba(0, 245, 160, 0.2) !important;
  border-color: rgba(0, 245, 160, 0.5) !important;
  box-shadow: 0 0 15px rgba(0, 245, 160, 0.3);
}

html.cyberpunk .code-snippet {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.25);
}

/* ========== Glassmorphism 主题 ========== */
html.glassmorphism .request-detail {
  /* 弹窗内容样式 */
}

html.glassmorphism .empty-icon-wrapper {
  background: #f3f4f6;
  border: 2px dashed #d1d5db;
}

html.glassmorphism .empty-icon {
  color: var(--app-text-tertiary);
}

html.glassmorphism .basic-info-panel {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
}

html.glassmorphism .info-header {
  background: #f3f4f6;
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .id-tag {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.glassmorphism .info-grid {
  background: #e5e7eb;
}

html.glassmorphism .info-item {
  background: #ffffff;
}

html.glassmorphism .provider-tag {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.glassmorphism .model-value,
html.glassmorphism .tokens-value,
html.glassmorphism .cost-value,
html.glassmorphism .duration-value {
  text-shadow: none;
}

html.glassmorphism .detail-tabs :deep(.el-tabs__header) {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .detail-tabs :deep(.el-tabs__item) {
  color: var(--app-text-secondary);
}

html.glassmorphism .detail-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(37, 99, 235, 0.08);
  color: var(--app-color-primary);
}

html.glassmorphism .detail-tabs :deep(.el-tabs__active-bar) {
  box-shadow: none;
}

html.glassmorphism .detail-tabs :deep(.el-tabs__content) {
  background: #fafafa;
}

html.glassmorphism .tab-badge {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.3) !important;
  color: var(--app-color-primary) !important;
}

html.glassmorphism .code-block {
  background: #ffffff;
  border: 1px solid #e5e7eb;
}

html.glassmorphism .code-header {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .detail-btn {
  background: var(--app-color-primary);
  border: none;
  color: #ffffff;
}

html.glassmorphism .detail-btn:hover {
  background: #1d4ed8;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

html.glassmorphism .mcp-collapse :deep(.el-collapse-item) {
  background: #ffffff;
  border: 1px solid #e5e7eb;
}

html.glassmorphism .mcp-collapse :deep(.el-collapse-item:hover) {
  border-color: #93c5fd;
}

html.glassmorphism .mcp-collapse :deep(.el-collapse-item__header) {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .mcp-index {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.3);
  color: var(--app-color-primary);
}

html.glassmorphism .mcp-server-tag {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

html.glassmorphism .code-snippet {
  background: #ffffff;
  border: 1px solid #e5e7eb;
}

/* ========== 明色主题 (html.light - 非玻璃拟态/非暗色) ========== */
html.light:not(.cyberpunk):not(.dark) .request-detail {
  /* 弹窗内容样式 */
}

html.light:not(.cyberpunk):not(.dark) .empty-icon-wrapper {
  background: var(--app-bg-hover);
  border: 2px dashed var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .empty-icon {
  color: var(--app-text-tertiary);
}

html.light:not(.cyberpunk):not(.dark) .basic-info-panel {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .info-header {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .id-tag {
  background: rgba(0, 168, 232, 0.1) !important;
  border: 1px solid rgba(0, 168, 232, 0.5) !important;
  color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .info-grid {
  background: var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .info-item {
  background: var(--app-bg-card);
}

html.light:not(.cyberpunk):not(.dark) .provider-tag {
  background: rgba(0, 168, 232, 0.1) !important;
  border: 1px solid rgba(0, 168, 232, 0.5) !important;
  color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .model-value,
html.light:not(.cyberpunk):not(.dark) .tokens-value,
html.light:not(.cyberpunk):not(.dark) .cost-value,
html.light:not(.cyberpunk):not(.dark) .duration-value {
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .detail-tabs :deep(.el-tabs__header) {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .detail-tabs :deep(.el-tabs__item) {
  color: var(--app-text-secondary);
}

html.light:not(.cyberpunk):not(.dark) .detail-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(0, 168, 232, 0.08);
  color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .detail-tabs :deep(.el-tabs__active-bar) {
  background: var(--app-color-primary);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .detail-tabs :deep(.el-tabs__content) {
  background: var(--app-bg-base);
}

html.light:not(.cyberpunk):not(.dark) .tab-badge {
  background: rgba(0, 168, 232, 0.1) !important;
  border: 1px solid rgba(0, 168, 232, 0.5) !important;
  color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .code-block {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .code-header {
  background: var(--app-bg-hover);
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .detail-btn {
  background: var(--app-color-primary);
  border: none;
  color: #ffffff;
}

html.light:not(.cyberpunk):not(.dark) .detail-btn:hover {
  background: var(--app-color-primary-hover);
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .mcp-collapse :deep(.el-collapse-item) {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .mcp-collapse :deep(.el-collapse-item:hover) {
  border-color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .mcp-collapse :deep(.el-collapse-item__header) {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .mcp-index {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid rgba(0, 168, 232, 0.5);
  color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .mcp-server-tag {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.5) !important;
  color: var(--app-color-success) !important;
}

html.light:not(.cyberpunk):not(.dark) .code-snippet {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
}

/* ========== 暗色主题 ========== */
html.dark .request-detail {
  /* 弹窗内容样式 */
}

html.dark .empty-icon-wrapper {
  background: rgba(0, 212, 255, 0.1);
  border: 2px dashed rgba(0, 212, 255, 0.3);
}

html.dark .empty-icon {
  color: var(--app-color-primary);
}

html.dark .basic-info-panel {
  background: rgba(26, 26, 46, 0.6);
  border: 1px solid var(--app-border-default);
}

html.dark .info-header {
  background: rgba(0, 212, 255, 0.05);
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .id-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.dark .info-grid {
  background: var(--app-border-default);
}

html.dark .info-item {
  background: var(--app-bg-card);
}

html.dark .provider-tag {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.dark .detail-tabs :deep(.el-tabs__header) {
  background: rgba(0, 212, 255, 0.05);
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .detail-tabs :deep(.el-tabs__item) {
  color: var(--app-text-secondary);
}

html.dark .detail-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(0, 212, 255, 0.1);
  color: var(--app-color-primary);
}

html.dark .detail-tabs :deep(.el-tabs__active-bar) {
  background: var(--app-color-primary);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.4);
}

html.dark .detail-tabs :deep(.el-tabs__content) {
  background: rgba(0, 0, 0, 0.2);
}

html.dark .tab-badge {
  background: rgba(0, 212, 255, 0.15) !important;
  border: 1px solid rgba(0, 212, 255, 0.4) !important;
  color: var(--app-color-primary) !important;
}

html.dark .code-block {
  background: rgba(26, 26, 46, 0.6);
  border: 1px solid var(--app-border-default);
}

html.dark .code-header {
  background: rgba(0, 212, 255, 0.05);
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .detail-btn {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.1));
  border: 1px solid rgba(0, 212, 255, 0.4);
  color: var(--app-color-primary);
}

html.dark .detail-btn:hover {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.3), rgba(0, 212, 255, 0.15));
  border-color: rgba(0, 212, 255, 0.6);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

html.dark .mcp-collapse :deep(.el-collapse-item) {
  background: rgba(26, 26, 46, 0.6);
  border: 1px solid var(--app-border-default);
}

html.dark .mcp-collapse :deep(.el-collapse-item:hover) {
  border-color: rgba(0, 212, 255, 0.4);
}

html.dark .mcp-collapse :deep(.el-collapse-item__header) {
  background: rgba(0, 212, 255, 0.05);
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .mcp-index {
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.4);
  color: var(--app-color-primary);
}

html.dark .mcp-server-tag {
  background: rgba(16, 185, 129, 0.15) !important;
  border: 1px solid rgba(16, 185, 129, 0.4) !important;
  color: var(--app-color-success) !important;
}

html.dark .code-snippet {
  background: rgba(26, 26, 46, 0.6);
  border: 1px solid var(--app-border-default);
}
</style>
