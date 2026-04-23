// 代理配置 composable
// 管理代理开关、证书状态检查和轮询

import { ref, watch, onUnmounted } from 'vue'
import { getProxyConfig, setProxyConfig, checkCaCertExists, getMonitorPorts } from '@/services/settingsStore'

export function useProxyConfig() {
  // 代理配置
  const proxyEnabled = ref(false)

  // 证书是否存在（null 表示未检查）
  const certExists = ref<boolean | null>(null)

  // 代理端口
  const proxyPort = ref(7101) // 默认端口

  // 证书检查定时器
  let certCheckTimer: ReturnType<typeof setInterval> | null = null

  // 检查证书状态
  async function checkCertStatus() {
    const exists = await checkCaCertExists()
    certExists.value = exists

    // 如果证书已存在，停止轮询
    if (exists && certCheckTimer) {
      clearInterval(certCheckTimer)
      certCheckTimer = null
    }
  }

  // 启动证书状态轮询
  function startCertPolling() {
    // 如果已经在轮询，不重复启动
    if (certCheckTimer) return

    // 每 2 秒检查一次证书状态
    certCheckTimer = setInterval(async () => {
      await checkCertStatus()
    }, 2000)
  }

  // 停止证书状态轮询
  function stopCertPolling() {
    if (certCheckTimer) {
      clearInterval(certCheckTimer)
      certCheckTimer = null
    }
  }

  // 加载代理配置
  async function loadProxyConfig() {
    const config = await getProxyConfig()
    proxyEnabled.value = config.enabled
    // 检查证书是否存在
    await checkCertStatus()
    // 获取代理端口配置
    const ports = await getMonitorPorts()
    proxyPort.value = ports.proxy

    // 如果启用了代理但证书不存在，启动轮询等待证书生成
    if (proxyEnabled.value && certExists.value === false) {
      startCertPolling()
    }
  }

  // 保存代理配置
  async function saveProxyConfig() {
    await setProxyConfig({
      enabled: proxyEnabled.value
    })
  }

  // 监听代理开关变化，自动保存并管理证书轮询
  watch(proxyEnabled, (enabled) => {
    // 切换时立即持久化
    saveProxyConfig()
    // 启用代理时，检查证书状态
    if (enabled && certExists.value === false) {
      startCertPolling()
    } else if (!enabled) {
      // 关闭代理时，停止轮询
      stopCertPolling()
    }
  })

  // 组件卸载时清理定时器
  onUnmounted(() => {
    stopCertPolling()
  })

  return {
    proxyEnabled,
    certExists,
    proxyPort,
    loadProxyConfig,
    saveProxyConfig,
    checkCertStatus,
    startCertPolling,
    stopCertPolling
  }
}
