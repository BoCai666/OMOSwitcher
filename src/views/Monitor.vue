<script setup lang="ts">
/**
 * 监控页面
 * 监控服务控制和数据展示主页面 - 暗黑科技风格
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import StatsCard from '@/components/monitor/StatsCard.vue'
import RequestList from '@/components/monitor/RequestList.vue'
import RequestDetail from '@/components/monitor/RequestDetail.vue'

// 使用状态管理
const store = useMonitorStore()

// 加载状态
const loading = ref(false)

// 详情弹窗显示状态
const detailDialogVisible = ref(false)

// 处理请求选中 - 打开弹窗
function handleRequestSelected() {
  detailDialogVisible.value = true
}

// 关闭详情弹窗
function closeDetailDialog() {
  detailDialogVisible.value = false
}

// 自动刷新开关
const autoRefresh = ref(false)

// 处理启动监控服务
async function handleStart() {
  loading.value = true
  try {
    await store.startMonitor()
    await store.checkStatus()
    // 启动成功后刷新数据
    await store.refresh()
  } catch (e) {
    console.error('启动监控服务失败:', e)
  } finally {
    loading.value = false
  }
}

// 处理停止监控服务
async function handleStop() {
  loading.value = true
  try {
    await store.stopMonitor()
    await store.checkStatus()
  } catch (e) {
    console.error('停止监控服务失败:', e)
  } finally {
    loading.value = false
  }
}

// 处理刷新
async function handleRefresh() {
  loading.value = true
  try {
    await store.refresh()
  } catch (e) {
    console.error('刷新数据失败:', e)
  } finally {
    loading.value = false
  }
}

// 处理清空数据
async function handleClear() {
  try {
    await store.clearData()
    await store.refresh()
  } catch (e) {
    console.error('清空数据失败:', e)
  }
}

// 切换自动刷新
function toggleAutoRefresh(enabled: boolean) {
  if (enabled) {
    store.startAutoRefresh(5000) // 5秒刷新一次
  } else {
    store.stopAutoRefresh()
  }
}

// 页面加载时检查状态
onMounted(async () => {
  await store.checkStatus()
  if (store.isRunning) {
    await store.refresh()
  }
})

// 页面卸载时停止自动刷新
onUnmounted(() => {
  store.stopAutoRefresh()
})
</script>

<template>
  <div class="monitor-page">
      <!-- 控制卡片 - 玻璃效果 + 状态指示 -->
      <div class="control-card">
        <div class="card-header">
          <div class="header-title">
            <div class="title-icon-wrapper">
              <el-icon class="title-icon"><VideoCamera /></el-icon>
            </div>
            <span class="title-text">监控服务控制</span>
          </div>
          <div class="header-actions">
            <div class="neon-button-group">
              <button
                class="neon-btn neon-btn-start"
                @click="handleStart"
                :disabled="loading || store.isRunning"
              >
                <el-icon class="btn-icon"><VideoPlay /></el-icon>
                <span>启动</span>
              </button>
              <button
                class="neon-btn neon-btn-stop"
                @click="handleStop"
                :disabled="loading || !store.isRunning"
              >
                <el-icon class="btn-icon"><VideoPause /></el-icon>
                <span>停止</span>
              </button>
            </div>
            <div class="divider-vertical"></div>
            <button
              class="neon-btn neon-btn-secondary"
              @click="handleRefresh"
              :disabled="loading"
            >
              <el-icon class="btn-icon"><Refresh /></el-icon>
              <span>刷新</span>
            </button>
            <el-popconfirm
              title="确定要清空所有监控数据吗？此操作不可恢复。"
              confirm-button-text="确定"
              cancel-button-text="取消"
              @confirm="handleClear"
            >
              <template #reference>
                <button class="neon-btn neon-btn-warning">
                  <el-icon class="btn-icon"><Delete /></el-icon>
                  <span>清空</span>
                </button>
              </template>
            </el-popconfirm>
          </div>
        </div>

        <div class="status-section">
          <div class="status-grid">
            <!-- 服务状态 -->
            <div class="status-card">
              <div class="status-indicator" :class="{ active: store.isRunning }">
                <div class="indicator-pulse"></div>
                <div class="indicator-core"></div>
              </div>
              <div class="status-content">
                <div class="status-label">服务状态</div>
                <div class="status-value" :class="{ active: store.isRunning }">
                  {{ store.isRunning ? '运行中' : '已停止' }}
                </div>
              </div>
            </div>

            <!-- 监听端口 -->
            <div class="status-card">
              <div class="status-icon">
                <el-icon><Connection /></el-icon>
              </div>
              <div class="status-content">
                <div class="status-label">监听端口</div>
                <div class="status-value port-value">{{ store.status.port || '-' }}</div>
              </div>
            </div>

            <!-- 自动刷新 -->
            <div class="status-card">
              <div class="status-icon auto-refresh-icon" :class="{ active: autoRefresh }">
                <el-icon><Timer /></el-icon>
              </div>
              <div class="status-content">
                <div class="status-label">自动刷新</div>
                <el-switch
                  v-model="autoRefresh"
                  @change="toggleAutoRefresh"
                  :disabled="!store.isRunning"
                  class="glass-switch"
                />
              </div>
            </div>
          </div>

          <!-- 错误提示 -->
          <div v-if="store.error" class="error-alert">
            <el-icon class="error-icon"><Warning /></el-icon>
            <span>{{ store.error }}</span>
          </div>
        </div>
      </div>

      <!-- 统计卡片行 -->
      <div class="stats-row">
        <div class="stats-col">
          <StatsCard title="今日" :stats="store.todayStats" />
        </div>
        <div class="stats-col">
          <StatsCard title="本周" :stats="store.weekStats" />
        </div>
        <div class="stats-col">
          <StatsCard title="本月" :stats="store.monthStats" />
        </div>
      </div>

      <!-- 请求列表 - 全宽布局 -->
      <div class="content-row">
        <div class="content-col requests-section">
          <div class="section-card">
            <div class="section-header">
              <div class="header-icon-wrapper list-icon">
                <el-icon><List /></el-icon>
              </div>
              <span class="header-title">请求列表</span>
              <div class="header-badge" v-if="store.requests.length > 0">
                {{ store.requests.length }}
              </div>
            </div>
            <div class="section-divider"></div>
            <div class="section-body">
              <RequestList @request-selected="handleRequestSelected" />
            </div>
          </div>
        </div>
      </div>

      <!-- 请求详情弹窗 -->
      <el-dialog
        v-model="detailDialogVisible"
        title="请求详情"
        width="900px"
        :close-on-click-modal="false"
        :close-on-press-escape="true"
        destroy-on-close
        append-to-body
        align-center
        class="detail-dialog"
        @close="closeDetailDialog"
      >
        <RequestDetail />
      </el-dialog>
    </div>
</template>

<style scoped>
/* ==================== CSS 变量 ==================== */
.monitor-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
  background: var(--app-bg-base);
  min-height: 100%;
}

