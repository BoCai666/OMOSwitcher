<script setup lang="ts">
/**
 * 模型管理页面
 * 以供应商为分组展示模型列表
 */
import { ref, onMounted, computed } from 'vue'
import type { Model } from '@/types'
import {
  listModels,
  addModel,
  updateModel,
  deleteModel,
  validateModelId,
  parseProvider
} from '@/services/modelStore'
import { showError, showSuccess, confirm, AppError, ErrorCode } from '@/utils/errorHandler'

// 加载状态
const loading = ref(true)

// 模型列表数据
const models = ref<Model[]>([])

// 按供应商分组的模型
const groupedModels = computed(() => {
  const groups = new Map<string, Model[]>()
  for (const model of models.value) {
    const provider = model.provider
    if (!groups.has(provider)) {
      groups.set(provider, [])
    }
    groups.get(provider)!.push(model)
  }
  return groups
})

// 供应商列表（按名称排序）
const providers = computed(() => {
  return Array.from(groupedModels.value.keys()).sort()
})

// 统计信息
const stats = computed(() => {
  const providerCount = groupedModels.value.size
  const modelCount = models.value.length
  return { providerCount, modelCount }
})

// 对话框显示状态
const dialogVisible = ref(false)
const isEditing = ref(false)
const editingOriginalId = ref('')

// 表单数据
const formData = ref<Model>({
  id: '',
  name: '',
  provider: ''
})

// 表单引用（用于验证）
const formRef = ref()

// 展开的折叠面板
const activeCollapse = ref<string[]>([])

// 表单验证规则
const formRules = {
  id: [
    { required: true, message: '请输入模型 ID', trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: Function) => {
        if (!value) {
          callback(new Error('请输入模型 ID'))
          return
        }
        if (!validateModelId(value)) {
          callback(new Error('格式无效，应为 provider/model-name'))
          return
        }
        // 检查是否已存在（添加时）
        if (!isEditing.value && models.value.some(m => m.id === value)) {
          callback(new Error('该模型 ID 已存在'))
          return
        }
        // 检查是否已存在（编辑时，排除自身）
        if (isEditing.value && value !== editingOriginalId.value && models.value.some(m => m.id === value)) {
          callback(new Error('该模型 ID 已存在'))
          return
        }
        callback()
      },
      trigger: 'blur'
    }
  ],
  name: [
    { required: true, message: '请输入模型名称', trigger: 'blur' },
    { min: 1, max: 50, message: '长度在 1 到 50 个字符', trigger: 'blur' }
  ],
  provider: [
    { required: true, message: '请输入提供商', trigger: 'blur' },
    { min: 1, max: 30, message: '长度在 1 到 30 个字符', trigger: 'blur' }
  ]
}

// 加载模型列表
const loadModels = async () => {
  loading.value = true
  try {
    models.value = await listModels()
    // 默认展开所有供应商
    activeCollapse.value = Array.from(groupedModels.value.keys())
  } finally {
    loading.value = false
  }
}

// 监听 ID 变化，自动解析 provider
const handleIdChange = (id: string) => {
  const parsed = parseProvider(id)
  if (parsed && !formData.value.provider) {
    formData.value.provider = parsed
  }
}

// 打开添加对话框
const handleAdd = () => {
  isEditing.value = false
  editingOriginalId.value = ''
  formData.value = {
    id: '',
    name: '',
    provider: ''
  }
  dialogVisible.value = true
}

// 打开编辑对话框
const handleEdit = (row: Model) => {
  isEditing.value = true
  editingOriginalId.value = row.id
  formData.value = {
    id: row.id,
    name: row.name,
    provider: row.provider
  }
  dialogVisible.value = true
}

