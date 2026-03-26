/**
 * 应用设置存储模块
 * 管理应用设置，存储在 ~/.config/opencode/settings.json
 */

// 设置类型定义
export interface AppSettings {
  // 工作路径
  workingPath?: string
  // 当前预设名称
  currentPreset?: string
  // 最近使用的预设
  lastUsedPreset?: string
  // 预设使用历史
  presetHistory?: string[]
}

// 默认设置
const DEFAULT_SETTINGS: AppSettings = {}

// 内存缓存
let settingsCache: AppSettings | null = null

/**
 * 动态导入 Tauri API
 */
async function getTauriInvoke() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke
  } catch {
    return null
  }
}

/**
 * 读取设置
 */
export async function readSettings(): Promise<AppSettings> {
  // 返回缓存
  if (settingsCache) {
    return { ...settingsCache }
  }

  try {
    const invoke = await getTauriInvoke()
    if (!invoke) {
      return { ...DEFAULT_SETTINGS }
    }

    const content = await invoke<string>('read_settings')
    if (content) {
      settingsCache = JSON.parse(content)
      return settingsCache ? { ...settingsCache } : { ...DEFAULT_SETTINGS }
    }
  } catch (error) {
    console.log('读取设置失败，使用默认值:', error)
  }

  return { ...DEFAULT_SETTINGS }
}

/**
 * 保存设置
 */
export async function writeSettings(settings: AppSettings): Promise<void> {
  const invoke = await getTauriInvoke()
  if (!invoke) {
    console.warn('无法保存设置：Tauri API 不可用')
    return
  }

  try {
    await invoke('write_settings', { content: JSON.stringify(settings, null, 2) })
    settingsCache = settings
  } catch (error) {
    console.error('保存设置失败:', error)
  }
}

/**
 * 获取工作路径
 */
export async function getWorkingPath(): Promise<string | undefined> {
  const settings = await readSettings()
  return settings.workingPath
}

/**
 * 设置工作路径
 */
export async function setWorkingPath(path: string): Promise<void> {
  const settings = await readSettings()
  settings.workingPath = path
  await writeSettings(settings)
}

/**
 * 获取当前预设名称
 */
export async function getCurrentPreset(): Promise<string | undefined> {
  const settings = await readSettings()
  return settings.currentPreset
}

/**
 * 设置当前预设名称
 */
export async function setCurrentPreset(name: string | undefined): Promise<void> {
  const settings = await readSettings()
  if (name === undefined) {
    delete settings.currentPreset
  } else {
    settings.currentPreset = name
  }
  await writeSettings(settings)
}

/**
 * 获取最近使用的预设
 */
export async function getLastUsedPreset(): Promise<string | undefined> {
  const settings = await readSettings()
  return settings.lastUsedPreset
}

/**
 * 设置最近使用的预设
 */
export async function setLastUsedPreset(name: string): Promise<void> {
  const settings = await readSettings()
  settings.lastUsedPreset = name
  await writeSettings(settings)
}

/**
 * 获取预设使用历史
 * @param limit 返回数量限制，默认 5 个
 */
export async function getPresetHistory(limit: number = 5): Promise<string[]> {
  const settings = await readSettings()
  return (settings.presetHistory || []).slice(0, limit)
}

/**
 * 记录预设使用历史
 */
export async function recordPresetUsage(name: string): Promise<void> {
  const settings = await readSettings()
  let history = settings.presetHistory || []
  
  // 移除旧记录（如果存在）
  history = history.filter(n => n !== name)
  
  // 添加到最前面
  history.unshift(name)
  
  // 限制历史记录数量（最多保留 20 个）
  history = history.slice(0, 20)
  
  settings.presetHistory = history
  await writeSettings(settings)
}

/**
 * 初始化设置存储
 * 在应用启动时调用
 */
export async function initSettings(): Promise<void> {
  await readSettings()
}
