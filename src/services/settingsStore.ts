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
  // 关闭行为：ask=每次询问, minimize=最小化到托盘, exit=直接退出
  closeBehavior?: 'ask' | 'minimize' | 'exit'
  // 关闭确认弹窗今日不显示的日期（YYYY-MM-DD）
  closeConfirmDismissedDate?: string
  // 上次关闭时选择的操作（配合 closeConfirmDismissedDate 使用）
  lastCloseAction?: 'minimize' | 'exit'
  // 热重载配置
  hotReload: {
    // 是否启用热重载
    enabled: boolean
    // OpenCode Server 端口
    port: number
  }
  // OpenCode Go 额度查询配置（网页抓取方式需要的认证参数）
  openCodeGo?: {
    // 用户/订阅 ID
    id?: string
    // 浏览器 Cookie（登录态）
    cookie?: string
    // 额度查询页面 URL（可自定义，默认 https://opencode.ai/auth）
    usageUrl?: string
  }
  // 待显示的更新日志（更新完成后重启时显示）
  pendingChangelog?: {
    version: string
    date?: string
    body?: string
  }
}

// 默认设置
const DEFAULT_SETTINGS: AppSettings = {
  closeBehavior: 'ask',
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

// ==================== 关闭行为相关函数 ====================

/**
 * 获取今日日期字符串（YYYY-MM-DD）
 */
function getTodayString(): string {
  return new Date().toISOString().split('T')[0]
}

/**
 * 获取关闭行为设置
 * 优先返回永久设置；若永久设置为 ask，则检查"今日不显示"
 */
export async function getCloseBehavior(): Promise<'ask' | 'minimize' | 'exit'> {
  const settings = await readSettings()

  // 永久设置（非 ask）优先
  if (settings.closeBehavior && settings.closeBehavior !== 'ask') {
    return settings.closeBehavior
  }

  // 检查是否今日已选择"今日不显示"
  if (settings.closeConfirmDismissedDate && settings.lastCloseAction) {
    const today = getTodayString()
    if (settings.closeConfirmDismissedDate === today) {
      return settings.lastCloseAction
    }
  }

  return 'ask'
}

/**
 * 设置关闭行为（永久设置）
 */
export async function setCloseBehavior(behavior: 'ask' | 'minimize' | 'exit'): Promise<void> {
  const settings = await readSettings()
  settings.closeBehavior = behavior
  await writeSettings(settings)
}

/**
 * 设置"今日不显示"关闭确认弹窗
 * @param action 用户本次选择的操作
 */
export async function setCloseConfirmDismissed(action: 'minimize' | 'exit'): Promise<void> {
  const settings = await readSettings()
  settings.closeConfirmDismissedDate = getTodayString()
  settings.lastCloseAction = action
  await writeSettings(settings)
}

/**
 * 清除"今日不显示"设置
 */
export async function clearCloseConfirmDismissed(): Promise<void> {
  const settings = await readSettings()
  delete settings.closeConfirmDismissedDate
  delete settings.lastCloseAction
  await writeSettings(settings)
}

// ==================== 更新日志相关函数 ====================

/**
 * 保存待显示的更新日志
 */
export async function savePendingChangelog(changelog: { version: string; date?: string; body?: string }): Promise<void> {
  const settings = await readSettings()
  settings.pendingChangelog = changelog
  await writeSettings(settings)
}

/**
 * 获取待显示的更新日志
 */
export async function getPendingChangelog(): Promise<{ version: string; date?: string; body?: string } | undefined> {
  const settings = await readSettings()
  return settings.pendingChangelog
}

/**
 * 清除待显示的更新日志
 */
export async function clearPendingChangelog(): Promise<void> {
  const settings = await readSettings()
  delete settings.pendingChangelog
  await writeSettings(settings)
}

// ==================== OpenCode Go 额度查询配置相关函数 ====================

/**
 * 获取 OpenCode Go 额度查询配置
 */
export async function getOpenCodeGoConfig(): Promise<{ id?: string; cookie?: string; usageUrl?: string }> {
  const settings = await readSettings()
  return {
    id: settings.openCodeGo?.id,
    cookie: settings.openCodeGo?.cookie,
    usageUrl: settings.openCodeGo?.usageUrl
  }
}

/**
 * 设置 OpenCode Go 额度查询配置
 */
export async function setOpenCodeGoConfig(config: { id?: string; cookie?: string; usageUrl?: string }): Promise<void> {
  const settings = await readSettings()
  settings.openCodeGo = { ...settings.openCodeGo, ...config }
  await writeSettings(settings)
}
