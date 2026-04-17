<script setup lang="ts">
/**
 * 编辑自定义供应商对话框
 * 包含完整的表单逻辑和变体配置
 */
import { ref, watch } from 'vue'
import { Plus, ArrowDown, ArrowRight } from '@element-plus/icons-vue'
import type { ProviderWithAvailability, CustomProviderConfig } from '@/services/opencodeModels'
import { updateCustomProvider, getCustomProviderFullConfig } from '@/services/opencodeModels'
import { showError, showSuccess } from '@/utils/errorHandler'
import { getVariantOptions } from '@/composables/useVariantPresets'

// ==================== Props & Emits ====================

const props = defineProps<{
  visible: boolean
  /** 要编辑的供应商，null 时不显示内容 */
  provider: ProviderWithAvailability | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  /** 编辑成功后触发，通知父组件刷新 */
  success: []
}>()

// 编辑对话框状态
const editProviderLoading = ref(false)
const editVariantExpanded = ref<boolean[]>([])

// 编辑表单数据
const editForm = ref({
  providerId: '',  // 不可修改，仅用于显示
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
      variants: [] as string[],
      variantFieldValues: {} as Record<string, Record<string, unknown>>,
    }
  ]
})

// 对话框打开时加载供应商配置
watch(() => props.visible, async (val) => {
  if (val && props.provider) {
    await openEditProvider(props.provider)
  }
})

/**
 * 打开编辑对话框
 * 从选中的供应商加载配置数据到表单
 */
async function openEditProvider(provider: ProviderWithAvailability) {
  // 从 opencode.json 获取完整配置（包含 API Key）
  const fullConfig = await getCustomProviderFullConfig(provider.id)
  
  // 填充基本信息
  editForm.value.providerId = provider.id
  editForm.value.name = fullConfig?.name || provider.name || ''
  editForm.value.npm = fullConfig?.npm || provider.npm || '@ai-sdk/openai-compatible'
  editForm.value.apiKey = fullConfig?.apiKey || ''
  editForm.value.baseURL = fullConfig?.baseURL || provider.api || ''
  
  // 填充模型列表（优先使用完整配置中的 models）
  const modelsMap = fullConfig?.models || provider.models
  const models = Object.entries(modelsMap || {})
  if (models.length > 0) {
    editForm.value.models = models.map(([modelId, m]) => {
      // 解析 variants 配置
      const variants: string[] = []
      const variantFieldValues: Record<string, Record<string, unknown>> = {}
      
      const modelVariants = (m as Record<string, unknown>).variants
      if (modelVariants && typeof modelVariants === 'object') {
        for (const [vKey, vConfig] of Object.entries(modelVariants as Record<string, unknown>)) {
          variants.push(vKey)
          if (vConfig && typeof vConfig === 'object') {
            variantFieldValues[vKey] = vConfig as Record<string, unknown>
          }
        }
      }
      
      const modelData = m as Record<string, unknown>
      const modalities = modelData.modalities
      
      return {
        id: modelId,
        name: (modelData.name as string) || modelId,
        reasoning: (modelData.reasoning as boolean) || false,
        context: (modelData.limit as Record<string, unknown>)?.context as number || 128000,
        output: (modelData.limit as Record<string, unknown>)?.output as number || 8192,
        inputText: (modalities as Record<string, unknown>)?.input
          ? ((modalities as Record<string, unknown>).input as string[]).includes('text')
          : true,
        inputImage: (modalities as Record<string, unknown>)?.input
          ? ((modalities as Record<string, unknown>).input as string[]).includes('image')
          : false,
        inputVideo: (modalities as Record<string, unknown>)?.input
          ? ((modalities as Record<string, unknown>).input as string[]).includes('video')
          : false,
        outputText: (modalities as Record<string, unknown>)?.output
          ? ((modalities as Record<string, unknown>).output as string[]).includes('text')
          : true,
        variants,
        variantFieldValues,
      }
    })
  } else {
    editForm.value.models = [{
      id: '',
      name: '',
      reasoning: false,
      context: 128000,
      output: 8192,
      inputText: true,
      inputImage: false,
      inputVideo: false,
      outputText: true,
      variants: [],
      variantFieldValues: {},
    }]
  }
  
  editVariantExpanded.value = editForm.value.models.map(() => false)
}

