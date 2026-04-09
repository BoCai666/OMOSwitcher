<script setup lang="ts">
/**
 * 模型管理页面
 * 以供应商卡片网格展示全量模型注册表
 * 数据来源：~/.cache/opencode/models.json + opencode.json + antigravity-accounts.json
 */
import { ref, computed, onMounted } from 'vue'
import { Search, Close, InfoFilled, Refresh, Plus } from '@element-plus/icons-vue'
import type { RegistryProvider, RegistryModel } from '@/types/config'
import type { ProviderWithAvailability, CustomProviderConfig } from '@/services/opencodeModels'
import {
  getProvidersWithAvailability,
  clearRegistryCache,
  deleteCustomProvider,
  addCustomProvider
} from '@/services/opencodeModels'
import { showError, showSuccess, confirm } from '@/utils/errorHandler'

// 加载状态
const loading = ref(true)
const errorMsg = ref('')

// 供应商卡片列表
const providers = ref<ProviderWithAvailability[]>([])

// 搜索关键词
const searchKeyword = ref('')

// 筛选标签：all / available / custom / builtin
const filterTab = ref<'all' | 'available' | 'custom' | 'builtin'>('all')

// 内置供应商 ID 集合（来自 OpenCode 内部注册表，非用户自定义）
const builtinProviderIds = ref<Set<string>>(new Set())

// 选中的供应商（查看模型列表）
const selectedProvider = ref<ProviderWithAvailability | null>(null)

// 模型详情对话框
const modelDetailVisible = ref(false)
const modelDetail = ref<(RegistryModel & { providerId: string }) | null>(null)

// 模型搜索关键词（在供应商详情内）
const modelSearchKeyword = ref('')

// 统计信息
const stats = computed(() => {
  const total = providers.value.length
  const available = providers.value.filter(p => p.available).length
  const custom = providers.value.filter(p => p.custom).length
  const builtin = providers.value.filter(p => builtinProviderIds.value.has(p.id)).length
  const modelCount = providers.value.reduce((sum, p) => sum + p.modelCount, 0)
  return { total, available, custom, builtin, modelCount }
})

// 筛选后的供应商列表
const filteredProviders = computed(() => {
  let list = providers.value

  // 按可用性筛选
  if (filterTab.value === 'available') {
    list = list.filter(p => p.available)
  }

  // 按自定义筛选
  if (filterTab.value === 'custom') {
    list = list.filter(p => p.custom)
  }

  // 按内置筛选（models.json 中包含的供应商）
  if (filterTab.value === 'builtin') {
    list = list.filter(p => builtinProviderIds.value.has(p.id))
  }

  // 按搜索关键词筛选
  if (searchKeyword.value) {
    const kw = searchKeyword.value.toLowerCase()
    list = list.filter(p => {
      const nameMatch = (p.name || p.id).toLowerCase().includes(kw)
      const idMatch = p.id.toLowerCase().includes(kw)
      // 也搜索模型名称
      if (!nameMatch && !idMatch) {
        return Object.values(p.models || {}).some(m =>
          m.name.toLowerCase().includes(kw) ||
          m.id.toLowerCase().includes(kw)
        )
      }
      return true
    })
  }

  return list
})

// 获取供应商的模型列表（搜索过滤后）
function getProviderModels(provider: RegistryProvider): RegistryModel[] {
  const models = Object.values(provider.models || {})
  if (!modelSearchKeyword.value) return models
  const kw = modelSearchKeyword.value.toLowerCase()
  return models.filter(m =>
    m.name.toLowerCase().includes(kw) ||
    m.id.toLowerCase().includes(kw)
  )
}

// 格式化上下文长度
function formatContextSize(limit?: number): string {
  if (!limit) return '-'
  if (limit >= 1048576) return `${(limit / 1048576).toFixed(1)}M`
  if (limit >= 1024) return `${(limit / 1024).toFixed(0)}K`
  return limit.toString()
}

// 加载数据（silent 为 true 时静默刷新，不显示 loading 遮罩，避免暗色模式白屏闪烁）
async function loadData(silent = false) {
  if (!silent) loading.value = true
  errorMsg.value = ''
  try {
    console.log('[ModelManage] 开始加载数据...')
    providers.value = await getProvidersWithAvailability()

    // 提取内置供应商 ID（非自定义供应商 = OpenCode 内部支持的供应商）
    builtinProviderIds.value = new Set(
      providers.value.filter(p => !p.custom).map(p => p.id)
    )

    console.log('[ModelManage] 获取到供应商数量:', providers.value.length)
    console.log('[ModelManage] 自定义供应商:', providers.value.filter(p => p.custom).map(p => ({ id: p.id, custom: p.custom })))
    console.log('[ModelManage] 可用供应商:', providers.value.filter(p => p.available).map(p => p.id))
    console.log('[ModelManage] 内置供应商:', builtinProviderIds.value)
    console.log('[ModelManage] stats:', stats.value)
    if (providers.value.length === 0) {
      errorMsg.value = '未找到模型注册表。请先运行一次 OpenCode 以生成缓存。'
    }
  } catch (e) {
    console.error('[ModelManage] 加载失败:', e)
    errorMsg.value = '加载失败：' + (e instanceof Error ? e.message : String(e))
    showError(errorMsg.value)
  } finally {
    loading.value = false
  }
}

// 选择供应商查看模型
function selectProvider(provider: ProviderWithAvailability) {
  if (selectedProvider.value?.id === provider.id) {
    selectedProvider.value = null
  } else {
    selectedProvider.value = provider
    modelSearchKeyword.value = ''
  }
}

