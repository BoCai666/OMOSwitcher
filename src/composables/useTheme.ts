// 主题切换 composable
// 支持 cyberpunk/glassmorphism/system 三种主题模式、主题色切换和特效开关

import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { ThemeMode, ThemeAccent, ThemeSettings } from '@/types/theme'
import { DEFAULT_THEME_SETTINGS } from '@/types/theme'

// localStorage 键名
const STORAGE_KEYS = {
  mode: 'theme-mode',
  accent: 'theme-accent',
  effects: 'effects-enabled',
  legacy: 'theme' // 旧键名，用于迁移
} as const

// 旧主题映射到新主题模式
const LEGACY_THEME_MAP: Record<string, ThemeMode> = {
  'dark': 'cyberpunk',
  'light': 'glassmorphism'
}

/**
 * 获取系统是否偏好暗色
 */
function getPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

/**
 * 获取系统是否偏好减少动画
 */
function getPrefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

/**
 * 初始化主题设置（立即执行，避免 FOUC）
 */
;(function initTheme() {
  const root = document.documentElement

  // 迁移旧主题设置
  const legacyTheme = localStorage.getItem(STORAGE_KEYS.legacy)
  if (legacyTheme && LEGACY_THEME_MAP[legacyTheme]) {
    localStorage.setItem(STORAGE_KEYS.mode, LEGACY_THEME_MAP[legacyTheme])
    localStorage.removeItem(STORAGE_KEYS.legacy)
  }

  // 读取当前设置
  const savedMode = localStorage.getItem(STORAGE_KEYS.mode) as ThemeMode | null
  const savedAccent = localStorage.getItem(STORAGE_KEYS.accent) as ThemeAccent | null
  const savedEffects = localStorage.getItem(STORAGE_KEYS.effects)

  // 解析特效设置（支持 boolean 和 string 类型）
  let effectsEnabled = savedEffects === null
    ? DEFAULT_THEME_SETTINGS.effectsEnabled
    : savedEffects === 'true' || savedEffects === 'false'
      ? savedEffects === 'true'
      : JSON.parse(savedEffects)

  // 如果系统偏好减少动画，则强制禁用特效
  if (getPrefersReducedMotion()) {
    effectsEnabled = false
  }

  // 确定最终主题模式
  const mode: ThemeMode = savedMode || DEFAULT_THEME_SETTINGS.mode
  const accent: ThemeAccent = savedAccent || DEFAULT_THEME_SETTINGS.accent

  // 应用 CSS class
  root.classList.remove('cyberpunk', 'glassmorphism', 'light', 'dark')
  if (mode === 'system') {
    if (getPrefersDark()) {
      root.classList.add('cyberpunk', 'dark')
    } else {
      root.classList.add('glassmorphism', 'light')
    }
  } else {
    root.classList.add(mode)
    // 同时添加 light 或 dark 类，用于滚动条等样式区分
    root.classList.add(mode === 'cyberpunk' ? 'dark' : 'light')
  }

  // 应用强调色
  root.classList.remove('accent-cyan', 'accent-magenta', 'accent-purple', 'accent-gold')
  root.classList.add(`accent-${accent}`)

  // 应用特效
  root.classList.toggle('reduce-motion', !effectsEnabled)
})()

// 响应式状态
const mode = ref<ThemeMode>(
  (localStorage.getItem(STORAGE_KEYS.mode) as ThemeMode) || DEFAULT_THEME_SETTINGS.mode
)

const accent = ref<ThemeAccent>(
  (localStorage.getItem(STORAGE_KEYS.accent) as ThemeAccent) || DEFAULT_THEME_SETTINGS.accent
)

const effectsEnabled = ref<boolean>(
  localStorage.getItem(STORAGE_KEYS.effects) !== null
    ? localStorage.getItem(STORAGE_KEYS.effects) === 'true'
    : DEFAULT_THEME_SETTINGS.effectsEnabled
)

/**
 * 主题 Composable
 * 提供主题切换、强调色切换和特效开关功能
 */
