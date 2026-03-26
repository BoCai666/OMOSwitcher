import Database from 'better-sqlite3';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { config } from '../config.js';

// pkg 环境中 __dirname 已被注入，使用不同的变量名
const pkgDirname = typeof __dirname !== 'undefined' 
  ? __dirname 
  : path.dirname(fileURLToPath(import.meta.url));

/**
 * 数据库管理器类
 * 
 * 封装 better-sqlite3 数据库连接，提供：
 * - 数据库初始化和连接管理
 * - 迁移脚本执行
 * - WAL 模式配置
 * - 连接错误处理
 */
export class DatabaseManager {
  private db: Database.Database | null = null;
  private dbPath: string;

  constructor() {
    this.dbPath = config.storage.sqlite?.path || './data/opencode.db';
  }

  /**
   * 初始化数据库连接
   * 
   * @returns 数据库连接实例
   * @throws 初始化失败时抛出错误
   */
  async initialize(): Promise<Database.Database> {
    // 确保数据目录存在
    const dataDir = path.dirname(this.dbPath);
    await fs.mkdir(dataDir, { recursive: true });

    // 打开数据库连接
    this.db = new Database(this.dbPath);

    // 配置数据库
    this.configureDatabase();

    // 执行迁移
    await this.runMigrations();

    console.log(`[Database] Connected to SQLite: ${this.dbPath}`);
    return this.db;
  }

  /**
   * 配置数据库参数
   * 
   * 包括：
   * - 外键约束启用
   * - WAL 模式配置
   * - 忙等待超时
   * - 性能优化参数
   */
  private configureDatabase(): void {
    if (!this.db) return;

    // 启用外键约束
    this.db.pragma('foreign_keys = ON');

    // 启用 WAL 模式（如果配置）
    if (config.storage.sqlite?.walMode) {
      this.db.pragma('journal_mode = WAL');
      console.log('[Database] WAL mode enabled');
    }

    // 设置忙等待超时
    const busyTimeout = config.storage.sqlite?.busyTimeout || 5000;
    this.db.pragma(`busy_timeout = ${busyTimeout}`);

    // 优化性能设置
    this.db.pragma('synchronous = NORMAL');
    this.db.pragma('cache_size = -64000'); // 64MB cache
    this.db.pragma('temp_store = MEMORY');
  }

  /**
   * 执行数据库迁移脚本
   * 
   * 从 migrations 目录读取所有 .sql 文件并按顺序执行
   */
  private async runMigrations(): Promise<void> {
    if (!this.db) return;

    const migrationsDir = path.join(pkgDirname, 'migrations');
    
    try {
      const files = await fs.readdir(migrationsDir);
      const sqlFiles = files
        .filter(f => f.endsWith('.sql'))
        .sort();

      for (const file of sqlFiles) {
        const filePath = path.join(migrationsDir, file);
        const sql = await fs.readFile(filePath, 'utf-8');
        
        try {
          // 执行迁移脚本
          this.db!.exec(sql);
          console.log(`[Database] Migration applied: ${file}`);
        } catch (execErr: any) {
          // 如果是"duplicate column name"错误，则忽略
          if (execErr.message && execErr.message.includes('duplicate column name')) {
            console.log(`[Database] Migration skipped (already applied): ${file}`);
            continue;
          }
          // 其他错误继续抛出
          throw execErr;
        }
      }
    } catch (err) {
      console.error('[Database] Migration error:', err);
      throw err;
    }
  }

  /**
   * 获取数据库连接实例
   * 
   * @returns 数据库连接实例
   * @throws 如果数据库未初始化则抛出错误
   */
  getDatabase(): Database.Database {
    if (!this.db) {
      throw new Error('Database not initialized. Call initialize() first.');
    }
    return this.db;
  }

  /**
   * 检查数据库是否已初始化
   * 
   * @returns 初始化状态
   */
  isInitialized(): boolean {
    return this.db !== null;
  }

  /**
   * 关闭数据库连接
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
      console.log('[Database] Connection closed');
    }
  }
}

// 导出单例实例
export const dbManager = new DatabaseManager();
