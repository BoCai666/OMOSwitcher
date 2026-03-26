<script setup lang="ts">
// 预设管理页面组件
// 提供预设列表展示、切换、删除和保存功能
import { ref, onMounted } from 'vue'
import type { Preset } from '@/types'
import { useConfigStore } from '@/stores'
import {
  listPresets,
  savePreset,
  deletePreset,
  switchPreset,
  getCurrentPreset,
  recordPresetUsage
} from '@/services/presetStore'
import { showError, showSuccess, showWarning, confirm, AppError, ErrorCode } from '@/utils/errorHandler'
import PresetDialog from '@/components/PresetDialog.vue'
import PresetDetailDialog from '@/components/PresetDetailDialog.vue'
import AppLayout from '@/components/layout/AppLayout.vue'

// 页面标题
const pageTitle = '预设管理'

// 配置 store
const configStore = useConfigStore()

// 预设列表数据
const presets = ref<Preset[]>([])

// 当前预设名称
const currentPresetName = ref<string | null>(null)

// 对话框可见性
const dialogVisible = ref(false)

// 预设详情对话框
const detailDialogVisible = ref(false)
const selectedPreset = ref<Preset | null>(null)

// 加载预设列表
const loadPresets = async () => {
  presets.value = await listPresets()
  currentPresetName.value = (await getCurrentPreset()) || null
}

// 页面加载时获取预设列表
onMounted(() => {
  loadPresets()
})

// 检查预设是否为当前预设
const isCurrentPreset = (preset: Preset) => {
  return currentPresetName.value === preset.name
}

// 切换预设
const handleSwitchPreset = async (preset: Preset) => {
  // 检查是否已经是当前预设
  if (isCurrentPreset(preset)) {
    showWarning('当前已是该预设')
    return
  }

  // 检查是否有未保存的更改
  if (configStore.hasUnsavedChanges) {
    const confirmed = await confirm(
      '当前有未保存的更改，切换预设将丢失这些更改。是否继续？',
      '未保存的更改'
    )
    
    if (!confirmed) return
  } else {
    // 没有未保存更改时，确认是否切换
    const confirmed = await confirm(
      `确定要切换到预设 "${preset.name}" 吗？`,
      '切换预设'
    )
    
    if (!confirmed) return
  }

  try {
    // 执行预设切换
    const result = await switchPreset(preset.name)
    
    if (!result.success || !result.preset) {
      showError(new AppError(result.error || '切换预设失败', ErrorCode.PRESET_NOT_FOUND))
      return
    }

    // 应用预设配置到 configStore
    configStore.applyPreset(result.preset.config)
    
    // 记录预设使用历史
    await recordPresetUsage(preset.name)
    
    // 更新当前预设名称
    currentPresetName.value = preset.name
    
    // 显示成功提示
    showSuccess(`已切换到预设: ${preset.name}`)
  } catch (error) {
    showError(error)
  }
}

// 删除预设
const handleDeletePreset = async (preset: Preset) => {
  const confirmed = await confirm(
    `确定要删除预设 "${preset.name}" 吗？此操作不可恢复。`,
    '删除预设'
  )
  
  if (!confirmed) return
  
  try {
    const success = await deletePreset(preset.name)
    if (success) {
      showSuccess('删除成功')
      await loadPresets()
    } else {
      showError(new AppError('预设不存在', ErrorCode.PRESET_NOT_FOUND))
    }
  } catch (error) {
    showError(error)
  }
}

// 保存新预设
const handleSavePreset = async (name: string, description?: string) => {
  // 从 configStore 获取当前配置
  const currentConfig = configStore.config
  
  if (!currentConfig) {
    showWarning('配置未加载，无法保存预设')
    return
  }
  
  try {
    await savePreset(name, currentConfig, description)
    showSuccess('预设保存成功')
    await loadPresets()
    dialogVisible.value = false
  } catch (error) {
    showError(error)
  }
}

// 加载预设（用于对话框回调）
const handleLoadPreset = async (preset: Preset) => {
  // 复用切换预设逻辑
  await handleSwitchPreset(preset)
}

// 格式化日期显示
const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 查看预设详情
const handleViewPreset = (preset: Preset) => {
  selectedPreset.value = preset
  detailDialogVisible.value = true
}
</script>

<template>
  <AppLayout :title="pageTitle">
    <div class="preset-manage">
      <!-- 页面标题和操作按钮 -->
      <div class="page-header">
        <div class="header-left">
          <span class="subtitle">管理配置预设，快速切换不同配置方案</span>
        </div>
        <el-button type="primary" @click="dialogVisible = true">
          <el-icon><Plus /></el-icon>
          保存当前配置为新预设
        </el-button>
      </div>

      <!-- 预设列表表格 -->
      <el-card class="preset-list-card">
        <el-table
          :data="presets"
          style="width: 100%"
          v-loading="false"
          empty-text="暂无预设，请点击上方按钮创建"
        >
          <!-- 预设名称列 -->
          <el-table-column prop="name" label="预设名称" min-width="150">
            <template #default="{ row }">
              <div class="preset-name-cell">
                <span class="preset-name">{{ row.name }}</span>
                <el-tag v-if="isCurrentPreset(row)" type="success" size="small" class="current-tag">
                  当前
                </el-tag>
              </div>
            </template>
          </el-table-column>

          <!-- 描述列 -->
          <el-table-column prop="description" label="描述" min-width="200">
            <template #default="{ row }">
              <span class="preset-description">
                {{ row.description || '无描述' }}
              </span>
            </template>
          </el-table-column>

          <!-- 创建时间列 -->
          <el-table-column prop="createdAt" label="创建时间" width="180">
            <template #default="{ row }">
              {{ formatDate(row.createdAt) }}
            </template>
          </el-table-column>

          <!-- 更新时间列 -->
          <el-table-column prop="updatedAt" label="更新时间" width="180">
            <template #default="{ row }">
              {{ formatDate(row.updatedAt) }}
            </template>
          </el-table-column>

          <!-- 操作列 -->
          <el-table-column label="操作" width="260" fixed="right">
            <template #default="{ row }">
              <el-button
                size="small"
                @click="handleViewPreset(row)"
              >
                查看
              </el-button>
              <el-button
                size="small"
                type="primary"
                @click="handleSwitchPreset(row)"
              >
                切换
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="handleDeletePreset(row)"
              >
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <!-- 预设对话框组件 -->
      <PresetDialog
        v-model:visible="dialogVisible"
        :presets="presets"
        @save="handleSavePreset"
        @load="handleLoadPreset"
        @delete="async (name) => { await deletePreset(name); await loadPresets(); }"
      />

      <!-- 预设详情对话框 -->
      <PresetDetailDialog
        v-model:visible="detailDialogVisible"
        :preset="selectedPreset"
      />
    </div>
  </AppLayout>
</template>

<style scoped>
.preset-manage {
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

.preset-list-card {
  margin-top: 20px;
}

.preset-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.preset-name {
  font-weight: 500;
  color: #303133;
}

.current-tag {
  margin-left: 4px;
}

.preset-description {
  color: #606266;
}
</style>