export function useTheme() {
  const root = document.documentElement
  const mediaQueryDark = window.matchMedia('(prefers-color-scheme: dark)')
  const mediaQueryMotion = window.matchMedia('(prefers-reduced-motion: reduce)')

  // 是否暗色主题（cyberpunk 算暗色，glassmorphism 算亮色）
  const isDark = computed(() => {
    if (mode.value === 'system') {
      return getPrefersDark()
    }
    return mode.value === 'cyberpunk'
  })

  // 是否 cyberpunk 模式
  const isCyberpunk = computed(() => {
    if (mode.value === 'system') {
      return getPrefersDark()
    }
    return mode.value === 'cyberpunk'
  })

  // 是否 glassmorphism 模式
  const isGlassmorphism = computed(() => {
    if (mode.value === 'system') {
      return !getPrefersDark()
    }
    return mode.value === 'glassmorphism'
  })

  // 当前完整设置
  const settings = computed<ThemeSettings>(() => ({
    mode: mode.value,
    accent: accent.value,
    effectsEnabled: effectsEnabled.value
  }))

  /**
   * 应用主题模式到 html 元素
   */
  const applyMode = (newMode: ThemeMode) => {
    root.classList.remove('cyberpunk', 'glassmorphism', 'light', 'dark')
    if (newMode === 'system') {
      if (getPrefersDark()) {
        root.classList.add('cyberpunk', 'dark')
      } else {
        root.classList.add('glassmorphism', 'light')
      }
    } else {
      root.classList.add(newMode)
      // 同时添加 light 或 dark 类，用于滚动条等样式区分
      root.classList.add(newMode === 'cyberpunk' ? 'dark' : 'light')
    }
    mode.value = newMode
  }

  /**
   * 应用强调色到 html 元素
   */
  const applyAccent = (newAccent: ThemeAccent) => {
    root.classList.remove('accent-cyan', 'accent-magenta', 'accent-purple', 'accent-gold')
    root.classList.add(`accent-${newAccent}`)
    accent.value = newAccent
  }

  /**
   * 应用特效设置到 html 元素
   */
  const applyEffects = (enabled: boolean) => {
    root.classList.toggle('reduce-motion', !enabled)
    effectsEnabled.value = enabled
  }

  /**
   * 设置主题模式
   * @param newMode 主题模式：'cyberpunk' | 'glassmorphism' | 'system'
   */
  const setThemeMode = (newMode: ThemeMode) => {
    applyMode(newMode)
    localStorage.setItem(STORAGE_KEYS.mode, newMode)
  }

  /**
   * 设置强调色
   * @param newAccent 强调色：'cyan' | 'magenta' | 'purple' | 'gold'
   */
  const setThemeAccent = (newAccent: ThemeAccent) => {
    applyAccent(newAccent)
    localStorage.setItem(STORAGE_KEYS.accent, newAccent)
  }

  /**
   * 设置特效开关
   * @param enabled 是否启用特效
   */
  const setEffectsEnabled = (enabled: boolean) => {
    applyEffects(enabled)
    localStorage.setItem(STORAGE_KEYS.effects, String(enabled))
  }

  /**
   * 切换主题（cyberpunk <-> glassmorphism）
   * 兼容旧 API
   */
  const toggleTheme = () => {
    // system 模式下切换等价于在 cyberpunk 和 glassmorphism 之间切换
    const newMode: ThemeMode = isCyberpunk.value ? 'glassmorphism' : 'cyberpunk'
    setThemeMode(newMode)
  }

  /**
   * 系统主题变化处理
   */
  const handleSystemThemeChange = () => {
    if (mode.value === 'system') {
      root.classList.remove('cyberpunk', 'glassmorphism', 'light', 'dark')
      if (getPrefersDark()) {
        root.classList.add('cyberpunk', 'dark')
      } else {
        root.classList.add('glassmorphism', 'light')
      }
    }
  }

  /**
   * 减少动画偏好变化处理
   */
  const handleReducedMotionChange = (e: MediaQueryListEvent) => {
    if (e.matches) {
      // 系统要求减少动画，强制禁用特效
      applyEffects(false)
      localStorage.setItem(STORAGE_KEYS.effects, 'false')
    }
    // 如果之前是启用状态，不自动恢复（用户需要手动开启）
  }

  // 监听系统主题变化
  onMounted(() => {
    mediaQueryDark.addEventListener('change', handleSystemThemeChange)
    mediaQueryMotion.addEventListener('change', handleReducedMotionChange)
  })

  onUnmounted(() => {
    mediaQueryDark.removeEventListener('change', handleSystemThemeChange)
    mediaQueryMotion.removeEventListener('change', handleReducedMotionChange)
  })

  // 监听 prefers-reduced-motion 变化，如果当前是禁用状态则保持
  watch(mediaQueryMotion, (query) => {
    if (!query.matches && !effectsEnabled.value) {
      // 系统动画偏好改变且用户之前禁用了特效，保持禁用状态
    }
  })

  return {
    // 状态
    mode,
    accent,
    effectsEnabled,
    settings,

    // 计算属性
    isDark,
    isCyberpunk,
    isGlassmorphism,

    // 方法
    setThemeMode,
    setThemeAccent,
    setEffectsEnabled,
    toggleTheme
  }
}
