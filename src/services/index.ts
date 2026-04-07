// 服务导出
// configReader 通过 Tauri 命令读写配置文件
export {
  readConfig,
  configExists,
  writeConfig,
  deleteConfig
} from './configReader'

// 预设存储服务
export {
  listPresets,
  savePreset,
  loadPreset,
  deletePreset,
  presetExists,
  clearPresets,
  getCurrentPreset,
  setCurrentPreset,
  getLastUsedPreset,
  setLastUsedPreset,
  getRecentPresets,
  recordPresetUsage,
  switchPreset,
  initPresetStore
} from './presetStore'

// 模型存储服务
export {
  listModels,
  addModel,
  updateModel,
  deleteModel,
  getModelById,
  resetModels,
  validateModelId,
  parseProvider,
  modelExists,
  getDefaultModels,
  groupModelsByProvider,
  getProviders
} from './modelStore'

// 应用设置服务
export {
  readSettings,
  writeSettings,
  getWorkingPath,
  setWorkingPath,
  initSettings,
  getMonitorPorts,
  setMonitorPorts,
  getMonitorWebPort,
  getMonitorProxyPort
} from './settingsStore'

// OpenCode Server 软重载服务
export {
  requestSoftReload,
  checkPendingSoftReload,
  type SoftReloadState,
  type SoftReloadResult
} from './opencodeApi'

// OpenCode 模型注册表服务
export {
  readModelsRegistry,
  getAvailableProviderIds,
  getProvidersWithAvailability,
  clearRegistryCache,
  type ProviderWithAvailability
} from './opencodeModels'

export type {
  RegistryProvider,
  RegistryModel
} from '@/types/config'
