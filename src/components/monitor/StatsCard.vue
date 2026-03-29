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
  <el-card class="stats-card" shadow="never">
    <template #header>
      <div class="card-header">
        <div class="header-icon-wrapper">
          <el-icon class="header-icon"><TrendCharts /></el-icon>
        </div>
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
          <div class="stat-value neon-text">{{ stats.requestCount }}</div>
          <div class="stat-label">请求数</div>
        </div>
      </div>

      <!-- Token 数统计 -->
      <div class="stat-item">
        <div class="stat-icon-wrapper tokens">
          <el-icon><Timer /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value neon-text">{{ formattedTokens }}</div>
          <div class="stat-label">Token 数</div>
        </div>
      </div>

      <!-- 费用统计 -->
      <div class="stat-item">
        <div class="stat-icon-wrapper cost">
          <el-icon><Coin /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value neon-text">{{ formattedCost }}</div>
          <div class="stat-label">费用</div>
        </div>
      </div>
    </div>
  </el-card>
</template>

<style scoped>
.stats-card {
  height: 100%;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.stats-card:hover {
  transform: translateY(-4px);
  box-shadow:
    0 16px 48px rgba(0, 212, 255, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  border-color: rgba(0, 212, 255, 0.3);
}

:deep(.el-card__header) {
  border-bottom: 1px solid var(--app-border-default);
  padding: 16px 20px;
}

:deep(.el-card__body) {
  padding: 20px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.05));
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 10px;
  box-shadow:
    0 0 20px rgba(0, 212, 255, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.header-icon {
  font-size: 18px;
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.6));
}

.header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-primary);
  letter-spacing: 0.5px;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: var(--app-bg-hover);
  border: 1px solid var(--app-border-default);
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-item:hover {
  background: var(--app-bg-active);
  border-color: var(--app-color-primary);
  transform: translateX(4px);
}

.stat-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, var(--app-color-primary) 0%, color-mix(in srgb, var(--app-color-primary) 80%, black) 100%);
  border-radius: 12px;
  color: var(--app-text-inverse);
  font-size: 22px;
  box-shadow:
    0 4px 15px rgba(0, 212, 255, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
  transition: all 0.3s ease;
}

.stat-icon-wrapper.tokens {
  background: linear-gradient(135deg, var(--app-color-success) 0%, color-mix(in srgb, var(--app-color-success) 80%, black) 100%);
  box-shadow:
    0 4px 15px rgba(0, 245, 160, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stat-icon-wrapper.cost {
  background: linear-gradient(135deg, var(--app-color-warning) 0%, color-mix(in srgb, var(--app-color-warning) 80%, black) 100%);
  box-shadow:
    0 4px 15px rgba(255, 215, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stat-item:hover .stat-icon-wrapper {
  transform: scale(1.05);
  box-shadow:
    0 6px 20px rgba(0, 212, 255, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stat-item:hover .stat-icon-wrapper.tokens {
  box-shadow:
    0 6px 20px rgba(0, 245, 160, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stat-item:hover .stat-icon-wrapper.cost {
  box-shadow:
    0 6px 20px rgba(255, 215, 0, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 26px;
  font-weight: 700;
  line-height: 1.2;
  margin-bottom: 4px;
}

.neon-text {
  color: var(--app-color-primary);
  text-shadow:
    0 0 10px rgba(0, 212, 255, 0.8),
    0 0 20px rgba(0, 212, 255, 0.5),
    0 0 30px rgba(0, 212, 255, 0.3);
}

.stat-label {
  font-size: 12px;
  color: var(--app-text-secondary);
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

/* ========== Cyberpunk 主题 ========== */
html.cyberpunk .stats-card {
  background: linear-gradient(135deg, rgba(10, 15, 30, 0.95), rgba(5, 10, 25, 0.98));
  border: 1px solid rgba(0, 212, 255, 0.4);
  box-shadow:
    0 0 30px rgba(0, 212, 255, 0.2),
    inset 0 1px 0 rgba(0, 212, 255, 0.1);
}

html.cyberpunk .stats-card:hover {
  border-color: rgba(0, 212, 255, 0.7);
  box-shadow:
    0 0 50px rgba(0, 212, 255, 0.3),
    0 0 100px rgba(255, 0, 128, 0.15),
    inset 0 1px 0 rgba(0, 212, 255, 0.2);
}

html.cyberpunk .header-icon-wrapper {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.25), rgba(255, 0, 128, 0.15));
  border-color: rgba(0, 212, 255, 0.6);
  box-shadow:
    0 0 25px rgba(0, 212, 255, 0.4),
    inset 0 0 15px rgba(0, 212, 255, 0.1);
}

html.cyberpunk .header-icon {
  filter: drop-shadow(0 0 10px rgba(0, 212, 255, 0.8));
}

html.cyberpunk .stat-item {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.08), transparent);
  border-color: rgba(0, 212, 255, 0.25);
}

html.cyberpunk .stat-item:hover {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.15), rgba(255, 0, 128, 0.08));
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.2);
}

html.cyberpunk .neon-text {
  text-shadow:
    0 0 5px rgba(0, 212, 255, 1),
    0 0 15px rgba(0, 212, 255, 0.8),
    0 0 30px rgba(0, 212, 255, 0.5),
    0 0 50px rgba(255, 0, 128, 0.3);
}

/* ========== Glassmorphism 主题 ========== */
html.glassmorphism .stats-card {
  background: var(--app-glass-bg, rgba(255, 255, 255, 0.25));
  border: 1px solid rgba(255, 255, 255, 0.3);
  backdrop-filter: blur(30px);
  -webkit-backdrop-filter: blur(30px);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

html.glassmorphism .stats-card:hover {
  background: rgba(255, 255, 255, 0.32);
  border-color: rgba(255, 255, 255, 0.5);
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  transform: translateY(-6px);
}

html.glassmorphism .header-icon-wrapper {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.4), rgba(255, 255, 255, 0.2));
  border-color: rgba(255, 255, 255, 0.5);
  box-shadow:
    0 4px 15px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
}

html.glassmorphism .header-icon {
  filter: none;
}

html.glassmorphism .stat-item {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.25);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

html.glassmorphism .stat-item:hover {
  background: rgba(255, 255, 255, 0.25);
  border-color: rgba(255, 255, 255, 0.4);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

html.glassmorphism .stat-icon-wrapper {
  background: linear-gradient(135deg, var(--app-color-primary), color-mix(in srgb, var(--app-color-primary) 70%, white));
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
}

html.glassmorphism .neon-text {
  text-shadow: none;
  color: var(--app-text-primary);
}
</style>