/* ==================== 控制卡片 - 玻璃效果 ==================== */
.control-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 20px;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  overflow: hidden;
  margin-bottom: 24px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.control-card:hover {
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
  border-color: var(--app-color-primary);
}

/* 卡片头部 */
.control-card > .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--app-border-default);
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, transparent 50%);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 14px;
}

.title-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, var(--app-color-primary) 20%, transparent 100%);
  border: 1px solid var(--app-color-primary);
  border-radius: 12px;
  box-shadow:
    0 0 20px rgba(0, 212, 255, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.title-icon {
  font-size: 22px;
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.6));
}

.title-text {
  font-size: 17px;
  font-weight: 600;
  color: var(--app-text-primary);
  letter-spacing: 0.5px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.divider-vertical {
  width: 1px;
  height: 32px;
  background: linear-gradient(180deg, transparent, var(--app-border-default), transparent);
}

/* ==================== 霓虹按钮组 ==================== */
.neon-button-group {
  display: flex;
  gap: 2px;
  background: var(--app-bg-hover);
  border-radius: 12px;
  padding: 4px;
  border: 1px solid var(--app-border-default);
}

.neon-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border: none;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: transparent;
  color: var(--app-text-secondary);
  position: relative;
  overflow: hidden;
}

.neon-btn::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.1), transparent);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.neon-btn:hover::before {
  opacity: 1;
}

.neon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-icon {
  font-size: 16px;
}

/* 启动按钮 */
.neon-btn-start {
  background: linear-gradient(135deg, rgba(0, 245, 160, 0.15), rgba(0, 245, 160, 0.05));
  border: 1px solid rgba(0, 245, 160, 0.3);
  color: var(--app-color-success);
  text-shadow: 0 0 10px rgba(0, 245, 160, 0.5);
}