// 保存模型（添加或编辑）
const handleSave = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (isEditing.value) {
      // 编辑模式
      const result = await updateModel(editingOriginalId.value, {
        id: formData.value.id,
        name: formData.value.name,
        provider: formData.value.provider
      })
      if (result) {
        showSuccess('模型更新成功')
        await loadModels()
        dialogVisible.value = false
      } else {
        showError(new AppError('模型不存在', ErrorCode.MODEL_NOT_FOUND))
      }
    } else {
      // 添加模式
      try {
        await addModel({
          id: formData.value.id,
          name: formData.value.name,
          provider: formData.value.provider
        })
        showSuccess('模型添加成功')
        await loadModels()
        dialogVisible.value = false
      } catch (error) {
        showError(error)
      }
    }
  } catch {
    // 验证失败，不执行保存
  }
}

// 删除模型
const handleDelete = async (row: Model) => {
  const confirmed = await confirm(
    `确定要删除模型 "${row.name}" (${row.id}) 吗？`,
    '确认删除'
  )

  if (!confirmed) return

  const success = await deleteModel(row.id)
  if (success) {
    showSuccess('模型删除成功')
    await loadModels()
  } else {
    showError(new AppError('模型不存在', ErrorCode.MODEL_NOT_FOUND))
  }
}

// 重置表单
const handleDialogClose = () => {
  formRef.value?.resetFields()
}

// 获取供应商的标签颜色
const getProviderTagType = (provider: string): string => {
  const colors = ['', 'success', 'warning', 'danger', 'info']
  // 根据供应商名称生成一个稳定的颜色索引
  let hash = 0
  for (let i = 0; i < provider.length; i++) {
    hash = provider.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
}

// 页面加载时获取模型列表
onMounted(() => {
  loadModels()
})
</script>

<template>
  <div class="model-manage" v-loading="loading" element-loading-text="加载模型列表...">
      <!-- 页面头部 -->
      <div class="page-header">
        <div class="header-left">
          <span class="subtitle">共 {{ stats.providerCount }} 个供应商，{{ stats.modelCount }} 个模型</span>
        </div>
        <el-button type="primary" @click="handleAdd">
          <el-icon><Plus /></el-icon>
          添加模型
        </el-button>
      </div>

      <!-- 加载骨架屏 -->
      <div v-if="loading" class="loading-skeleton">
        <el-skeleton :rows="5" animated />
      </div>

      <!-- 供应商分组展示 -->
      <div v-else class="provider-groups">
        <el-collapse v-model="activeCollapse" class="provider-collapse">
          <el-collapse-item
            v-for="provider in providers"
            :key="provider"
            :name="provider"
          >
            <template #title>
              <div class="provider-header">
                <el-tag :type="getProviderTagType(provider)" size="large">
                  {{ provider }}
                </el-tag>
                <span class="model-count">
                  {{ groupedModels.get(provider)?.length || 0 }} 个模型
                </span>
              </div>
            </template>

            <!-- 供应商下的模型表格 -->
            <el-table
              :data="groupedModels.get(provider)"
              style="width: 100%"
              stripe
              size="small"
            >
              <el-table-column prop="id" label="模型 ID" min-width="220">
                <template #default="{ row }">
                  <code class="model-id">{{ row.id }}</code>
                </template>
              </el-table-column>

              <el-table-column prop="name" label="模型名称" min-width="150" />

              <el-table-column label="操作" width="150" fixed="right">
                <template #default="{ row }">
                  <el-button
                    type="primary"
                    link
                    size="small"
                    @click="handleEdit(row)"
                  >
                    编辑
                  </el-button>
                  <el-button
                    type="danger"
                    link
                    size="small"
                    @click="handleDelete(row)"
                  >
                    删除
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </el-collapse-item>
        </el-collapse>

        <!-- 空状态 -->
        <el-empty v-if="models.length === 0" description="暂无模型，点击上方按钮添加" />
      </div>

      <!-- 添加/编辑对话框 -->
      <el-dialog
        v-model="dialogVisible"
        :title="isEditing ? '编辑模型' : '添加模型'"
        width="500px"
        :close-on-click-modal="false"
        append-to-body
        align-center
        @closed="handleDialogClose"
      >
        <el-form
          ref="formRef"
          :model="formData"
          :rules="formRules"
          label-width="100px"
          label-position="right"
        >
          <el-form-item label="模型 ID" prop="id">
            <el-input
              v-model="formData.id"
              placeholder="provider/model-name"
              :disabled="isEditing"
              @change="handleIdChange"
            />
            <div class="form-tip">
              格式：provider/model-name（如：wuwen/glm-5）
            </div>
          </el-form-item>

          <el-form-item label="模型名称" prop="name">
            <el-input
              v-model="formData.name"
              placeholder="请输入模型显示名称"
              maxlength="50"
              show-word-limit
            />
          </el-form-item>

          <el-form-item label="提供商" prop="provider">
            <el-input
              v-model="formData.provider"
              placeholder="请输入提供商名称"
              maxlength="30"
              show-word-limit
            />
          </el-form-item>
        </el-form>

        <template #footer>
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSave">
            {{ isEditing ? '保存' : '添加' }}
          </el-button>
        </template>
      </el-dialog>
    </div>
</template>

<style scoped>
/* 容器样式 */
.model-manage {
  max-width: 1200px;
  margin: 0 auto;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-5);
  padding: var(--app-spacing-4) var(--app-spacing-5);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.subtitle {
  color: var(--app-text-tertiary);
  font-size: 14px;
}

/* 加载骨架屏 */
.loading-skeleton {
  padding: var(--app-spacing-6);
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-lg);
}