// 查看模型详情
function showModelDetail(model: RegistryModel, providerId: string) {
  modelDetail.value = { ...model, providerId }
  modelDetailVisible.value = true
}

// 刷新数据（静默刷新，不触发 loading 遮罩）
async function refreshData() {
  clearRegistryCache()
  await loadData(true)
}

// 删除自定义供应商
async function handleDeleteProvider(provider: ProviderWithAvailability) {
  const confirmed = await confirm(
    `确定要删除自定义供应商 "${provider.name || provider.id}" 吗？此操作将从 opencode.json 中移除该供应商配置，不可恢复。`,
    '删除供应商'
  )
  if (!confirmed) return

  try {
    await deleteCustomProvider(provider.id)
    // 如果当前选中的就是被删除的供应商，清除选中状态
    if (selectedProvider.value?.id === provider.id) {
      selectedProvider.value = null
    }
    showSuccess(`已删除供应商: ${provider.name || provider.id}`)
    await refreshData()
  } catch (e) {
    showError(e)
  }
}

// ==================== 添加自定义供应商 ====================

// 对话框状态
const addProviderVisible = ref(false)
const addProviderLoading = ref(false)

// 表单数据
const addForm = ref({
  providerId: '',
  name: '',
  npm: '@ai-sdk/openai-compatible',
  apiKey: '',
  baseURL: '',
  // 模型列表（至少一个模型）
  models: [
    {
      id: '',
      name: '',
      reasoning: false,
      context: 128000,
      output: 8192,
      inputText: true,
      inputImage: false,
      inputVideo: false,
      outputText: true,
      variants: [] as { name: string; config: string }[]
    }
  ]
})

// 打开添加对话框
function openAddProvider() {
  addForm.value = {
    providerId: '',
    name: '',
    npm: '@ai-sdk/openai-compatible',
    apiKey: '',
    baseURL: '',
    models: [
      {
        id: '',
        name: '',
        reasoning: false,
        context: 128000,
        output: 8192,
        inputText: true,
        inputImage: false,
        inputVideo: false,
        outputText: true,
        variants: []
      }
    ]
  }
  addProviderVisible.value = true
}

// 添加一行模型配置
function addModelRow() {
  addForm.value.models.push({
    id: '', name: '', reasoning: false,
    context: 128000, output: 8192, inputText: true, inputImage: false, inputVideo: false, outputText: true,
    variants: []
  })
}

// 移除指定行的模型配置
function removeModelRow(index: number) {
  addForm.value.models.splice(index, 1)
}

// 表单验证
function validateAddForm(): string | null {
  const form = addForm.value
  if (!form.providerId.trim()) return '请输入供应商 ID'
  if (!/^[a-zA-Z0-9_-]+$/.test(form.providerId.trim())) {
    return '供应商 ID 只能包含字母、数字、下划线和连字符'
  }
  // 检查是否与已有供应商冲突
  if (providers.value.some(p => p.id === form.providerId.trim())) {
    return `供应商 "${form.providerId}" 已存在，请使用其他 ID`
  }
  if (!form.apiKey.trim()) return '请输入 API Key'

  // 验证模型列表
  const validModels = form.models.filter(m => m.id.trim())
  if (validModels.length === 0) return '请至少添加一个模型'
  for (const model of validModels) {
    if (/\s/.test(model.id.trim())) return `模型 ID "${model.id}" 不能包含空格`
  }
  return null
}

