/**
 * 运行时配置
 * 
 * 端口优先级：环境变量 > 配置文件 > 默认值
 * 初始值使用环境变量/默认值，启动时会从配置文件更新
 */
export const config = {
  // 现有配置
  maxMemoryRecords: parseInt(process.env.MAX_RECORDS || '1000'),
  autoCleanup: process.env.AUTO_CLEANUP !== 'false',
  port: parseInt(process.env.PORT || '7100'),
  proxyPort: parseInt(process.env.PROXY_PORT || '7101'),
  
  // 新增存储配置
  storage: {
    type: (process.env.STORAGE_TYPE || 'memory') as 'memory' | 'sqlite', // 'memory' | 'sqlite' (memory 用于 pkg 打包)
    sqlite: {
      path: process.env.SQLITE_PATH || './data/opencode.db',
      walMode: true, // 启用 WAL 模式提高并发性能
      busyTimeout: 5000, // 5秒超时
    }
  },
  
  // 数据保留配置（保留最近90天）
  retention: {
    enabled: true, // 启用自动清理
    days: 90, // 保留90天
    archiveBeforeDelete: true // 删除前归档到压缩文件
  },

  // 企业代理 CA 证书路径（用于信任企业代理的自签名证书）
  enterpriseCaCertPath: process.env.ENTERPRISE_CA_CERT_PATH || '',
};

/**
 * 从配置管理器更新端口配置
 * 优先级：环境变量 > 配置文件 > 当前值（默认值）
 * 
 * @param configManager 配置管理器实例
 */
export function updatePortsFromConfig(configManager: { get: (key: string) => unknown }): void {
  // 仅当环境变量未设置时，才使用配置文件的值
  if (!process.env.PORT) {
    const webPort = configManager.get('ports.web') as number | undefined;
    if (webPort !== undefined) {
      config.port = webPort;
    }
  }
  
  if (!process.env.PROXY_PORT) {
    const proxyPort = configManager.get('ports.proxy') as number | undefined;
    if (proxyPort !== undefined) {
      config.proxyPort = proxyPort;
    }
  }
  
  console.log(`[Config] Ports: web=${config.port}, proxy=${config.proxyPort}`);
}
