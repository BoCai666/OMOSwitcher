<script setup lang="ts">
/**
 * 统计卡片组件
 * 显示指定时间段的统计数据
 */
import { computed } from 'vue'
import { TrendCharts, Document, Coin, Timer } from '@element-plus/icons-vue'

// 组件属性定义
interface Props {
  title: string // 卡片标题（如：今日、本周、本月）
  stats: {
    requestCount: number
    totalTokens: number
    totalCost: number
  }
}

const props = defineProps<Props>()

// 格式化费用显示
const formattedCost = computed(() => {
  return `$${props.stats.totalCost.toFixed(4)}`
})

// 格式化 Token 数显示
const formattedTokens = computed(() => {
  if (props.stats.totalTokens >= 1000000) {
    return `${(props.stats.totalTokens / 1000000).toFixed(2)}M`
  }
  if (props.stats.totalTokens >= 1000) {
    return `${(props.stats.totalTokens / 1000).toFixed(1)}K`
  }
  return props.stats.totalTokens.toString()
})
</script>

<template>
  <el-card class="stats-card" shadow="hover">
    <template #header>
      <div class="card-header">
        <el-icon class="header-icon"><TrendCharts /></el-icon>
        <span class="header-title">{{ title }}</span>
      </div>
    </template>

    <div class="stats-content">
      <!-- 请求数统计 -->
      <div class="stat-item">
        <div class="stat-icon-wrapper">
          <el-icon><Document /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.requestCount" title="请求数" />
        </div>
      </div>

      <!-- Token 数统计 -->
      <div class="stat-item">
        <div class="stat-icon-wrapper tokens">
          <el-icon><Timer /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="formattedTokens" title="Token 数" />
        </div>
      </div>

      <!-- 费用统计 -->
      <div class="stat-item">
        <div class="stat-icon-wrapper cost">
          <el-icon><Coin /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="formattedCost" title="费用" />
        </div>
      </div>
    </div>
  </el-card>
</template>

<style scoped>
.stats-card {
  height: 100%;
  transition: all 0.3s ease;
}

.stats-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  font-size: 18px;
  color: #409eff;
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background-color: #f5f7fa;
  border-radius: 8px;
  transition: background-color 0.3s ease;
}

.stat-item:hover {
  background-color: #ecf5ff;
}

.stat-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background-color: #409eff;
  border-radius: 8px;
  color: #fff;
  font-size: 20px;
}

.stat-icon-wrapper.tokens {
  background-color: #67c23a;
}

.stat-icon-wrapper.cost {
  background-color: #e6a23c;
}

.stat-info {
  flex: 1;
}

:deep(.el-statistic__content) {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

:deep(.el-statistic__title) {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}
</style>
