<script setup lang="ts">
// 预设管理页面组件
// 提供预设列表展示、切换、删除和保存功能
import { ref, onMounted, computed } from 'vue'
import { Plus, Refresh } from '@element-plus/icons-vue'
import type { Preset } from '@/types'
import { useConfigStore } from '@/stores'
import {
  listPresets,
  savePreset,
  deletePreset,
  switchPreset,
  getCurrentPreset,
  recordPresetUsage,
  invalidatePresetsCache
} from '@/services/presetStore'
import { showError, showSuccess, showWarning, confirm, AppError, ErrorCode } from '@/utils/errorHandler'
import PresetDialog from '@/components/PresetDialog.vue'
import PresetDetailDialog from '@/components/PresetDetailDialog.vue'

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
const selectedPresetName = ref<string | null>(null)

// 选中的预设（响应式，从列表中获取最新数据）
const selectedPreset = computed(() => {
  if (!selectedPresetName.value) return null
  return presets.value.find(p => p.name === selectedPresetName.value) || null
})

// 加载预设列表
const loadPresets = async () => {
  presets.value = await listPresets()
  currentPresetName.value = (await getCurrentPreset()) || null
}

// 刷新预设列表（清除缓存后重新从磁盘读取）
const refreshing = ref(false)
const handleRefresh = async () => {
  refreshing.value = true
  try {
    invalidatePresetsCache()
    await loadPresets()
    showSuccess('预设列表已刷新')
  } finally {
    refreshing.value = false
  }
}

// 页面加载时获取预设列表
onMounted(() => {
  loadPresets()
})

// 检查预设是否为当前预设
const isCurrentPreset = (preset: Preset) => {
  return currentPresetName.value === preset.name
}

// 获取行类名（用于高亮当前预设）
const getRowClassName = ({ row }: { row: Preset }) => {
  return isCurrentPreset(row) ? 'is-current-preset' : ''
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

    // 应用预设配置到 configStore，传入预设名称以便跟踪
    configStore.applyPreset(result.preset.config, preset.name)
    
    // 立即保存配置到文件
    await configStore.saveConfig()
    
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
      // 如果删除的是当前预设，清除 configStore 中的当前预设状态
      if (configStore.currentPresetName === preset.name) {
        configStore.clearCurrentPreset()
      }
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
  selectedPresetName.value = preset.name
  detailDialogVisible.value = true
}
</script>

