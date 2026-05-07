<script lang="ts">
// 模块级缓存：跨组件实例保持，3 分钟内不重复查询
import type { ProviderQuota } from '@/types/quota'
import type { ZhipuUsageDetails } from '@/types/quota'
const CACHE_DURATION = 3 * 60 * 1000
let lastFetchTimestamp = 0
let cachedQuotaData: ProviderQuota[] = []
let cachedRefreshTime = ''
// zhipu 详情缓存：按 providerId 缓存，与额度列表共享同一时间戳
let cachedZhipuDetails: Record<string, { data: ZhipuUsageDetails; timestamp: number }> = {}
</script>

<script setup lang="ts">
/**
 * 模型额度仪表盘
 * 显示已接入供应商的额度/余额卡片
 */
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { Refresh, Coin } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { quotaApi } from '@/services/quotaApi'
import { log, error } from '@/utils/logger'
import QuotaCard from '@/components/QuotaCard.vue'
import OpenCodeGoSettingsDialog from '@/components/OpenCodeGoSettingsDialog.vue'
import {
  formatBalance,
  formatTokens,
  formatResetTime,
  getBalancePercentage,
  getProgressColor
} from '@/composables/useQuotaFormatter'
import type { ModelUsageSummary } from '@/types/quota'

// 数据
const quotaData = ref<ProviderQuota[]>([])
const loading = ref(false)
const lastRefreshTime = ref<string>('')

// 判断是否为 OpenCodeGo（需要保留卡片以便显示设置按钮）
function isOpenCodeGoProvider(providerId: string): boolean {
  const id = providerId.toLowerCase()
  return id.includes('opencode') && id.includes('go')
}

// 获取所有供应商额度
// Rust 端并发查询，每完成一个就通过 "quota-progress" 事件实时推送到前端
let quotaUnlisten: UnlistenFn | null = null

async function fetchQuotas() {
  loading.value = true

  // 1. 设置事件监听，Rust 每完成一个查询就实时更新卡片
  //    首次刷新时不预填充骨架卡片，查询成功的卡片逐个出现
  if (quotaUnlisten) { quotaUnlisten(); quotaUnlisten = null }
  quotaUnlisten = await listen<ProviderQuota>('quota-progress', (event) => {
    const q = event.payload
    // 跳过无需展示的供应商：unsupported 且非 OpenCodeGo 且非 error
    if (q.quotaType === 'unsupported' && !isOpenCodeGoProvider(q.providerId) && q.status !== 'error') return
    const idx = quotaData.value.findIndex(item => item.providerId === q.providerId)
    if (idx !== -1) {
      quotaData.value[idx] = q  // 更新已有卡片（非首次刷新）
    } else {
      quotaData.value.push(q)  // 首次刷新：查询成功的卡片逐个弹出
    }
  })

  try {
    // 2. 全并发查询（Rust JoinSet），事件逐步更新卡片内容
    const all = await quotaApi.fetchAllProviderQuotas()

    // 3. 最终整理：在事件已更新的 quotaData 基础上过滤并排序
    //    保留：balance/token_limit + OpenCodeGo（可设置参数）+ error（需展示错误）
    //    注意：不在 all 中的卡片说明 Rust 未返回，移除
    const allIds = new Set(all.map(q => q.providerId))
    quotaData.value = quotaData.value
      .filter(q =>
        allIds.has(q.providerId) && (
          q.quotaType !== 'unsupported' ||
          isOpenCodeGoProvider(q.providerId) ||
          q.status === 'error'
        )
      )
      .sort((a, b) => a.providerId.localeCompare(b.providerId))

    const refreshTime = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    lastRefreshTime.value = refreshTime
    cachedQuotaData = [...quotaData.value]
    cachedRefreshTime = refreshTime
    lastFetchTimestamp = Date.now()
    cachedZhipuDetails = {}
    log('[额度加载] 供应商列表:', quotaData.value.map(q => `${q.providerId}(${q.quotaType})`).join(', '))
  } catch (e) {
    error('获取额度数据失败:', e)
  } finally {
    loading.value = false
    quotaUnlisten?.()
    quotaUnlisten = null
  }
}

