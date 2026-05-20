<script setup lang="ts">
/**
 * 额度卡片组件
 * 展示单个供应商的额度/余额信息
 */
import { computed } from 'vue'
import { WarningFilled, Setting } from '@element-plus/icons-vue'
import type { ProviderQuota } from '@/types/quota'
import {
  getProviderMeta,
  getBalancePercentage,
  getProgressColor,
  formatBalance,
  formatTokens,
  formatResetTime
} from '@/composables/useQuotaFormatter'
import { useTheme } from '@/composables/useTheme'

const props = defineProps<{
  quota: ProviderQuota
}>()

// 供应商元数据（缓存，避免重复调用）
const providerMeta = computed(() => getProviderMeta(props.quota.providerId))

// 暗色模式下，暗色品牌供应商使用灰色替代色
const { isDark } = useTheme()
const providerEffectiveColor = computed(() => {
  if (isDark.value && providerMeta.value.darkColor) return providerMeta.value.darkColor
  return providerMeta.value.color
})

// 纯余额型供应商：只有 totalBalance，没有 usedBalance/resetTime（如 DeepSeek）
const isPureBalance = computed(() =>
  props.quota.quotaType === 'balance' &&
  props.quota.usedBalance == null &&
  props.quota.resetTime == null
)

// OpenCode Go：三维度额度（滚动/周/月）
const isOpenCodeGo = computed(() =>
  props.quota.providerId.toLowerCase().includes('opencode') &&
  props.quota.providerId.toLowerCase().includes('go')
)

const emit = defineEmits<{
  retry: [quota: ProviderQuota]
  detail: [quota: ProviderQuota]
  settings: [quota: ProviderQuota]
}>()
</script>