.neon-btn-start:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(0, 245, 160, 0.25), rgba(0, 245, 160, 0.1));
  box-shadow:
    0 0 20px rgba(0, 245, 160, 0.4),
    0 4px 15px rgba(0, 245, 160, 0.2);
  transform: translateY(-2px);
}

/* 停止按钮 */
.neon-btn-stop {
  background: linear-gradient(135deg, rgba(255, 71, 87, 0.15), rgba(255, 71, 87, 0.05));
  border: 1px solid rgba(255, 71, 87, 0.3);
  color: var(--app-color-danger);
  text-shadow: 0 0 10px rgba(255, 71, 87, 0.5);
}

.neon-btn-stop:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(255, 71, 87, 0.25), rgba(255, 71, 87, 0.1));
  box-shadow:
    0 0 20px rgba(255, 71, 87, 0.4),
    0 4px 15px rgba(255, 71, 87, 0.2);
  transform: translateY(-2px);
}

/* 次要按钮 */
.neon-btn-secondary {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.1), rgba(0, 212, 255, 0.03));
  border: 1px solid rgba(0, 212, 255, 0.25);
  color: var(--app-color-primary);
}

.neon-btn-secondary:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.08));
  box-shadow:
    0 0 20px rgba(0, 212, 255, 0.3),
    0 4px 15px rgba(0, 212, 255, 0.15);
  transform: translateY(-2px);
}

/* 警告按钮 */
.neon-btn-warning {
  background: linear-gradient(135deg, rgba(255, 215, 0, 0.1), rgba(255, 215, 0, 0.03));
  border: 1px solid rgba(255, 215, 0, 0.25);
  color: var(--app-color-warning);
}

.neon-btn-warning:hover {
  background: linear-gradient(135deg, rgba(255, 215, 0, 0.2), rgba(255, 215, 0, 0.08));
  box-shadow:
    0 0 20px rgba(255, 215, 0, 0.3),
    0 4px 15px rgba(255, 215, 0, 0.15);
  transform: translateY(-2px);
}

/* ==================== 状态区域 - 状态指示 ==================== */
.status-section {
  padding: 24px;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-bottom: 20px;
}

.status-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  transition: all 0.3s ease;
}

.status-card:hover {
  background: var(--app-bg-active);
  border-color: var(--app-color-primary);
  transform: translateX(4px);
}

/* 状态指示器 */
.status-indicator {
  position: relative;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.indicator-pulse {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: rgba(128, 128, 128, 0.2);
  border: 2px solid rgba(128, 128, 128, 0.4);
  transition: all 0.3s ease;
}

.indicator-core {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--app-text-disabled);
  box-shadow: 0 0 10px rgba(128, 128, 128, 0.5);
  transition: all 0.3s ease;
}

.status-indicator.active .indicator-pulse {
  background: rgba(0, 245, 160, 0.15);
  border-color: rgba(0, 245, 160, 0.5);
  animation: pulse-ring 2s ease-out infinite;
}

.status-indicator.active .indicator-core {
  background: var(--app-color-success);
  box-shadow:
    0 0 20px rgba(0, 245, 160, 0.8),
    0 0 40px rgba(0, 245, 160, 0.4);
}

@keyframes pulse-ring {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  100% {
    transform: scale(1.6);
    opacity: 0;
  }
}

/* 状态图标 */
.status-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.15), rgba(0, 212, 255, 0.05));
  border: 1px solid rgba(0, 212, 255, 0.25);
  border-radius: 12px;
  font-size: 24px;
  color: var(--app-color-primary);
  transition: all 0.3s ease;
}

.status-icon.auto-refresh-icon.active {
  background: linear-gradient(135deg, rgba(0, 245, 160, 0.15), rgba(0, 245, 160, 0.05));
  border-color: rgba(0, 245, 160, 0.3);
  color: var(--app-color-success);
  animation: rotate-slow 3s linear infinite;
}

@keyframes rotate-slow {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.status-content {
  flex: 1;
}

.status-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 6px;
}

.status-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--app-text-secondary);
  transition: all 0.3s ease;
}

.status-value.active {
  color: var(--app-color-success);
  text-shadow: 0 0 20px rgba(0, 245, 160, 0.5);
}

