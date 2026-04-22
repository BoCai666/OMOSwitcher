<script setup lang="ts">
/**
 * 备用模型链编辑器组件
 * 用于编辑 Agent/Category 配置的 fallback_models 字段
 * 支持字符串、对象、混合数组格式的展示与操作
 * 支持内联编辑每条备用模型的参数（variant、reasoningEffort 等）
 */
import { computed, ref } from 'vue'
import { CirclePlus, Close, ArrowUp, ArrowDown, ArrowRight } from '@element-plus/icons-vue'
import type { FallbackModels, FallbackModelObject } from '@/types'

// 类型守卫：判断条目是否为对象形式
function isObjectEntry(entry: string | FallbackModelObject): entry is FallbackModelObject {
  return typeof entry === 'object' && entry !== null
}

// 编辑表单数据结构
interface EditFormData {
  variant: string
  reasoningEffort: string
  temperature: number | undefined
  top_p: number | undefined
  maxTokens: number | undefined
  thinkingType: 'enabled' | 'disabled' | ''
  thinkingBudget: number | undefined
}

// 创建空表单
function createEmptyForm(): EditFormData {
  return {
    variant: '',
    reasoningEffort: '',
    temperature: undefined,
    top_p: undefined,
    maxTokens: undefined,
    thinkingType: '',
    thinkingBudget: undefined,
  }
}

const props = defineProps<{
  fallbackModels: FallbackModels | undefined
  currentModel: string
}>()

const emit = defineEmits<{
  'update:fallbackModels': [value: FallbackModels | undefined]
  'add-model': []
}>()

// 当前正在编辑的条目索引
const editingIndex = ref<number | null>(null)
// 编辑表单数据
const editForm = ref<EditFormData>(createEmptyForm())

// reasoningEffort 选项
const reasoningEffortOptions = ['none', 'minimal', 'low', 'medium', 'high', 'xhigh'] as const
// variant 选项
const variantOptions = ['max', 'xhigh', 'high', 'medium', 'low', 'xlow'] as const

// 将 props 标准化为数组形式，过滤掉与主模型相同的条目
const normalizedEntries = computed<(string | FallbackModelObject)[]>(() => {
  if (!props.fallbackModels) return []
  // 单字符串 → 包装为单元素数组
  if (typeof props.fallbackModels === 'string') {
    return props.fallbackModels === props.currentModel ? [] : [props.fallbackModels]
  }
  // 数组 → 过滤掉主模型
  return props.fallbackModels.filter((entry) => {
    const modelId = isObjectEntry(entry) ? entry.model : entry
    return modelId !== props.currentModel
  })
})

// 提取条目的模型 ID
function getModelId(entry: string | FallbackModelObject): string {
  return isObjectEntry(entry) ? entry.model : entry
}

// 提取显示名称（取 provider 后的部分）
function getDisplayName(entry: string | FallbackModelObject): string {
  const id = getModelId(entry)
  return id.includes('/') ? id.split('/').pop()! : id
}

// 提取供应商前缀
function getProviderTag(entry: string | FallbackModelObject): string {
  const id = getModelId(entry)
  return id.includes('/') ? id.split('/')[0] : ''
}

// 获取对象条目的附加标签信息
function getEntryBadges(entry: string | FallbackModelObject): string[] {
  if (!isObjectEntry(entry)) return []
  const badges: string[] = []
  if (entry.variant) badges.push(entry.variant)
  if (entry.reasoningEffort) badges.push(`推理: ${entry.reasoningEffort}`)
  if (entry.thinking?.type === 'enabled') {
    const budget = entry.thinking.budgetTokens ? ` (${(entry.thinking.budgetTokens / 1000).toFixed(0)}k)` : ''
    badges.push(`思考${budget}`)
  }
  return badges
}

// 发射更新后的数组
function emitUpdate(newList: (string | FallbackModelObject)[]) {
  if (newList.length === 0) {
    emit('update:fallbackModels', undefined)
  } else {
    emit('update:fallbackModels', newList)
  }
}

// 移除条目
function removeEntry(index: number) {
  if (editingIndex.value === index) cancelEdit()
  const list = [...normalizedEntries.value]
  list.splice(index, 1)
  emitUpdate(list)
}

// 上移条目
function moveUp(index: number) {
  if (index <= 0) return
  const list = [...normalizedEntries.value]
  const temp = list[index]
  list[index] = list[index - 1]
  list[index - 1] = temp
  emitUpdate(list)
}

