<script setup lang="ts">
/**
 * Category 配置页面
 * 用于管理 8 个 Category 的模型配置
 */
import { ref, onMounted, computed, watch } from 'vue'
import type { CategoryName, Model, OhMyOpenCodeConfig } from '@/types'
import { CATEGORY_NAMES, CATEGORY_INFO, createDefaultConfig } from '@/types'
import ConfigCard from '@/components/ConfigCard.vue'
import ConfigDetailDialog from '@/components/ConfigDetailDialog.vue'
import AppLayout from '@/components/layout/AppLayout.vue'
import { listModels } from '@/services/modelStore'
import { useConfigStore } from '@/stores/config'

// 页面标题
const pageTitle = 'Category 配置'

// 使用共享的配置 store
const configStore = useConfigStore()

// 模型列表
const models = ref<Model[]>([])

// 保存消息
const saveMessage = ref('')

// 详情对话框
const detailDialogVisible = ref(false)
const selectedCategoryName = ref<CategoryName | null>(null)

// 配置数据（使用 store 中的配置，如果未加载则使用默认值）
const config = computed<OhMyOpenCodeConfig>({
  get: () => configStore.config || createDefaultConfig(),
  set: (value) => { configStore.config = value }
})

// 初始化数据
onMounted(async () => {
  // 加载模型列表
  models.value = await listModels()

  // 如果配置未加载，从文件加载
  if (!configStore.isLoaded) {
    await configStore.loadConfig()
  }
})

// 更新 Category 配置
function updateCategoryModel(categoryName: CategoryName, modelId: string) {
  configStore.updateCategoryModel(categoryName, modelId)
}

// 保存配置
async function handleSave() {
  saveMessage.value = ''

  try {
    await configStore.saveConfig()
    saveMessage.value = '保存成功'
    setTimeout(() => {
      saveMessage.value = ''
    }, 3000)
  } catch (error) {
    saveMessage.value = '保存失败: ' + (error as Error).message
  }
}

// 查看详情
function handleViewDetail(categoryName: CategoryName) {
  selectedCategoryName.value = categoryName
  detailDialogVisible.value = true
}

// 监听 store 中的错误
watch(() => configStore.error, (newError) => {
  if (newError) {
    saveMessage.value = '错误: ' + newError
  }
})
</script>

<template>
  <AppLayout :title="pageTitle">
    <div class="category-config">
      <!-- 页面头部 -->
      <div class="page-header">
        <div class="header-left">
          <span class="subtitle">共 {{ CATEGORY_NAMES.length }} 个 Category</span>
          <el-tag v-if="configStore.hasUnsavedChanges" type="warning" size="small">
            未保存
          </el-tag>
        </div>
        <div class="header-actions">
          <el-button
            type="primary"
            :loading="configStore.isSaving"
            @click="handleSave"
          >
            保存配置
          </el-button>
        </div>
      </div>

      <!-- 保存状态提示 -->
      <el-alert
        v-if="saveMessage"
        :title="saveMessage"
        :type="saveMessage.includes('失败') || saveMessage.includes('错误') ? 'error' : 'success'"
        show-icon
        class="save-alert"
        closable
        @close="saveMessage = ''"
      />

      <!-- Category 卡片网格 -->
      <div class="category-grid">
        <ConfigCard
          v-for="name in CATEGORY_NAMES"
          :key="name"
          :name="CATEGORY_INFO[name].displayName"
          :description="CATEGORY_INFO[name].description"
          v-model="config.categories[name].model"
          :models="models"
          :editable="true"
          :clickable="true"
          @update:model-value="(val: string) => updateCategoryModel(name, val)"
          @click="handleViewDetail(name)"
        />
      </div>

      <!-- Category 详情对话框 -->
      <ConfigDetailDialog
        v-if="selectedCategoryName"
        v-model:visible="detailDialogVisible"
        type="category"
        :name="selectedCategoryName"
        :current-model="config.categories[selectedCategoryName].model"
        :models="models"
      />
    </div>
  </AppLayout>
</template>

<style scoped>
.category-config {
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

.save-alert {
  margin-bottom: 20px;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

/* 响应式布局：大屏幕显示 3 列 */
@media (min-width: 992px) {
  .category-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

/* 中等屏幕显示 2 列 */
@media (min-width: 768px) and (max-width: 991px) {
  .category-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* 小屏幕显示 1 列 */
@media (max-width: 767px) {
  .category-grid {
    grid-template-columns: 1fr;
  }
}
</style>
