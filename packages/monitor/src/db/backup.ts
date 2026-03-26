import Database from 'better-sqlite3';
import fs from 'fs/promises';
import path from 'path';
import { config } from '../config.js';

export class DatabaseBackup {
  private db: Database.Database;
  private backupDir: string;
  private maxBackups: number;
  
  constructor(db: Database.Database) {
    this.db = db;
    this.backupDir = path.join(process.cwd(), 'backups');
    this.maxBackups = 7;
  }
  
  async initialize(): Promise<void> {
    await fs.mkdir(this.backupDir, { recursive: true });
    
    // 设置数据库文件权限（仅所有者可读写）
    const dbPath = config.storage.sqlite?.path || './data/opencode.db';
    try {
      await fs.chmod(dbPath, 0o600);
    } catch (err) {
      console.warn('[Backup] 无法设置数据库文件权限:', err);
    }
  }
  
  async createBackup(): Promise<string> {
    const timestamp = new Date().toISOString()
      .replace(/[:.]/g, '-')
      .slice(0, 19);
    const backupPath = path.join(this.backupDir, `backup-${timestamp}.db`);
    
    console.log(`[Backup] 创建备份: ${backupPath}`);
    
    await this.db.backup(backupPath);
    console.log('[Backup] 备份完成');
    
    await this.cleanupOldBackups();
    
    return backupPath;
  }
  
  private async cleanupOldBackups(): Promise<void> {
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
    
    // 按时间排序
    backupsWithTime.sort((a, b) => b.mtime.getTime() - a.mtime.getTime());
    
    // 删除旧的备份
    for (let i = this.maxBackups; i < backupsWithTime.length; i++) {
      console.log(`[Backup] 删除旧备份: ${backupsWithTime[i].name}`);
      await fs.unlink(backupsWithTime[i].path);
    }
  }
  
  scheduleDailyBackup(): void {
    const scheduleBackup = () => {
      const now = new Date();
      const next2AM = new Date(now);
      next2AM.setHours(2, 0, 0, 0);
      if (next2AM <= now) {
        next2AM.setDate(next2AM.getDate() + 1);
      }
      
      const delay = next2AM.getTime() - now.getTime();
      
      setTimeout(() => {
        this.createBackup().catch(console.error);
        scheduleBackup();
      }, delay);
    };
    
    scheduleBackup();
  }
  
  async getBackupList(): Promise<Array<{ filename: string; size: number; created: Date }>> {
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
  }
}