/* 赛博朋克主题 - 统计信息使用更亮的灰色 */
html.cyberpunk .subtitle {
  color: #a0a0c0;
}

/* 暗色主题 - 统计信息使用更亮的灰色 */
html.dark .subtitle {
  color: #a3a6ad;
}

/* 暗色主题 - 折叠面板头部样式 */
html.dark .provider-collapse :deep(.el-collapse-item__header) {
  background: rgba(18, 18, 26, 0.85);
  border: 1px solid var(--app-border-default);
  color: var(--app-text-primary);
}

html.dark .provider-collapse :deep(.el-collapse-item__header:hover) {
  background: var(--app-bg-elevated);
  border-color: var(--app-border-hover);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.08);
}

html.dark .provider-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 3px rgba(0, 212, 255, 0.5));
}

html.dark .provider-collapse :deep(.el-collapse-item__arrow.is-active) {
  color: var(--app-color-secondary);
  filter: drop-shadow(0 0 6px rgba(0, 255, 213, 0.6));
}

/* 暗色主题 - 页面头部按钮 */
html.dark .page-header .el-button--primary {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  border: none;
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

html.dark .page-header .el-button--primary:hover {
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.5);
}

/* 暗色主题 - 空状态 */
html.dark .provider-groups .el-empty {
  --el-empty-fill-color-0: var(--app-bg-card);
  --el-empty-fill-color-1: var(--app-bg-elevated);
  --el-empty-description-color: var(--app-text-tertiary);
}

/* 霓虹发光按钮 */
.page-header .el-button--primary {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  border: none;
  box-shadow:
    0 0 10px rgba(0, 212, 255, 0.4),
    0 0 20px rgba(0, 212, 255, 0.2),
    0 0 30px rgba(0, 212, 255, 0.1);
  transition: all var(--app-transition-normal);
}

.page-header .el-button--primary:hover {
  box-shadow:
    0 0 15px rgba(0, 212, 255, 0.6),
    0 0 30px rgba(0, 212, 255, 0.4),
    0 0 45px rgba(0, 212, 255, 0.2);
  transform: translateY(-1px);
}

/* 供应商分组区域 */
.provider-groups {
  min-height: 400px;
}

/* 折叠面板 - 玻璃背景 + 霓虹指示器 */
.provider-collapse {
  border: none;
}

