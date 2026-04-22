<script setup lang="ts">
/**
 * 配置卡片组件
 * 用于显示 Agent 或 Category 的配置信息
 * 点击卡片可查看详情
 */
import { computed } from 'vue'
import { ArrowRight, Edit } from '@element-plus/icons-vue'
import type { Model } from '@/types'

const props = defineProps<{
  name: string
  modelValue: string
  models: Model[]
  description?: string
  clickable?: boolean
  fallbackCount?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'click': []
  'click-model': []
}>()

// 当前模型信息
const currentModelInfo = computed(() => {
  return props.models.find(m => m.id === props.modelValue)
})

// 供应商名称
const providerName = computed(() => {
  return currentModelInfo.value?.provider || props.modelValue.split('/')[0]
})

// 模型显示名称
const displayModel = computed(() => {
  return currentModelInfo.value?.name || props.modelValue.split('/').pop() || props.modelValue
})

// 备用模型计数，默认0
const fallbackCount = computed(() => props.fallbackCount ?? 0)
</script>

<template>
  <el-card 
    class="config-card" 
    shadow="hover"
    :class="{ 'clickable': clickable }"
    @click="clickable && emit('click')"
  >
    <template #header>
      <div class="card-header">
        <span class="name">{{ name }}</span>
        <el-icon v-if="clickable" class="click-hint"><ArrowRight /></el-icon>
      </div>
    </template>
    
    <div class="card-content">
      <!-- 描述信息 -->
      <div v-if="description" class="description">
        {{ description }}
      </div>
      
      <!-- 当前模型显示 -->
      <div 
        class="model-display" 
        :class="{ 'model-clickable': clickable }"
        @click.stop="clickable && emit('click-model')"
      >
        <div class="model-info">
          <span class="label">当前模型</span>
          <div class="model-details">
            <el-tag type="info" size="small" class="provider-tag">{{ providerName }}</el-tag>
            <span class="model-name">{{ displayModel }}</span>
          </div>
        </div>
        <el-icon v-if="clickable" class="model-click-hint"><Edit /></el-icon>
      </div>

      <!-- 备用模型指示器 -->
      <div v-if="fallbackCount > 0" class="fallback-indicator">
        <el-tag type="warning" size="small">{{ fallbackCount }} 个备用模型</el-tag>
      </div>
    </div>
  </el-card>
</template>

<style scoped>
.config-card {
  --app-glow-intense: 0 0 40px rgba(0, 212, 255, 0.5);
}

