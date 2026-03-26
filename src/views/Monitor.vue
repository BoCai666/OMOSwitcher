<script setup lang="ts">
/**
 * 监控页面
 * 监控服务控制和数据展示主页面
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useMonitorStore } from '@/stores/monitor'
import AppLayout from '@/components/layout/AppLayout.vue'
import StatsCard from '@/components/monitor/StatsCard.vue'
import RequestList from '@/components/monitor/RequestList.vue'
import RequestDetail from '@/components/monitor/RequestDetail.vue'

// 页面标题
const pageTitle = '监控'

// 使用状态管理
const store = useMonitorStore()

// 加载状态
const loading = ref(false)

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
  <AppLayout :title="pageTitle">
    <div class="monitor-page">
      <!-- 控制卡片 -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <div class="header-title">
              <el-icon :size="20"><VideoCamera /></el-icon>
              <span>监控服务控制</span>
            </div>
            <div class="header-actions">
              <el-button-group>
                <el-button
                  type="primary"
                  @click="handleStart"
                  :loading="loading"
                  :disabled="store.isRunning"
                >
                  <el-icon><VideoPlay /></el-icon>
                  启动
                </el-button>
                <el-button
                  type="danger"
                  @click="handleStop"
                  :loading="loading"
                  :disabled="!store.isRunning"
                >
                  <el-icon><VideoPause /></el-icon>
                  停止
                </el-button>
              </el-button-group>
              <el-divider direction="vertical" />
              <el-button @click="handleRefresh" :loading="loading">
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
              <el-popconfirm
                title="确定要清空所有监控数据吗？此操作不可恢复。"
                confirm-button-text="确定"
                cancel-button-text="取消"
                @confirm="handleClear"
              >
                <template #reference>
                  <el-button type="warning">
                    <el-icon><Delete /></el-icon>
                    清空
                  </el-button>
                </template>
              </el-popconfirm>
            </div>
          </div>
        </template>

        <div class="status-info">
          <div class="status-item">
            <span class="status-label">服务状态:</span>
            <el-tag :type="store.isRunning ? 'success' : 'info'" size="large" effect="light">
              <el-icon v-if="store.isRunning"><CircleCheck /></el-icon>
              <el-icon v-else><CircleClose /></el-icon>
              {{ store.isRunning ? '运行中' : '已停止' }}
            </el-tag>
          </div>
          <div class="status-item">
            <span class="status-label">监听端口:</span>
            <el-tag type="info" size="large" effect="plain">
              {{ store.status.port }}
            </el-tag>
          </div>
          <div class="status-item">
            <span class="status-label">自动刷新:</span>
            <el-switch
              v-model="autoRefresh"
              @change="toggleAutoRefresh"
              :disabled="!store.isRunning"
            />
          </div>
          <div v-if="store.error" class="status-item error">
            <el-alert :title="store.error" type="error" :closable="false" show-icon />
          </div>
        </div>
      </el-card>

      <!-- 统计卡片行 -->
      <el-row :gutter="20" class="stats-row">
        <el-col :span="8">
          <StatsCard title="今日" :stats="store.todayStats" />
        </el-col>
        <el-col :span="8">
          <StatsCard title="本周" :stats="store.weekStats" />
        </el-col>
        <el-col :span="8">
          <StatsCard title="本月" :stats="store.monthStats" />
        </el-col>
      </el-row>

      <!-- 请求列表和详情 -->
      <el-row :gutter="20" class="content-row">
        <el-col :span="14">
          <el-card class="requests-card" shadow="hover">
            <template #header>
              <div class="section-header">
                <el-icon><List /></el-icon>
                <span>请求列表</span>
              </div>
            </template>
            <RequestList />
          </el-card>
        </el-col>
        <el-col :span="10">
          <el-card class="detail-card" shadow="hover">
            <template #header>
              <div class="section-header">
                <el-icon><Document /></el-icon>
                <span>请求详情</span>
              </div>
            </template>
            <RequestDetail />
          </el-card>
        </el-col>
      </el-row>
    </div>
  </AppLayout>
</template>

<style scoped>
.monitor-page {
  max-width: 1400px;
  margin: 0 auto;
}

.control-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-info {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: center;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-item.error {
  flex: 1;
  min-width: 300px;
}

.status-label {
  color: #606266;
  font-size: 14px;
}

.stats-row {
  margin-bottom: 20px;
}

.content-row {
  margin-bottom: 20px;
}

.requests-card,
.detail-card {
  height: 600px;
  display: flex;
  flex-direction: column;
}

.requests-card :deep(.el-card__body),
.detail-card :deep(.el-card__body) {
  flex: 1;
  padding: 0;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

/* 响应式布局 */
@media (max-width: 1200px) {
  .stats-row .el-col {
    margin-bottom: 16px;
  }

  .stats-row .el-col:last-child {
    margin-bottom: 0;
  }

  .content-row .el-col {
    margin-bottom: 16px;
  }

  .content-row .el-col:last-child {
    margin-bottom: 0;
  }
}
</style>