// 添加模型行（编辑表单）
function addEditModelRow() {
  editForm.value.models.push({
    id: '', name: '', reasoning: false,
    context: 128000, output: 8192, inputText: true, inputImage: false, inputVideo: false, outputText: true,
    variants: [],
    variantFieldValues: {},
  })
  editVariantExpanded.value.push(false)
}

// 移除模型行（编辑表单）
function removeEditModelRow(index: number) {
  editForm.value.models.splice(index, 1)
  editVariantExpanded.value.splice(index, 1)
}

// 切换变体展开状态（编辑表单）
function toggleEditVariantExpanded(index: number) {
  editVariantExpanded.value[index] = !editVariantExpanded.value[index]
}

// API 格式变更处理（编辑表单）
function handleEditNpmChange() {
  const validKeys = new Set(getVariantOptions(editForm.value.npm).map(v => v.key))
  for (const model of editForm.value.models) {
    model.variants = model.variants.filter(k => validKeys.has(k))
    for (const key of Object.keys(model.variantFieldValues)) {
      if (!validKeys.has(key)) {
        delete model.variantFieldValues[key]
      }
    }
  }
}

// 处理 variant 勾选变更（编辑表单）
function handleEditVariantChange(model: typeof editForm.value.models[0], variantKey: string, checked: boolean) {
  if (checked) {
    if (!model.variants.includes(variantKey)) {
      model.variants.push(variantKey)
    }
    const variantOpt = getVariantOptions(editForm.value.npm).find(v => v.key === variantKey)
    if (variantOpt) {
      const fieldValues: Record<string, unknown> = {}
      for (const field of variantOpt.fields) {
        fieldValues[field.key] = field.default
      }
      model.variantFieldValues[variantKey] = fieldValues
    }
  } else {
    const index = model.variants.indexOf(variantKey)
    if (index > -1) {
      model.variants.splice(index, 1)
    }
    delete model.variantFieldValues[variantKey]
  }
}

// 验证编辑表单
function validateEditForm(): string | null {
  const form = editForm.value
  if (!form.apiKey.trim()) return '请输入 API Key'
  
  const validModels = form.models.filter(m => m.id.trim())
  if (validModels.length === 0) return '请至少添加一个模型'
  for (const model of validModels) {
    if (/\s/.test(model.id.trim())) return `模型 ID "${model.id}" 不能包含空格`
  }
  return null
}

// 提交编辑
async function handleEditProvider() {
  const validationError = validateEditForm()
  if (validationError) {
    showError(validationError)
    return
  }
  
  editProviderLoading.value = true
  try {
    const form = editForm.value
    
    // 构建供应商配置
    const config: Record<string, unknown> = {}
    
    if (form.name.trim()) {
      config.name = form.name.trim()
    }
    
    if (form.npm.trim()) {
      config.npm = form.npm.trim()
    }
    
    const options: Record<string, unknown> = {}
    if (form.apiKey.trim()) {
      options.apiKey = form.apiKey.trim()
    }
    if (form.baseURL.trim()) {
      options.baseURL = form.baseURL.trim()
    }
    if (Object.keys(options).length > 0) {
      config.options = options
    }
    
    // 构建模型配置
    const models: Record<string, Record<string, unknown>> = {}
    for (const model of form.models) {
      const modelId = model.id.trim()
      if (!modelId) continue
      
      const modelConfig: Record<string, unknown> = {}
      modelConfig.name = model.name.trim() || modelId
      modelConfig.limit = {
        context: model.context || 128000,
        output: model.output || 8192
      }
      
      const inputModalities: string[] = []
      if (model.inputText) inputModalities.push('text')
      if (model.inputImage) {
        inputModalities.push('image')
        inputModalities.push('pdf')
      }
      if (model.inputVideo) inputModalities.push('video')
      
      const modalitiesConfig: Record<string, unknown> = {}
      if (inputModalities.length > 0) {
        modalitiesConfig.input = inputModalities
      }
      if (model.outputText) {
        modalitiesConfig.output = ['text']
      }
      if (Object.keys(modalitiesConfig).length > 0) {
        modelConfig.modalities = modalitiesConfig
      }
      
      // 推理能力：勾选后写入 reasoning + options.thinking
      if (model.reasoning) {
        modelConfig.reasoning = true
        modelConfig.options = { thinking: { type: 'enabled' } }
      }

      // 处理 variants
      if (model.variants && model.variants.length > 0) {
        const variants: Record<string, Record<string, unknown>> = {}
        for (const vKey of model.variants) {
          const fieldValues = model.variantFieldValues[vKey]
          if (!fieldValues) continue
          
          const npm = editForm.value.npm
          if (npm === '@ai-sdk/openai-compatible' || npm === '@ai-sdk/openai') {
            variants[vKey] = { ...fieldValues }
          } else if (npm === '@ai-sdk/anthropic') {
            variants[vKey] = {
              thinking: {
                type: 'enabled',
                budgetTokens: fieldValues.budgetTokens || 16000
              }
            }
          } else if (npm === '@ai-sdk/google') {
            const thinkingConfig: Record<string, unknown> = { includeThoughts: true }
            if (fieldValues.thinkingLevel) {
              thinkingConfig.thinkingLevel = fieldValues.thinkingLevel
            }
            if (fieldValues.thinkingBudget) {
              thinkingConfig.thinkingBudget = fieldValues.thinkingBudget
            }
            variants[vKey] = { thinkingConfig }
          }
        }
        if (Object.keys(variants).length > 0) {
          modelConfig.variants = variants
        }
      }
      
      models[modelId] = modelConfig
    }
    
    config.models = models
    
    await updateCustomProvider(form.providerId, config as CustomProviderConfig)
    showSuccess(`已更新供应商: ${form.name || form.providerId}`)
    emit('update:visible', false)
    emit('success')
  } catch (e) {
    showError(e)
  } finally {
    editProviderLoading.value = false
  }
}

