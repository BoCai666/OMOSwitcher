// 热重载配置 composable
// 管理模型热重载开关和端口配置

import { ref, watch } from 'vue'
import { getHotReloadConfig, setHotReloadConfig } from '@/services/settingsStore'

export function useHotReloadConfig() {
  // 热重载配置
  const hotReloadEnabled = ref(false)
  const hotReloadPort = ref(4096)

  // 加载热重载配置
  async function loadHotReloadConfig() {
    const config = await getHotReloadConfig()
    hotReloadEnabled.value = config.enabled
    hotReloadPort.value = config.port
  }

  // 保存热重载配置
  async function saveHotReloadConfig() {
    await setHotReloadConfig({
      enabled: hotReloadEnabled.value,
      port: hotReloadPort.value
    })
  }

  // 监听热重载开关变化，自动保存
  watch(hotReloadEnabled, () => {
    saveHotReloadConfig()
  })

  return {
    hotReloadEnabled,
    hotReloadPort,
    loadHotReloadConfig,
    saveHotReloadConfig
  }
}