<template>
  <div class="preset-manage">
      <!-- 页面标题和操作按钮 -->
      <div class="page-header">
        <div class="header-left">
          <span class="subtitle">管理配置预设，快速切换不同配置方案</span>
        </div>
        <div class="header-right">
          <el-button :icon="Refresh" :loading="refreshing" @click="handleRefresh">
            刷新
          </el-button>
          <el-button type="primary" @click="dialogVisible = true">
            <el-icon><Plus /></el-icon>
            保存当前配置为新预设
          </el-button>
        </div>
      </div>

      <!-- 预设列表表格 -->
      <el-card class="preset-list-card">
        <el-table
          :data="presets"
          style="width: 100%"
          v-loading="false"
          :row-class-name="getRowClassName"
          @row-click="handleViewPreset"
        >
          <!-- 预设名称列 -->
          <el-table-column prop="name" label="预设名称" min-width="120">
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
          <el-table-column prop="createdAt" label="创建时间" min-width="150">
            <template #default="{ row }">
              {{ formatDate(row.createdAt) }}
            </template>
          </el-table-column>

          <!-- 更新时间列 -->
          <el-table-column prop="updatedAt" label="更新时间" min-width="150">
            <template #default="{ row }">
              {{ formatDate(row.updatedAt) }}
            </template>
          </el-table-column>

          <!-- 操作列 -->
          <el-table-column label="操作" width="150" fixed="right" class-name="action-column">
            <template #default="{ row }">
              <div class="action-buttons">
                <el-button
                  size="small"
                  class="neon-button-switch"
                  @click.stop="handleSwitchPreset(row)"
                >
                  切换
                </el-button>
                <el-button
                  size="small"
                  class="neon-button-delete"
                  @click.stop="handleDeletePreset(row)"
                >
                  删除
                </el-button>
              </div>
            </template>
          </el-table-column>
        </el-table>

        <!-- 自定义空状态 -->
        <template #empty>
          <div class="empty-state">
            <div class="empty-illustration">
              <svg viewBox="0 0 200 160" fill="none" xmlns="http://www.w3.org/2000/svg">
                <!-- 主文件夹 -->
                <path d="M40 50C40 44.4772 44.4772 40 50 40H75L85 55H150C155.523 55 160 59.4772 160 65V120C160 125.523 155.523 130 150 130H50C44.4772 130 40 125.523 40 120V50Z" 
                  fill="url(#folderGradient)" stroke="var(--app-color-primary)" stroke-width="1.5"/>
                <!-- 内部发光效果 -->
                <path d="M50 65H150V115H50V65Z" fill="var(--app-bg-base)" opacity="0.5"/>
                <!-- 小文件图标 -->
                <rect x="65" y="80" width="25" height="30" rx="3" fill="var(--app-bg-card)" stroke="var(--app-border-default)"/>
                <rect x="72" y="88" width="11" height="2" rx="1" fill="var(--app-color-primary)" opacity="0.6"/>
                <rect x="72" y="94" width="11" height="2" rx="1" fill="var(--app-color-primary)" opacity="0.4"/>
                <!-- 星形装饰 -->
                <circle cx="150" cy="45" r="8" fill="var(--app-color-primary)" opacity="0.2"/>
                <circle cx="150" cy="45" r="4" fill="var(--app-color-primary)" opacity="0.6"/>
                <!-- 底部装饰线 -->
                <path d="M60 140H140" stroke="var(--app-border-default)" stroke-width="2" stroke-linecap="round" opacity="0.5"/>
                <defs>
                  <linearGradient id="folderGradient" x1="40" y1="40" x2="160" y2="130" gradientUnits="userSpaceOnUse">
                    <stop offset="0%" stop-color="var(--app-bg-card)"/>
                    <stop offset="100%" stop-color="var(--app-bg-base)"/>
                  </linearGradient>
                </defs>
              </svg>
            </div>
            <h3 class="empty-title">暂无预设配置</h3>
            <p class="empty-description">
              您还没有保存任何配置预设<br>
              点击下方按钮，将当前配置保存为新预设
            </p>
            <el-button type="primary" class="neon-button-primary" @click="dialogVisible = true">
              <el-icon><Plus /></el-icon>
              创建第一个预设
            </el-button>
          </div>
        </template>
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
        @updated="loadPresets"
      />
    </div>
</template>

<style scoped>
/* ============================================
   预设管理页面样式 - 赛博朋克霓虹主题
   使用 CSS 变量实现主题适配
   ============================================ */

