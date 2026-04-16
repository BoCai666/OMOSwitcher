<script lang="ts">
// 模块级缓存：跨组件实例保持，3 分钟内不重复查询
import type { ProviderQuota } from '@/types/quota'
const CACHE_DURATION = 3 * 60 * 1000
let lastFetchTimestamp = 0
let cachedQuotaData: ProviderQuota[] = []
let cachedRefreshTime = ''
</script>

<script setup lang="ts">
/**
 * 模型额度仪表盘
 * 显示已接入供应商的额度/余额卡片
 */
import { ref, onMounted, computed } from 'vue'
import { Refresh, Coin, WarningFilled } from '@element-plus/icons-vue'
import { quotaApi } from '@/services/quotaApi'
import type { ZhipuUsageDetails } from '@/types/quota'
import { getProviderMetadata } from '@/data/providerMetadata'
import { log, error } from '@/utils/logger'

// 数据
const quotaData = ref<ProviderQuota[]>([])
const loading = ref(false)
const lastRefreshTime = ref<string>('')

// 获取所有供应商额度
async function fetchQuotas() {
  loading.value = true
  try {
    const all = await quotaApi.fetchAllProviderQuotas()
    // 只显示已实现查询接口的供应商，过滤掉 unsupported
    const filtered = all.filter(q => q.quotaType !== 'unsupported')
    log('[额度加载] 供应商列表:', filtered.map(q => `${q.providerId}(${q.quotaType})`).join(', '))
    const refreshTime = new Date().toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
    quotaData.value = filtered
    lastRefreshTime.value = refreshTime
    // 写入模块级缓存
    cachedQuotaData = filtered
    cachedRefreshTime = refreshTime
    lastFetchTimestamp = Date.now()
  } catch (e) {
    error('获取额度数据失败:', e)
  } finally {
    loading.value = false
  }
}

// 重试单个供应商
async function retryProvider(quota: ProviderQuota) {
  // 将该供应商状态设为 loading，再整体刷新
  const idx = quotaData.value.findIndex(q => q.providerId === quota.providerId)
  if (idx !== -1) {
    quotaData.value[idx] = { ...quotaData.value[idx], status: 'loading' }
  }
  try {
    const all = await quotaApi.fetchAllProviderQuotas()
    const filtered = all.filter(q => q.quotaType !== 'unsupported')
    const refreshTime = new Date().toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
    quotaData.value = filtered
    lastRefreshTime.value = refreshTime
    cachedQuotaData = filtered
    cachedRefreshTime = refreshTime
    lastFetchTimestamp = Date.now()
  } catch (e) {
    error('重试获取额度失败:', e)
    if (idx !== -1) {
      quotaData.value[idx] = {
        ...quotaData.value[idx],
        status: 'error',
        errorMessage: '重试失败'
      }
    }
  }
}

// 格式化余额显示
function formatBalance(value: number | null | undefined, currency?: string | null): string {
  if (value == null) return '--'
  const prefix = currency === 'USD' ? '$' : '¥'
  return `${prefix}${value.toFixed(2)}`
}

// 格式化 token 数量
function formatTokens(value: number | null | undefined): string {
  if (value == null) return '--'
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return String(value)
}

// 计算余额剩余百分比（用于进度条）
function getBalancePercentage(quota: ProviderQuota): number {
  if (quota.totalBalance == null || quota.totalBalance === 0) return 0
  const used = quota.usedBalance ?? 0
  return Math.min(100, Math.max(0, ((quota.totalBalance - used) / quota.totalBalance) * 100))
}

// 进度条颜色
function getProgressColor(percentage: number | null | undefined): string {
  if (percentage == null) return '#909399'
  if (percentage > 70) return '#67c23a'
  if (percentage > 30) return '#e6a23c'
  return '#f56c6c'
}

// 获取供应商品牌色
function getProviderColor(providerId: string): string {
  return getProviderMetadata(providerId).color
}

// 获取供应商完整元数据（图标、渐变等）
function getProviderMeta(providerId: string) {
  return getProviderMetadata(providerId)
}

// 判断是否为智谱供应商（含 Z.ai，两者共用同一套监控 API）
function isZhipuProvider(providerId: string): boolean {
  const id = providerId.toLowerCase()
  return id.includes('zhipu') || id.includes('glm') || id.includes('zai')
}