<template>
  <div
    class="quota-card"
    :style="{
      '--provider-color': providerEffectiveColor
    }"
  >
    <!-- 底部发光线装饰 -->
    <div class="card-glow-line"></div>

    <!-- 加载状态 -->
    <div v-if="quota.status === 'loading'" class="card-body">
      <el-skeleton :rows="4" animated />
    </div>

    <!-- 错误状态 -->
    <div v-else-if="quota.status === 'error'" class="card-body error-state">
      <!-- OpenCodeGo: 显示头部和设置按钮 -->
      <div v-if="isOpenCodeGo" class="card-header-row error-header-row">
        <div class="provider-icon-badge" :style="{ background: providerMeta.gradient || providerEffectiveColor }">
          <svg :viewBox="providerMeta.iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
            <path :d="providerMeta.iconPath" />
          </svg>
        </div>
        <span class="provider-name">{{ quota.providerName }}</span>
        <span class="header-spacer"></span>
        <button
          class="settings-btn"
          title="设置额度查询参数"
          @click.stop="emit('settings', quota)"
        >
          <el-icon :size="16"><Setting /></el-icon>
        </button>
      </div>
      <div class="error-content">
        <el-icon class="error-icon"><WarningFilled /></el-icon>
        <p class="error-text">{{ quota.errorMessage || '查询失败' }}</p>
        <el-button size="small" @click="emit('retry', quota)">重试</el-button>
      </div>
    </div>

    <!-- 不支持额度查询 -->
    <div v-else-if="quota.quotaType === 'unsupported'" class="card-body unsupported-state">
      <div class="card-header-row">
        <div class="provider-icon-badge" :style="{ background: providerMeta.gradient || providerEffectiveColor }">
          <svg :viewBox="providerMeta.iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
            <path :d="providerMeta.iconPath" />
          </svg>
        </div>
        <span class="provider-name">{{ quota.providerName }}</span>
        <span class="header-spacer"></span>
        <button
          v-if="isOpenCodeGo"
          class="settings-btn"
          title="设置额度查询参数"
          @click.stop="emit('settings', quota)"
        >
          <el-icon :size="16"><Setting /></el-icon>
        </button>
      </div>
      <el-empty
        description="该供应商暂不支持额度查询"
        :image-size="64"
        class="unsupported-empty"
      />
    </div>

    <!-- 余额型 -->
    <div v-else-if="quota.quotaType === 'balance'" class="card-body clickable" @click="emit('detail', quota)">
      <div class="card-header-row">
        <div class="provider-icon-badge" :style="{ background: providerMeta.gradient || providerEffectiveColor }">
          <svg :viewBox="providerMeta.iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
            <path :d="providerMeta.iconPath" />
          </svg>
        </div>
        <span class="provider-name">{{ quota.providerName }}</span>
        <span class="header-spacer"></span>
        <button
          v-if="isOpenCodeGo"
          class="settings-btn"
          title="设置额度查询参数"
          @click.stop="emit('settings', quota)"
        >
          <el-icon :size="16"><Setting /></el-icon>
        </button>
      </div>
      <div class="balance-main">
        <span class="balance-label">{{ isPureBalance ? '余额' : '配额使用' }}</span>
        <span class="balance-value">
          {{ isPureBalance ? formatBalance(quota.totalBalance, quota.currency) : getBalancePercentage(quota).toFixed(1) + '%' }}
        </span>
        <span class="balance-detail">
          {{ formatBalance(isPureBalance ? null : quota.usedBalance, quota.currency) }} / {{ formatBalance(quota.totalBalance, quota.currency) }}
        </span>
      </div>
      <el-progress
        :percentage="isPureBalance ? ((quota.totalBalance ?? 0) > 0 ? 100 : 0) : getBalancePercentage(quota)"
        :color="getProgressColor(isPureBalance ? ((quota.totalBalance ?? 0) > 0 ? 100 : 0) : getBalancePercentage(quota))"
        :stroke-width="8"
        :show-text="false"
        class="quota-progress"
      />
      <div class="reset-badge" :class="{ 'reset-badge-placeholder': !quota.resetTime }">
        <template v-if="quota.resetTime">{{ formatResetTime(quota.resetTime) }}</template>
        <template v-else>暂无重置时间信息</template>
      </div>
    </div>

    <!-- 配额型 (token_limit) -->
    <div v-else-if="quota.quotaType === 'token_limit'" class="card-body clickable" @click="emit('detail', quota)">
      <div class="card-header-row">
        <div class="provider-icon-badge" :style="{ background: providerMeta.gradient || providerEffectiveColor }">
          <svg :viewBox="providerMeta.iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
            <path :d="providerMeta.iconPath" />
          </svg>
        </div>
        <span class="provider-name">{{ quota.providerName }}</span>
        <span class="header-spacer"></span>
        <button
          v-if="isOpenCodeGo"
          class="settings-btn"
          title="设置额度查询参数"
          @click.stop="emit('settings', quota)"
        >
          <el-icon :size="16"><Setting /></el-icon>
        </button>
      </div>
      <div class="balance-main">
        <span class="balance-label">配额使用</span>
        <span class="balance-value">
          {{ quota.quotaPercentage != null ? `${quota.quotaPercentage.toFixed(1)}%` : '--' }}
        </span>
        <span class="balance-detail">
          {{ quota.quotaUsed != null && quota.quotaLimit != null ? `${formatTokens(quota.quotaUsed)} / ${formatTokens(quota.quotaLimit)}` : '--/--' }}
        </span>
      </div>
      <el-progress
        :percentage="quota.quotaPercentage ?? 0"
        :color="getProgressColor(quota.quotaPercentage ?? 0)"
        :stroke-width="8"
        :show-text="false"
        class="quota-progress"
      />
      <div class="reset-badge" :class="{ 'reset-badge-placeholder': !quota.resetTime }">
        <template v-if="quota.resetTime">{{ formatResetTime(quota.resetTime) }}</template>
        <template v-else>5小时滚动窗口</template>
      </div>
    </div>

    <!-- Kimi Code 专用展示 -->
    <div v-else-if="quota.quotaType === 'token_limit' && quota.isKimiCode" class="card-body clickable" @click="emit('detail', quota)">
      <div class="card-header-row">
        <div class="provider-icon-badge" :style="{ background: providerMeta.gradient || providerEffectiveColor }">
          <svg :viewBox="providerMeta.iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
            <path :d="providerMeta.iconPath" />
          </svg>
        </div>
        <span class="provider-name">{{ quota.providerName }}</span>
      </div>
      <div class="balance-main">
        <span class="balance-label">5小时额度</span>
        <span class="balance-value">
          {{ quota.quotaPercentage != null ? `${quota.quotaPercentage.toFixed(1)}%` : '--' }}
        </span>
        <span class="balance-detail">
          {{ formatTokens(quota.quotaUsed) }} / {{ formatTokens(quota.quotaLimit) }}
        </span>
      </div>
      <el-progress
        :percentage="quota.quotaPercentage ?? 0"
        :color="getProgressColor(quota.quotaPercentage ?? 0)"
        :stroke-width="8"
        :show-text="false"
        class="quota-progress"
      />
      <div class="reset-badge" :class="{ 'reset-badge-placeholder': !quota.resetTime }">
        <template v-if="quota.resetTime">{{ formatResetTime(quota.resetTime) }}</template>
        <template v-else>暂无重置时间信息</template>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ==================== 额度卡片 ==================== */