/* 基础变量定义 - 使用全局 CSS 变量 */
.preset-manage {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 0 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* 刷新按钮 - hover 时保持文字可读 */
.header-right .el-button:not(.el-button--primary) {
  background: transparent !important;
  border: 1px solid var(--app-border-default) !important;
  color: var(--app-text-primary) !important;
  transition: all 0.3s ease !important;
}

.header-right .el-button:not(.el-button--primary):hover {
  background: var(--app-color-primary) !important;
  border-color: var(--app-color-primary) !important;
  color: #ffffff !important;
}

/* 暗色模式下刷新按钮 hover 使用紫色，避免天蓝色 */
html.dark .header-right .el-button:not(.el-button--primary):hover {
  background: var(--app-color-purple) !important;
  border-color: var(--app-color-purple) !important;
}

.subtitle {
  color: var(--app-text-secondary);
  font-size: 14px;
  letter-spacing: 0.5px;
}

/* 霓虹按钮 - 主按钮 */
:deep(.el-button[type="primary"]) {
  background: transparent !important;
  border: 1px solid var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  box-shadow: 
    0 0 10px rgba(0, 212, 255, 0.3),
    inset 0 0 10px rgba(0, 212, 255, 0.05) !important;
  transition: all 0.3s ease !important;
}

:deep(.el-button[type="primary"]:hover) {
  background: rgba(0, 212, 255, 0.1) !important;
  box-shadow: 
    0 0 20px rgba(0, 212, 255, 0.5),
    0 0 40px rgba(0, 212, 255, 0.3),
    inset 0 0 15px rgba(0, 212, 255, 0.1) !important;
  transform: translateY(-1px);
}

/* 预设卡片 */
.preset-list-card {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
  border-radius: 12px !important;
  box-shadow: 
    0 4px 20px rgba(0, 0, 0, 0.4),
    0 0 0 1px rgba(0, 212, 255, 0.05) !important;
  overflow: hidden;
}

:deep(.el-card__body) {
  padding: 0 !important;
}

/* 表格样式覆盖 */
:deep(.el-table) {
  background: transparent !important;
  color: var(--app-text-primary) !important;
}

:deep(.el-table__header-wrapper) {
  background: var(--app-bg-base) !important;
}

:deep(.el-table__header) {
  background: transparent !important;
}

:deep(.el-table th.el-table__cell) {
  background: var(--app-bg-base) !important;
  color: var(--app-color-primary) !important;
  font-weight: 600 !important;
  border-bottom: 1px solid var(--app-border-default) !important;
  padding: 16px 12px !important;
  text-transform: uppercase;
  letter-spacing: 1px;
  font-size: 12px;
}

:deep(.el-table td.el-table__cell) {
  background: transparent !important;
  border-bottom: 1px solid var(--app-border-default) !important;
  padding: 16px 12px !important;
}

:deep(.el-table__body tr) {
  transition: all 0.3s ease;
}

/* 表格行悬停效果 */
:deep(.el-table__body tr:hover > td.el-table__cell) {
  background: rgba(0, 212, 255, 0.05) !important;
}

:deep(.el-table__body tr:hover) {
  box-shadow: inset 0 0 30px rgba(0, 212, 255, 0.05);
}

/* 当前预设高亮 - 改进版 */
:deep(.el-table__body tr.is-current-preset td.el-table__cell) {
  background: linear-gradient(90deg, rgba(0, 212, 255, 0.12), rgba(0, 212, 255, 0.05)) !important;
  border-left: 3px solid var(--app-color-primary) !important;
  position: relative;
}

/* 高亮行的过渡效果 */
:deep(.el-table__body tr.is-current-preset) {
  position: relative;
}

/* 当前预设的预设名称加粗 */
:deep(.el-table__body tr.is-current-preset .preset-name) {
  font-weight: 600;
  color: var(--app-color-primary);
}

/* 预设名称单元格 */
.preset-name-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.preset-name {
  font-weight: 500;
  color: var(--app-text-primary);
  font-size: 14px;
}

/* 当前标签 */
:deep(.current-tag.el-tag--success) {
  background: rgba(0, 255, 157, 0.15) !important;
  border: 1px solid var(--app-color-success) !important;
  color: var(--app-color-success) !important;
  box-shadow: 0 0 10px rgba(0, 255, 157, 0.2);
  font-weight: 600;
}

/* 描述单元格 */
.preset-description {
  color: var(--app-text-secondary);
  font-size: 13px;
}

/* 操作按钮容器 */
.action-buttons {
  display: flex;
  gap: 6px;
}

/* 霓虹效果按钮 - 切换 */
.neon-button-switch {
  background: transparent !important;
  border: 1px solid var(--app-color-success) !important;
  color: var(--app-color-success) !important;
  transition: all 0.3s ease !important;
  padding: 5px 10px !important;
  font-size: 12px !important;
}

.neon-button-switch:hover {
  background: rgba(0, 255, 157, 0.1) !important;
  box-shadow: 
    0 0 15px rgba(0, 255, 157, 0.4),
    0 0 30px rgba(0, 255, 157, 0.2),
    inset 0 0 10px rgba(0, 255, 157, 0.1) !important;
}

/* 霓虹效果按钮 - 删除 */
.neon-button-delete {
  background: transparent !important;
  border: 1px solid var(--app-color-danger) !important;
  color: var(--app-color-danger) !important;
  transition: all 0.3s ease !important;
  padding: 5px 10px !important;
  font-size: 12px !important;
}

.neon-button-delete:hover {
  background: rgba(255, 71, 87, 0.1) !important;
  box-shadow: 
    0 0 15px rgba(255, 71, 87, 0.4),
    0 0 30px rgba(255, 71, 87, 0.2),
    inset 0 0 10px rgba(255, 71, 87, 0.1) !important;
}

/* 空状态样式 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-illustration {
  width: 200px;
  height: 160px;
  margin-bottom: 24px;
  filter: drop-shadow(0 0 20px rgba(0, 212, 255, 0.2));
}

.empty-illustration svg {
  width: 100%;
  height: 100%;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--app-text-primary);
  margin-bottom: 12px;
  letter-spacing: 1px;
}

.empty-description {
  font-size: 14px;
  color: var(--app-text-secondary);
  line-height: 1.6;
  margin-bottom: 24px;
  max-width: 400px;
}

/* 空状态霓虹按钮 */
.neon-button-primary {
  background: transparent !important;
  border: 1px solid var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  box-shadow: 
    0 0 15px rgba(0, 212, 255, 0.3),
    inset 0 0 15px rgba(0, 212, 255, 0.05) !important;
  transition: all 0.3s ease !important;
  padding: 12px 24px !important;
  font-size: 14px !important;
}

.neon-button-primary:hover {
  background: rgba(0, 212, 255, 0.15) !important;
  box-shadow: 
    0 0 25px rgba(0, 212, 255, 0.6),
    0 0 50px rgba(0, 212, 255, 0.3),
    inset 0 0 20px rgba(0, 212, 255, 0.1) !important;
  transform: translateY(-2px);
}

/* 输入框样式覆盖 */
:deep(.el-input__wrapper) {
  background: var(--app-bg-base) !important;
  border: 1px solid var(--app-border-default) !important;
  box-shadow: none !important;
}

:deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary) !important;
}