.port-value {
  color: var(--app-color-primary);
  font-family: 'Consolas', monospace;
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

/* 错误提示 */
.error-alert {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(255, 71, 87, 0.1), rgba(255, 71, 87, 0.03));
  border: 1px solid rgba(255, 71, 87, 0.3);
  border-radius: 12px;
  color: var(--app-color-danger);
  font-size: 14px;
}

.error-icon {
  font-size: 20px;
  filter: drop-shadow(0 0 8px rgba(255, 71, 87, 0.5));
}

/* ==================== 统计行 ==================== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-bottom: 24px;
}

.stats-col {
  min-width: 0;
}

/* ==================== 内容区 - 请求列表全宽布局 ==================== */
.content-row {
  display: block;
}

.content-col {
  width: 100%;
}

/* 区域卡片 */
.section-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.03) 0%, transparent 50%);
}

.header-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  font-size: 18px;
}

.header-icon-wrapper.list-icon {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.15), rgba(0, 212, 255, 0.05));
  border: 1px solid rgba(0, 212, 255, 0.25);
  color: var(--app-color-primary);
}

.header-icon-wrapper.detail-icon {
  background: linear-gradient(135deg, rgba(255, 215, 0, 0.15), rgba(255, 215, 0, 0.05));
  border: 1px solid rgba(255, 215, 0, 0.25);
  color: var(--color-warning);
}

.section-header .header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.header-badge {
  margin-left: auto;
  padding: 4px 12px;
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  color: var(--app-color-primary);
}

.section-divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--app-border-default), transparent);
}

.section-body {
  flex: 1;
  padding: 0;
  overflow: hidden;
}

/* ==================== 玻璃开关 ==================== */
:deep(.glass-switch) {
  --el-switch-on-color: var(--app-color-success);
  --el-switch-off-color: var(--app-bg-hover);
}

:deep(.glass-switch .el-switch__core) {
  border-color: var(--app-border-default);
  background: var(--app-bg-hover);
}

:deep(.glass-switch.is-checked .el-switch__core) {
  background: rgba(0, 245, 160, 0.2);
  border-color: rgba(0, 245, 160, 0.4);
  box-shadow: 0 0 15px rgba(0, 245, 160, 0.3);
}

:deep(.glass-switch .el-switch__action) {
  background: linear-gradient(135deg, var(--app-text-inverse), var(--app-text-tertiary));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

/* ==================== 详情弹窗样式 ==================== */
.detail-dialog :deep(.el-dialog) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.detail-dialog :deep(.el-dialog__header) {
  padding: 16px 24px;
  border-bottom: 1px solid var(--app-border-default);
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, transparent 50%);
  margin-right: 0;
}

.detail-dialog :deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
  font-size: 16px;
}

.detail-dialog :deep(.el-dialog__headerbtn) {
  top: 16px;
  right: 20px;
}

.detail-dialog :deep(.el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-secondary);
  font-size: 20px;
}

.detail-dialog :deep(.el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
}

.detail-dialog :deep(.el-dialog__body) {
  padding: 0;
  flex: 1;
  overflow: hidden;
}

/* ==================== 响应式布局 ==================== */
@media (max-width: 1200px) {
  .status-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .section-card {
    min-height: 400px;
  }
}

@media (max-width: 768px) {
  .monitor-page {
    padding: 12px;
  }
  
  .control-card > .card-header {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }
  
  .header-actions {
    flex-wrap: wrap;
    width: 100%;
  }
  
  .status-grid {
    grid-template-columns: 1fr;
  }
  
  .stats-row {
    grid-template-columns: 1fr;
  }
  
  .neon-button-group {
    order: -1;
    width: 100%;
    margin-bottom: 8px;
  }
  
  .neon-btn {
    flex: 1;
    justify-content: center;
  }
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk .monitor-page {
  background: linear-gradient(180deg, rgba(10, 10, 18, 0.95) 0%, rgba(18, 18, 26, 0.9) 100%);
}

html.cyberpunk .control-card {
  background: rgba(26, 26, 46, 0.85);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 40px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .control-card:hover {
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.6),
    0 0 60px rgba(0, 255, 255, 0.15);
}

html.cyberpunk .control-card > .card-header {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.08) 0%, rgba(255, 0, 255, 0.05) 50%, transparent 100%);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .title-icon-wrapper {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 20%, transparent 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  box-shadow:
    0 0 25px rgba(0, 255, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

html.cyberpunk .title-icon {
  filter: drop-shadow(0 0 12px rgba(0, 255, 255, 0.8));
}

html.cyberpunk .title-text {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .neon-button-group {
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk .neon-btn-start {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.2), rgba(0, 255, 136, 0.05));
  border: 1px solid rgba(0, 255, 136, 0.4);
  text-shadow: 0 0 15px rgba(0, 255, 136, 0.6);
}

html.cyberpunk .neon-btn-start:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.3), rgba(0, 255, 136, 0.1));
  box-shadow:
    0 0 30px rgba(0, 255, 136, 0.5),
    0 0 60px rgba(0, 255, 136, 0.2);
}