// 格式化重置时间为人类可读倒计时
function formatResetTime(resetTime: string | number | null | undefined): string {
  if (resetTime == null) return ''

  // 如果已经是可读字符串（非数字、非ISO格式），直接返回
  if (typeof resetTime === 'string') {
    const trimmed = resetTime.trim()
    // 特殊标记：滚动窗口
    if (trimmed === '5h-rolling') return '5小时滚动窗口'
    // 尝试解析为日期
    const parsed = new Date(trimmed)
    if (isNaN(parsed.getTime())) {
      // 无法解析为日期，当作已格式化的字符串直接返回
      return trimmed
    }
    // 成功解析为日期，继续走倒计时逻辑
    return formatCountdown(parsed)
  }

  // 数字类型（毫秒时间戳）
  if (typeof resetTime === 'number') {
    const date = new Date(resetTime)
    if (isNaN(date.getTime())) return ''
    return formatCountdown(date)
  }

  return ''
}

// 将目标日期格式化为倒计时或日期字符串
function formatCountdown(targetDate: Date): string {
  const now = Date.now()
  const diff = targetDate.getTime() - now

  // 已过期
  if (diff <= 0) {
    return '已重置'
  }

  // 不到1小时
  if (diff < 3600_000) {
    const minutes = Math.ceil(diff / 60_000)
    return `${minutes}分钟后重置`
  }

  // 不到24小时
  if (diff < 86_400_000) {
    const hours = Math.floor(diff / 3600_000)
    const minutes = Math.ceil((diff % 3600_000) / 60_000)
    if (minutes >= 60) {
      return `${hours + 1}小时后重置`
    }
    return minutes > 0 ? `${hours}小时${minutes}分钟后重置` : `${hours}小时后重置`
  }

  // 超过24小时，显示日期
  const month = String(targetDate.getMonth() + 1).padStart(2, '0')
  const day = String(targetDate.getDate()).padStart(2, '0')
  const hour = String(targetDate.getHours()).padStart(2, '0')
  const minute = String(targetDate.getMinutes()).padStart(2, '0')
  return `重置于 ${month}-${day} ${hour}:${minute}`
}

// ==================== 详情弹窗 ====================
const detailDialogVisible = ref(false)
const detailLoading = ref(false)
const selectedQuota = ref<ProviderQuota | null>(null)
const zhipuDetails = ref<ZhipuUsageDetails | null>(null)

// 判断卡片是否可点击
function isCardClickable(quota: ProviderQuota): boolean {
  return quota.quotaType === 'balance' || quota.quotaType === 'token_limit'
}

// 打开详情弹窗
async function openDetail(quota: ProviderQuota) {
  if (!isCardClickable(quota)) return
  selectedQuota.value = quota
  zhipuDetails.value = null
  detailDialogVisible.value = true

  // 智谱供应商需要额外获取用量详情
  if (isZhipuProvider(quota.providerId)) {
    detailLoading.value = true
    log(`[额度详情] providerId=${quota.providerId}, isZhipuProvider=true, 开始请求`)
    try {
      const result = await quotaApi.fetchZhipuUsageDetails(quota.providerId)
      zhipuDetails.value = result
      log(`[额度详情] 请求成功:`, JSON.stringify(result).substring(0, 500))
    } catch (e) {
      error('[额度详情] 请求失败:', e)
    } finally {
      detailLoading.value = false
    }
  } else {
    log(`[额度详情] providerId=${quota.providerId}, isZhipuProvider=false, 跳过详情请求`)
  }
}

// 弹窗标题
const dialogTitle = computed(() => {
  if (!selectedQuota.value) return '额度详情'
  return `${selectedQuota.value.providerName} - 额度详情`
})

// 格式化限制类型名称
function formatLimitType(type: string): string {
  const map: Record<string, string> = {
    TOKENS_LIMIT: 'Token 限额',
    TIME_LIMIT: '时间限额',
    RATE_LIMIT: '频率限额',
    TIMES_LIMIT: '次数限额',
    SESSION_LIMIT: '会话限额'
  }
  return map[type] || type
}