:deep(.el-input__inner) {
  color: var(--app-text-primary) !important;
  background: transparent !important;
}

:deep(.el-input__inner::placeholder) {
  color: var(--app-text-secondary) !important;
}

/* 加载动画覆盖 */
:deep(.el-loading-mask) {
  background: rgba(10, 10, 15, 0.8) !important;
  backdrop-filter: blur(4px);
}

/* 滚动条美化 */
:deep(.el-table__body-wrapper::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-track) {
  background: var(--app-bg-base);
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-thumb) {
  background: var(--app-border-default);
  border-radius: 4px;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-thumb:hover) {
  background: var(--app-color-primary);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk .preset-manage {
  background: transparent;
}

html.cyberpunk .page-header {
  background: rgba(26, 26, 46, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.15);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.1);
}

html.cyberpunk .subtitle {
  color: var(--app-color-primary);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.4);
}

html.cyberpunk .preset-list-card {
  background: rgba(26, 26, 46, 0.9) !important;
  border: 1px solid rgba(0, 255, 255, 0.2) !important;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 40px rgba(0, 255, 255, 0.1) !important;
}

html.cyberpunk :deep(.el-table th.el-table__cell) {
  background: rgba(0, 255, 255, 0.08) !important;
  color: var(--app-color-primary) !important;
  border-bottom: 1px solid rgba(0, 255, 255, 0.2) !important;
}

html.cyberpunk :deep(.el-table__body tr:hover > td.el-table__cell) {
  background: rgba(0, 255, 255, 0.08) !important;
  box-shadow: inset 0 0 20px rgba(0, 255, 255, 0.05);
}

html.cyberpunk :deep(.el-table__body tr.is-current-preset td.el-table__cell) {
  background: linear-gradient(90deg, rgba(0, 255, 136, 0.15), rgba(0, 255, 136, 0.05)) !important;
  border-left: 3px solid var(--app-color-success) !important;
}

