import initSqlJs, { Database as SqlJsDatabase, SqlJsStatic } from 'sql.js';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';
import { MONITOR_ROOT, DATABASE_FILE } from '../paths.js';

// 使用 createRequire 来加载 WASM 文件（pkg 兼容）
const require = createRequire(import.meta.url);

// pkg 环境中 __dirname 已被注入，使用不同的变量名
const pkgDirname = typeof __dirname !== 'undefined' 
  ? __dirname 
  : path.dirname(fileURLToPath(import.meta.url));

/**
 * 获取 WASM 文件路径
 */
function getWasmPath(): string {
  // 尝试多个可能的位置
  const possiblePaths = [
    path.join(pkgDirname, '..', 'wasm', 'sql-wasm.wasm'),
    path.join(pkgDirname, 'wasm', 'sql-wasm.wasm'),
    path.join(process.cwd(), 'dist', 'wasm', 'sql-wasm.wasm'),
  ];
  
  for (const p of possiblePaths) {
    try {
      if (require('fs').existsSync(p)) {
        return p;
      }
    } catch {
      // 继续尝试
    }
  }
  
  // 返回默认路径（sql.js 会尝试自动加载）
  return path.join(pkgDirname, '..', 'wasm', 'sql-wasm.wasm');
}

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
 * Statement 包装类
 * 封装 sql.js 的 Statement，提供类似 better-sqlite3 的 API
 */
class StatementWrapper {
  private stmt: any;
  private db: SqlJsDatabase;

  constructor(db: SqlJsDatabase, sql: string) {
    this.db = db;
    this.stmt = db.prepare(sql);
  }

  /**
   * 绑定参数并执行
   * 支持 positional (?) 和 named ($param) 参数
   */
  run(...params: any[]): { changes: number; lastInsertRowid: number | bigint } {
    // 转换参数格式
    const boundParams = params.map(p => {
      if (p === null) return null;
      if (p === undefined) return null;
      if (typeof p === 'boolean') return p ? 1 : 0;
      if (typeof p === 'object') return JSON.stringify(p);
      return p;
    });

    this.stmt.bind(boundParams);
    this.stmt.step();
    this.stmt.reset();

    return {
      changes: this.db.getRowsModified(),
      lastInsertRowid: this.db.exec('SELECT last_insert_rowid() as id')[0]?.values[0]?.[0] as number || 0
    };
  }

  /**
   * 执行查询并返回所有结果
   */
  all(...params: any[]): any[] {
    const boundParams = params.map(p => {
      if (p === null) return null;
      if (p === undefined) return null;
      if (typeof p === 'boolean') return p ? 1 : 0;
      if (typeof p === 'object') return JSON.stringify(p);
      return p;
    });

    this.stmt.bind(boundParams);
    
    const results: any[] = [];
    while (this.stmt.step()) {
      results.push(this.stmt.getAsObject());
    }
    this.stmt.reset();

    return results;
  }

  /**
   * 执行查询并返回第一条结果
   */
  get(...params: any[]): any | undefined {
    const boundParams = params.map(p => {
      if (p === null) return null;
      if (p === undefined) return null;
      if (typeof p === 'boolean') return p ? 1 : 0;
      if (typeof p === 'object') return JSON.stringify(p);
      return p;
    });

    this.stmt.bind(boundParams);
    
    let result: any = undefined;
    if (this.stmt.step()) {
      result = this.stmt.getAsObject();
    }
    this.stmt.reset();

    return result;
  }

  /**
   * 释放语句资源
   */
  free(): void {
    this.stmt.free();
  }
}

/**
 * 数据库管理器类
 * 
 * 使用 sql.js（纯 WASM SQLite）实现持久化存储
 * 可以在 pkg 打包环境中正常工作
 */
export class DatabaseManager {
  private db: SqlJsDatabase | null = null;
  private SQL: SqlJsStatic | null = null;
  private dbPath: string;
  private statements: Map<string, StatementWrapper> = new Map();
  private saveTimeout: ReturnType<typeof setTimeout> | null = null;
  private pendingSave = false;

  constructor() {
    this.dbPath = getDatabasePath();
  }

