// OpenCode 启动功能 composable
// 提供启动 opencode 命令行工具的功能

import { ref } from 'vue'

export function useOpenCode() {
  // 是否正在启动
  const isLaunching = ref(false)
  
  // 错误信息
  const error = ref<string | null>(null)

  /**
   * 启动 OpenCode
   * @param workingPath 工作目录路径，为空则使用用户主目录
   * @param proxyEnabled 是否启用监控代理
   * @param proxyCaCertPath 代理 CA 证书路径（可选）
   * 调用 Tauri 命令在后台启动 opencode
   */
  const launchOpenCode = async (
    workingPath: string = '',
    proxyEnabled: boolean = false,
    proxyCaCertPath: string = ''
  ) => {
    isLaunching.value = true
    error.value = null
    
    try {
      // 动态导入 Tauri API（仅在 Tauri 环境中可用）
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('launch_opencode', { 
        workingPath,
        proxyEnabled,
        proxyCaCertPath
      })
      // 启动成功，不显示消息，让用户继续操作
    } catch (e) {
      // 命令不存在或执行失败时显示错误
      error.value = '启动 opencode 失败: ' + String(e)
    } finally {
      isLaunching.value = false
    }
  }

  return {
    launchOpenCode,
    isLaunching,
    error
  }
}