// 下移条目
function moveDown(index: number) {
  const list = [...normalizedEntries.value]
  if (index >= list.length - 1) return
  const temp = list[index]
  list[index] = list[index + 1]
  list[index + 1] = temp
  emitUpdate(list)
}

// 清空全部
function clearAll() {
  cancelEdit()
  emit('update:fallbackModels', undefined)
}

// 添加模型
function handleAdd() {
  emit('add-model')
}

// 开始编辑某条目（点击行切换）
function startEdit(index: number) {
  if (editingIndex.value === index) {
    saveEdit()
    return
  }
  // 先保存之前正在编辑的
  if (editingIndex.value !== null) {
    saveEdit()
  }
  const entry = normalizedEntries.value[index]
  if (isObjectEntry(entry)) {
    editForm.value = {
      variant: entry.variant ?? '',
      reasoningEffort: entry.reasoningEffort ?? '',
      temperature: entry.temperature,
      top_p: entry.top_p,
      maxTokens: entry.maxTokens,
      thinkingType: entry.thinking?.type ?? '',
      thinkingBudget: entry.thinking?.budgetTokens,
    }
  } else {
    editForm.value = createEmptyForm()
  }
  editingIndex.value = index
}

// 保存编辑
function saveEdit() {
  if (editingIndex.value === null) return
  const entry = normalizedEntries.value[editingIndex.value]
  const modelId = getModelId(entry)

  // 构建对象，只包含有值的字段
  const obj: FallbackModelObject = { model: modelId }
  if (editForm.value.variant) obj.variant = editForm.value.variant
  if (editForm.value.reasoningEffort) {
    obj.reasoningEffort = editForm.value.reasoningEffort as FallbackModelObject['reasoningEffort']
  }
  if (editForm.value.temperature !== undefined && editForm.value.temperature !== null) {
    obj.temperature = editForm.value.temperature
  }
  if (editForm.value.top_p !== undefined && editForm.value.top_p !== null) {
    obj.top_p = editForm.value.top_p
  }
  if (editForm.value.maxTokens !== undefined && editForm.value.maxTokens !== null) {
    obj.maxTokens = editForm.value.maxTokens
  }
  if (editForm.value.thinkingType) {
    obj.thinking = { type: editForm.value.thinkingType }
    if (editForm.value.thinkingBudget) {
      obj.thinking.budgetTokens = editForm.value.thinkingBudget
    }
  }

  // 如果除了 model 没有其他参数，回退为字符串
  const hasExtraParams = obj.variant || obj.reasoningEffort ||
    obj.temperature !== undefined || obj.top_p !== undefined ||
    obj.maxTokens !== undefined || obj.thinking

  const list = [...normalizedEntries.value]
  list[editingIndex.value] = hasExtraParams ? obj : modelId
  emitUpdate(list)
  editingIndex.value = null
  editForm.value = createEmptyForm()
}

// 取消编辑
function cancelEdit() {
  editingIndex.value = null
  editForm.value = createEmptyForm()
}
</script>

