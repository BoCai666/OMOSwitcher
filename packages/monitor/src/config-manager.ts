import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { fileURLToPath } from 'url';
import logger from './logger.js';

// pkg 环境中 __dirname 和 __filename 已被注入，使用不同的变量名
const pkgFilename = typeof __filename !== 'undefined' 
  ? __filename 
  : fileURLToPath(import.meta.url);
const pkgDirname = typeof __dirname !== 'undefined' 
  ? __dirname 
  : path.dirname(pkgFilename);

// 检测是否在 pkg 打包环境中运行
// pkg 会设置 process.pkg 和特定的入口点路径
const isPkgEnvironment = typeof (process as any).pkg !== 'undefined' || 
                         pkgFilename.includes('snapshot') ||
                         pkgDirname.includes('snapshot');

// 用户配置目录
const USER_CONFIG_DIR = path.join(os.homedir(), '.config', 'omoswitcher');
const USER_CONFIG_FILE = path.join(USER_CONFIG_DIR, 'monitor-config.jsonc');

// 配置变更回调类型
export type ConfigChangeCallback = (key: string, value: any, oldValue: any) => void;

// 默认配置
const DEFAULT_CONFIG = {
  domains: [
    { domain: "api.openai.com", provider: "OpenAI", enabled: true },
    { domain: "api.anthropic.com", provider: "Anthropic", enabled: true },
    { domain: "api.groq.com", provider: "Groq", enabled: true },
    { domain: "openrouter.ai", provider: "OpenRouter", enabled: true },
    { domain: "api.kimi.com", provider: "Kimi", enabled: true }
  ],
  pricing: {
    matchStrategy: "prefix",
    models: [
      { model: "gpt-4", input: 30, output: 60 },
      { model: "gpt-4-turbo", input: 10, output: 30 },
      { model: "gpt-3.5-turbo", input: 0.5, output: 1.5 },
      { model: "gpt-4o", input: 5, output: 15 },
      { model: "gpt-4o-mini", input: 0.15, output: 0.6 },
      { model: "claude-3-opus", input: 15, output: 75 },
      { model: "claude-3-sonnet", input: 3, output: 15 },
      { model: "kimi", input: 1, output: 3 }
    ]
  },
  ports: { web: 3000, proxy: 8080 },
  monitor: { refreshInterval: 3000, maxRequestsInList: 100 },
  nebula: { theme: "blue", starCount: 150 }
};

/**
 * ConfigManager - 配置管理器类
 * 支持 JSONC 文件加载、热更新、路径访问和监听
 */
export class ConfigManager {
  private configPath: string;
  private config: any = null;
  private watcher: fs.FSWatcher | null = null;
  private callbacks: Set<ConfigChangeCallback> = new Set();
  private isReloading = false;
  private lastMtime = 0;

  /**
   * 构造函数
   * @param configPath 配置文件路径（可选，默认为用户配置目录）
   */
  constructor(configPath?: string) {
    if (configPath) {
      this.configPath = path.resolve(configPath);
    } else {
      // pkg 打包环境：只使用用户配置目录
      if (isPkgEnvironment) {
        this.configPath = USER_CONFIG_FILE;
        logger.debug(`[ConfigManager] pkg 环境，使用用户配置目录: ${this.configPath}`);
      } else {
        // 开发环境：按优先级查找
        const possiblePaths = [
          USER_CONFIG_FILE,                                    // 用户配置目录
          path.resolve(pkgDirname, '..', 'config.jsonc'),      // 项目根目录
          path.resolve(process.cwd(), 'config.jsonc'),         // 当前工作目录
        ];
        
        // 找到第一个存在的配置文件
        const existingPath = possiblePaths.find(p => fs.existsSync(p));
        
        if (existingPath) {
          this.configPath = existingPath;
        } else {
          // 没有找到配置文件，使用用户配置目录（会自动创建）
          this.configPath = USER_CONFIG_FILE;
        }
        
        logger.debug(`[ConfigManager] 开发环境，配置文件路径: ${this.configPath}`);
      }
    }
  }

  /**
   * 加载配置文件
   */
  async load(): Promise<void> {
    try {
      if (fs.existsSync(this.configPath)) {
        const content = await fs.promises.readFile(this.configPath, 'utf-8');
        this.config = this.parseJSONC(content);
        logger.info(`[ConfigManager] 已加载配置文件: ${this.configPath}`);
      } else {
        // 文件不存在，使用默认配置
        this.config = JSON.parse(JSON.stringify(DEFAULT_CONFIG));
        // 确保目录存在后写入默认配置文件
        const configDir = path.dirname(this.configPath);
        if (!fs.existsSync(configDir)) {
          await fs.promises.mkdir(configDir, { recursive: true });
        }
        await this.writeConfigFile(this.config);
        logger.info(`[ConfigManager] 已创建默认配置文件: ${this.configPath}`);
      }
      
      // 启动文件监听
      this.startWatching();
    } catch (error) {
      logger.error('[ConfigManager] 加载配置失败:', error);
      // 加载失败时使用默认配置
      this.config = JSON.parse(JSON.stringify(DEFAULT_CONFIG));
    }
  }

