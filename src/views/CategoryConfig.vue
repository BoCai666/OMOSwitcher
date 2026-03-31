<script setup lang="ts">
/**
 * Category 配置页面
 * 为每个 Category 配置模型
 */
import { ref, onMounted, computed } from 'vue'
import { useConfigStore } from '@/stores/config'
import { listModels } from '@/services/modelStore'
import { showSuccess, showError } from '@/utils/errorHandler'
import type { CategoryName, Model } from '@/types'
import { CATEGORY_NAMES, CATEGORY_INFO } from '@/types/config'
import { CATEGORY_DETAILS } from '@/data/categoryDetails'

const configStore = useConfigStore()

// 加载状态
const loading = ref(true)

// 模型列表
const models = ref<Model[]>([])

// 搜索关键词
const searchKeyword = ref('')

// 过滤后的 Category 列表
const filteredCategories = computed(() => {
  if (!searchKeyword.value) {
    return CATEGORY_NAMES
  }
  const keyword = searchKeyword.value.toLowerCase()
  return CATEGORY_NAMES.filter(name => {
    const info = CATEGORY_INFO[name]
    return (
      name.toLowerCase().includes(keyword) ||
      info.displayName.toLowerCase().includes(keyword) ||
      info.description.toLowerCase().includes(keyword)
    )
  })
})

// 获取 Category 当前配置的模型
function getCategoryModel(name: CategoryName): string {
  return configStore.config?.categories?.[name]?.model || ''
}

// 获取 Category 详情
function getCategoryDetail(name: CategoryName) {
  return CATEGORY_DETAILS[name]
}

// 更新 Category 模型
function updateCategoryModel(name: CategoryName, model: string) {
  configStore.updateCategoryModel(name, model)
}

// 保存配置
async function saveConfig() {
  try {
    await configStore.saveConfig()
    showSuccess('配置已保存')
  } catch (error) {
    showError('保存配置失败')
  }
}

// 加载数据
onMounted(async () => {
  loading.value = true
  try {
    // 加载配置
    if (!configStore.isLoaded) {
      await configStore.loadConfig()
    }
    // 加载模型列表
    models.value = await listModels()
  } catch (error) {
    showError('加载数据失败')
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="category-config">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">Category 配置</h1>
      <p class="page-desc">为每个任务类别配置使用的模型</p>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索 Category..."
        clearable
        class="search-input"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      
      <el-button 
        type="primary" 
        :disabled="!configStore.hasUnsavedChanges"
        @click="saveConfig"
      >
        <el-icon><DocumentChecked /></el-icon>
        保存配置
      </el-button>
    </div>

    <!-- 加载状态 -->
    <el-skeleton v-if="loading" :rows="10" animated />

    <!-- Category 列表 -->
    <div v-else class="category-list">
      <el-card 
        v-for="name in filteredCategories" 
        :key="name" 
        class="category-card"
        shadow="hover"
      >
        <div class="category-header">
          <div class="category-info">
            <h3 class="category-name">{{ CATEGORY_INFO[name].displayName }}</h3>
            <el-tag size="small" type="info">{{ name }}</el-tag>
          </div>
          <div class="category-model-select">
            <el-select
              :model-value="getCategoryModel(name)"
              placeholder="选择模型"
              filterable
              @change="(model: string) => updateCategoryModel(name, model)"
            >
              <el-option
                v-for="model in models"
                :key="model.id"
                :label="model.name"
                :value="model.id"
              />
            </el-select>
          </div>
        </div>
        
        <p class="category-desc">{{ CATEGORY_INFO[name].description }}</p>
        
        <!-- Category 详细信息 -->
        <el-collapse class="category-details">
          <el-collapse-item title="推荐模型">
            <div v-if="getCategoryDetail(name)" class="detail-content">
              <div class="recommended-model">
                <el-tag type="success">{{ getCategoryDetail(name).recommendedModel }}</el-tag>
              </div>
              <div class="fallback-chain">
                <h4>备选模型链</h4>
                <div 
                  v-for="(fallback, idx) in getCategoryDetail(name).fallbackChain" 
                  :key="idx"
                  class="fallback-item"
                >
                  <span class="fallback-model">{{ fallback.model }}</span>
                  <span class="fallback-providers">
                    {{ fallback.providers.join(', ') }}
                  </span>
                </div>
              </div>
            </div>
          </el-collapse-item>
        </el-collapse>
      </el-card>
    </div>

    <!-- 空状态 -->
    <el-empty v-if="!loading && filteredCategories.length === 0" description="没有找到匹配的 Category" />
  </div>
</template>

<style scoped>
.category-config {
  padding: var(--app-spacing-6);
}

.page-header {
  margin-bottom: var(--app-spacing-6);
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0 0 var(--app-spacing-2) 0;
  color: var(--app-text-primary);
}

.page-desc {
  margin: 0;
  color: var(--app-text-tertiary);
}

.toolbar {
  display: flex;
  gap: var(--app-spacing-3);
  margin-bottom: var(--app-spacing-6);
}

.search-input {
  max-width: 300px;
}

.category-list {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-4);
}

.category-card {
  transition: all 0.3s ease;
}

.category-card:hover {
  transform: translateY(-2px);
}

.category-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--app-spacing-3);
}

.category-info {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-3);
}

.category-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.category-model-select {
  min-width: 200px;
}

.category-desc {
  margin: 0 0 var(--app-spacing-3) 0;
  color: var(--app-text-secondary);
  line-height: 1.6;
}

.category-details {
  border: none;
}

.detail-content {
  padding: var(--app-spacing-2) 0;
}

.recommended-model {
  margin-bottom: var(--app-spacing-4);
}

.fallback-chain h4 {
  margin: 0 0 var(--app-spacing-2) 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text-primary);
}

.fallback-item {
  display: flex;
  gap: var(--app-spacing-3);
  padding: var(--app-spacing-1) 0;
  font-size: 13px;
}

.fallback-model {
  color: var(--app-text-primary);
  font-weight: 500;
}

.fallback-providers {
  color: var(--app-text-tertiary);
}
</style>