// 提交添加
async function handleAddProvider() {
  const validationError = validateAddForm()
  if (validationError) {
    showError(validationError)
    return
  }

  addProviderLoading.value = true
  try {
    const form = addForm.value
    const config: CustomProviderConfig = {
      npm: form.npm.trim() || undefined,
      name: form.name.trim() || undefined,
      options: {}
    }

    // 设置 options
    if (form.apiKey.trim()) {
      config.options!.apiKey = form.apiKey.trim()
    }
    if (form.baseURL.trim()) {
      config.options!.baseURL = form.baseURL.trim()
    }
    // 清理空 options
    if (Object.keys(config.options!).length === 0) {
      delete config.options
    }

    // 构建模型配置
    const models: CustomProviderConfig['models'] = {}
    for (const model of form.models) {
      const modelId = model.id.trim()
      if (!modelId) continue

      const inputModalities: string[] = []
      if (model.inputText) inputModalities.push('text')
      if (model.inputImage) {
        inputModalities.push('image')
        inputModalities.push('pdf')
      }
      if (model.inputVideo) inputModalities.push('video')

      models[modelId] = {
        name: model.name.trim() || modelId,
        reasoning: model.reasoning || undefined,
        ...(model.reasoning ? {
          options: {
            thinking: {
              type: 'enabled'
            }
          }
        } : {}),
        limit: {
          context: model.context || 128000,
          output: model.output || 8192
        },
        modalities: {
          input: inputModalities.length > 0 ? inputModalities : undefined,
          output: model.outputText ? ['text'] : undefined
        }
      }

      // 构建变体配置
      if (model.variants && model.variants.length > 0) {
        const variants: Record<string, Record<string, unknown>> = {}
        for (const v of model.variants) {
          const vName = v.name.trim()
          if (!vName) continue
          try {
            variants[vName] = JSON.parse(v.config)
          } catch {
            // 无效 JSON 跳过
          }
        }
        if (Object.keys(variants).length > 0) {
          models[modelId]!.variants = variants
        }
      }

      // 清理 undefined 值
      const m = models[modelId] as Record<string, unknown>
      for (const key of Object.keys(m)) {
        if (m[key] === undefined) delete m[key]
      }
    }
    config.models = models

    await addCustomProvider(form.providerId.trim(), config)
    showSuccess(`已添加供应商: ${form.name || form.providerId}`)
    addProviderVisible.value = false
    await refreshData()
  } catch (e) {
    showError(e)
  } finally {
    addProviderLoading.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>

<template>
  <div class="model-manage" v-loading="loading" element-loading-text="加载模型注册表...">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <div class="stats-row">
          <span class="stat-item stat-available">
            <span class="stat-dot available" />
            {{ stats.available }} 可用
          </span>
          <span class="stat-divider">/</span>
          <span class="stat-item">{{ stats.total }} 供应商</span>
          <span class="stat-divider">/</span>
          <span class="stat-item">{{ stats.modelCount }} 模型</span>
        </div>
      </div>
      <div class="header-right">
        <el-input
          v-model="searchKeyword"
          placeholder="搜索供应商或模型..."
          :prefix-icon="Search"
          clearable
          class="search-input"
        />
        <el-button @click="openAddProvider" type="primary" plain>
          <el-icon><Plus /></el-icon>
          <span>添加供应商</span>
        </el-button>
        <el-button @click="refreshData" :loading="loading">
          <el-icon><Refresh /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 筛选标签 -->
    <div class="filter-tabs">
      <button
        class="filter-tab"
        :class="{ active: filterTab === 'all' }"
        @click="filterTab = 'all'"
      >
        全部 ({{ stats.total }})
      </button>
      <button
        class="filter-tab"
        :class="{ active: filterTab === 'available' }"
        @click="filterTab = 'available'"
      >
        <span class="tab-dot available" />
        可用 ({{ stats.available }})
      </button>
      <button
        class="filter-tab"
        :class="{ active: filterTab === 'custom' }"
        @click="filterTab = 'custom'"
      >
        <span class="tab-dot custom" />
        自定义 ({{ stats.custom }})
      </button>
      <button
        class="filter-tab"
        :class="{ active: filterTab === 'builtin' }"
        @click="filterTab = 'builtin'"
      >
        <span class="tab-dot builtin" />
        内置 ({{ stats.builtin }})
      </button>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMsg" class="error-banner">
      <el-icon><InfoFilled /></el-icon>
      <span>{{ errorMsg }}</span>
    </div>

    <!-- 加载骨架屏 -->
    <div v-if="loading" class="loading-skeleton">
      <el-skeleton :rows="5" animated />
    </div>

    <!-- 主内容区：左右分栏 -->
    <div v-else-if="providers.length > 0" class="content-area">
      <!-- 左侧：供应商卡片网格 -->
      <div class="provider-grid-wrap">
        <div class="provider-grid">
          <div
            v-for="provider in filteredProviders"
            :key="provider.id"
            class="provider-card"
            :class="{
              'is-available': provider.available,
              'is-selected': selectedProvider?.id === provider.id,
              'is-unavailable': !provider.available
            }"
            @click="selectProvider(provider)"
          >
            <div class="card-header">
              <span class="provider-name">{{ provider.name || provider.id }}</span>
              <span class="status-dot" :class="provider.available ? 'available' : 'unavailable'" />
            </div>
            <div class="card-meta">
              <span class="model-count">{{ provider.modelCount }} 模型</span>
              <span v-if="provider.available" class="available-badge">已配置</span>
              <span v-if="provider.custom" class="custom-badge">自定义</span>
            </div>
            <div v-if="provider.id !== (provider.name || provider.id)" class="card-id" :class="{ 'is-available': provider.available }">
              {{ provider.id }}
            </div>
          </div>
        </div>
        <div v-if="filteredProviders.length === 0" class="empty-state">
          <el-empty description="没有匹配的供应商" />
        </div>
      </div>

      <!-- 右侧：模型列表面板 -->
      <div v-if="selectedProvider" class="model-panel">
        <div class="panel-header">
          <div class="panel-title-row">
            <span class="panel-title">{{ selectedProvider.name }}</span>
            <span class="status-dot" :class="selectedProvider.available ? 'available' : 'unavailable'" />
            <button
              v-if="selectedProvider.custom"
              class="delete-provider-btn"
              title="删除此自定义供应商"
              @click="handleDeleteProvider(selectedProvider)"
            >
              <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" width="14" height="14">
                <path d="M3 6H21M8 6V4C8 3.44772 8.44772 3 9 3H15C15.5523 3 16 3.44772 16 4V6M19 6V20C19 20.5523 18.5523 21 18 21H6C5.44772 21 5 20.5523 5 20V6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <span>删除</span>
            </button>
            <el-button text @click="selectedProvider = null">
              <el-icon><Close /></el-icon>
            </el-button>
          </div>
          <div class="panel-subtitle">
            {{ selectedProvider.id }} · {{ selectedProvider.modelCount }} 个模型
            <span v-if="selectedProvider.api" class="api-url">· {{ selectedProvider.api }}</span>
          </div>
          <el-input
            v-model="modelSearchKeyword"
            placeholder="搜索模型..."
            :prefix-icon="Search"
            clearable
            size="small"
            class="model-search"
          />
        </div>
        <div class="model-list">
          <div
            v-for="model in getProviderModels(selectedProvider)"
            :key="model.id"
            class="model-item"
            @click="showModelDetail(model, selectedProvider.id)"
          >
            <div class="model-item-header">
              <span class="model-name">{{ model.name || model.id }}</span>
              <div class="model-badges">
                <span v-if="model.tool_call" class="badge badge-tool">工具</span>
                <span v-if="model.reasoning" class="badge badge-reasoning">推理</span>
                <span v-if="model.modalities?.input?.includes('image')" class="badge badge-image">图片</span>
              </div>
            </div>
            <div class="model-item-meta">
              <code class="model-id-code">{{ selectedProvider.id }}/{{ model.id }}</code>
              <span v-if="model.limit?.context" class="context-size">{{ formatContextSize(model.limit.context) }}</span>
            </div>
          </div>
          <div v-if="getProviderModels(selectedProvider).length === 0" class="empty-state small">
            <el-empty description="没有匹配的模型" :image-size="60" />
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state full">
      <el-empty description="暂无模型注册表数据">
        <el-button type="primary" @click="refreshData">重新加载</el-button>
      </el-empty>
    </div>

    <!-- 模型详情对话框 -->
    <el-dialog
      v-model="modelDetailVisible"
      :title="modelDetail?.name || modelDetail?.id"
      width="520px"
      append-to=".app-main"
      align-center
      class="model-detail-dialog"
    >
      <div v-if="modelDetail" class="detail-content">
        <div class="detail-row">
          <span class="detail-label">模型 ID</span>
          <code class="detail-value">{{ modelDetail.providerId }}/{{ modelDetail.id }}</code>
        </div>
        <div v-if="modelDetail.family" class="detail-row">
          <span class="detail-label">模型家族</span>
          <span class="detail-value">{{ modelDetail.family }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">工具调用</span>
          <span class="detail-value">{{ modelDetail.tool_call ? '✓ 支持' : '✗ 不支持' }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">推理能力</span>
          <span class="detail-value">{{ modelDetail.reasoning ? '✓ 支持' : '✗ 不支持' }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">附件上传</span>
          <span class="detail-value">{{ modelDetail.attachment ? '✓ 支持' : '✗ 不支持' }}</span>
        </div>
        <div v-if="modelDetail.limit" class="detail-row">
          <span class="detail-label">上下文长度</span>
          <span class="detail-value">{{ formatContextSize(modelDetail.limit.context) }}</span>
        </div>
        <div v-if="modelDetail.limit?.output" class="detail-row">
          <span class="detail-label">最大输出</span>
          <span class="detail-value">{{ formatContextSize(modelDetail.limit.output) }}</span>
        </div>
        <div v-if="modelDetail.modalities" class="detail-row">
          <span class="detail-label">输入模态</span>
          <span class="detail-value">{{ modelDetail.modalities.input?.join(', ') || '-' }}</span>
        </div>
        <div v-if="modelDetail.modalities" class="detail-row">
          <span class="detail-label">输出模态</span>
          <span class="detail-value">{{ modelDetail.modalities.output?.join(', ') || '-' }}</span>
        </div>
        <div v-if="modelDetail.release_date" class="detail-row">
          <span class="detail-label">发布日期</span>
          <span class="detail-value">{{ modelDetail.release_date }}</span>
        </div>
        <div v-if="modelDetail.open_weights !== undefined" class="detail-row">
          <span class="detail-label">开放权重</span>
          <span class="detail-value">{{ modelDetail.open_weights ? '是' : '否' }}</span>
        </div>
      </div>
    </el-dialog>

    <!-- 添加自定义供应商对话框 -->
    <el-dialog
      v-model="addProviderVisible"
      title="添加自定义供应商"
      width="620px"
      append-to=".app-main"
      align-center
      :close-on-click-modal="true"
      class="add-provider-dialog"
    >
      <div class="add-form">
        <!-- 供应商 ID -->
        <div class="form-section">
          <div class="form-section-title">基本信息</div>
          <div class="form-row">
            <label class="form-label required">供应商 ID</label>
            <el-input
              v-model="addForm.providerId"
              placeholder="例如: my-provider（仅字母数字下划线连字符）"
              maxlength="50"
            />
          </div>
          <div class="form-row">
            <label class="form-label">显示名称</label>
            <el-input
              v-model="addForm.name"
              placeholder="例如: My Custom Provider（可选）"
              maxlength="100"
            />
          </div>
        </div>

        <!-- API 配置 -->
        <div class="form-section">
          <div class="form-section-title">API 配置</div>
          <div class="form-row">
            <label class="form-label">API 格式</label>
            <el-select v-model="addForm.npm" placeholder="选择 API 格式">
              <el-option
                label="OpenAI"
                value="@ai-sdk/openai-compatible"
              />
              <el-option
                label="Anthropic"
                value="@ai-sdk/anthropic"
              />
            </el-select>
          </div>
          <div class="form-row">
            <label class="form-label required">API Key</label>
            <el-input
              v-model="addForm.apiKey"
              placeholder="sk-... 或 {env:API_KEY_NAME}"
              show-password
            />
            <div class="form-hint">支持直接填写密钥或使用 {env:环境变量名} 引用</div>
          </div>
          <div class="form-row">
            <label class="form-label">Base URL</label>
            <el-input
              v-model="addForm.baseURL"
              placeholder="https://api.example.com/v1（OpenAI 兼容接口地址）"
            />
          </div>
        </div>

        <!-- 模型配置 -->
        <div class="form-section">
          <div class="form-section-header">
            <div class="form-section-title">模型列表</div>
            <el-button size="small" @click="addModelRow" plain>
              <el-icon><Plus /></el-icon>
              添加模型
            </el-button>
          </div>

          <div v-for="(model, index) in addForm.models" :key="index" class="model-form-item">
            <div class="model-form-header">
              <span class="model-form-index">模型 {{ index + 1 }}</span>
              <el-button
                v-if="addForm.models.length > 1"
                size="small"
                text
                type="danger"
                @click="removeModelRow(index)"
              >
                移除
              </el-button>
            </div>
            <div class="model-form-body">
              <div class="model-form-row">
                <div class="model-form-field">
                  <label class="form-label-sm required">模型 ID</label>
                  <el-input
                    v-model="model.id"
                    placeholder="例如: gpt-4o"
                    size="small"
                  />
                </div>
                <div class="model-form-field">
                  <label class="form-label-sm">显示名称</label>
                  <el-input
                    v-model="model.name"
                    placeholder="例如: GPT-4o"
                    size="small"
                  />
                </div>
              </div>
              <div class="model-form-row">
                <div class="model-form-field">
                  <label class="form-label-sm">上下文长度</label>
                  <el-input-number
                    v-model="model.context"
                    :min="1024"
                    :step="1024"
                    size="small"
                    controls-position="right"
                  />
                </div>
                <div class="model-form-field">
                  <label class="form-label-sm">最大输出</label>
                  <el-input-number
                    v-model="model.output"
                    :min="256"
                    :step="1024"
                    size="small"
                    controls-position="right"
                  />
                </div>
              </div>
              <div class="model-form-row">
                <div class="model-form-field checkbox-field">
                  <el-checkbox v-model="model.reasoning" label="推理能力" size="small" />
                </div>
              </div>
              <div class="model-form-row">
                <div class="model-form-field checkbox-field">
                  <label class="form-label-sm" style="margin-bottom:0">输入模态</label>
                  <el-checkbox v-model="model.inputText" label="文本" size="small" />
                  <el-checkbox v-model="model.inputImage" label="图片" size="small" />
                  <el-checkbox v-model="model.inputVideo" label="视频" size="small" />
                </div>
                <div class="model-form-field checkbox-field">
                  <label class="form-label-sm" style="margin-bottom:0">输出模态</label>
                  <el-checkbox v-model="model.outputText" label="文本" size="small" />
                </div>
              </div>
              <!-- 变体配置 -->
              <div class="variant-section">
                <div class="variant-section-header">
                  <span class="form-label-sm">变体 (Variants)</span>
                  <el-button size="small" text @click="model.variants.push({ name: '', config: '' })">
                    <el-icon><Plus /></el-icon> 添加变体
                  </el-button>
                </div>
                <div v-if="model.variants.length === 0" class="variant-empty">
                  暂无变体，点击"添加变体"配置
                </div>
                <div v-for="(variant, vi) in model.variants" :key="vi" class="variant-row">
                  <div class="variant-field">
                    <el-input
                      v-model="variant.name"
                      placeholder="变体名称，如 high"
                      size="small"
                    />
                  </div>
                  <div class="variant-field variant-field-config">
                    <el-input
                      v-model="variant.config"
                      placeholder='JSON 配置，如 {"reasoningEffort":"high"}'
                      size="small"
                    />
                  </div>
                  <el-button size="small" text type="danger" @click="model.variants.splice(vi, 1)">
                    移除
                  </el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="addProviderVisible = false">取消</el-button>
        <el-button
          type="primary"
          @click="handleAddProvider"
          :loading="addProviderLoading"
        >
          添加供应商
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
/* 容器样式 */
.model-manage {
  max-width: 1400px;
  margin: 0 auto;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-4);
  padding: var(--app-spacing-4) var(--app-spacing-5);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
  gap: var(--app-spacing-4);
  flex-wrap: wrap;
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.stats-row {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-size: 14px;
  color: var(--app-text-tertiary);
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stat-available {
  color: var(--app-color-success, #22c55e);
  font-weight: 600;
}

.stat-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.stat-dot.available {
  background: var(--app-color-success, #22c55e);
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
}

.stat-divider {
  color: var(--app-border-default);
}

.search-input {
  width: 260px;
}

/* 筛选标签 */
.filter-tabs {
  display: flex;
  gap: var(--app-spacing-2);
  margin-bottom: var(--app-spacing-4);
}

.filter-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  background: var(--app-bg-card);
  color: var(--app-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--app-transition-fast);
}

.filter-tab:hover {
  border-color: var(--app-border-hover);
  color: var(--app-text-primary);
}

.filter-tab.active {
  background: rgba(0, 212, 255, 0.1);
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
}

.tab-dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.tab-dot.available {
  background: var(--app-color-success, #22c55e);
}

.tab-dot.custom {
  background: #a855f7;
}

.tab-dot.builtin {
  background: #3b82f6;
}

/* 错误提示 */
.error-banner {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--app-radius-md);
  color: var(--app-color-danger);
  font-size: 13px;
  margin-bottom: var(--app-spacing-4);
}

/* 加载骨架屏 */
.loading-skeleton {
  padding: var(--app-spacing-6);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
}

/* 主内容区：左右分栏 */
.content-area {
  display: flex;
  gap: var(--app-spacing-4);
  min-height: 500px;
}

/* 左侧供应商网格 */
.provider-grid-wrap {
  flex: 1;
  min-width: 0;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--app-spacing-3);
}

/* 供应商卡片 */
.provider-card {
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  cursor: pointer;
  transition: all var(--app-transition-fast);
  position: relative;
  overflow: hidden;
}

.provider-card:hover {
  border-color: var(--app-border-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.provider-card.is-available {
  border-left: 3px solid var(--app-color-success, #22c55e);
}

.provider-card.is-selected {
  border-color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.05);
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.15);
}

.provider-card.is-unavailable {
  opacity: 0.85;
  border-style: dashed;
}

.provider-card.is-unavailable:hover {
  opacity: 1;
}

/* 自定义徽章 */
.custom-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: var(--app-radius-sm);
  background: rgba(168, 85, 247, 0.15);
  color: #a855f7;
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-2);
}

.provider-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--app-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 170px;
}

/* 状态圆点 */
.status-dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid transparent;
}

.status-dot.available {
  background: var(--app-color-success, #22c55e);
  border-color: var(--app-color-success, #22c55e);
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.status-dot.unavailable {
  background: var(--app-text-disabled, #9ca3af);
  border-color: var(--app-border-default, #d1d5db);
}

.card-meta {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.available-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: var(--app-radius-sm);
  background: rgba(34, 197, 94, 0.15);
  color: var(--app-color-success, #22c55e);
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.card-id {
  margin-top: var(--app-spacing-1);
  font-size: 11px;
  font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Consolas', 'Liberation Mono', monospace;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.38);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-id.is-available {
  color: rgba(22, 163, 74, 0.7);
}

/* 右侧模型面板 */
.model-panel {
  width: 420px;
  flex-shrink: 0;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 280px);
  position: sticky;
  top: var(--app-spacing-4);
}

.panel-header {
  padding: var(--app-spacing-4);
  border-bottom: 1px solid var(--app-border-default);
}

.panel-title-row {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.panel-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--app-text-primary);
}

/* 面板中的删除供应商按钮 */
.delete-provider-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  padding: 4px 10px;
  border-radius: var(--app-radius-md);
  border: 1px solid rgba(239, 68, 68, 0.3);
  background: transparent;
  color: var(--app-color-danger);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--app-transition-fast);
  flex-shrink: 0;
}

.delete-provider-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--app-color-danger);
}

.panel-subtitle {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin-bottom: var(--app-spacing-3);
  margin-top: 2px;
}

.api-url {
  word-break: break-all;
}

.model-search {
  width: 100%;
}

/* 模型列表 */
.model-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--app-spacing-2);
}

.model-item {
  padding: var(--app-spacing-3);
  border: 1px solid transparent;
  border-radius: var(--app-radius-md);
  cursor: pointer;
  transition: all var(--app-transition-fast);
  margin-bottom: var(--app-spacing-1);
}

.model-item:hover {
  background: rgba(0, 212, 255, 0.05);
  border-color: var(--app-border-hover);
}

.model-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--app-spacing-2);
}