html.cyberpunk .neon-btn-stop {
  background: linear-gradient(135deg, rgba(255, 51, 102, 0.2), rgba(255, 51, 102, 0.05));
  border: 1px solid rgba(255, 51, 102, 0.4);
  text-shadow: 0 0 15px rgba(255, 51, 102, 0.6);
}

html.cyberpunk .neon-btn-stop:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(255, 51, 102, 0.3), rgba(255, 51, 102, 0.1));
  box-shadow:
    0 0 30px rgba(255, 51, 102, 0.5),
    0 0 60px rgba(255, 51, 102, 0.2);
}

html.cyberpunk .neon-btn-secondary {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15), rgba(0, 255, 255, 0.05));
  border: 1px solid rgba(0, 255, 255, 0.3);
}

html.cyberpunk .neon-btn-secondary:hover:not(:disabled) {
  box-shadow:
    0 0 30px rgba(0, 255, 255, 0.4),
    0 0 60px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .neon-btn-warning {
  background: linear-gradient(135deg, rgba(255, 170, 0, 0.15), rgba(255, 170, 0, 0.05));
  border: 1px solid rgba(255, 170, 0, 0.3);
}

html.cyberpunk .neon-btn-warning:hover {
  box-shadow:
    0 0 30px rgba(255, 170, 0, 0.4),
    0 0 60px rgba(255, 170, 0, 0.2);
}

html.cyberpunk .status-card {
  background: rgba(26, 26, 46, 0.7);
  border: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .status-card:hover {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 25px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .status-icon {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(0, 255, 255, 0.05));
  border: 1px solid rgba(0, 255, 255, 0.3);
}

html.cyberpunk .status-value.active {
  text-shadow: 0 0 25px rgba(0, 255, 136, 0.6);
}

html.cyberpunk .port-value {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

html.cyberpunk .section-card {
  background: rgba(26, 26, 46, 0.85);
  border: 1px solid rgba(0, 255, 255, 0.15);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

html.cyberpunk .section-header {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.05) 0%, transparent 50%);
}

html.cyberpunk .header-icon-wrapper.list-icon {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2), rgba(0, 255, 255, 0.05));
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .header-icon-wrapper.detail-icon {
  background: linear-gradient(135deg, rgba(255, 170, 0, 0.2), rgba(255, 170, 0, 0.05));
  border: 1px solid rgba(255, 170, 0, 0.3);
  box-shadow: 0 0 15px rgba(255, 170, 0, 0.3);
}

html.cyberpunk .header-badge {
  background: rgba(0, 255, 255, 0.15);
  border: 1px solid rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2);
}