/* 折叠面板头部 - 玻璃拟态 */
.provider-collapse :deep(.el-collapse-item__header) {
  background: var(--app-glass-bg, rgba(18, 18, 26, 0.75));
  backdrop-filter: var(--app-glass-blur, blur(12px));
  -webkit-backdrop-filter: var(--app-glass-blur, blur(12px));
  border: 1px solid var(--app-glass-border, rgba(255, 255, 255, 0.08));
  border-radius: var(--app-radius-md);
  padding: 0 var(--app-spacing-4);
  margin-bottom: var(--app-spacing-2);
  height: 56px;
  font-size: 15px;
  color: var(--app-text-primary);
  transition: all var(--app-transition-normal);
}

.provider-collapse :deep(.el-collapse-item__header:hover) {
  background: var(--app-bg-elevated);
  border-color: var(--app-border-hover);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.1);
}

/* 霓虹展开指示器 */
.provider-collapse :deep(.el-collapse-item__arrow) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 212, 255, 0.6));
  transition: all var(--app-transition-bounce);
}

.provider-collapse :deep(.el-collapse-item__arrow.is-active) {
  color: var(--app-color-secondary);
  filter: drop-shadow(0 0 8px rgba(0, 255, 213, 0.8));
  transform: rotate(90deg);
}

.provider-collapse :deep(.el-collapse-item__wrap) {
  border: none;
  background: transparent;
}

.provider-collapse :deep(.el-collapse-item__content) {
  padding: var(--app-spacing-3) 0 var(--app-spacing-4) 0;
}

/* 供应商头部信息 */
.provider-header {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
  flex: 1;
}

.provider-header .el-tag {
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: var(--app-color-primary);
  font-weight: 500;
}

/* 赛博朋克主题 - 供应商标签使用更亮的颜色 */
html.cyberpunk .provider-header .el-tag {
  color: #4dd0e1;
  background: rgba(0, 255, 255, 0.15);
  border-color: rgba(0, 255, 255, 0.4);
}

/* 暗色主题 - 供应商标签使用主题色 */
html.dark .provider-header .el-tag {
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.15);
  border-color: rgba(0, 212, 255, 0.3);
}

/* 玻璃拟态主题 - 供应商标签使用深蓝色 */
html.glassmorphism .provider-header .el-tag {
  color: #1d4ed8;
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(37, 99, 235, 0.4);
}

.model-count {
  color: var(--app-text-tertiary);
  font-size: 13px;
}

/* 表格样式 - 悬停高亮 + 斑马纹 */
.provider-collapse :deep(.el-table) {
  background: transparent;
  --el-table-header-bg-color: var(--app-bg-elevated);
  --el-table-row-hover-bg-color: rgba(0, 212, 255, 0.08);
  --el-table-border-color: var(--app-border-default);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  overflow: hidden;
}

.provider-collapse :deep(.el-table__header) {
  background: var(--app-bg-elevated);
}

.provider-collapse :deep(.el-table__header th) {
  background: var(--app-bg-elevated);
  color: var(--app-text-primary);
  font-weight: 600;
  border-bottom: 1px solid var(--app-border-default);
}

.provider-collapse :deep(.el-table__body tr) {
  background: var(--app-bg-card);
  transition: background-color var(--app-transition-fast);
}

/* 斑马纹 - 交替行颜色 */
.provider-collapse :deep(.el-table__body tr:nth-child(even)) {
  background: var(--app-bg-base);
}

.provider-collapse :deep(.el-table__body tr:nth-child(odd)) {
  background: var(--app-bg-card);
}

/* 悬停高亮效果 */
.provider-collapse :deep(.el-table__body tr:hover > td) {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow: inset 0 0 20px rgba(0, 212, 255, 0.05);
}

.provider-collapse :deep(.el-table__body td) {
  color: var(--app-text-secondary);
  border-bottom: 1px solid var(--app-border-default);
}

/* 表格内按钮 */
.provider-collapse :deep(.el-button--primary.is-link) {
  color: var(--app-color-primary);
  transition: all var(--app-transition-fast);
}

.provider-collapse :deep(.el-button--primary.is-link:hover) {
  color: var(--app-color-secondary);
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.6);
}

