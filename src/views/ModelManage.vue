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
import AppLayout from '@/components/layout/AppLayout.vue'

// 页面标题
const pageTitle = '模型管理'

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
  models.value = await listModels()
  // 默认展开所有供应商
  activeCollapse.value = Array.from(groupedModels.value.keys())
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
  <AppLayout :title="pageTitle">
    <div class="model-manage">
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

      <!-- 供应商分组展示 -->
      <div class="provider-groups">
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
  </AppLayout>
</template>

<style scoped>
.model-manage {
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.subtitle {
  color: #909399;
  font-size: 14px;
}

.provider-groups {
  min-height: 400px;
}

.provider-collapse {
  border: none;
}

.provider-collapse :deep(.el-collapse-item__header) {
  background-color: #f5f7fa;
  border-radius: 8px;
  padding: 0 16px;
  margin-bottom: 8px;
  height: 50px;
  font-size: 15px;
}

.provider-collapse :deep(.el-collapse-item__wrap) {
  border: none;
}

.provider-collapse :deep(.el-collapse-item__content) {
  padding: 0 0 16px 0;
}

.provider-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.model-count {
  color: #909399;
  font-size: 13px;
}

.model-id {
  background-color: #f5f7fa;
  padding: 2px 8px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #409eff;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
  line-height: 1.4;
}
</style>