html.cyberpunk :deep(.el-table__body tr.is-current-preset .preset-name) {
  color: var(--app-color-success) !important;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

html.cyberpunk .preset-name {
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .neon-button-view:hover {
  box-shadow:
    0 0 15px rgba(0, 255, 255, 0.4),
    0 0 30px rgba(0, 255, 255, 0.2) !important;
}

html.cyberpunk .neon-button-switch {
  border-color: var(--app-color-success) !important;
  color: var(--app-color-success) !important;
}

html.cyberpunk .neon-button-switch:hover {
  background: rgba(0, 255, 136, 0.15) !important;
  box-shadow:
    0 0 20px rgba(0, 255, 136, 0.5),
    0 0 40px rgba(0, 255, 136, 0.2) !important;
}

html.cyberpunk .neon-button-delete:hover {
  background: rgba(255, 51, 102, 0.15) !important;
  box-shadow:
    0 0 20px rgba(255, 51, 102, 0.5),
    0 0 40px rgba(255, 51, 102, 0.2) !important;
}

html.cyberpunk .empty-illustration {
  filter: drop-shadow(0 0 30px rgba(0, 255, 255, 0.3));
}

html.cyberpunk .empty-title {
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism .preset-manage {
  background: transparent;
}

html.glassmorphism .page-header {
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

html.glassmorphism .subtitle {
  color: var(--app-text-secondary);
}

html.glassmorphism .preset-list-card {
  background: rgba(255, 255, 255, 0.7) !important;
  border: 1px solid rgba(255, 255, 255, 0.9) !important;
  backdrop-filter: blur(16px) !important;
  -webkit-backdrop-filter: blur(16px) !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1) !important;
}

html.glassmorphism :deep(.el-table) {
  background: transparent !important;
}

html.glassmorphism :deep(.el-table th.el-table__cell) {
  background: rgba(37, 99, 235, 0.08) !important;
  color: var(--app-color-primary) !important;
  border-bottom: 1px solid rgba(37, 99, 235, 0.15) !important;
}

html.glassmorphism :deep(.el-table td.el-table__cell) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05) !important;
}

html.glassmorphism :deep(.el-table__body tr:hover > td.el-table__cell) {
  background: rgba(37, 99, 235, 0.05) !important;
}

html.glassmorphism :deep(.el-table__body tr.is-current-preset td.el-table__cell) {
  background: linear-gradient(90deg, rgba(16, 185, 129, 0.12), rgba(16, 185, 129, 0.04)) !important;
  border-left: 3px solid var(--app-color-success) !important;
}

html.glassmorphism :deep(.el-table__body tr.is-current-preset .preset-name) {
  color: var(--app-color-success) !important;
  font-weight: 600;
}

html.glassmorphism :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.8) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
}

html.glassmorphism :deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary) !important;
}

html.glassmorphism .neon-button-view {
  background: rgba(255, 255, 255, 0.6) !important;
  border: 1px solid rgba(0, 0, 0, 0.15) !important;
  color: var(--app-text-primary) !important;
}

html.glassmorphism .neon-button-view:hover {
  background: rgba(37, 99, 235, 0.1) !important;
  border-color: var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  box-shadow: none !important;
}

html.glassmorphism .neon-button-switch {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
}

html.glassmorphism .neon-button-switch:hover {
  background: rgba(16, 185, 129, 0.2) !important;
  box-shadow: 0 4px 16px rgba(16, 185, 129, 0.2) !important;
}

html.glassmorphism .neon-button-delete {
  background: rgba(239, 68, 68, 0.1) !important;
  border: 1px solid rgba(239, 68, 68, 0.3) !important;
  color: var(--app-color-danger) !important;
}

html.glassmorphism .neon-button-delete:hover {
  background: rgba(239, 68, 68, 0.2) !important;
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.2) !important;
}

html.glassmorphism .empty-state {
  background: rgba(255, 255, 255, 0.5);
  border-radius: var(--app-radius-lg);
}

html.glassmorphism .neon-button-primary {
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-secondary)) !important;
  border: none !important;
  color: white !important;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.3) !important;
}

html.glassmorphism .neon-button-primary:hover {
  box-shadow: 0 8px 24px rgba(37, 99, 235, 0.4) !important;
  transform: translateY(-2px);
}

/* 玻璃拟态主题 - 取消按钮增强对比度 */
html.glassmorphism .edit-actions :deep(.el-button:not(.el-button--primary)) {
  background: rgba(255, 255, 255, 0.8) !important;
  border: 1px solid rgba(0, 0, 0, 0.2) !important;
  color: #374151 !important;
  font-weight: 500;
}

html.glassmorphism .edit-actions :deep(.el-button:not(.el-button--primary):hover) {
  border-color: var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  background: rgba(255, 255, 255, 0.95) !important;
}