// 关闭对话框
function handleClose() {
  emit('update:visible', false)
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="编辑自定义供应商"
    width="620px"
    append-to=".app-main"
    align-center
    :close-on-click-modal="true"
    class="add-provider-dialog"
    @update:model-value="emit('update:visible', $event)"
  >
    <div class="add-form">
      <!-- 供应商 ID（只读显示） -->
      <div class="form-section">
        <div class="form-section-title">基本信息</div>
        <div class="form-row">
          <label class="form-label">供应商 ID</label>
          <el-input
            v-model="editForm.providerId"
            disabled
            placeholder="供应商 ID 不可修改"
          />
        </div>
        <div class="form-row">
          <label class="form-label">显示名称</label>
          <el-input
            v-model="editForm.name"
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
          <el-select v-model="editForm.npm" placeholder="选择 API 格式" @change="handleEditNpmChange">
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
            v-model="editForm.apiKey"
            placeholder="sk-... 或 {env:API_KEY_NAME}"
            show-password
          />
        </div>
        <div class="form-row">
          <label class="form-label">Base URL</label>
          <el-input
            v-model="editForm.baseURL"
            placeholder="https://api.example.com/v1（OpenAI 兼容接口地址）"
          />
        </div>
      </div>

      <!-- 模型配置 -->
      <div class="form-section">
        <div class="form-section-header">
          <div class="form-section-title">模型列表</div>
          <el-button size="small" @click="addEditModelRow" plain>
            <el-icon><Plus /></el-icon>
            添加模型
          </el-button>
        </div>

        <div v-for="(model, index) in editForm.models" :key="index" class="model-form-item">
          <div class="model-form-header">
            <span class="model-form-index">模型 {{ index + 1 }}</span>
            <el-button
              v-if="editForm.models.length > 1"
              size="small"
              text
              type="danger"
              @click="removeEditModelRow(index)"
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
            <!-- 变体配置（可折叠） -->
            <div v-if="getVariantOptions(editForm.npm).length > 0" class="variant-section">
              <div class="variant-section-header" @click="toggleEditVariantExpanded(index)">
                <div class="variant-section-title">
                  <el-icon class="variant-expand-icon">
                    <ArrowRight v-if="!editVariantExpanded[index]" />
                    <ArrowDown v-else />
                  </el-icon>
                  <span>变体 (Variants)</span>
                  <span v-if="model.variants.length > 0" class="variant-count">
                    已选 {{ model.variants.length }} 个
                  </span>
                </div>
              </div>
              <!-- 变体列表（折叠内容） -->
              <div v-show="editVariantExpanded[index]" class="variant-list">
                <div
                  v-for="opt in getVariantOptions(editForm.npm)"
                  :key="opt.key"
                  class="variant-item"
                  :class="{ 'is-selected': model.variants.includes(opt.key) }"
                >
                  <div class="variant-header">
                    <el-checkbox
                      :model-value="model.variants.includes(opt.key)"
                      @change="(checked: boolean) => handleEditVariantChange(model, opt.key, checked)"
                      size="small"
                    >
                      <span class="variant-label">{{ opt.label }}</span>
                      <span class="variant-desc">{{ opt.description }}</span>
                    </el-checkbox>
                  </div>
                  <!-- 参数编辑区 -->
                  <div v-if="model.variants.includes(opt.key) && opt.fields.length > 0" class="variant-fields">
                    <div v-for="field in opt.fields" :key="field.key" class="variant-field-row">
                      <label class="variant-field-label">{{ field.label }}</label>
                      <!-- select 类型 -->
                      <el-select
                        v-if="field.type === 'select'"
                        v-model="model.variantFieldValues[opt.key][field.key]"
                        size="small"
                        class="variant-field-input"
                      >
                        <el-option
                          v-for="optItem in field.options"
                          :key="optItem.value"
                          :label="optItem.label"
                          :value="optItem.value"
                        />
                      </el-select>
                      <!-- number 类型 -->
                      <el-input-number
                        v-if="field.type === 'number'"
                        v-model="model.variantFieldValues[opt.key][field.key]"
                        :min="field.min"
                        :max="field.max"
                        :step="field.step"
                        size="small"
                        controls-position="right"
                        class="variant-field-input"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        @click="handleEditProvider"
        :loading="editProviderLoading"
      >
        保存更改
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
/* ==================== 编辑供应商对话框 ==================== */
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
  gap: 4px;
  padding-top: 4px;
  white-space: nowrap;
}