.provider-collapse :deep(.el-button--danger.is-link) {
  color: var(--app-color-danger);
  transition: all var(--app-transition-fast);
}

.provider-collapse :deep(.el-button--danger.is-link:hover) {
  text-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
}

/* 模型 ID 代码样式 */
.model-id {
  background: rgba(0, 212, 255, 0.1);
  padding: 2px var(--app-spacing-2);
  border-radius: var(--app-radius-sm);
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;
  font-size: 13px;
  color: var(--app-color-primary);
  border: 1px solid rgba(0, 212, 255, 0.2);
}

/* 赛博朋克主题 - 模型ID使用更亮的青色 */
html.cyberpunk .model-id {
  color: #4dd0e1;
  text-shadow: 0 0 5px rgba(0, 255, 255, 0.3);
}

/* 暗色主题 - 模型ID使用主题色 */
html.dark .model-id {
  color: var(--app-color-primary);
  background: rgba(0, 212, 255, 0.1);
  border-color: rgba(0, 212, 255, 0.2);
}

/* 玻璃拟态主题 - 模型ID使用深蓝色 */
html.glassmorphism .model-id {
  color: #1d4ed8;
  background: rgba(37, 99, 235, 0.1);
  border-color: rgba(37, 99, 235, 0.3);
}

/* 暗色主题 - 表格内按钮样式 */
html.dark .provider-collapse :deep(.el-button--primary.is-link) {
  color: var(--app-color-primary);
  transition: all var(--app-transition-fast);
}

html.dark .provider-collapse :deep(.el-button--primary.is-link:hover) {
  color: var(--app-color-secondary);
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.6);
}

html.dark .provider-collapse :deep(.el-button--danger.is-link) {
  color: var(--app-color-danger);
  transition: all var(--app-transition-fast);
}

html.dark .provider-collapse :deep(.el-button--danger.is-link:hover) {
  text-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
}

/* ==================== 暗色主题表格样式修复 ==================== */
html.dark .provider-collapse :deep(.el-table) {
  --el-table-header-bg-color: var(--app-bg-elevated);
  --el-table-row-hover-bg-color: rgba(0, 212, 255, 0.08);
  --el-table-border-color: var(--app-border-default);
  background: var(--app-bg-card);
}

html.dark .provider-collapse :deep(.el-table__header th) {
  background: var(--app-bg-elevated);
  color: var(--app-text-primary);
  border-bottom: 1px solid var(--app-border-default);
}

html.dark .provider-collapse :deep(.el-table__body tr) {
  background: var(--app-bg-card);
}

html.dark .provider-collapse :deep(.el-table__body tr:nth-child(even)) {
  background: var(--app-bg-base);
}

html.dark .provider-collapse :deep(.el-table__body tr:nth-child(odd)) {
  background: var(--app-bg-card);
}

html.dark .provider-collapse :deep(.el-table__body tr:hover > td) {
  background: rgba(0, 212, 255, 0.08) !important;
}

html.dark .provider-collapse :deep(.el-table__body td) {
  color: var(--app-text-secondary);
  border-bottom: 1px solid var(--app-border-default);
}

/* 玻璃拟态主题 - 表格内按钮增强对比度 */
html.glassmorphism .provider-collapse :deep(.el-button--primary.is-link) {
  color: #1d4ed8;
  font-weight: 500;
}

html.glassmorphism .provider-collapse :deep(.el-button--primary.is-link:hover) {
  color: #1e40af;
  text-shadow: none;
}

html.glassmorphism .provider-collapse :deep(.el-button--danger.is-link) {
  color: #dc2626;
  font-weight: 500;
}

html.glassmorphism .provider-collapse :deep(.el-button--danger.is-link:hover) {
  color: #b91c1c;
  text-shadow: none;
}

/* 空状态 */
.provider-groups .el-empty {
  --el-empty-fill-color-0: var(--app-bg-card);
  --el-empty-fill-color-1: var(--app-bg-elevated);
  --el-empty-description-color: var(--app-text-tertiary);
  padding: var(--app-spacing-12) 0;
}

