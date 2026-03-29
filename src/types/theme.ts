/** 主题模式 */
export type ThemeMode = 'cyberpunk' | 'glassmorphism' | 'system'

/** 主题强调色 */
export type ThemeAccent = 'cyan' | 'magenta' | 'purple' | 'gold'

/** 主题设置 */
export interface ThemeSettings {
  /** 主题模式 */
  mode: ThemeMode
  /** 强调色预设 */
  accent: ThemeAccent
  /** 是否启用特效 */
  effectsEnabled: boolean
}

/** 默认主题设置 */
export const DEFAULT_THEME_SETTINGS: ThemeSettings = {
  mode: 'system',
  accent: 'cyan',
  effectsEnabled: true
}