.quota-card {
  position: relative;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-top: 3px solid var(--provider-color, var(--app-color-primary));
  border-radius: 16px;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: var(--app-shadow-sm);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 品牌色渐变背景层（真实 DOM，不走伪元素） */
.card-brand-bg {
  position: absolute;
  inset: 0;
  background: var(--provider-gradient, var(--provider-color, var(--app-color-primary)));
  opacity: 0.08;
  pointer-events: none;
  z-index: 0;
  transition: opacity 0.3s ease;
}

.quota-card:hover .card-brand-bg {
  opacity: 0.15;
}

.quota-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--app-shadow-hover);
  border-color: var(--provider-color, var(--app-color-primary));
  border-top-color: var(--provider-color, var(--app-color-primary));
}

/* 底部发光线装饰 */
.card-glow-line {
  position: absolute;
  bottom: 0;
  left: 10%;
  right: 10%;
  height: 2px;
  background: var(--provider-color, var(--app-color-primary));
  border-radius: 2px;
  opacity: 0.3;
  pointer-events: none;
  z-index: 2;
  box-shadow: 0 0 8px var(--provider-color, var(--app-color-primary));
  transition: all 0.3s ease;
}

.quota-card:hover .card-glow-line {
  opacity: 0.7;
  left: 5%;
  right: 5%;
  height: 3px;
  box-shadow: 0 0 12px var(--provider-color, var(--app-color-primary));
}

/* ==================== 卡片内容 ==================== */
.card-body {
  position: relative;
  z-index: 1;
  padding: 20px;
  min-height: 170px;
  display: flex;
  flex-direction: column;
}

/* 供应商头部行 */
.card-header-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.provider-icon-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  color: #ffffff;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  line-height: 1.3;
}

.header-spacer {
  flex: 1;
}

/* 设置按钮（齿轮图标） */
.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--app-border-default);
  border-radius: 6px;
  background: transparent;
  color: var(--app-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
  padding: 0;
}

.settings-btn:hover {
  background: var(--app-color-primary);
  border-color: var(--app-color-primary);
  color: #ffffff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.settings-btn:active {
  transform: scale(0.95);
}

/* 余额主区域 */
.balance-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 14px;
  flex: 1;
}

.balance-label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  letter-spacing: 1px;
  text-transform: uppercase;
  font-weight: 500;
}

.balance-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--app-text-primary);
  font-family: 'Consolas', 'Monaco', 'JetBrains Mono', monospace;
  line-height: 1.2;
  letter-spacing: -0.5px;
}

.balance-detail {
  font-size: 12px;
  color: var(--app-text-tertiary);
  font-family: 'Consolas', 'Monaco', monospace;
  margin-top: 2px;
}

/* 进度条 */
.quota-progress {
  margin-bottom: 12px;
}

.quota-progress :deep(.el-progress-bar__outer) {
  border-radius: 6px;
  background: var(--app-bg-hover);
  height: 8px;
}

.quota-progress :deep(.el-progress-bar__inner) {
  border-radius: 6px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 重置时间徽章 */
.reset-badge {
  margin-top: auto;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  color: var(--app-text-tertiary);
  background: var(--app-bg-hover);
  display: inline-block;
  width: fit-content;
}

/* 无重置时间时的占位提示 */
.reset-badge-placeholder {
  color: var(--app-text-quaternary);
  font-style: italic;
}

/* ==================== 错误状态 ==================== */
.error-state {
  align-items: center;
  justify-content: center;
}

/* OpenCodeGo 错误状态头部覆盖 */
.error-header-row {
  width: 100%;
  margin-bottom: 12px;
  align-items: center;
}

.error-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
}

.error-icon {
  font-size: 32px;
  color: var(--app-color-danger);
}

.error-text {
  font-size: 13px;
  color: var(--app-text-secondary);
  margin: 0;
}

/* ==================== 不支持状态 ==================== */
.unsupported-state {
  align-items: center;
}

.unsupported-empty {
  padding: 0;
}