/* 对话框 - 玻璃效果 */
:deep(.el-dialog) {
  background: var(--app-glass-bg, rgba(18, 18, 26, 0.9));
  backdrop-filter: var(--app-glass-blur, blur(20px));
  -webkit-backdrop-filter: var(--app-glass-blur, blur(20px));
  border: 1px solid var(--app-glass-border, rgba(255, 255, 255, 0.1));
  border-radius: var(--app-radius-xl);
  box-shadow:
    0 25px 50px rgba(0, 0, 0, 0.5),
    0 0 40px rgba(0, 212, 255, 0.1);
}

/* ==================== 暗色主题对话框修复 ==================== */
/* 修复暗色模式下对话框背景铺满问题 */
html.cyberpunk :deep(.el-dialog),
html.dark :deep(.el-dialog) {
  background: rgba(26, 26, 46, 0.95);
  border: 1px solid var(--app-border-default);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5), 0 0 30px rgba(0, 212, 255, 0.08);
}

html.cyberpunk :deep(.el-dialog__body),
html.dark :deep(.el-dialog__body) {
  background: transparent;
}

html.dark :deep(.el-dialog__header) {
  border-bottom: 1px solid var(--app-border-default);
}

html.dark :deep(.el-dialog__title) {
  color: var(--app-text-primary);
}

html.dark :deep(.el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 212, 255, 0.6));
}

html.dark :deep(.el-dialog__footer) {
  border-top: 1px solid var(--app-border-default);
}

html.dark :deep(.el-dialog__footer .el-button:not(.el-button--primary)) {
  background: transparent;
  border-color: var(--app-border-default);
  color: var(--app-text-secondary);
}

html.dark :deep(.el-dialog__footer .el-button:not(.el-button--primary):hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
}

html.dark :deep(.el-dialog__footer .el-button--primary) {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

/* 对话框内输入框暗色主题适配 */
html.dark :deep(.el-input__wrapper) {
  background: var(--app-bg-elevated);
  box-shadow: 0 0 0 1px var(--app-border-default) inset;
}

html.dark :deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--app-border-hover) inset;
}

html.dark :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 1px var(--app-color-primary) inset, 0 0 10px rgba(0, 212, 255, 0.2);
}

html.dark :deep(.el-input__inner) {
  color: var(--app-text-primary);
  background: transparent;
}

html.dark :deep(.el-input__inner::placeholder) {
  color: var(--app-text-disabled);
}

html.dark :deep(.el-form-item__label) {
  color: var(--app-text-primary);
}

:deep(.el-dialog__header) {
  padding: var(--app-spacing-5);
  margin-right: 0;
  border-bottom: 1px solid var(--app-border-default);
}

:deep(.el-dialog__title) {
  color: var(--app-text-primary);
  font-weight: 600;
  font-size: 18px;
}

:deep(.el-dialog__headerbtn .el-dialog__close) {
  color: var(--app-text-tertiary);
  transition: all var(--app-transition-fast);
}

:deep(.el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: drop-shadow(0 0 5px rgba(0, 212, 255, 0.6));
}

:deep(.el-dialog__body) {
  padding: var(--app-spacing-5);
  color: var(--app-text-secondary);
}

:deep(.el-dialog__footer) {
  padding: var(--app-spacing-4) var(--app-spacing-5);
  border-top: 1px solid var(--app-border-default);
}

/* 对话框内表单样式 */
:deep(.el-form-item__label) {
  color: var(--app-text-primary);
  font-weight: 500;
}

:deep(.el-input__wrapper) {
  background: var(--app-bg-elevated);
  box-shadow: 0 0 0 1px var(--app-border-default) inset;
  transition: all var(--app-transition-fast);
}

:deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--app-border-hover) inset;
}

:deep(.el-input__wrapper.is-focus) {
  box-shadow:
    0 0 0 1px var(--app-color-primary) inset,
    0 0 10px rgba(0, 212, 255, 0.2);
}