.model-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-badges {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 500;
}

.badge-tool {
  background: rgba(0, 212, 255, 0.15);
  color: var(--app-color-primary);
  border: 1px solid rgba(0, 212, 255, 0.3);
}

.badge-reasoning {
  background: rgba(168, 85, 247, 0.15);
  color: #a855f7;
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.badge-image {
  background: rgba(234, 179, 8, 0.15);
  color: #eab308;
  border: 1px solid rgba(234, 179, 8, 0.3);
}

.model-item-meta {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  margin-top: 4px;
}

.model-id-code {
  font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Consolas', 'Liberation Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  color: var(--app-text-disabled);
  background: rgba(0, 212, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-size {
  font-size: 11px;
  color: var(--app-text-tertiary);
}

/* 空状态 */
.empty-state {
  padding: var(--app-spacing-8) 0;
}

.empty-state.full {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state.small {
  padding: var(--app-spacing-6) 0;
}

/* ==================== 模型详情对话框 ==================== */
.detail-content {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-3);
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--app-spacing-2) 0;
  border-bottom: 1px solid var(--app-border-default);
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-label {
  color: var(--app-text-tertiary);
  font-size: 13px;
  flex-shrink: 0;
}

.detail-value {
  color: var(--app-text-primary);
  font-size: 13px;
  text-align: right;
}

code.detail-value {
  font-family: 'Cascadia Code', 'SF Mono', 'Fira Code', 'Consolas', 'Liberation Mono', monospace;
  font-weight: 500;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(0, 212, 255, 0.05);
  color: var(--app-color-primary);
}

/* ==================== 暗色主题 ==================== */
html.dark .provider-card {
  background: rgba(18, 18, 26, 0.9);
  border-color: rgba(255, 255, 255, 0.12);
}

html.dark .provider-card:hover {
  border-color: rgba(255, 255, 255, 0.2);
  background: rgba(18, 18, 26, 0.95);
}

html.dark .provider-card.is-available {
  border-left-color: #22c55e;
  border-left-width: 3px;
}

html.dark .provider-card.is-available:hover {
  border-color: rgba(34, 197, 94, 0.3);
}

html.dark .provider-card.is-selected {
  background: rgba(0, 212, 255, 0.08);
  border-color: var(--app-color-primary);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

html.dark .provider-card.is-unavailable {
  opacity: 0.75;
  border-color: rgba(255, 255, 255, 0.08);
  background: rgba(18, 18, 26, 0.6);
}

html.dark .provider-card.is-unavailable:hover {
  opacity: 0.95;
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(18, 18, 26, 0.8);
}

html.dark .status-dot.available {
  box-shadow: 0 0 10px rgba(34, 197, 94, 0.8);
}

html.dark .status-dot.unavailable {
  background: #6b7280;
  border-color: #4b5563;
}

html.dark .custom-badge {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
  border-color: rgba(192, 132, 252, 0.3);
}

html.dark .badge-reasoning {
  color: #c084fc;
  border-color: rgba(192, 132, 252, 0.3);
}

html.dark .badge-image {
  color: #facc15;
  border-color: rgba(250, 204, 21, 0.3);
}

html.dark .model-panel {
  background: rgba(18, 18, 26, 0.95);
  border-color: rgba(255, 255, 255, 0.1);
}

html.dark .model-item:hover {
  background: rgba(0, 212, 255, 0.08);
}

html.dark .model-name {
  color: rgba(255, 255, 255, 0.95);
}

html.dark .model-id-code {
  background: rgba(0, 212, 255, 0.08);
  color: rgba(0, 212, 255, 0.8);
  border: 1px solid rgba(0, 212, 255, 0.15);
}

html.dark .provider-name {
  color: rgba(255, 255, 255, 0.95);
}

html.dark .card-id {
  color: rgba(255, 255, 255, 0.38);
}

html.dark .card-id.is-available {
  color: rgba(34, 197, 94, 0.75);
}

html.dark .filter-tab.active {
  background: rgba(0, 212, 255, 0.15);
}

/* 暗色对话框 */
html.dark .model-detail-dialog :deep(.el-dialog) {
  background: rgba(26, 26, 46, 0.95);
  border: 1px solid var(--app-border-default);
}

html.dark .detail-value code {
  background: rgba(0, 212, 255, 0.08);
  color: rgba(0, 212, 255, 0.85);
  padding: 2px 6px;
  border-radius: 4px;
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk .provider-card {
  background: rgba(10, 10, 20, 0.9);
  border-color: rgba(0, 255, 255, 0.15);
}

html.cyberpunk .provider-card:hover {
  border-color: rgba(0, 255, 255, 0.3);
  background: rgba(10, 10, 20, 0.95);
}

html.cyberpunk .provider-card.is-available {
  border-left-color: #00ff88;
  border-left-width: 3px;
  box-shadow: inset 0 0 15px rgba(0, 255, 136, 0.05);
}

html.cyberpunk .provider-card.is-available:hover {
  box-shadow: inset 0 0 20px rgba(0, 255, 136, 0.1);
}

html.cyberpunk .provider-card.is-selected {
  border-color: #00ffff;
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.2), inset 0 0 20px rgba(0, 255, 255, 0.05);
}

html.cyberpunk .provider-card.is-unavailable {
  opacity: 0.8;
  border-color: rgba(0, 255, 255, 0.08);
  background: rgba(10, 10, 20, 0.5);
}

html.cyberpunk .provider-card.is-unavailable:hover {
  opacity: 0.95;
  border-color: rgba(0, 255, 255, 0.2);
  background: rgba(10, 10, 20, 0.7);
}

html.cyberpunk .status-dot.available {
  background: #00ff88;
  box-shadow: 0 0 10px rgba(0, 255, 136, 0.9);
  border-color: #00ff88;
}

html.cyberpunk .status-dot.unavailable {
  background: #4a5568;
  border-color: #2d3748;
}

html.cyberpunk .custom-badge {
  background: rgba(0, 255, 255, 0.15);
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .available-badge {
  background: rgba(0, 255, 136, 0.15);
  color: #00ff88;
  border-color: rgba(0, 255, 136, 0.3);
}

html.cyberpunk .badge-tool {
  background: rgba(0, 255, 255, 0.15);
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .filter-tab.active {
  background: rgba(0, 255, 255, 0.15);
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.4);
}

html.cyberpunk .model-panel {
  background: rgba(5, 5, 15, 0.95);
  border-color: rgba(0, 255, 255, 0.2);
}

html.cyberpunk .model-item:hover {
  background: rgba(0, 255, 255, 0.08);
}

html.cyberpunk .model-name,
html.cyberpunk .provider-name {
  color: rgba(255, 255, 255, 0.95);
}

html.cyberpunk .model-id-code {
  background: rgba(0, 255, 255, 0.1);
  color: rgba(0, 255, 255, 0.85);
  border: 1px solid rgba(0, 255, 255, 0.2);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .delete-provider-btn:hover {
  background: rgba(255, 51, 102, 0.2);
  color: #ff3366;
  box-shadow: 0 0 10px rgba(255, 51, 102, 0.4);
}

html.cyberpunk .card-id {
  color: rgba(255, 255, 255, 0.38);
}

html.cyberpunk .card-id.is-available {
  color: rgba(0, 255, 136, 0.75);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism .provider-card {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(8px);
  border-color: rgba(255, 255, 255, 0.3);
}

html.glassmorphism .provider-card:hover {
  background: rgba(255, 255, 255, 0.75);
  border-color: rgba(255, 255, 255, 0.5);
}

html.glassmorphism .provider-card.is-available {
  border-left-color: #16a34a;
  border-left-width: 3px;
}

html.glassmorphism .provider-card.is-available:hover {
  border-color: rgba(22, 163, 74, 0.3);
}

html.glassmorphism .provider-card.is-selected {
  background: rgba(37, 99, 235, 0.1);
  border-color: #2563eb;
}

html.glassmorphism .provider-card.is-unavailable {
  opacity: 0.85;
  background: rgba(255, 255, 255, 0.5);
  border-color: rgba(200, 200, 200, 0.4);
}

html.glassmorphism .provider-card.is-unavailable:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.65);
  border-color: rgba(180, 180, 180, 0.5);
}

html.glassmorphism .status-dot.available {
  background: #16a34a;
  border-color: #16a34a;
  box-shadow: 0 0 8px rgba(22, 163, 74, 0.5);
}

html.glassmorphism .status-dot.unavailable {
  background: #9ca3af;
  border-color: #d1d5db;
}

html.glassmorphism .custom-badge {
  background: rgba(37, 99, 235, 0.1);
  color: #2563eb;
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .model-panel {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(12px);
}

html.glassmorphism .badge-tool {
  background: rgba(37, 99, 235, 0.1);
  color: #2563eb;
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .filter-tab.active {
  background: rgba(37, 99, 235, 0.1);
  color: #2563eb;
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .model-panel {
  background: rgba(255, 255, 255, 0.7);
  border-color: rgba(200, 200, 200, 0.3);
}

html.glassmorphism .model-item:hover {
  background: rgba(37, 99, 235, 0.05);
}

html.glassmorphism .model-name,
html.glassmorphism .provider-name {
  color: rgba(0, 0, 0, 0.85);
}

html.glassmorphism .model-id-code {
  background: rgba(37, 99, 235, 0.08);
  color: rgba(30, 64, 175, 0.9);
  border: 1px solid rgba(37, 99, 235, 0.15);
}

html.glassmorphism .card-id {
  color: rgba(0, 0, 0, 0.35);
}

html.glassmorphism .card-id.is-available {
  color: rgba(22, 163, 74, 0.7);
}

html.glassmorphism .badge-tool {
  background: rgba(37, 99, 235, 0.1);
  color: #2563eb;
  border-color: rgba(37, 99, 235, 0.3);
}

/* 对话框样式 */
:deep(.el-dialog) {
  background: var(--app-glass-bg, rgba(18, 18, 26, 0.9));
  backdrop-filter: var(--app-glass-blur, blur(20px));
  border: 1px solid var(--app-glass-border, rgba(255, 255, 255, 0.1));
  border-radius: var(--app-radius-xl);
}

:deep(.el-dialog__header) {
  padding: var(--app-spacing-4) var(--app-spacing-5);
  border-bottom: 1px solid var(--app-border-default);
  margin-right: 0;
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
}

:deep(.el-dialog__body) {
  padding: var(--app-spacing-5);
  color: var(--app-text-secondary);
}

/* 响应式 */
@media (max-width: 900px) {
  .content-area {
    flex-direction: column;
  }

  .model-panel {
    width: 100%;
    max-height: 50vh;
    position: static;
  }

  .search-input {
    width: 200px;
  }
}

@media (max-width: 600px) {
  .provider-grid {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  }

  .search-input {
    width: 100%;
  }

  .page-header {
    flex-direction: column;
    align-items: stretch;
  }
}

/* ==================== 添加供应商对话框 ==================== */
.add-form {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-5);
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-3);
}

.form-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.form-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
  padding-bottom: var(--app-spacing-2);
  border-bottom: 1px solid var(--app-border-default);
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text-secondary);
}

.form-label.required::before {
  content: '*';
  color: var(--app-color-danger);
  margin-right: 2px;
}

.form-label-sm {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin-bottom: 2px;
}

.form-label-sm.required::before {
  content: '*';
  color: var(--app-color-danger);
  margin-right: 2px;
}

.form-hint {
  font-size: 11px;
  color: var(--app-text-disabled);
  margin-top: 2px;
}

/* 模型表单条目 */
.model-form-item {
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  padding: var(--app-spacing-3);
  background: var(--app-bg-secondary, rgba(0, 0, 0, 0.02));
}

.model-form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--app-spacing-2);
}

.model-form-index {
  font-size: 12px;
  font-weight: 600;
  color: var(--app-text-secondary);
}

.model-form-body {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
}

.model-form-row {
  display: flex;
  gap: var(--app-spacing-3);
}

.model-form-field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.model-form-field.checkbox-field {
  flex-direction: row;
  align-items: center;
  gap: var(--app-spacing-3);
  padding-top: 4px;
}

/* 变体配置 */
.variant-section {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
  margin-top: var(--app-spacing-1);
  padding-top: var(--app-spacing-2);
  border-top: 1px dashed var(--app-border-default);
}

.variant-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.variant-empty {
  font-size: 11px;
  color: var(--app-text-disabled);
  text-align: center;
  padding: var(--app-spacing-2) 0;
}

.variant-row {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.variant-field {
  width: 140px;
  flex-shrink: 0;
}

.variant-field-config {
  flex: 1;
  width: auto;
}

/* 暗色主题 - 添加对话框 */
html.dark .model-form-item {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.08);
}

html.dark .form-section-title {
  border-bottom-color: rgba(255, 255, 255, 0.1);
}

/* 赛博朋克主题 - 添加对话框 */
html.cyberpunk .model-form-item {
  background: rgba(0, 255, 255, 0.03);
  border-color: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .form-section-title {
  border-bottom-color: rgba(0, 255, 255, 0.15);
  color: #00ffff;
}

/* 玻璃拟态主题 - 添加对话框 */
html.glassmorphism .model-form-item {
  background: rgba(255, 255, 255, 0.3);
  border-color: rgba(200, 200, 200, 0.3);
}

html.glassmorphism .form-section-title {
  border-bottom-color: rgba(200, 200, 200, 0.3);
}
</style>

<!-- 非 scoped 样式：对话框 teleport 到 .app-main 后 scoped 失效，需用全局样式控制 -->
<style>
.add-provider-dialog .el-dialog__body {
  max-height: calc(100vh - 240px);
  overflow-y: auto;
}

/* 明色模式 checkbox 选中色柔和处理 */
html.light .el-checkbox.is-checked .el-checkbox__inner {
  background-color: #409eff !important;
  border-color: #409eff !important;
}
html.light .el-checkbox.is-checked .el-checkbox__label {
  color: #409eff !important;
}
</style>
