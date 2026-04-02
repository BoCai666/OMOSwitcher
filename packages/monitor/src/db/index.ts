import Database from 'better-sqlite3';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { DATABASE_FILE } from '../paths.js';

// 获取当前目录路径
const currentDir = typeof __dirname !== 'undefined'
  ? __dirname
  : path.dirname(fileURLToPath(import.meta.url));

/**
 * 获取数据库路径
 * 优先级：环境变量 > 默认用户目录
 */
function getDatabasePath(): string {
  if (process.env.SQLITE_PATH) {
    return process.env.SQLITE_PATH;
  }
  return DATABASE_FILE;
}

/**
 * 数据库管理器类
 * 
 * 使用 better-sqlite3（原生 SQLite 绑定）实现持久化存储
 * - 同步 API，性能更好
 * - 自动持久化，无需手动保存
 * - 原生 C 绑定，无需 WASM
 */
export class DatabaseManager {
  private db: Database.Database | null = null;
  private dbPath: string;

  constructor() {
    this.dbPath = getDatabasePath();
  }

  /**
   * 初始化数据库连接
   */
  async initialize(): Promise<Database.Database> {
    // 确保数据目录存在
    const dataDir = path.dirname(this.dbPath);
    await fs.promises.mkdir(dataDir, { recursive: true });

    // 创建数据库连接
    this.db = new Database(this.dbPath);

    console.log(`[Database] Connected to: ${this.dbPath}`);

    // 配置数据库
    this.configureDatabase();

    // 执行迁移
    await this.runMigrations();

    return this.db;
  }

  /**
   * 配置数据库参数
   */
  private configureDatabase(): void {
    if (!this.db) return;

    // 启用外键约束
    this.db.pragma('foreign_keys = ON');
    
    // 性能优化：WAL 模式
    this.db.pragma('journal_mode = WAL');
    
    console.log('[Database] Configuration applied');
  }

  /**
   * 执行数据库迁移脚本
   */
  private async runMigrations(): Promise<void> {
    if (!this.db) return;

    const migrationsDir = path.join(currentDir, 'migrations');

    try {
      const files = await fs.promises.readdir(migrationsDir);
      const sqlFiles = files
        .filter(f => f.endsWith('.sql'))
        .sort();

      for (const file of sqlFiles) {
        const filePath = path.join(migrationsDir, file);
        const sql = await fs.promises.readFile(filePath, 'utf-8');

        try {
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
   */
  getDatabase(): Database.Database {
    if (!this.db) {
      throw new Error('Database not initialized. Call initialize() first.');
    }
    return this.db;
  }

  /**
   * 检查数据库是否已初始化
   */
  isInitialized(): boolean {
    return this.db !== null;
  }

  /**
   * 准备语句
   */
  prepare(sql: string): Database.Statement {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db.prepare(sql);
  }

  /**
   * 执行 SQL 语句
   */
  exec(sql: string): void {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    this.db.exec(sql);
  }

  /**
   * 执行查询并返回所有结果
   */
  query(sql: string, params: any[] = []): any[] {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db.prepare(sql).all(...params) as any[];
  }

  /**
   * 执行查询并返回第一条结果
   */
  queryOne(sql: string, params: any[] = []): any | undefined {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db.prepare(sql).get(...params);
  }

  /**
   * 执行 PRAGMA 命令
   */
  pragma(pragmaSql: string): void {
    if (!this.db) return;
    this.db.pragma(pragmaSql);
  }

  /**
   * 创建事务
   */
  transaction<T>(fn: () => T): T {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db.transaction(fn)();
  }

  /**
   * 关闭数据库连接
   */
  async close(): Promise<void> {
    if (this.db) {
      this.db.close();
      this.db = null;
      console.log('[Database] Connection closed');
    }
  }
}

// 导出单例实例
export const dbManager = new DatabaseManager();