/* ==================== 赛博朋克主题 ==================== */
html.cyberpunk .config-card :deep(.el-card) {
  background: rgba(26, 26, 46, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: 
    0 4px 24px rgba(0, 0, 0, 0.5),
    0 0 20px rgba(0, 255, 255, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  border-radius: var(--app-radius-lg);
  overflow: hidden;
  transition: all var(--app-transition-normal);
}

html.cyberpunk .config-card :deep(.el-card__header) {
  background: linear-gradient(135deg, 
    rgba(0, 255, 255, 0.15) 0%, 
    rgba(255, 0, 255, 0.1) 50%,
    rgba(0, 0, 0, 0) 100%
  );
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
  padding: var(--app-spacing-4);
}

html.cyberpunk .config-card.clickable:hover :deep(.el-card) {
  transform: translateY(-2px);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.6),
    0 0 30px rgba(0, 255, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  border-color: rgba(0, 255, 255, 0.5);
}

html.cyberpunk .description {
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-left: 3px solid var(--app-color-primary);
}

html.cyberpunk .model-display {
  background: linear-gradient(135deg, 
    rgba(0, 255, 255, 0.1) 0%, 
    rgba(255, 0, 255, 0.05) 100%
  );
  border: 1px solid rgba(0, 255, 255, 0.2);
}

html.cyberpunk .model-display::before {
  background: radial-gradient(circle, 
    rgba(0, 255, 255, 0.2) 0%, 
    transparent 70%
  );
}

html.cyberpunk .provider-tag {
  background: rgba(0, 255, 255, 0.15) !important;
  border: 1px solid rgba(0, 255, 255, 0.3) !important;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
}

/* ==================== 玻璃拟态主题 ==================== */
html.glassmorphism .config-card :deep(.el-card) {
  background: rgba(255, 255, 255, 0.75);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.9);
  box-shadow: 
    0 4px 24px rgba(0, 0, 0, 0.06),
    0 0 15px rgba(37, 99, 235, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  border-radius: var(--app-radius-lg);
  overflow: hidden;
  transition: all var(--app-transition-normal);
}

html.glassmorphism .config-card :deep(.el-card__header) {
  background: linear-gradient(135deg, 
    rgba(37, 99, 235, 0.08) 0%, 
    rgba(139, 92, 246, 0.05) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  padding: var(--app-spacing-4);
}

html.glassmorphism .config-card.clickable:hover :deep(.el-card) {
  transform: translateY(-2px);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.1),
    0 0 20px rgba(37, 99, 235, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  border-color: rgba(37, 99, 235, 0.3);
}

html.glassmorphism .description {
  background: rgba(37, 99, 235, 0.05);
  border: 1px solid rgba(37, 99, 235, 0.1);
  border-left: 3px solid var(--app-color-primary);
}

html.glassmorphism .model-display {
  background: linear-gradient(135deg, 
    rgba(37, 99, 235, 0.06) 0%, 
    rgba(139, 92, 246, 0.03) 100%
  );
  border: 1px solid rgba(37, 99, 235, 0.12);
}

html.glassmorphism .model-display::before {
  background: radial-gradient(circle, 
    rgba(37, 99, 235, 0.1) 0%, 
    transparent 70%
  );
}

html.glassmorphism .provider-tag {
  background: rgba(37, 99, 235, 0.1) !important;
  border: 1px solid rgba(37, 99, 235, 0.2) !important;
  box-shadow: none;
}

/* ==================== 基础样式 ==================== */
.config-card {
  margin-bottom: var(--app-spacing-4);
}

/* 可点击光晕动画 */
.config-card.clickable :deep(.el-card)::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, 
    transparent, 
    rgba(0, 212, 255, 0.05), 
    transparent
  );
  transition: left 0.6s ease;
  pointer-events: none;
}

.config-card.clickable:hover :deep(.el-card)::before {
  left: 100%;
}

.config-card {
  margin-bottom: var(--app-spacing-4);
}

.config-card.clickable {
  cursor: pointer;
}

.name {
  font-weight: 600;
  font-size: 16px;
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.click-hint {
  color: var(--app-text-tertiary);
  font-size: 14px;
  transition: all var(--app-transition-normal);
}

.config-card.clickable:hover .click-hint {
  color: var(--app-color-primary);
  transform: translateX(4px);
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-3);
  padding: var(--app-spacing-2) 0;
}

/* 描述区域 */
.description {
  font-size: 13px;
  color: var(--app-text-secondary);
  line-height: 1.6;
  padding: var(--app-spacing-3) var(--app-spacing-4);
  background: rgba(0, 168, 232, 0.05);
  border-radius: var(--app-radius-md);
  border: 1px solid rgba(0, 168, 232, 0.1);
  border-left: 3px solid var(--app-color-primary);
  backdrop-filter: blur(4px);
}

/* 模型显示区域 */
.model-display {
  padding: var(--app-spacing-4);
  border-radius: var(--app-radius-md);
  position: relative;
  overflow: hidden;
  background: linear-gradient(135deg, 
    rgba(0, 168, 232, 0.06) 0%, 
    rgba(139, 92, 246, 0.04) 100%
  );
  border: 1px solid rgba(0, 168, 232, 0.15);
}

/* 模型区域可点击样式 */
.model-display.model-clickable {
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.model-display.model-clickable:hover {
  border-color: var(--app-color-primary);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
  transform: translateY(-1px);
}

.model-click-hint {
  color: var(--app-text-tertiary);
  font-size: 16px;
  opacity: 0;
  transition: all 0.3s ease;
  flex-shrink: 0;
  margin-left: var(--app-spacing-3);
}

.model-display.model-clickable:hover .model-click-hint {
  opacity: 1;
  color: var(--app-color-primary);
}

/* 赛博朋克主题 - 模型区域可点击 */
html.cyberpunk .model-display.model-clickable:hover {
  border-color: rgba(0, 255, 255, 0.5);
  box-shadow: 
    0 0 20px rgba(0, 255, 255, 0.3),
    inset 0 0 15px rgba(0, 255, 255, 0.05);
}

html.cyberpunk .model-click-hint {
  color: var(--app-text-tertiary);
}

html.cyberpunk .model-display.model-clickable:hover .model-click-hint {
  color: var(--app-color-primary);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.5);
}

/* 玻璃拟态主题 - 模型区域可点击 */
html.glassmorphism .model-display.model-clickable:hover {
  border-color: rgba(37, 99, 235, 0.4);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.15);
}

html.glassmorphism .model-display.model-clickable:hover .model-click-hint {
  color: var(--app-color-primary);
}

/* 模型显示区域微光装饰 */
.model-display::before {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  width: 60px;
  height: 60px;
  background: radial-gradient(circle, 
    rgba(0, 212, 255, 0.15) 0%, 
    transparent 70%
  );
  pointer-events: none;
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-2);
}

.label {
  font-size: 11px;
  color: var(--app-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.model-details {
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
  flex-wrap: wrap;
}

/* 供应商标签 - 基础样式 */
.provider-tag {
  font-size: 11px !important;
  color: var(--app-color-primary) !important;
  backdrop-filter: blur(4px);
}

.model-name {
  font-weight: 600;
  font-size: 14px;
  background: linear-gradient(135deg, var(--app-color-primary), var(--app-color-accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 备用模型指示器 */
.fallback-indicator {
  margin-top: var(--app-spacing-2);
  padding-top: var(--app-spacing-2);
  border-top: 1px dashed rgba(0, 168, 232, 0.15);
}

/* 卡片进入动画 */
@keyframes card-enter {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.config-card {
  animation: card-enter 0.4s ease-out;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .config-card :deep(.el-card__header),
  .config-card :deep(.el-card__body) {
    padding: var(--app-spacing-3);
  }
  
  .name {
    font-size: 14px;
  }
  
  .model-name {
    font-size: 13px;
  }
}
</style>