  /**
   * 解析 JSONC 格式（支持注释）
   * @param content JSONC 字符串
   */
  private parseJSONC(content: string): any {
    try {
      // 简单的 JSONC 解析：移除注释
      const json = content
        // 移除单行注释
        .replace(/\/\/.*$/gm, '')
        // 移除多行注释
        .replace(/\/\*[\s\S]*?\*\//g, '')
        // 移除尾随逗号
        .replace(/,(\s*[}\]])/g, '$1');
      
      return JSON.parse(json);
    } catch (error) {
      throw new Error(`Failed to parse config: ${(error as Error).message}`);
    }
  }

  /**
   * 写入配置文件（保留原有格式和注释）
   * @param config 配置对象
   */
  private async writeConfigFile(config: any): Promise<void> {
    try {
      // 如果文件存在，尝试保留注释
      let content: string;
      if (fs.existsSync(this.configPath)) {
        content = await fs.promises.readFile(this.configPath, 'utf-8');
        content = this.mergeConfigWithComments(content, config);
      } else {
        // 生成带注释的默认配置
        content = this.generateDefaultConfigWithComments();
      }
      
      await fs.promises.writeFile(this.configPath, content, 'utf-8');
    } catch (error) {
      throw new Error(`Failed to write config: ${(error as Error).message}`);
    }
  }

  /**
   * 合并配置并保留注释
   * @param content 原文件内容
   * @param config 新配置对象
   */
  private mergeConfigWithComments(content: string, config: any): string {
    // 简单实现：直接格式化输出，后续可优化为智能合并
    return JSON.stringify(config, null, 2);
  }

  /**
   * 生成带注释的默认配置
   */
  private generateDefaultConfigWithComments(): string {
    return `// OpenCode Monitor 配置文件
// 支持 JSONC 格式（带注释的 JSON）

{
  // 域名配置：需要监控的 API 域名列表
  "domains": [
    {
      "domain": "api.openai.com",
      "provider": "OpenAI",
      "enabled": true
    },
    {
      "domain": "api.anthropic.com",
      "provider": "Anthropic",
      "enabled": true
    },
    {
      "domain": "api.groq.com",
      "provider": "Groq",
      "enabled": true
    },
    {
      "domain": "openrouter.ai",
      "provider": "OpenRouter",
      "enabled": true
    },
    {
      "domain": "api.kimi.com",
      "provider": "Kimi",
      "enabled": true
    }
  ],

  // 定价配置：各模型的输入输出定价（美元 per 1M tokens）
  "pricing": {
    "matchStrategy": "prefix",
    "models": [
      {
        "model": "gpt-4",
        "input": 30,
        "output": 60
      },
      {
        "model": "gpt-4-turbo",
        "input": 10,
        "output": 30
      },
      {
        "model": "gpt-3.5-turbo",
        "input": 0.5,
        "output": 1.5
      },
      {
        "model": "gpt-4o",
        "input": 5,
        "output": 15
      },
      {
        "model": "gpt-4o-mini",
        "input": 0.15,
        "output": 0.6
      },
      {
        "model": "claude-3-opus",
        "input": 15,
        "output": 75
      },
      {
        "model": "claude-3-sonnet",
        "input": 3,
        "output": 15
      },
      {
        "model": "kimi",
        "input": 1,
        "output": 3
      }
    ]
  },

  // 端口配置：各服务监听的端口
  "ports": {
    "web": 3000,
    "proxy": 8080
  },

  // 监控配置
  "monitor": {
    "refreshInterval": 3000,
    "maxRequestsInList": 100
  },

  // 星空特效配置
  "nebula": {
    "theme": "blue",
    "starCount": 150
  }
}
`;
  }

  /**
   * 启动文件监听
   */
  private startWatching(): void {
    if (this.watcher) {
      return;
    }

    // 文件不存在时跳过监听
    if (!fs.existsSync(this.configPath)) {
      logger.debug('[ConfigManager] 配置文件不存在，跳过文件监听');
      return;
    }

    try {
      this.watcher = fs.watch(this.configPath, (eventType) => {
        if (eventType === 'change' && !this.isReloading) {
          this.handleConfigChange();
        }
      });
      logger.debug('[ConfigManager] 已启动配置文件监听');
    } catch (error) {
      // 监听失败不影响正常运行
      logger.warn('[ConfigManager] 无法监听配置文件:', error);
    }
  }

  /**
   * 处理配置文件变更
   */
  private async handleConfigChange(): Promise<void> {
    if (this.isReloading) {
      return;
    }

    try {
      // 检查文件修改时间，避免重复触发
      const stats = await fs.promises.stat(this.configPath);
      if (stats.mtimeMs <= this.lastMtime) {
        return;
      }
      this.lastMtime = stats.mtimeMs;

      this.isReloading = true;
      
      // 保存旧配置
      const oldConfig = JSON.parse(JSON.stringify(this.config));
      
      // 重新加载
      const content = await fs.promises.readFile(this.configPath, 'utf-8');
      const newConfig = this.parseJSONC(content);
      
      // 更新配置
      this.config = newConfig;
      
      // 找出变更并触发回调
      this.notifyChanges(oldConfig, newConfig);
      
    } catch (error) {
      logger.error('[ConfigManager] 重新加载配置失败:', error);
    } finally {
      this.isReloading = false;
    }
  }

  /**
   * 通知配置变更
   */
  private notifyChanges(oldConfig: any, newConfig: any, prefix: string = ''): void {
    const allKeys = new Set([
      ...Object.keys(oldConfig || {}),
      ...Object.keys(newConfig || {})
    ]);

    for (const key of allKeys) {
      const fullKey = prefix ? `${prefix}.${key}` : key;
      const oldValue = oldConfig?.[key];
      const newValue = newConfig?.[key];
      
      if (JSON.stringify(oldValue) !== JSON.stringify(newValue)) {
        // 如果是对象，递归检查子项
        if (typeof oldValue === 'object' && oldValue !== null && 
            typeof newValue === 'object' && newValue !== null &&
            !Array.isArray(oldValue) && !Array.isArray(newValue)) {
          this.notifyChanges(oldValue, newValue, fullKey);
        } else {
          // 触发回调
          for (const callback of this.callbacks) {
            try {
              callback(fullKey, newValue, oldValue);
            } catch (error) {
              logger.error('[ConfigManager] 配置变更回调错误:', error);
            }
          }
        }
      }
    }
  }

  /**
   * 获取配置值
   * @param key 配置键，支持点号路径和数组索引（如 'pricing.models[0].model'）
   */
  get<T = any>(key: string): T | undefined {
    if (!this.config) {
      return undefined;
    }

    // 解析路径，支持数组索引如 [0]
    const pathParts = this.parsePath(key);
    
    let current: any = this.config;
    for (const part of pathParts) {
      if (current === null || current === undefined) {
        return undefined;
      }
      current = current[part];
    }
    
    return current;
  }

  /**
   * 解析路径字符串为路径数组
   */
  private parsePath(key: string): (string | number)[] {
    // 处理数组索引格式，如 models[0] -> models.0
    const normalized = key.replace(/\[(\d+)\]/g, '.$1');
    return normalized.split('.').map(part => {
      const num = parseInt(part, 10);
      return isNaN(num) ? part : num;
    });
  }

  /**
   * 设置配置值
   * @param key 配置键
   * @param value 配置值
   */
  async set(key: string, value: any): Promise<void> {
    if (!this.config) {
      await this.load();
    }

    const oldValue = this.get(key);
    if (JSON.stringify(oldValue) === JSON.stringify(value)) {
      return; // 值未变化，无需操作
    }

    // 更新内存中的配置
    const pathParts = this.parsePath(key);
    let current: any = this.config;
    
    // 遍历到倒数第二个部分
    for (let i = 0; i < pathParts.length - 1; i++) {
      const part = pathParts[i];
      const nextPart = pathParts[i + 1];
      
      if (current[part] === undefined) {
        // 如果下一个部分是数字，创建数组，否则创建对象
        current[part] = typeof nextPart === 'number' ? [] : {};
      }
      current = current[part];
    }
    
    // 设置值
    const lastPart = pathParts[pathParts.length - 1];
    current[lastPart] = value;

    // 写入文件
    await this.writeConfigFile(this.config);

    // 触发回调
    for (const callback of this.callbacks) {
      try {
        callback(key, value, oldValue);
      } catch (error) {
        logger.error('[ConfigManager] 配置变更回调错误:', error);
      }
    }
  }

  /**
   * 监听配置变更
   * @param callback 回调函数
   */
  watch(callback: ConfigChangeCallback): void {
    this.callbacks.add(callback);
  }

  /**
   * 取消监听配置变更
   * @param callback 回调函数
   */
  unwatch(callback: ConfigChangeCallback): void {
    this.callbacks.delete(callback);
  }

  /**
   * 获取完整配置对象（深拷贝，防止外部修改）
   */
  getConfig(): any {
    return JSON.parse(JSON.stringify(this.config));
  }

  /**
   * 停止监听并清理资源
   */
  destroy(): void {
    if (this.watcher) {
      this.watcher.close();
      this.watcher = null;
    }
    this.callbacks.clear();
  }
}
