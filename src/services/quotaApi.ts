/**
 * 额度查询 API
 * 封装 Tauri invoke 调用
 */
import { invoke } from '@tauri-apps/api/core'
import type { ProviderQuota, ZhipuUsageDetails } from '@/types/quota'
import { getAvailableProviderIds } from '@/services/opencodeModels'

export const quotaApi = {
  /**
   * 获取所有已接入供应商的额度信息
   * 先通过 getAvailableProviderIds 获取完整供应商列表（与模型管理页面同一数据源），
   * 再将 ID 列表传给 Rust 端进行额度查询
   */
  async fetchAllProviderQuotas(): Promise<ProviderQuota[]> {
    // 获取可用供应商 ID 列表（统一数据源：opencode.json + antigravity + auth.json）
    const providerIds = await getAvailableProviderIds()
    if (providerIds.length === 0) return []

    const result = await invoke<string>('fetch_all_provider_quotas', { providerIds })
    return JSON.parse(result) as ProviderQuota[]
  },

  /**
   * 查询智谱供应商的模型用量和工具用量详情（近 7 天）
   */
  async fetchZhipuUsageDetails(providerId: string): Promise<ZhipuUsageDetails> {
    const result = await invoke<string>('fetch_zhipu_usage_details', { providerId })
    return JSON.parse(result) as ZhipuUsageDetails
  }
}
