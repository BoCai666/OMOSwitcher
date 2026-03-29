<script setup lang="ts">
import { useTheme } from '@/composables/useTheme'

const { isDark, toggleTheme } = useTheme()
</script>

<template>
  <button
    class="theme-toggle"
    :title="isDark ? '切换到明色主题' : '切换到暗色主题'"
    @click="toggleTheme"
  >
    <span class="icon-wrapper">
      <!-- 太阳图标：暗色主题显示，点击切换到明色 -->
      <svg v-if="isDark" class="icon sun" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="5" stroke="currentColor" fill="none" stroke-width="2"/>
        <line x1="12" y1="1" x2="12" y2="3" stroke="currentColor" stroke-width="2"/>
        <line x1="12" y1="21" x2="12" y2="23" stroke="currentColor" stroke-width="2"/>
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" stroke="currentColor" stroke-width="2"/>
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" stroke="currentColor" stroke-width="2"/>
        <line x1="1" y1="12" x2="3" y2="12" stroke="currentColor" stroke-width="2"/>
        <line x1="21" y1="12" x2="23" y2="12" stroke="currentColor" stroke-width="2"/>
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" stroke="currentColor" stroke-width="2"/>
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" stroke="currentColor" stroke-width="2"/>
      </svg>
      <!-- 月亮图标：明色主题显示，点击切换到暗色 -->
      <svg v-else class="icon moon" viewBox="0 0 24 24">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" stroke="currentColor" fill="none" stroke-width="2"/>
      </svg>
    </span>
  </button>
</template>

<style scoped>
.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  transition: background-color var(--app-transition-normal),
              box-shadow var(--app-transition-normal);
}

.theme-toggle:hover {
  background-color: var(--app-bg-hover);
  box-shadow: 0 0 12px var(--app-color-primary);
}

.icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
}

.icon {
  width: 24px;
  height: 24px;
  color: var(--app-text-secondary);
  transition: transform var(--app-transition-normal),
              opacity var(--app-transition-normal),
              color var(--app-transition-normal);
}

/* 太阳和月亮图标旋转 + 淡入淡出 */
.sun {
  animation: sun-rotate 200ms ease forwards;
}

.moon {
  animation: moon-rotate 200ms ease forwards;
}

.theme-toggle:hover .icon {
  color: var(--app-color-primary);
}

@keyframes sun-rotate {
  0% {
    transform: rotate(0deg) scale(0.8);
    opacity: 0.5;
  }
  100% {
    transform: rotate(90deg) scale(1);
    opacity: 1;
  }
}

@keyframes moon-rotate {
  0% {
    transform: rotate(0deg) scale(0.8);
    opacity: 0.5;
  }
  100% {
    transform: rotate(-90deg) scale(1);
    opacity: 1;
  }
}
</style>