.unsupported-empty :deep(.el-empty__description p) {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

/* ==================== 骨架屏 ==================== */
.card-body :deep(.el-skeleton) {
  width: 100%;
}

/* ==================== 可点击卡片 ==================== */
.card-body.clickable {
  cursor: pointer;
}

.card-body.clickable:hover {
  background: color-mix(in srgb, var(--provider-color, var(--app-color-primary)) 5%, transparent);
}

.card-body.clickable:active {
  background: color-mix(in srgb, var(--provider-color, var(--app-color-primary)) 10%, transparent);
  transform: scale(0.99);
}

/* ============================================================
   赛博朋克主题 - 卡片样式
   ============================================================ */

/* 卡片 - 赛博朋克 */
html.cyberpunk .quota-card {
  background: rgba(18, 18, 31, 0.85);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-top: 3px solid var(--provider-color, #00ffff);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
}

html.cyberpunk .card-brand-bg {
  opacity: 0.12;
}

html.cyberpunk .quota-card:hover {
  transform: translateY(-4px);
  border-color: var(--provider-color, rgba(0, 255, 255, 0.5));
  border-top-color: var(--provider-color, #00ffff);
  box-shadow:
    0 8px 30px rgba(0, 0, 0, 0.5),
    0 0 20px var(--provider-color, rgba(0, 255, 255, 0.3));
}

html.cyberpunk .quota-card:hover .card-brand-bg {
  opacity: 0.22;
}

/* 底部发光线 - 赛博朋克增强 */
html.cyberpunk .card-glow-line {
  opacity: 0.5;
  height: 2px;
  box-shadow: 0 0 10px var(--provider-color, #00ffff);
}

html.cyberpunk .quota-card:hover .card-glow-line {
  opacity: 1;
  height: 3px;
  box-shadow: 0 0 15px var(--provider-color, #00ffff);
}

/* 供应商名 - 赛博朋克 */
html.cyberpunk .provider-name {
  color: #e0e0ff;
}

/* 设置按钮 - 赛博朋克 */
html.cyberpunk .settings-btn {
  border-color: rgba(0, 255, 255, 0.25);
  color: rgba(0, 255, 255, 0.5);
}

html.cyberpunk .settings-btn:hover {
  background: rgba(0, 255, 255, 0.15);
  border-color: rgba(0, 255, 255, 0.5);
  color: #00ffff;
  box-shadow: 0 0 12px rgba(0, 255, 255, 0.2);
}

/* 百分比数字 - 赛博朋克霓虹 */
html.cyberpunk .balance-value {
  color: #00ffff;
  text-shadow: 0 0 12px rgba(0, 255, 255, 0.5);
}

/* 进度条 - 赛博朋克 */
html.cyberpunk .quota-progress :deep(.el-progress-bar__outer) {
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
}

/* 重置徽章 - 赛博朋克 */
html.cyberpunk .reset-badge {
  background: rgba(0, 255, 255, 0.08);
  border: 1px solid rgba(0, 255, 255, 0.15);
  color: rgba(0, 255, 255, 0.7);
}

/* 可点击卡片悬停 - 赛博朋克 */
html.cyberpunk .card-body.clickable:hover {
  background: rgba(0, 255, 255, 0.04);
}

/* 错误图标 - 赛博朋克 */
html.cyberpunk .error-icon {
  color: #ff3366;
  text-shadow: 0 0 10px rgba(255, 51, 102, 0.5);
}

/* ============================================================
   玻璃拟态主题 - 卡片样式
   ============================================================ */

/* 卡片 - 玻璃拟态 */
html.glassmorphism .quota-card {
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(37, 99, 235, 0.12);
  border-top: 3px solid var(--provider-color, #2563eb);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
}

html.glassmorphism .card-brand-bg {
  opacity: 0.05;
}

html.glassmorphism .quota-card:hover {
  transform: translateY(-4px);
  border-color: var(--provider-color, rgba(37, 99, 235, 0.35));
  border-top-color: var(--provider-color, #2563eb);
  box-shadow: 0 12px 32px rgba(37, 99, 235, 0.12);
}

html.glassmorphism .quota-card:hover .card-brand-bg {
  opacity: 0.10;
}

/* 底部发光线 - 玻璃拟态 */
html.glassmorphism .card-glow-line {
  opacity: 0.2;
}

html.glassmorphism .quota-card:hover .card-glow-line {
  opacity: 0.5;
}

/* 供应商名 - 玻璃拟态 */
html.glassmorphism .provider-name {
  color: #1e293b;
}

/* 百分比数字 - 玻璃拟态 */
html.glassmorphism .balance-value {
  color: #1e293b;
}

/* 进度条 - 玻璃拟态 */
html.glassmorphism .quota-progress :deep(.el-progress-bar__outer) {
  background: rgba(37, 99, 235, 0.08);
}

/* 重置徽章 - 玻璃拟态 */
html.glassmorphism .reset-badge {
  background: rgba(37, 99, 235, 0.06);
  color: #475569;
}

/* 可点击卡片悬停 - 玻璃拟态 */
html.glassmorphism .card-body.clickable:hover {
  background: rgba(255, 255, 255, 0.4);
}

/* ============================================================
   暗色模式 - 卡片样式
   ============================================================ */
html.dark .quota-card {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

html.dark .quota-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
}

html.dark .card-body.clickable:hover {
  background: color-mix(in srgb, var(--provider-color, var(--app-color-primary)) 6%, transparent);
}

/* ==================== 响应式 ==================== */
@media (max-width: 768px) {
  .balance-value {
    font-size: 26px;
  }
}
</style>