/* ==================== 明色主题 (html.light - 非玻璃拟态/非暗色) ==================== */
html.light:not(.cyberpunk):not(.dark) .preset-manage {
  background: transparent;
}

html.light:not(.cyberpunk):not(.dark) .page-header {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: var(--app-radius-md);
  padding: 16px 20px;
}

html.light:not(.cyberpunk):not(.dark) .subtitle {
  color: var(--app-text-secondary);
}

html.light:not(.cyberpunk):not(.dark) .preset-list-card {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
  box-shadow: var(--app-shadow-sm) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-table th.el-table__cell) {
  background: var(--app-bg-elevated) !important;
  color: var(--app-text-primary) !important;
  border-bottom: 1px solid var(--app-border-default) !important;
  text-shadow: none !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-table td.el-table__cell) {
  border-bottom: 1px solid var(--app-border-default) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-table__body tr:hover > td.el-table__cell) {
  background: rgba(0, 168, 232, 0.05) !important;
  box-shadow: none !important;
}

/* 明色主题 - 当前预设高亮 */
html.light:not(.cyberpunk):not(.dark) :deep(.el-table__body tr.is-current-preset td.el-table__cell) {
  background: linear-gradient(90deg, rgba(0, 168, 232, 0.1), rgba(0, 168, 232, 0.03)) !important;
  border-left: 3px solid var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-table__body tr.is-current-preset .preset-name) {
  color: var(--app-color-primary) !important;
  font-weight: 600;
}

html.light:not(.cyberpunk):not(.dark) .preset-name {
  color: var(--app-text-primary);
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) :deep(.current-tag.el-tag--success) {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.4) !important;
  color: var(--app-color-success) !important;
  box-shadow: none;
}

/* 明色主题 - 按钮样式 */
html.light:not(.cyberpunk):not(.dark) :deep(.el-button[type="primary"]) {
  background: var(--app-color-primary) !important;
  border: none !important;
  color: #ffffff !important;
  box-shadow: 0 2px 8px rgba(0, 168, 232, 0.25) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-button[type="primary"]:hover) {
  background: var(--app-color-primary-hover) !important;
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.35) !important;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-view {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
  color: var(--app-text-secondary) !important;
  box-shadow: none !important;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-view:hover {
  background: var(--app-bg-hover) !important;
  border-color: var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  box-shadow: none !important;
  transform: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-switch {
  background: rgba(16, 185, 129, 0.1) !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  color: var(--app-color-success) !important;
  box-shadow: none !important;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-switch:hover {
  background: rgba(16, 185, 129, 0.15) !important;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.2) !important;
  transform: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-delete {
  background: rgba(239, 68, 68, 0.1) !important;
  border: 1px solid rgba(239, 68, 68, 0.3) !important;
  color: var(--app-color-danger) !important;
  box-shadow: none !important;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-delete:hover {
  background: rgba(239, 68, 68, 0.15) !important;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.2) !important;
  transform: none;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-primary {
  background: var(--app-color-primary) !important;
  border: none !important;
  color: #ffffff !important;
  box-shadow: 0 4px 12px rgba(0, 168, 232, 0.25) !important;
}

html.light:not(.cyberpunk):not(.dark) .neon-button-primary:hover {
  box-shadow: 0 6px 16px rgba(0, 168, 232, 0.35) !important;
  transform: translateY(-1px);
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-input__wrapper) {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-input__wrapper:hover) {
  border-color: var(--app-color-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) :deep(.el-input__inner) {
  color: var(--app-text-primary) !important;
}

html.light:not(.cyberpunk):not(.dark) .empty-illustration {
  filter: none;
}

html.light:not(.cyberpunk):not(.dark) .empty-title {
  text-shadow: none;
}

html.light:not(.cyberpunk):not(.dark) .edit-actions :deep(.el-button:not(.el-button--primary)) {
  background: var(--app-bg-card) !important;
  border: 1px solid var(--app-border-default) !important;
  color: var(--app-text-secondary) !important;
}

html.light:not(.cyberpunk):not(.dark) .edit-actions :deep(.el-button:not(.el-button--primary):hover) {
  border-color: var(--app-color-primary) !important;
  color: var(--app-color-primary) !important;
  background: var(--app-bg-hover) !important;
}
</style>
