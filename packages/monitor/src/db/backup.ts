import fs from 'fs/promises';
import path from 'path';
import os from 'os';

// 用户配置目录
const USER_CONFIG_DIR = path.join(os.homedir(), '.config', 'omoswitcher');
const BACKUP_DIR = path.join(USER_CONFIG_DIR, 'backups');

/**
 * 数据库备份管理器
 * 
 * 使用 sql.js 的 export() 方法创建备份
 */
export class DatabaseBackup {
  private db: any; // sql.js Database
  private backupDir: string;
  private maxBackups: number;
  private dbPath: string;
  
  constructor(db: any, dbPath?: string) {
    this.db = db;
    this.backupDir = BACKUP_DIR;
    this.maxBackups = 7;
    this.dbPath = dbPath || path.join(USER_CONFIG_DIR, 'monitor.db');
  }
  
  async initialize(): Promise<void> {
    await fs.mkdir(this.backupDir, { recursive: true });
    console.log(`[Backup] Backup directory: ${this.backupDir}`);
  }
  
  /**
   * 创建备份
   * 使用 sql.js 的 export() 方法导出数据库
   */
  async createBackup(): Promise<string> {
    const timestamp = new Date().toISOString()
      .replace(/[:.]/g, '-')
      .slice(0, 19);
    const backupPath = path.join(this.backupDir, `backup-${timestamp}.db`);
    
    console.log(`[Backup] Creating backup: ${backupPath}`);
    
    try {
      // 使用 sql.js 的 export() 方法
      const data = this.db.export();
      const buffer = Buffer.from(data);
      await fs.writeFile(backupPath, buffer);
      
      console.log(`[Backup] Backup completed: ${backupPath} (${buffer.length} bytes)`);
      
      await this.cleanupOldBackups();
      
      return backupPath;
    } catch (err) {
      console.error('[Backup] Failed to create backup:', err);
      throw err;
    }
  }
  
  /**
   * 清理旧备份
   */
  private async cleanupOldBackups(): Promise<void> {
    try {
      const files = await fs.readdir(this.backupDir);
      const backups = files
        .filter(f => f.startsWith('backup-') && f.endsWith('.db'))
        .map(f => ({
          name: f,
          path: path.join(this.backupDir, f)
        }));
      
      // 获取文件状态
      const backupsWithTime = await Promise.all(
        backups.map(async b => ({
          ...b,
          mtime: (await fs.stat(b.path)).mtime
        }))
      );
      
      // 按时间排序（最新的在前）
      backupsWithTime.sort((a, b) => b.mtime.getTime() - a.mtime.getTime());
      
      // 删除超过最大数量的旧备份
      for (let i = this.maxBackups; i < backupsWithTime.length; i++) {
        console.log(`[Backup] Deleting old backup: ${backupsWithTime[i].name}`);
        await fs.unlink(backupsWithTime[i].path);
      }
    } catch (err) {
      console.warn('[Backup] Failed to cleanup old backups:', err);
    }
  }
  
  /**
   * 调度每日备份（凌晨2点）
   */
  scheduleDailyBackup(): void {
    const scheduleBackup = () => {
      const now = new Date();
      const next2AM = new Date(now);
      next2AM.setHours(2, 0, 0, 0);
      if (next2AM <= now) {
        next2AM.setDate(next2AM.getDate() + 1);
      }
      
      const delay = next2AM.getTime() - now.getTime();
      
      console.log(`[Backup] Next backup scheduled at: ${next2AM.toISOString()}`);
      
      setTimeout(() => {
        this.createBackup().catch(console.error);
        scheduleBackup();
      }, delay);
    };
    
    scheduleBackup();
  }
  
  /**
   * 获取备份列表
   */
  async getBackupList(): Promise<Array<{ filename: string; size: number; created: Date }>> {
    try {
      const files = await fs.readdir(this.backupDir);
      const backups = files.filter(f => f.startsWith('backup-') && f.endsWith('.db'));
      
      return Promise.all(
        backups.map(async filename => {
          const filePath = path.join(this.backupDir, filename);
          const stats = await fs.stat(filePath);
          return {
            filename,
            size: stats.size,
            created: stats.mtime
          };
        })
      );
    } catch (err) {
      console.warn('[Backup] Failed to get backup list:', err);
      return [];
    }
  }
}