// 重试单个供应商
async function retryProvider(quota: ProviderQuota) {
  const idx = quotaData.value.findIndex(q => q.providerId === quota.providerId)
  if (idx !== -1) {
    quotaData.value[idx] = { ...quotaData.value[idx], status: 'loading' }
  }
  // 直接复用 fetchQuotas（含事件监听，支持渐进更新）
  await fetchQuotas()
}

// ==================== 详情弹窗 ====================
const detailDialogVisible = ref(false)
const detailLoading = ref(false)
const selectedQuota = ref<ProviderQuota | null>(null)
const zhipuDetails = ref<ZhipuUsageDetails | null>(null)

// 判断是否为智谱供应商（含 Z.ai，两者共用同一套监控 API）
function isZhipuProvider(providerId: string): boolean {
  const id = providerId.toLowerCase()
  return id.includes('zhipu') || id.includes('glm') || id.includes('zai')
}

// 判断是否为 Kimi Code 供应商
function isKimiCodeProvider(providerId: string): boolean {
  const id = providerId.toLowerCase()
  return id === 'kimi-for-coding' || id.includes('kimi-code') || id.includes('kimicode')
}

// 判断是否为纯余额型供应商（无 usedBalance/resetTime，如 DeepSeek）
function isPureBalanceProvider(quota: ProviderQuota): boolean {
  return quota.quotaType === 'balance' && quota.usedBalance == null && quota.resetTime == null
}

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
    // 检查缓存：3 分钟内且额度列表未刷新则复用
    const cached = cachedZhipuDetails[quota.providerId]
    if (cached && (Date.now() - cached.timestamp) < CACHE_DURATION) {
      log(`[额度详情] providerId=${quota.providerId} 命中缓存，跳过请求`)
      zhipuDetails.value = cached.data
      return
    }

    detailLoading.value = true
    log(`[额度详情] providerId=${quota.providerId}, isZhipuProvider=true, 开始请求`)
    try {
      const result = await quotaApi.fetchZhipuUsageDetails(quota.providerId)
      zhipuDetails.value = result
      // 写入缓存
      cachedZhipuDetails[quota.providerId] = { data: result, timestamp: Date.now() }
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

// ==================== OpenCode Go 设置弹窗 ====================
const settingsDialogVisible = ref(false)

function openSettings(_quota: ProviderQuota) {
  settingsDialogVisible.value = true
}

function onSettingsSaved() {
  settingsDialogVisible.value = false
  setTimeout(() => fetchQuotas(), 500)
}

// Kimi Code 额度详情（从 limits._kimiCodeUsage 中提取）
const kimiCodeUsage = computed(() => {
  if (!selectedQuota.value?.limits) return null
  const limits = selectedQuota.value.limits as any
  return limits?._kimiCodeUsage || null
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

// 用量详情 tab 切换（今日 / 近7天）
const usageTab = ref<'today' | 'week'>('today')

// 当前 tab 对应的用量数据
const currentUsage = computed<ModelUsageSummary | null>(() => {
  if (!zhipuDetails.value) return null
  return usageTab.value === 'today'
    ? zhipuDetails.value.todayModelUsage
    : zhipuDetails.value.modelUsage
})

// 模型占比计算
function getModelPercentage(modelTokens: number, totalTokens: number): string {
  return totalTokens > 0
    ? ((modelTokens / totalTokens) * 100).toFixed(1) + '%'
    : '--'
}

onMounted(() => {
  const now = Date.now()
  if (now - lastFetchTimestamp < CACHE_DURATION && cachedQuotaData.length > 0) {
    quotaData.value = cachedQuotaData
    lastRefreshTime.value = cachedRefreshTime
  } else {
    fetchQuotas()
  }
})

onUnmounted(() => {
  quotaUnlisten?.()
  quotaUnlisten = null
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
          <QuotaCard
            :quota="quota"
            @retry="retryProvider"
            @detail="openDetail"
            @settings="openSettings"
          />
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
      append-to=".app-main"
      align-center
    >
      <!-- 加载状态：保持与内容区等高，避免弹窗尺寸跳变 -->
      <div v-if="detailLoading" class="dialog-loading">
        <div class="loading-spinner-wrapper">
          <el-icon class="loading-spinner is-loading"><Refresh /></el-icon>
          <span class="loading-text">加载用量详情...</span>
        </div>
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

          <!-- 用量详情 (仅智谱) -->
          <template v-if="zhipuDetails">
            <!-- Tab 切换 -->
            <div class="usage-tabs">
              <button
                class="usage-tab"
                :class="{ active: usageTab === 'today' }"
                @click="usageTab = 'today'"
              >今日</button>
              <button
                class="usage-tab"
                :class="{ active: usageTab === 'week' }"
                @click="usageTab = 'week'"
              >近7天</button>
            </div>

            <!-- 模型用量汇总 -->
            <template v-if="currentUsage">
              <el-descriptions :column="2" border class="detail-descriptions">
                <el-descriptions-item label="调用次数">
                  {{ currentUsage.totalCalls.toLocaleString() }}
                </el-descriptions-item>
                <el-descriptions-item label="Token 消耗">
                  {{ formatTokens(currentUsage.totalTokens) }}
                </el-descriptions-item>
              </el-descriptions>

              <!-- 各模型 Token 明细 -->
              <template v-if="currentUsage.modelList.length > 0">
                <div class="detail-section-title">各模型 Token 消耗</div>
                <el-table :data="currentUsage.modelList" size="small" class="detail-table">
                  <el-table-column prop="modelName" label="模型" min-width="140" />
                  <el-table-column label="Token 消耗" width="140">
                    <template #default="{ row }">
                      {{ formatTokens(row.totalTokens) }}
                    </template>
                  </el-table-column>
                  <el-table-column label="占比" width="100">
                    <template #default="{ row }">
                      {{ getModelPercentage(row.totalTokens, currentUsage.totalTokens) }}
                    </template>
                  </el-table-column>
                </el-table>
              </template>
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
            <!-- 纯余额型：只显示余额 -->
            <template v-if="isPureBalanceProvider(selectedQuota)">
              <el-descriptions-item label="余额" :span="2">
                {{ formatBalance(selectedQuota.totalBalance, selectedQuota.currency) }}
              </el-descriptions-item>
            </template>
            <!-- 标准余额型：显示完整字段 -->
            <template v-else>
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
            </template>
          </el-descriptions>
          <!-- OpenRouter 专用字段（纯余额型不展示） -->
          <template v-if="!isPureBalanceProvider(selectedQuota) && (selectedQuota.dailyUsage != null || selectedQuota.weeklyUsage != null || selectedQuota.monthlyUsage != null)">
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

        <!-- Kimi Code 供应商详情 -->
        <template v-else-if="isKimiCodeProvider(selectedQuota.providerId)">
          <el-descriptions :column="2" border class="detail-descriptions">
            <el-descriptions-item label="供应商">
              {{ selectedQuota.providerName }}
            </el-descriptions-item>
            <el-descriptions-item label="平台">
              Kimi Code (Coding Plan)
            </el-descriptions-item>
          </el-descriptions>

          <!-- 5小时额度 -->
          <template v-if="kimiCodeUsage?.fiveHour">
            <div class="detail-section-title">5小时滚动窗口</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item label="限额">
                {{ formatTokens(kimiCodeUsage.fiveHour.limit) }}
              </el-descriptions-item>
              <el-descriptions-item label="已用">
                {{ formatTokens(kimiCodeUsage.fiveHour.used) }}
              </el-descriptions-item>
              <el-descriptions-item label="剩余">
                {{ formatTokens(kimiCodeUsage.fiveHour.remaining) }}
              </el-descriptions-item>
              <el-descriptions-item label="重置时间">
                {{ formatResetTime(kimiCodeUsage.fiveHour.resetTime) }}
              </el-descriptions-item>
            </el-descriptions>
          </template>

          <!-- 周额度 -->
          <template v-if="kimiCodeUsage?.weekly">
            <div class="detail-section-title">周额度</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item label="限额">
                {{ formatTokens(kimiCodeUsage.weekly.limit) }}
              </el-descriptions-item>
              <el-descriptions-item label="已用">
                {{ formatTokens(kimiCodeUsage.weekly.used) }}
              </el-descriptions-item>
              <el-descriptions-item label="剩余">
                {{ formatTokens(kimiCodeUsage.weekly.remaining) }}
              </el-descriptions-item>
              <el-descriptions-item label="重置时间">
                {{ formatResetTime(kimiCodeUsage.weekly.resetTime) }}
              </el-descriptions-item>
            </el-descriptions>
          </template>

          <!-- 月额度 -->
          <template v-if="kimiCodeUsage?.monthly">
            <div class="detail-section-title">月额度</div>
            <el-descriptions :column="2" border class="detail-descriptions">
              <el-descriptions-item label="限额">
                {{ formatTokens(kimiCodeUsage.monthly.limit) }}
              </el-descriptions-item>
              <el-descriptions-item label="剩余">
                {{ formatTokens(kimiCodeUsage.monthly.remaining) }}
              </el-descriptions-item>
            </el-descriptions>
          </template>

          <!-- Kimi Code limits 数组展示 -->
          <template v-if="selectedQuota.limits && selectedQuota.limits.length > 0">
            <div class="detail-section-title">限制详情</div>
            <el-table :data="selectedQuota.limits" size="small" class="detail-table">
              <el-table-column label="名称" min-width="140">
                <template #default="{ row }">
                  {{ row.detail?.name || '--' }}
                </template>
              </el-table-column>
              <el-table-column label="限额" width="120">
                <template #default="{ row }">
                  {{ row.detail?.limit != null ? formatTokens(row.detail.limit) : '--' }}
                </template>
              </el-table-column>
              <el-table-column label="已用" width="120">
                <template #default="{ row }">
                  {{ row.detail?.used != null ? formatTokens(row.detail.used) : '--' }}
                </template>
              </el-table-column>
              <el-table-column label="窗口" width="140">
                <template #default="{ row }">
                  <span v-if="row.window">
                    {{ row.window.duration }} {{ row.window.timeUnit?.toLowerCase() }}
                  </span>
                  <span v-else>--</span>
                </template>
              </el-table-column>
            </el-table>
          </template>
        </template>

        <!-- OpenCode Go 三维度详情 -->
        <template v-else-if="isOpenCodeGoProvider(selectedQuota.providerId)">
          <el-descriptions :column="2" border class="detail-descriptions">
            <el-descriptions-item label="供应商">
              {{ selectedQuota.providerName }}
            </el-descriptions-item>
            <el-descriptions-item label="计划">
              OpenCode Go
            </el-descriptions-item>
          </el-descriptions>

          <!-- 三维度用量 -->
          <div class="detail-section-title">用量详情</div>
          <template v-for="dim in ['rolling', 'weekly', 'monthly']" :key="dim">
            <template v-if="selectedQuota.limits">
              <template v-for="item in selectedQuota.limits.filter((l: any) => l.type === dim)" :key="dim">
                <div style="font-size:13px;margin-top:12px;margin-bottom:4px">{{ item.label }}</div>
                <div style="display:flex;align-items:center;gap:8px;margin-bottom:2px">
                  <el-progress
                    :percentage="item.usagePercent ?? 0"
                    :color="getProgressColor(item.usagePercent ?? 0)"
                    :stroke-width="8"
                    :show-text="false"
                    style="flex:1"
                  />
                  <span style="font-size:13px;color:var(--app-text-primary);white-space:nowrap">
                    {{ item.usagePercent != null ? `${item.usagePercent}%` : '--' }}
                  </span>
                </div>
                <div v-if="item.resetTime" style="font-size:12px;color:var(--app-text-tertiary);margin-bottom:8px">
                  {{ item.resetTime }}
                </div>
              </template>
            </template>
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

    <!-- OpenCode Go 设置弹窗 -->
    <OpenCodeGoSettingsDialog
      :visible="settingsDialogVisible"
      @update:visible="settingsDialogVisible = $event"
      @saved="onSettingsSaved"
    />
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

/* ==================== 详情弹窗 ==================== */
.detail-dialog .dialog-loading {
  min-height: 320px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-loading .loading-spinner-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.dialog-loading .loading-spinner {
  font-size: 32px;
  color: var(--app-color-primary);
}

.dialog-loading .loading-text {
  font-size: 14px;
  color: var(--app-text-tertiary);
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

/* 固定表格布局，防止数值变化导致列宽抖动 */
.detail-descriptions :deep(.el-descriptions__table) {
  table-layout: fixed;
  width: 100%;
}

.detail-descriptions :deep(.el-descriptions__label) {
  color: var(--app-text-tertiary);
  background: var(--app-bg-hover);
  font-size: 13px;
  width: 120px;
}

.detail-descriptions :deep(.el-descriptions__content) {
  color: var(--app-text-primary);
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

.detail-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin: 20px 0 12px;
  padding-left: 10px;
  border-left: 3px solid var(--app-color-primary);
}

/* 用量 Tab 切换 */
.usage-tabs {
  display: inline-flex;
  gap: 4px;
  margin: 16px 0 0;
  background: var(--app-bg-hover);
  border-radius: 10px;
  padding: 4px;
  border: 1px solid var(--app-border-default);
}

.usage-tab {
  padding: 6px 20px;
  border: 1px solid transparent;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
  background: transparent;
  color: var(--app-text-tertiary);
  letter-spacing: 0.3px;
}

.usage-tab:hover {
  color: var(--app-text-primary);
}

.usage-tab.active {
  background: var(--app-bg-card);
  color: var(--app-color-primary);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
  border-color: var(--app-border-default);
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

html.cyberpunk .usage-tabs {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk .usage-tab {
  color: #8080a0;
}

html.cyberpunk .usage-tab:hover {
  color: #00ffff;
}

html.cyberpunk .usage-tab.active {
  background: rgba(0, 255, 255, 0.12);
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.15);
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

html.glassmorphism .usage-tabs {
  background: #f1f5f9;
  border-color: #e2e8f0;
}

html.glassmorphism .usage-tab {
  color: #94a3b8;
}

html.glassmorphism .usage-tab:hover {
  color: #1e293b;
}

html.glassmorphism .usage-tab.active {
  background: #ffffff;
  color: #2563eb;
  border-color: #e2e8f0;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
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
}
</style>

<!-- 非 scoped：el-dialog teleport 到 .app-main 后 scoped 选择器失效，需要全局样式 -->
<style>
.detail-dialog .el-dialog__body {
  max-height: calc(100vh - 200px) !important;
  overflow-y: auto !important;
}

.detail-dialog .el-dialog__body::-webkit-scrollbar {
  width: 6px;
}

.detail-dialog .el-dialog__body::-webkit-scrollbar-track {
  background: transparent;
}

.detail-dialog .el-dialog__body::-webkit-scrollbar-thumb {
  background: var(--app-border-default);
  border-radius: 3px;
}

.detail-dialog .el-dialog__body::-webkit-scrollbar-thumb:hover {
  background: var(--app-color-primary);
}
</style>