onMounted(() => {
  const now = Date.now()
  if (now - lastFetchTimestamp < CACHE_DURATION && cachedQuotaData.length > 0) {
    // 缓存命中，恢复数据，不发请求
    quotaData.value = cachedQuotaData
    lastRefreshTime.value = cachedRefreshTime
  } else {
    fetchQuotas()
  }
})
</script>

<template>
  <div class="quota-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <div class="title-icon-wrapper">
          <el-icon class="title-icon"><Coin /></el-icon>
        </div>
        <div class="header-title-group">
          <span class="title-text">配额</span>
          <span v-if="lastRefreshTime" class="refresh-time">
            最后刷新: {{ lastRefreshTime }}
          </span>
        </div>
      </div>
      <div class="header-right">
        <el-button
          class="refresh-btn"
          :icon="Refresh"
          :loading="loading"
          @click="fetchQuotas"
        >
          刷新
        </el-button>
      </div>
    </div>

    <!-- 额度卡片网格 -->
    <div v-if="quotaData.length > 0" class="quota-grid">
      <el-row :gutter="20">
        <el-col
          v-for="quota in quotaData"
          :key="quota.providerId"
          :xs="24"
          :sm="12"
          :md="8"
          :lg="6"
        >
          <div
            class="quota-card"
            :style="{
              '--provider-color': getProviderColor(quota.providerId)
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
              <div class="error-content">
                <el-icon class="error-icon"><WarningFilled /></el-icon>
                <p class="error-text">{{ quota.errorMessage || '查询失败' }}</p>
                <el-button size="small" @click="retryProvider(quota)">重试</el-button>
              </div>
            </div>

            <!-- 不支持额度查询 -->
            <div v-else-if="quota.quotaType === 'unsupported'" class="card-body unsupported-state">
              <div class="card-header-row">
                <div class="provider-icon-badge" :style="{ background: getProviderMeta(quota.providerId).gradient || getProviderColor(quota.providerId) }">
                  <svg :viewBox="getProviderMeta(quota.providerId).iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
                    <path :d="getProviderMeta(quota.providerId).iconPath" />
                  </svg>
                </div>
                <span class="provider-name">{{ quota.providerName }}</span>
              </div>
              <el-empty
                description="该供应商暂不支持额度查询"
                :image-size="64"
                class="unsupported-empty"
              />
            </div>

            <!-- 余额型 -->
            <div v-else-if="quota.quotaType === 'balance'" class="card-body clickable" @click="openDetail(quota)">
              <div class="card-header-row">
                <div class="provider-icon-badge" :style="{ background: getProviderMeta(quota.providerId).gradient || getProviderColor(quota.providerId) }">
                  <svg :viewBox="getProviderMeta(quota.providerId).iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
                    <path :d="getProviderMeta(quota.providerId).iconPath" />
                  </svg>
                </div>
                <span class="provider-name">{{ quota.providerName }}</span>
              </div>
              <div class="balance-main">
                <span class="balance-label">配额使用</span>
                <span class="balance-value">
                  {{ getBalancePercentage(quota).toFixed(1) }}%
                </span>
                <span class="balance-detail">
                  {{ formatBalance(quota.usedBalance, quota.currency) }} / {{ formatBalance(quota.totalBalance, quota.currency) }}
                </span>
              </div>
              <el-progress
                :percentage="getBalancePercentage(quota)"
                :color="getProgressColor(getBalancePercentage(quota))"
                :stroke-width="8"
                :show-text="false"
                class="quota-progress"
              />
              <div v-if="quota.resetTime" class="reset-badge">
                {{ formatResetTime(quota.resetTime) }}
              </div>
            </div>

            <!-- 配额型 (token_limit) -->
            <div v-else-if="quota.quotaType === 'token_limit'" class="card-body clickable" @click="openDetail(quota)">
              <div class="card-header-row">
                <div class="provider-icon-badge" :style="{ background: getProviderMeta(quota.providerId).gradient || getProviderColor(quota.providerId) }">
                  <svg :viewBox="getProviderMeta(quota.providerId).iconViewBox || '0 0 24 24'" fill="currentColor" width="18" height="18">
                    <path :d="getProviderMeta(quota.providerId).iconPath" />
                  </svg>
                </div>
                <span class="provider-name">{{ quota.providerName }}</span>
              </div>
              <div class="balance-main">
                <span class="balance-label">配额使用</span>
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
              <div v-if="quota.resetTime" class="reset-badge">
                {{ formatResetTime(quota.resetTime) }}
              </div>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading" class="empty-state">
      <el-empty description="请先在模型管理中配置供应商 API Key">
        <el-button type="primary" @click="$router.push('/models')">
          前往模型管理
        </el-button>
      </el-empty>
    </div>

    <!-- 详情弹窗 -->
    <el-dialog
      v-model="detailDialogVisible"
      :title="dialogTitle"
      width="640px"
      class="detail-dialog"
      :close-on-click-modal="true"
      destroy-on-close
    >
      <div v-if="detailLoading" class="dialog-loading">
        <el-skeleton :rows="5" animated />
      </div>

      <template v-else-if="selectedQuota">
        <!-- 智谱供应商：模型用量 + 工具用量 -->
        <template v-if="isZhipuProvider(selectedQuota.providerId)">
          <!-- 基础配额信息 -->
          <el-descriptions :column="2" border class="detail-descriptions">
            <el-descriptions-item label="供应商">
              {{ selectedQuota.providerName }}
            </el-descriptions-item>
            <el-descriptions-item label="配额使用">
              {{ selectedQuota.quotaPercentage != null ? `${selectedQuota.quotaPercentage.toFixed(1)}%` : '--' }}
            </el-descriptions-item>
            <el-descriptions-item label="已用">
              {{ formatTokens(selectedQuota.quotaUsed) }}
            </el-descriptions-item>
            <el-descriptions-item label="总量">
              {{ formatTokens(selectedQuota.quotaLimit) }}
            </el-descriptions-item>
            <el-descriptions-item v-if="selectedQuota.resetTime" label="重置时间" :span="2">
              {{ formatResetTime(selectedQuota.resetTime) }}
            </el-descriptions-item>
          </el-descriptions>

          <!-- 限制列表 (limits 数组) -->
          <template v-if="selectedQuota.limits && selectedQuota.limits.length > 0">
            <div class="detail-section-title">限制详情</div>
            <el-table :data="selectedQuota.limits" size="small" class="detail-table">
              <el-table-column prop="type" label="类型" width="120">
                <template #default="{ row }">
                  {{ formatLimitType(row.type) }}
                </template>
              </el-table-column>
              <el-table-column label="使用率" width="140">
                <template #default="{ row }">
                  <div class="limit-progress-cell">
                    <el-progress
                      :percentage="row.percentage ?? 0"
                      :color="getProgressColor(row.percentage ?? 0)"
                      :stroke-width="6"
                      :show-text="false"
                    />
                    <span class="limit-percentage-text">
                      {{ row.percentage != null ? `${row.percentage.toFixed(1)}%` : '--' }}
                    </span>
                  </div>
                </template>
              </el-table-column>
              <el-table-column label="用量" min-width="120">
                <template #default="{ row }">
                  <span v-if="row.currentValue != null && row.usage != null">
                    {{ formatTokens(row.currentValue) }} / {{ formatTokens(row.usage) }}
                  </span>
                  <span v-else-if="row.usage != null">{{ formatTokens(row.usage) }}</span>
                  <span v-else>--</span>
                </template>
              </el-table-column>
              <el-table-column label="剩余" width="100">
                <template #default="{ row }">
                  {{ row.remaining != null ? formatTokens(row.remaining) : '--' }}
                </template>
              </el-table-column>
              <el-table-column label="重置时间" width="140">
                <template #default="{ row }">
                  {{ formatResetTime(row.nextResetTime) || '--' }}
                </template>
              </el-table-column>
            </el-table>
          </template>

          <!-- 7天用量汇总 (仅智谱) -->
          <template v-if="zhipuDetails">
            <!-- 模型用量汇总 -->
            <div class="detail-section-title">模型用量 (近7天)</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item label="总调用次数">
                {{ zhipuDetails.modelUsage.totalCalls.toLocaleString() }}
              </el-descriptions-item>
              <el-descriptions-item label="总 Token 消耗">
                {{ formatTokens(zhipuDetails.modelUsage.totalTokens) }}
              </el-descriptions-item>
            </el-descriptions>

            <!-- 各模型 Token 明细 -->
            <template v-if="zhipuDetails.modelUsage.modelList.length > 0">
              <div class="detail-section-title">各模型 Token 消耗</div>
              <el-table :data="zhipuDetails.modelUsage.modelList" size="small" class="detail-table">
                <el-table-column prop="modelName" label="模型" min-width="140" />
                <el-table-column label="Token 消耗" width="140">
                  <template #default="{ row }">
                    {{ formatTokens(row.totalTokens) }}
                  </template>
                </el-table-column>
                <el-table-column label="占比" width="100">
                  <template #default="{ row }">
                    {{ zhipuDetails.modelUsage.totalTokens > 0
                      ? ((row.totalTokens / zhipuDetails.modelUsage.totalTokens) * 100).toFixed(1) + '%'
                      : '--' }}
                  </template>
                </el-table-column>
              </el-table>
            </template>

            <!-- 工具用量 -->
            <template v-if="zhipuDetails.toolUsage.networkSearchCount > 0 || zhipuDetails.toolUsage.webReadCount > 0 || zhipuDetails.toolUsage.zreadCount > 0">
              <div class="detail-section-title">工具用量 (近7天)</div>
              <el-descriptions :column="3" border class="detail-descriptions">
                <el-descriptions-item label="联网搜索">
                  {{ zhipuDetails.toolUsage.networkSearchCount.toLocaleString() }} 次
                </el-descriptions-item>
                <el-descriptions-item label="网页阅读">
                  {{ zhipuDetails.toolUsage.webReadCount.toLocaleString() }} 次
                </el-descriptions-item>
                <el-descriptions-item label="仓库搜索">
                  {{ zhipuDetails.toolUsage.zreadCount.toLocaleString() }} 次
                </el-descriptions-item>
              </el-descriptions>
            </template>
          </template>

          <!-- 智谱无详情数据 -->
          <el-empty
            v-if="!zhipuDetails && (!selectedQuota.limits || selectedQuota.limits.length === 0)"
            description="暂无用量数据"
            :image-size="64"
          />
        </template>

        <!-- 非智谱供应商：余额型详情 -->
        <template v-else-if="selectedQuota.quotaType === 'balance'">
          <el-descriptions :column="2" border class="detail-descriptions">
            <el-descriptions-item label="供应商">
              {{ selectedQuota.providerName }}
            </el-descriptions-item>
            <el-descriptions-item label="货币">
              {{ selectedQuota.currency === 'USD' ? '美元 (USD)' : '人民币 (CNY)' }}
            </el-descriptions-item>
            <el-descriptions-item label="可用余额">
              {{ formatBalance(selectedQuota.availableBalance, selectedQuota.currency) }}
            </el-descriptions-item>
            <el-descriptions-item label="总额度">
              {{ formatBalance(selectedQuota.totalBalance, selectedQuota.currency) }}
            </el-descriptions-item>
            <el-descriptions-item label="已用额度">
              {{ formatBalance(selectedQuota.usedBalance, selectedQuota.currency) }}
            </el-descriptions-item>
            <el-descriptions-item label="剩余比例">
              {{ getBalancePercentage(selectedQuota).toFixed(1) }}%
            </el-descriptions-item>
          </el-descriptions>
          <!-- OpenRouter 专用字段 -->
          <template v-if="selectedQuota.dailyUsage != null || selectedQuota.weeklyUsage != null || selectedQuota.monthlyUsage != null">
            <div class="detail-section-title">周期用量</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item v-if="selectedQuota.dailyUsage != null" label="今日用量">
                ${{ selectedQuota.dailyUsage.toFixed(2) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.weeklyUsage != null" label="本周用量">
                ${{ selectedQuota.weeklyUsage.toFixed(2) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.monthlyUsage != null" label="本月用量">
                ${{ selectedQuota.monthlyUsage.toFixed(2) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.spendingLimit != null" label="消费上限">
                ${{ selectedQuota.spendingLimit.toFixed(2) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.limitRemaining != null" label="剩余额度">
                ${{ selectedQuota.limitRemaining.toFixed(2) }}
              </el-descriptions-item>
            </el-descriptions>
          </template>
        </template>

        <!-- 非智谱供应商：配额型详情 -->
        <template v-else-if="selectedQuota.quotaType === 'token_limit'">
          <el-descriptions :column="2" border class="detail-descriptions">
            <el-descriptions-item label="供应商">
              {{ selectedQuota.providerName }}
            </el-descriptions-item>
            <el-descriptions-item label="使用率">
              {{ selectedQuota.quotaPercentage != null ? `${selectedQuota.quotaPercentage.toFixed(1)}%` : '--' }}
            </el-descriptions-item>
            <el-descriptions-item label="已用 Token">
              {{ formatTokens(selectedQuota.quotaUsed) }}
            </el-descriptions-item>
            <el-descriptions-item label="总限额">
              {{ formatTokens(selectedQuota.quotaLimit) }}
            </el-descriptions-item>
            <el-descriptions-item v-if="selectedQuota.resetTime" label="重置时间" :span="2">
              {{ formatResetTime(selectedQuota.resetTime) }}
            </el-descriptions-item>
          </el-descriptions>

          <!-- 周期用量与剩余额度 -->
          <template v-if="selectedQuota.weeklyUsage != null || selectedQuota.monthlyUsage != null || selectedQuota.spendingLimit != null || selectedQuota.limitRemaining != null">
            <div class="detail-section-title">周期用量与剩余额度</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item v-if="selectedQuota.weeklyUsage != null" label="周期用量">
                {{ formatTokens(selectedQuota.weeklyUsage) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.monthlyUsage != null" label="长期用量">
                {{ formatTokens(selectedQuota.monthlyUsage) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.spendingLimit != null" label="配额上限">
                {{ formatTokens(selectedQuota.spendingLimit) }}
              </el-descriptions-item>
              <el-descriptions-item v-if="selectedQuota.limitRemaining != null" label="当前窗口剩余">
                {{ formatTokens(selectedQuota.limitRemaining) }}
              </el-descriptions-item>
            </el-descriptions>
          </template>

          <!-- limits 数组展示 -->
          <template v-if="selectedQuota.limits && selectedQuota.limits.length > 0">
            <div class="detail-section-title">限制详情</div>
            <el-table :data="selectedQuota.limits" size="small" class="detail-table">
              <el-table-column prop="type" label="类型" width="120">
                <template #default="{ row }">
                  {{ formatLimitType(row.type) }}
                </template>
              </el-table-column>
              <el-table-column label="使用率" width="140">
                <template #default="{ row }">
                  <div class="limit-progress-cell">
                    <el-progress
                      :percentage="row.percentage ?? 0"
                      :color="getProgressColor(row.percentage ?? 0)"
                      :stroke-width="6"
                      :show-text="false"
                    />
                    <span class="limit-percentage-text">
                      {{ row.percentage != null ? `${row.percentage.toFixed(1)}%` : '--' }}
                    </span>
                  </div>
                </template>
              </el-table-column>
              <el-table-column label="用量" min-width="120">
                <template #default="{ row }">
                  <span v-if="row.currentValue != null && row.usage != null">
                    {{ formatTokens(row.currentValue) }} / {{ formatTokens(row.usage) }}
                  </span>
                  <span v-else-if="row.usage != null">{{ formatTokens(row.usage) }}</span>
                  <span v-else>--</span>
                </template>
              </el-table-column>
              <el-table-column label="剩余" width="100">
                <template #default="{ row }">
                  {{ row.remaining != null ? formatTokens(row.remaining) : '--' }}
                </template>
              </el-table-column>
              <el-table-column label="重置时间" width="140">
                <template #default="{ row }">
                  {{ formatResetTime(row.nextResetTime) || '--' }}
                </template>
              </el-table-column>
            </el-table>
          </template>
        </template>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
/* ==================== 页面容器 ==================== */
.quota-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
  background: var(--app-bg-base);
  min-height: 100%;
}

/* ==================== 页面头部 ==================== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow: var(--app-shadow-md);
  margin-bottom: 24px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.page-header:hover {
  box-shadow: var(--app-shadow-lg);
  border-color: var(--app-color-primary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.header-title-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
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
  transition: all 0.3s ease;
}

.title-icon {
  font-size: 22px;
  color: var(--app-color-primary);
}

.title-text {
  font-size: 17px;
  font-weight: 600;
  color: var(--app-text-primary);
  letter-spacing: 0.5px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.refresh-time {
  font-size: 12px;
  color: var(--app-text-tertiary);
  letter-spacing: 0.5px;
}

.refresh-btn {
  border-radius: 10px;
  font-weight: 600;
}

.refresh-btn:not(.el-button--primary) {
  background: transparent !important;
  border: 1px solid var(--app-border-default) !important;
  color: var(--app-text-primary) !important;
  transition: all 0.3s ease !important;
}

.refresh-btn:not(.el-button--primary):hover {
  background: var(--app-color-primary) !important;
  border-color: var(--app-color-primary) !important;
  color: #ffffff !important;
}

/* ==================== 额度卡片网格 ==================== */
.quota-grid {
  margin-bottom: 24px;
}

.quota-grid :deep(.el-col) {
  margin-bottom: 20px;
}

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

/* ==================== 错误状态 ==================== */
.error-state {
  align-items: center;
  justify-content: center;
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

/* ==================== 空状态 ==================== */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
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

/* ==================== 详情弹窗 ==================== */
.detail-dialog .dialog-loading {
  padding: 16px 0;
}

.detail-dialog :deep(.el-dialog) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
}

.detail-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid var(--app-border-default);
  padding: 16px 20px;
  margin: 0;
}

.detail-dialog :deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
  font-size: 15px;
}

.detail-dialog :deep(.el-dialog__body) {
  padding: 20px;
}

.detail-dialog :deep(.el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-secondary);
}

.detail-descriptions {
  margin-bottom: 16px;
}

.detail-descriptions :deep(.el-descriptions__label) {
  color: var(--app-text-tertiary);
  background: var(--app-bg-hover);
  font-size: 13px;
}

.detail-descriptions :deep(.el-descriptions__content) {
  color: var(--app-text-primary);
  font-size: 13px;
}

.detail-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 20px 0 12px;
  padding-left: 10px;
  border-left: 3px solid var(--app-color-primary);
}

.detail-table {
  border-radius: 8px;
  overflow: hidden;
}

.detail-table :deep(.el-table__header th) {
  background: var(--app-bg-hover) !important;
  color: var(--app-text-secondary);
  font-size: 12px;
  font-weight: 600;
}

.detail-table :deep(.el-table__body td) {
  color: var(--app-text-primary);
  font-size: 13px;
}

.detail-table :deep(.el-table__row:hover > td) {
  background: var(--app-bg-hover) !important;
}

.detail-table :deep(.el-table__empty-block) {
  background: transparent;
}

.limit-progress-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.limit-progress-cell .el-progress {
  flex: 1;
}

.limit-percentage-text {
  font-size: 12px;
  color: var(--app-text-secondary);
  min-width: 42px;
  text-align: right;
}

/* ============================================================
   赛博朋克主题 - 霓虹发光效果
   ============================================================ */

/* 页面头部 */
html.cyberpunk .page-header {
  background: rgba(18, 18, 31, 0.9);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow:
    0 0 20px rgba(0, 255, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.cyberpunk .page-header:hover {
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 0 0 30px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .title-icon-wrapper {
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 0%, rgba(0, 255, 255, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .title-icon {
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.6);
}

html.cyberpunk .title-text {
  color: #e0e0ff;
  text-shadow: 0 0 6px rgba(0, 255, 255, 0.2);
}

/* 刷新按钮 - 赛博朋克 */
html.cyberpunk .refresh-btn:not(.el-button--primary) {
  border: 1px solid rgba(0, 255, 255, 0.3) !important;
  color: #00ffff !important;
  background: rgba(0, 255, 255, 0.05) !important;
}

html.cyberpunk .refresh-btn:not(.el-button--primary):hover {
  background: rgba(0, 255, 255, 0.15) !important;
  border-color: rgba(0, 255, 255, 0.6) !important;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
  color: #00ffff !important;
}

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

/* 弹窗 - 赛博朋克 */
html.cyberpunk .detail-dialog :deep(.el-dialog) {
  background: rgba(18, 18, 31, 0.95);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: 0 0 40px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .detail-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

html.cyberpunk .detail-dialog :deep(.el-dialog__title) {
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .detail-section-title {
  color: #00ffff;
  border-left-color: #00ffff;
  text-shadow: 0 0 6px rgba(0, 255, 255, 0.2);
}

html.cyberpunk .detail-descriptions :deep(.el-descriptions__label) {
  background: rgba(0, 255, 255, 0.05);
  color: #8080a0;
  border-color: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .detail-descriptions :deep(.el-descriptions__content) {
  color: #e0e0ff;
  border-color: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .detail-table :deep(.el-table__header th) {
  background: rgba(0, 255, 255, 0.06) !important;
  color: rgba(0, 255, 255, 0.7);
  border-color: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .detail-table :deep(.el-table__body td) {
  color: #e0e0ff;
  border-color: rgba(0, 255, 255, 0.06);
}

html.cyberpunk .detail-table :deep(.el-table__row:hover > td) {
  background: rgba(0, 255, 255, 0.05) !important;
}

/* 空状态 - 赛博朋克 */
html.cyberpunk .empty-state {
  background: rgba(18, 18, 31, 0.85);
  border: 1px solid rgba(0, 255, 255, 0.1);
}

/* 错误图标 - 赛博朋克 */
html.cyberpunk .error-icon {
  color: #ff3366;
  text-shadow: 0 0 10px rgba(255, 51, 102, 0.5);
}

/* ============================================================
   玻璃拟态主题 - 毛玻璃效果
   ============================================================ */

/* 页面头部 */
html.glassmorphism .page-header {
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(37, 99, 235, 0.2);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  box-shadow: 0 8px 32px rgba(37, 99, 235, 0.08);
}

html.glassmorphism .page-header:hover {
  border-color: rgba(37, 99, 235, 0.4);
  box-shadow: 0 12px 40px rgba(37, 99, 235, 0.12);
}

html.glassmorphism .title-icon-wrapper {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(37, 99, 235, 0.05) 100%);
  border: 1px solid rgba(37, 99, 235, 0.3);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.15);
}

html.glassmorphism .title-icon {
  color: #2563eb;
}

html.glassmorphism .title-text {
  color: #1e293b;
}

/* 刷新按钮 - 玻璃拟态 */
html.glassmorphism .refresh-btn:not(.el-button--primary) {
  border: 1px solid rgba(37, 99, 235, 0.25) !important;
  color: #2563eb !important;
  background: rgba(255, 255, 255, 0.5) !important;
  backdrop-filter: blur(8px);
}

html.glassmorphism .refresh-btn:not(.el-button--primary):hover {
  background: #2563eb !important;
  border-color: #2563eb !important;
  color: #ffffff !important;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.3);
}

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

/* 弹窗 - 玻璃拟态 */
html.glassmorphism .detail-dialog :deep(.el-dialog) {
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(37, 99, 235, 0.2);
  box-shadow: 0 12px 48px rgba(37, 99, 235, 0.12);
}

html.glassmorphism .detail-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(37, 99, 235, 0.12);
}

html.glassmorphism .detail-dialog :deep(.el-dialog__title) {
  color: #1e293b;
}

html.glassmorphism .detail-section-title {
  color: #1e293b;
  border-left-color: #2563eb;
}

html.glassmorphism .detail-descriptions :deep(.el-descriptions__label) {
  background: rgba(37, 99, 235, 0.04);
  color: #94a3b8;
}

html.glassmorphism .detail-descriptions :deep(.el-descriptions__content) {
  color: #1e293b;
}

html.glassmorphism .detail-table :deep(.el-table__header th) {
  background: rgba(37, 99, 235, 0.04) !important;
  color: #475569;
}

html.glassmorphism .detail-table :deep(.el-table__body td) {
  color: #1e293b;
}

html.glassmorphism .detail-table :deep(.el-table__row:hover > td) {
  background: rgba(37, 99, 235, 0.04) !important;
}

/* 空状态 - 玻璃拟态 */
html.glassmorphism .empty-state {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(37, 99, 235, 0.15);
}

/* ============================================================
   暗色模式 - 补充样式
   ============================================================ */
html.dark .refresh-btn:not(.el-button--primary):hover {
  background: var(--app-color-purple) !important;
  border-color: var(--app-color-purple) !important;
}

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
  .quota-page {
    padding: 12px;
  }

  .page-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
    padding: 16px;
  }

  .header-right {
    width: 100%;
    justify-content: space-between;
  }

  .balance-value {
    font-size: 26px;
  }
}
</style>
