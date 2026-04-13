/**
 * 应用设置存储模块
 * 管理应用设置，存储在 ~/.config/omoswitcher/settings.json
 */

import { invoke } from '@tauri-apps/api/core'

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
  // 代理配置
  proxy: {
    // 是否启用代理（监控代理）
    enabled: boolean
  }
  // Monitor 服务端口配置
  monitorPorts: {
    // Web API 端口（Tauri 前端调用）
    web: number
    // 代理服务端口（拦截 LLM API）
    proxy: number
  }
  // 热重载配置
  hotReload: {
    // 是否启用热重载
    enabled: boolean
    // OpenCode Server 端口
    port: number
  }
}

// 默认设置
const DEFAULT_SETTINGS: AppSettings = {
  proxy: {
    enabled: false
  },
  monitorPorts: {
    web: 7100,
    proxy: 7101
  },
  hotReload: {
    enabled: false,
    port: 4096
  }
}

// 内存缓存
let settingsCache: AppSettings | null = null

/**
 * 读取设置
 */
export async function readSettings(): Promise<AppSettings> {
  // 返回缓存
  if (settingsCache) {
    return { ...settingsCache }
  }

  try {
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
 * 在应用启动时调用，确保设置文件存在并包含默认值
 */
export async function initSettings(): Promise<void> {
  try {
    // 尝试读取现有设置
    const content = await invoke<string>('read_settings')
    if (content) {
      // 文件存在，解析并缓存
      const parsed = JSON.parse(content) as Partial<AppSettings>
      // 合并默认值，确保 monitorPorts 和 hotReload 存在
      settingsCache = {
        ...DEFAULT_SETTINGS,
        ...parsed,
        monitorPorts: parsed.monitorPorts || DEFAULT_SETTINGS.monitorPorts,
        hotReload: parsed.hotReload || DEFAULT_SETTINGS.hotReload
      }
      // 如果原来没有 monitorPorts 或 hotReload，写入更新后的设置
      if (!parsed.monitorPorts || !parsed.hotReload) {
        await invoke('write_settings', { content: JSON.stringify(settingsCache, null, 2) })
      }
    } else {
      // 文件不存在，创建默认设置
      settingsCache = { ...DEFAULT_SETTINGS }
      await invoke('write_settings', { content: JSON.stringify(settingsCache, null, 2) })
    }
  } catch (error) {
    console.log('初始化设置失败:', error)
    settingsCache = { ...DEFAULT_SETTINGS }
  }
}

// ==================== 代理配置相关函数 ====================

/**
 * 获取代理配置
 */
export async function getProxyConfig(): Promise<{ enabled: boolean }> {
  const settings = await readSettings()
  return { enabled: settings.proxy?.enabled ?? false }
}

/**
 * 设置代理配置
 */
export async function setProxyConfig(config: { enabled: boolean }): Promise<void> {
  const settings = await readSettings()
  settings.proxy = { enabled: config.enabled }
  await writeSettings(settings)
}

/**
 * 检查代理是否启用
 */
export async function isProxyEnabled(): Promise<boolean> {
  const settings = await readSettings()
  return settings.proxy?.enabled ?? false
}

/**
 * 设置代理启用状态
 */
export async function setProxyEnabled(enabled: boolean): Promise<void> {
  const settings = await readSettings()
  if (!settings.proxy) {
    settings.proxy = { enabled }
  } else {
    settings.proxy.enabled = enabled
  }
  await writeSettings(settings)
}

// ==================== Monitor 端口配置相关函数 ====================

/**
 * 获取 Monitor 端口配置
 * 从 ~/.config/omoswitcher/monitor/config.jsonc 读取
 */
export async function getMonitorPorts(): Promise<{ web: number; proxy: number }> {
  try {
    const [web, proxy] = await invoke<[number, number]>('get_monitor_ports_config')
    return { web, proxy }
  } catch (error) {
    console.error('获取 Monitor 端口配置失败:', error)
    return { web: 7100, proxy: 7101 }
  }
}

/**
 * 设置 Monitor 端口配置
 */
export async function setMonitorPorts(ports: { web: number; proxy: number }): Promise<void> {
  const settings = await readSettings()
  settings.monitorPorts = ports
  await writeSettings(settings)
}

/**
 * 获取 Web API 端口
 */
export async function getMonitorWebPort(): Promise<number> {
  const ports = await getMonitorPorts()
  return ports.web
}

/**
 * 获取 Proxy 端口
 */
export async function getMonitorProxyPort(): Promise<number> {
  const ports = await getMonitorPorts()
  return ports.proxy
}

// ==================== 证书相关函数 ====================

/**
 * 检查 CA 证书是否存在（通过 Tauri 后端调用 Monitor API）
 */
export async function checkCaCertExists(): Promise<boolean> {
  try {
    return await invoke<boolean>('check_ca_cert_exists')
  } catch (error) {
    console.error('检查证书存在失败:', error)
    return false
  }
}

// ==================== 热重载配置相关函数 ====================

/**
 * 获取热重载配置
 */
export async function getHotReloadConfig(): Promise<{ enabled: boolean; port: number }> {
  const settings = await readSettings()
  return settings.hotReload ?? { enabled: false, port: 4096 }
}

/**
 * 设置热重载配置
 */
export async function setHotReloadConfig(config: { enabled: boolean; port: number }): Promise<void> {
  const settings = await readSettings()
  settings.hotReload = config
  await writeSettings(settings)
}