<template>
  <div class="fallback-chain-editor">
    <!-- 列表区域 -->
    <div v-if="normalizedEntries.length > 0" class="chain-list">
      <div
        v-for="(entry, index) in normalizedEntries"
        :key="getModelId(entry) + '-' + index"
        class="chain-item-wrapper"
      >
        <div
          class="chain-item"
          :class="{ 'is-editing': editingIndex === index }"
          @click.stop="startEdit(index)"
        >
          <!-- 展开箭头 -->
          <span class="expand-arrow" :class="{ 'is-expanded': editingIndex === index }">
            <el-icon :size="14"><ArrowRight /></el-icon>
          </span>

          <!-- 序号圆点 -->
          <span class="item-index">{{ index + 1 }}</span>

          <!-- 模型信息 -->
          <div class="item-info">
            <span v-if="getProviderTag(entry)" class="provider-tag">{{ getProviderTag(entry) }}</span>
            <span class="model-name">{{ getDisplayName(entry) }}</span>
            <!-- 对象条目的附加标签 -->
            <span
              v-for="badge in getEntryBadges(entry)"
              :key="badge"
              class="badge"
            >{{ badge }}</span>
          </div>

          <!-- 操作按钮 -->
          <div class="item-actions" @click.stop>
            <button
              class="action-btn"
              :disabled="index === 0"
              title="上移"
              @click="moveUp(index)"
            >
              <el-icon :size="14"><ArrowUp /></el-icon>
            </button>
            <button
              class="action-btn"
              :disabled="index === normalizedEntries.length - 1"
              title="下移"
              @click="moveDown(index)"
            >
              <el-icon :size="14"><ArrowDown /></el-icon>
            </button>
            <button class="action-btn remove-btn" title="移除" @click="removeEntry(index)">
              <el-icon :size="14"><Close /></el-icon>
            </button>
          </div>
        </div>

        <!-- 内联参数编辑器 -->
        <el-collapse-transition>
          <div v-if="editingIndex === index" class="edit-panel" @click.stop>
            <el-form label-position="top" size="small" class="edit-form">
              <div class="edit-grid">
                <!-- 第 1 行: variant + reasoningEffort -->
                <el-form-item label="Variant">
                  <el-select
                    v-model="editForm.variant"
                    placeholder="选择变体"
                    clearable
                    style="width: 100%"
                  >
                    <el-option
                      v-for="opt in variantOptions"
                      :key="opt"
                      :label="opt"
                      :value="opt"
                    />
                  </el-select>
                </el-form-item>
                <el-form-item label="Reasoning Effort">
                  <el-select
                    v-model="editForm.reasoningEffort"
                    placeholder="选择推理强度"
                    clearable
                    style="width: 100%"
                  >
                    <el-option
                      v-for="opt in reasoningEffortOptions"
                      :key="opt"
                      :label="opt"
                      :value="opt"
                    />
                  </el-select>
                </el-form-item>

                <!-- 第 2 行: temperature + top_p -->
                <el-form-item label="Temperature">
                  <el-input-number
                    v-model="editForm.temperature"
                    :min="0"
                    :max="2"
                    :step="0.1"
                    :precision="1"
                    controls-position="right"
                    placeholder="0.0 - 2.0"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="Top P">
                  <el-input-number
                    v-model="editForm.top_p"
                    :min="0"
                    :max="1"
                    :step="0.1"
                    :precision="1"
                    controls-position="right"
                    placeholder="0.0 - 1.0"
                    style="width: 100%"
                  />
                </el-form-item>

                <!-- 第 3 行: maxTokens + thinking -->
                <el-form-item label="Max Tokens">
                  <el-input-number
                    v-model="editForm.maxTokens"
                    :min="1"
                    :step="1"
                    controls-position="right"
                    placeholder="最大输出 token"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="Thinking">
                  <el-select
                    v-model="editForm.thinkingType"
                    placeholder="选择思考模式"
                    clearable
                    style="width: 100%"
                  >
                    <el-option label="enabled" value="enabled" />
                    <el-option label="disabled" value="disabled" />
                  </el-select>
                </el-form-item>

                <!-- 第 4 行: budgetTokens（仅 thinking=enabled 时显示） -->
                <el-form-item
                  v-if="editForm.thinkingType === 'enabled'"
                  label="Thinking Budget (tokens)"
                  class="edit-full-width"
                >
                  <el-input-number
                    v-model="editForm.thinkingBudget"
                    :min="1"
                    :step="1000"
                    controls-position="right"
                    placeholder="如 12000"
                    style="width: 100%"
                  />
                </el-form-item>
              </div>
            </el-form>
          </div>
        </el-collapse-transition>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <span class="empty-text">未配置备用模型，点击下方按钮添加</span>
    </div>

    <!-- 底部操作栏 -->
    <div class="footer-actions">
      <el-button size="small" class="add-model-btn" @click="handleAdd">
        <el-icon><CirclePlus /></el-icon>
        <span>添加备用模型</span>
      </el-button>
      <button
        v-if="normalizedEntries.length > 0"
        class="clear-btn"
        @click="clearAll"
      >清空</button>
    </div>
  </div>
</template>

<style scoped>
.fallback-chain-editor {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-3, 12px);
}

/* ==================== 列表 ==================== */

.chain-list {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-1, 4px);
}

/* ==================== 条目包装器（含编辑面板） ==================== */

.chain-item-wrapper {
  display: flex;
  flex-direction: column;
}

/* ==================== 单条记录 ==================== */

.chain-item {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2, 8px);
  padding: var(--app-spacing-2, 8px) var(--app-spacing-3, 12px);
  background-color: var(--app-bg-card, #12121a);
  border: 1px solid var(--app-border-default, #2a2a3a);
  border-radius: var(--app-radius-md, 8px);
  transition: border-color var(--app-transition-fast, 150ms ease),
              box-shadow var(--app-transition-fast, 150ms ease);
  cursor: pointer;
}

.chain-item:hover {
  border-color: rgba(0, 212, 255, 0.3);
  box-shadow: 0 2px 8px rgba(0, 212, 255, 0.1);
}

/* 正在编辑时的高亮边框 */
.chain-item.is-editing {
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 0 1px rgba(0, 212, 255, 0.2);
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}

/* ==================== 展开箭头 ==================== */

.expand-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--app-text-tertiary);
  transition: transform var(--app-transition-fast, 150ms ease),
              color var(--app-transition-fast, 150ms ease);
  flex-shrink: 0;
}

