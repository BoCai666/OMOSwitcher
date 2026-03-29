<script setup lang="ts">
import { ElSwitch } from 'element-plus'
import { useTheme } from '@/composables/useTheme'
import type { ThemeAccent } from '@/types/theme'

// 使用主题管理 composable
const { accent, effectsEnabled, setThemeAccent, setEffectsEnabled } = useTheme()

// 可选的主题色列表
const colorOptions: ThemeAccent[] = ['cyan', 'magenta', 'purple', 'gold']

// 颜色显示名称映射
const colorLabels: Record<ThemeAccent, string> = {
  cyan: '青色',
  magenta: '品红',
  purple: '紫色',
  gold: '金色'
}

/**
 * 处理特效开关变化
 * @param value 开关状态
 */
function handleEffectsChange(value: boolean | string | number): void {
  setEffectsEnabled(Boolean(value))
}

/**
 * 处理颜色选择
 * @param color 选中的主题色
 */
function handleColorSelect(color: ThemeAccent): void {
  setThemeAccent(color)
}
</script>

<template>
  <div class="effect-settings">
    <!-- 特效总开关 -->
    <div class="setting-item">
      <div class="label-group">
        <span class="label">特效</span>
        <span class="sub-label">启用动画和视觉效果</span>
      </div>
      <ElSwitch
        v-model="effectsEnabled"
        class="effects-switch"
        @change="handleEffectsChange"
      />
    </div>

    <!-- 主题色选择器 -->
    <div class="setting-item">
      <span class="label">主题色</span>
      <div class="color-picker">
        <button
          v-for="color in colorOptions"
          :key="color"
          :class="['color-option', color, { active: accent === color }]"
          :title="colorLabels[color]"
          :aria-label="`选择${colorLabels[color]}主题`"
          @click="handleColorSelect(color)"
        />
      </div>
    </div>

    <!-- 性能提示 -->
    <p class="hint">
      <span class="hint-icon">💡</span>
      关闭特效可提升低配置设备性能
    </p>
  </div>
</template>

<style scoped>
/* 容器样式 */
.effect-settings {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-5);
  padding: var(--app-spacing-4);
  background: var(--app-bg-card);
  border-radius: var(--app-radius-lg);
  border: 1px solid var(--app-border-default);
}

/* 设置项容器 */
.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--app-spacing-4);
}

/* 标签组 */
.label-group {
  display: flex;
  flex-direction: column;
  gap: var(--app-spacing-1);
}

/* 主标签 */
.label {
  font-size: 14px;
  font-weight: 500;
  color: var(--app-text-primary);
}

/* 副标签 */
.sub-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

/* 颜色选择器容器 */
.color-picker {
  display: flex;
  gap: var(--app-spacing-3);
  align-items: center;
}

/* 颜色选项按钮 */
.color-option {
  width: 28px;
  height: 28px;
  border-radius: var(--app-radius-full);
  border: 2px solid transparent;
  cursor: pointer;
  background-color: var(--color);
  transition: transform var(--app-transition-normal),
              box-shadow var(--app-transition-normal),
              border-color var(--app-transition-fast);
  position: relative;
  padding: 0;
}

/* 颜色定义 */
.color-option.cyan {
  --color: var(--app-color-neon-cyan);
}

.color-option.magenta {
  --color: var(--app-color-neon-magenta);
}

.color-option.purple {
  --color: var(--app-color-neon-purple);
}

.color-option.gold {
  --color: var(--app-color-neon-gold);
}

/* 悬停效果 */
.color-option:hover {
  transform: scale(1.15);
  box-shadow: 0 0 12px var(--color);
}

/* 选中状态 */
.color-option.active {
  border-color: var(--app-text-primary);
  box-shadow: 0 0 0 2px var(--app-bg-card), 0 0 0 4px var(--color);
  transform: scale(1.1);
}

/* 禁用 reduce-motion 时的简化过渡 */
:global(.reduce-motion) .color-option {
  transition: none;
}

:global(.reduce-motion) .color-option:hover {
  transform: none;
}

:global(.reduce-motion) .color-option.active {
  transform: none;
}

/* 性能提示 */
.hint {
  margin: 0;
  padding: var(--app-spacing-3) var(--app-spacing-4);
  font-size: 12px;
  color: var(--app-text-tertiary);
  background: var(--app-bg-base);
  border-radius: var(--app-radius-md);
  border: 1px dashed var(--app-border-default);
  display: flex;
  align-items: center;
  gap: var(--app-spacing-2);
}

.hint-icon {
  font-size: 14px;
  line-height: 1;
}

/* 暗色/亮色主题适配 */
/* cyberpunk 主题下的额外发光效果 */
:global(.cyberpunk) .color-option.active {
  box-shadow: 0 0 0 2px var(--app-bg-card), 0 0 0 4px var(--color), 0 0 20px var(--color);
}

:global(.cyberpunk) .effect-settings {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

/* glassmorphism 主题下的玻璃效果 */
:global(.glassmorphism) .effect-settings {
  background: var(--app-color-glass-bg);
  backdrop-filter: var(--app-effect-glass-blur);
  border-color: var(--app-color-glass-border);
}

/* 响应式适配 */
@media (max-width: 480px) {
  .effect-settings {
    padding: var(--app-spacing-3);
  }

  .setting-item {
    flex-wrap: wrap;
  }

  .color-option {
    width: 24px;
    height: 24px;
  }
}
</style>