:deep(.el-input__inner) {
  color: var(--app-text-primary);
  background: transparent;
}

:deep(.el-input__inner::placeholder) {
  color: var(--app-text-disabled);
}

:deep(.el-input__count-inner) {
  background: transparent;
  color: var(--app-text-tertiary);
}

.form-tip {
  font-size: 12px;
  color: var(--app-text-tertiary);
  margin-top: var(--app-spacing-1);
  line-height: 1.4;
}

/* 对话框底部按钮 */
:deep(.el-dialog__footer .el-button:not(.el-button--primary)) {
  background: transparent;
  border-color: var(--app-border-default);
  color: var(--app-text-secondary);
}

:deep(.el-dialog__footer .el-button:not(.el-button--primary):hover) {
  border-color: var(--app-color-primary);
  color: var(--app-color-primary);
}

:deep(.el-dialog__footer .el-button--primary) {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  border: none;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

:deep(.el-dialog__footer .el-button--primary:hover) {
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.5);
}

/* ==================== 明色主题适配 ==================== */

/* 明色主题 - 对话框样式 */
html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog) {
  background: rgba(255, 255, 255, 0.98);
  border: 1px solid var(--app-border-default);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.15);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__header) {
  border-bottom: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__title) {
  color: var(--app-text-primary);
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__headerbtn:hover .el-dialog__close) {
  color: var(--app-color-primary);
  filter: none;
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__body) {
  background: transparent;
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__footer) {
  border-top: 1px solid var(--app-border-default);
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__footer .el-button--primary) {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary));
  box-shadow: 0 4px 15px rgba(0, 168, 232, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .model-manage :deep(.el-dialog__footer .el-button--primary:hover) {
  box-shadow: 0 6px 20px rgba(0, 168, 232, 0.4);
}

/* 明色主题 - 表格内按钮样式优化 */
html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-button--primary.is-link) {
  color: #ffffff;
  font-weight: 500;
  padding: 4px 12px;
  border-radius: 4px;
  background: var(--app-color-primary);
  border: none;
  transition: all 0.2s ease;
}

html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-button--primary.is-link:hover) {
  background: var(--app-color-primary-hover);
  color: #ffffff;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 168, 232, 0.3);
}

html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-button--danger.is-link) {
  color: #ffffff;
  font-weight: 500;
  padding: 4px 12px;
  border-radius: 4px;
  background: var(--app-color-danger);
  border: none;
  transition: all 0.2s ease;
}

html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-button--danger.is-link:hover) {
  background: #dc2626;
  color: #ffffff;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
}

/* 明色主题 - 折叠面板样式 */
html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-collapse-item__header) {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  box-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-collapse-item__header:hover) {
  background: var(--app-bg-hover);
  border-color: var(--app-border-hover);
}

/* 明色主题 - 供应商标签 */
html.light:not(.cyberpunk):not(.dark) .provider-header .el-tag {
  background: rgba(0, 168, 232, 0.1);
  border: 1px solid rgba(0, 168, 232, 0.3);
  color: var(--app-color-primary);
}

/* 明色主题 - 模型ID样式 */
html.light:not(.cyberpunk):not(.dark) .model-id {
  background: rgba(0, 168, 232, 0.08);
  border: 1px solid rgba(0, 168, 232, 0.2);
  color: var(--app-color-primary);
}

/* 明色主题 - 表格样式 */
html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-table) {
  --el-table-row-hover-bg-color: rgba(0, 168, 232, 0.05);
}

html.light:not(.cyberpunk):not(.dark) .provider-collapse :deep(.el-table__body tr:hover > td) {
  background: rgba(0, 168, 232, 0.05) !important;
}

/* 响应式优化 */
@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    gap: var(--app-spacing-3);
    align-items: stretch;
  }

  .provider-header {
    flex-wrap: wrap;
  }

  .model-id {
    font-size: 12px;
  }
}
</style>