.model-form-field.checkbox-field .form-label-sm {
  margin-right: 6px;
}

/* 变体配置 */
.variant-section {
  margin-top: var(--app-spacing-1);
  padding-top: var(--app-spacing-2);
  border-top: 1px dashed var(--app-border-default);
}

.variant-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--app-spacing-1) var(--app-spacing-2);
  border-radius: var(--app-radius-sm);
  cursor: pointer;
  user-select: none;
  transition: background var(--app-transition-fast);
}

.variant-section-header:hover {
  background: rgba(0, 0, 0, 0.03);
}

.variant-section-title {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text-primary);
}

.variant-expand-icon {
  font-size: 12px;
  color: var(--app-text-tertiary);
  transition: transform var(--app-transition-fast);
}

.variant-count {
  font-size: 11px;
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 6px;
  border-radius: var(--app-radius-sm);
}

.variant-list {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
  padding-top: var(--app-spacing-2);
}

.variant-item {
  padding: var(--app-spacing-2);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-sm);
  background: var(--app-bg-secondary, rgba(0, 0, 0, 0.02));
  transition: all var(--app-transition-fast);
}

.variant-item.is-selected {
  border-color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.05);
}

.variant-header {
  display: flex;
  align-items: center;
}

.variant-label {
  font-weight: 500;
  margin-right: var(--app-spacing-2);
}

.variant-desc {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.variant-fields {
  margin-top: var(--app-spacing-2);
  padding-top: var(--app-spacing-2);
  border-top: 1px dashed var(--app-border-default);
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
}

.variant-field-row {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.variant-field-label {
  font-size: 12px;
  color: var(--app-text-secondary);
  min-width: 100px;
}

.variant-field-input {
  flex: 1;
  max-width: 200px;
}

/* 暗色主题 - 变体配置 */
html.dark .variant-section-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

html.dark .variant-item {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.08);
}

html.dark .variant-item.is-selected {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.3);
}

/* 暗色主题 - 编辑对话框 */
html.dark .model-form-item {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.08);
}

html.dark .form-section-title {
  border-bottom-color: rgba(255, 255, 255, 0.1);
}

/* 赛博朋克主题 - 编辑对话框 */
html.cyberpunk .model-form-item {
  background: rgba(0, 255, 255, 0.03);
  border-color: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .form-section-title {
  border-bottom-color: rgba(0, 255, 255, 0.15);
  color: #00ffff;
}

/* 玻璃拟态主题 - 编辑对话框 */
html.glassmorphism .model-form-item {
  background: rgba(255, 255, 255, 0.3);
  border-color: rgba(200, 200, 200, 0.3);
}

html.glassmorphism .form-section-title {
  border-bottom-color: rgba(200, 200, 200, 0.3);
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
</style>

<!-- 非 scoped 样式：对话框 teleport 到 .app-main 后 scoped 失效，需用全局样式控制 -->
<style>
.add-provider-dialog .el-dialog__body {
  max-height: calc(100vh - 240px);
  overflow-y: auto;
}
</style>