html.cyberpunk :deep(.glass-switch.is-checked .el-switch__core) {
  background: rgba(0, 255, 136, 0.25);
  border-color: rgba(0, 255, 136, 0.5);
  box-shadow: 0 0 20px rgba(0, 255, 136, 0.4);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism .monitor-page {
  background: linear-gradient(180deg, rgba(241, 245, 249, 0.95) 0%, rgba(248, 250, 252, 0.9) 100%);
}

html.glassmorphism .control-card {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid #d1d5db;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

html.glassmorphism .control-card:hover {
  border-color: #93c5fd;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.1);
}

html.glassmorphism .control-card > .card-header {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.06) 0%, transparent 50%);
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .title-icon-wrapper {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: 0 2px 10px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .title-icon {
  color: var(--app-color-primary);
  filter: none;
}

html.glassmorphism .neon-button-group {
  background: #f3f4f6;
  border: 1px solid #d1d5db;
}

html.glassmorphism .neon-btn-start {
  background: var(--app-color-success);
  border: none;
  color: #ffffff;
  text-shadow: none;
}

html.glassmorphism .neon-btn-start:hover:not(:disabled) {
  background: #0ea06e;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

html.glassmorphism .neon-btn-stop {
  background: var(--app-color-danger);
  border: none;
  color: #ffffff;
  text-shadow: none;
}

html.glassmorphism .neon-btn-stop:hover:not(:disabled) {
  background: #dc2626;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

html.glassmorphism .neon-btn-secondary {
  background: #ffffff;
  border: 1px solid #d1d5db;
  color: var(--app-color-primary);
}

html.glassmorphism .neon-btn-secondary:hover:not(:disabled) {
  border-color: var(--app-color-primary);
  box-shadow: none;
}

html.glassmorphism .neon-btn-warning {
  background: #ffffff;
  border: 1px solid #d1d5db;
  color: var(--app-color-warning);
}

html.glassmorphism .neon-btn-warning:hover {
  border-color: var(--app-color-warning);
  box-shadow: none;
}

html.glassmorphism .status-card {
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #e5e7eb;
}

html.glassmorphism .status-card:hover {
  background: #ffffff;
  border-color: #93c5fd;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.1);
}

html.glassmorphism .status-icon {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.3);
}

html.glassmorphism .status-value.active {
  color: var(--app-color-success);
  text-shadow: none;
}

html.glassmorphism .port-value {
  text-shadow: none;
}

html.glassmorphism .section-card {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid #d1d5db;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

html.glassmorphism .section-header {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

html.glassmorphism .header-icon-wrapper.list-icon {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: none;
}

html.glassmorphism .header-icon-wrapper.detail-icon {
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  box-shadow: none;
}

html.glassmorphism .header-badge {
  background: rgba(37, 99, 235, 0.1);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: none;
}

html.glassmorphism :deep(.glass-switch.is-checked .el-switch__core) {
  background: rgba(16, 185, 129, 0.2);
  border-color: var(--app-color-success);
  box-shadow: none;
}

html.glassmorphism .error-alert {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

/* ==================== 明色主题 (html.light - 非玻璃拟态/非暗色) ==================== */
html.light:not(.cyberpunk):not(.dark) .monitor-page {
  background: var(--app-bg-base);
}

html.light:not(.cyberpunk):not(.dark) .control-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  box-shadow: var(--app-shadow-sm);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

html.light:not(.cyberpunk):not(.dark) .control-card:hover {
  border-color: var(--app-color-primary);
  box-shadow: var(--app-shadow-md), 0 0 0 1px rgba(0, 168, 232, 0.1);
}

html.light:not(.cyberpunk):not(.dark) .control-card > .card-header {
  background: var(--app-bg-elevated);
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .title-icon-wrapper {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid var(--app-color-primary);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .title-icon {
  filter: none;
  color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .title-text {
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-group {
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-start {
  background: var(--app-color-success);
  border: 1px solid transparent;
  color: #ffffff;
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-start:hover:not(:disabled) {
  background: #0ea06e;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-stop {
  background: var(--app-color-danger);
  border: 1px solid transparent;
  color: #ffffff;
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-stop:hover:not(:disabled) {
  background: #dc2626;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-secondary {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  color: var(--app-color-primary);
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-secondary:hover:not(:disabled) {
  background: rgba(0, 168, 232, 0.1);
  border-color: var(--app-color-primary);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-warning {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  color: var(--app-color-warning);
}

html.light:not(.cyberpunk):not(.dark) .neon-btn-warning:hover {
  background: rgba(245, 158, 11, 0.1);
  border-color: var(--app-color-warning);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .status-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .status-card:hover {
  background: var(--app-bg-hover);
  border-color: var(--app-color-primary);
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.1);
}

html.light:not(.cyberpunk):not(.dark) .status-icon {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid rgba(0, 168, 232, 0.5);
}

html.light:not(.cyberpunk):not(.dark) .status-value.active {
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .port-value {
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .section-card {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  box-shadow: var(--app-shadow-sm);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

html.light:not(.cyberpunk):not(.dark) .section-card:hover {
  border-color: var(--app-border-hover);
}

html.light:not(.cyberpunk):not(.dark) .section-header {
  background: var(--app-bg-elevated);
}

html.light:not(.cyberpunk):not(.dark) .header-icon-wrapper.list-icon {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid rgba(0, 168, 232, 0.5);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .header-icon-wrapper.detail-icon {
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.5);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .header-badge {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid rgba(0, 168, 232, 0.5);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) :deep(.glass-switch.is-checked .el-switch__core) {
  background: rgba(16, 185, 129, 0.2);
  border-color: var(--app-color-success);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .error-alert {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.5);
}
</style>