.expand-arrow.is-expanded {
  transform: rotate(90deg);
  color: var(--app-color-primary, #00d4ff);
}

/* ==================== 序号 ==================== */

.item-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  background: linear-gradient(135deg, var(--app-color-primary, #00d4ff), #00a8ff);
  color: #fff;
  border-radius: 50%;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
  box-shadow: 0 0 8px rgba(0, 212, 255, 0.35);
}

/* ==================== 模型信息 ==================== */

.item-info {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-1, 4px);
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.provider-tag {
  font-size: 11px;
  color: var(--app-text-tertiary);
  background-color: rgba(0, 212, 255, 0.08);
  padding: 2px 6px;
  border-radius: var(--app-radius-sm, 4px);
  border: 1px solid rgba(0, 212, 255, 0.2);
  flex-shrink: 0;
}

.model-name {
  font-weight: 500;
  color: var(--app-text-primary, #e5eaf3);
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.badge {
  font-size: 10px;
  color: var(--app-color-success, #10b981);
  background-color: rgba(16, 185, 129, 0.1);
  padding: 2px 6px;
  border-radius: var(--app-radius-sm, 4px);
  border: 1px solid rgba(16, 185, 129, 0.3);
  flex-shrink: 0;
  white-space: nowrap;
}

/* ==================== 操作按钮 ==================== */

.item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--app-transition-fast, 150ms ease);
}

.chain-item:hover .item-actions,
.chain-item.is-editing .item-actions {
  opacity: 1;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--app-text-tertiary);
  border-radius: var(--app-radius-sm, 4px);
  cursor: pointer;
  transition: all var(--app-transition-fast, 150ms ease);
}

.action-btn:hover:not(:disabled) {
  background-color: var(--app-bg-hover, #1f1f35);
  color: var(--app-color-primary, #00d4ff);
}

.action-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.action-btn.remove-btn:hover:not(:disabled) {
  color: var(--app-color-danger, #ef4444);
  background-color: rgba(239, 68, 68, 0.1);
}

/* ==================== 内联编辑面板 ==================== */

.edit-panel {
  background-color: var(--app-bg-hover, #f0f2f5);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-top: none;
  border-radius: 0 0 var(--app-radius-md, 8px) var(--app-radius-md, 8px);
  padding: var(--app-spacing-3, 12px) var(--app-spacing-4, 16px);
}

.edit-form {
  margin-bottom: var(--app-spacing-2, 8px);
}

/* 2 列网格布局 */
.edit-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0 var(--app-spacing-3, 12px);
}

.edit-grid .edit-full-width {
  grid-column: 1 / -1;
}

/* 表单标签样式 */.edit-panel :deep(.el-form-item__label) {
  font-size: 11px;
  color: var(--app-text-tertiary);
  padding-bottom: 2px;
}

.edit-panel :deep(.el-form-item) {
  margin-bottom: 10px;
}

/* ==================== 空状态 ==================== */

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--app-spacing-6, 24px) var(--app-spacing-4, 16px);
  border: 1px dashed var(--app-border-default, #2a2a3a);
  border-radius: var(--app-radius-md, 8px);
  background-color: rgba(0, 212, 255, 0.02);
}

.empty-text {
  font-size: 13px;
  color: var(--app-text-tertiary);
}

/* ==================== 底部操作栏 ==================== */

.footer-actions {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3, 12px);
  padding-top: var(--app-spacing-1, 4px);
}

.clear-btn {
  border: none;
  background: none;
  color: var(--app-text-tertiary);
  font-size: 12px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--app-radius-sm, 4px);
  transition: color var(--app-transition-fast, 150ms ease);
}

.clear-btn:hover {
  color: var(--app-color-danger, #ef4444);
}

/* ==================== 添加按钮 ==================== */

.add-model-btn {
  color: var(--app-color-primary, #00d4ff) !important;
  border-color: var(--app-color-primary, #00d4ff) !important;
  background-color: transparent !important;
}

.add-model-btn:hover {
  color: #fff !important;
  border-color: var(--app-color-primary, #00d4ff) !important;
  background-color: var(--app-color-primary, #00d4ff) !important;
  opacity: 0.85;
}
</style>