  /**
   * 初始化数据库连接
   */
  async initialize(): Promise<SqlJsDatabase> {
    // 确保 SQL.js 已加载，指定 WASM 文件路径
    if (!this.SQL) {
      const wasmPath = getWasmPath();
      console.log(`[Database] Loading SQL.js WASM from: ${wasmPath}`);
      
      this.SQL = await initSqlJs({
        locateFile: (file: string) => {
          // 对于 sql-wasm.wasm，使用我们找到的路径
          if (file === 'sql-wasm.wasm') {
            return wasmPath;
          }
          return file;
        }
      });
    }

    // 确保数据目录存在
    const dataDir = path.dirname(this.dbPath);
    await fs.mkdir(dataDir, { recursive: true });

    // 尝试从文件加载数据库
    try {
      const buffer = await fs.readFile(this.dbPath);
      this.db = new this.SQL.Database(buffer);
      console.log(`[Database] Loaded existing database: ${this.dbPath}`);
    } catch {
      // 文件不存在，创建新数据库
      this.db = new this.SQL.Database();
      console.log(`[Database] Created new database: ${this.dbPath}`);
    }

    // 配置数据库
    this.configureDatabase();

    // 执行迁移
    await this.runMigrations();

    // 初始保存（强制保存，不检查 pendingSave 标志）
    this.pendingSave = true;
    await this.saveToFile();

    return this.db;
  }

  /**
   * 配置数据库参数
   */
  private configureDatabase(): void {
    if (!this.db) return;

    // 启用外键约束
    this.db.run('PRAGMA foreign_keys = ON');
    
    console.log('[Database] Configuration applied');
  }

  /**
   * 执行数据库迁移脚本
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
          this.db!.run(sql);
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
  getDatabase(): SqlJsDatabase {
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
   * 准备语句（类似 better-sqlite3 的 prepare）
   */
  prepare(sql: string): StatementWrapper {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return new StatementWrapper(this.db, sql);
  }

  /**
   * 执行 SQL 语句
   */
  exec(sql: string): void {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    this.db.run(sql);
    this.scheduleSave();
  }

  /**
   * 执行查询并返回结果
   */
  query(sql: string, params: any[] = []): any[] {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    
    const stmt = new StatementWrapper(this.db, sql);
    const result = stmt.all(...params);
    stmt.free();
    
    return result;
  }

  /**
   * 执行单条语句并返回结果
   */
  queryOne(sql: string, params: any[] = []): any | undefined {
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    
    const stmt = new StatementWrapper(this.db, sql);
    const result = stmt.get(...params);
    stmt.free();
    
    return result;
  }

  /**
   * 执行 PRAGMA 命令
   */
  pragma(pragmaSql: string): void {
    if (!this.db) return;
    this.db.run(`PRAGMA ${pragmaSql}`);
  }

  /**
   * 调度保存操作（防抖）
   */
  scheduleSave(): void {
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }
    this.pendingSave = true;
    this.saveTimeout = setTimeout(() => {
      this.saveToFile();
    }, 1000); // 1秒后保存
  }

  /**
   * 保存数据库到文件
   */
  async saveToFile(): Promise<void> {
    if (!this.db) return;
    
    // 如果 pendingSave 为 false，强制设置并保存
    if (!this.pendingSave) {
      this.pendingSave = true;
    }
    
    try {
      const data = this.db.export();
      const buffer = Buffer.from(data);
      await fs.writeFile(this.dbPath, buffer);
      this.pendingSave = false;
      console.log(`[Database] Saved to ${this.dbPath}`);
    } catch (err) {
      console.error('[Database] Failed to save:', err);
    }
  }

  /**
   * 立即保存数据库到文件
   */
  async flush(): Promise<void> {
    if (!this.db) return;
    
    const data = this.db.export();
    const buffer = Buffer.from(data);
    await fs.writeFile(this.dbPath, buffer);
    this.pendingSave = false;
  }

  /**
   * 关闭数据库连接
   */
  async close(): Promise<void> {
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }
    
    if (this.db) {
      // 保存最终数据
      await this.flush();
      
      // 释放所有语句
      for (const stmt of this.statements.values()) {
        stmt.free();
      }
      this.statements.clear();
      
      this.db.close();
      this.db = null;
      console.log('[Database] Connection closed');
    }
  }
}

// 导出单例实例
export const dbManager = new DatabaseManager();
