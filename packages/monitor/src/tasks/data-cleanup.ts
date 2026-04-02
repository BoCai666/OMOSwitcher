import fs from 'fs/promises';
import path from 'path';
import { createGzip } from 'zlib';
import { promisify } from 'util';
import { pipeline } from 'stream';
import { createReadStream, createWriteStream } from 'fs';
import { dbManager } from '../db/index.js';
import { ARCHIVE_DIR } from '../paths.js';

const pipelineAsync = promisify(pipeline);

// 数据保留配置
const RETENTION_CONFIG = {
  enabled: true,
  days: 90,
  archiveBeforeDelete: true
};

export class DataCleanupTask {
  private db: any; // sql.js Database
  private archiveDir: string;
  
  constructor(db: any) {
    this.db = db;
    this.archiveDir = ARCHIVE_DIR;
  }
  
  async initialize(): Promise<void> {
    await fs.mkdir(this.archiveDir, { recursive: true });
    console.log(`[Cleanup] Archive directory: ${this.archiveDir}`);
  }
  
  /**
   * 执行数据清理
   */
  async cleanup(): Promise<{ archived: number; deleted: number }> {
    if (!RETENTION_CONFIG.enabled) {
      return { archived: 0, deleted: 0 };
    }
    
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - RETENTION_CONFIG.days);
    const cutoffTimestamp = cutoffDate.getTime();
    
    console.log(`[Cleanup] Cleaning data older than ${RETENTION_CONFIG.days} days (cutoff: ${cutoffDate.toISOString()})`);
    
    // 查询将要删除的数据
    const oldRequests = dbManager.query(
      'SELECT id FROM requests WHERE timestamp < ?',
      [cutoffTimestamp]
    );
    
    if (oldRequests.length === 0) {
      console.log('[Cleanup] No data to clean');
      return { archived: 0, deleted: 0 };
    }
    
    console.log(`[Cleanup] Found ${oldRequests.length} expired records`);
    
    // 归档数据
    let archived = 0;
    if (RETENTION_CONFIG.archiveBeforeDelete) {
      archived = await this.archiveData(cutoffTimestamp);
    }
    
    // 删除数据
    const deleteStmt = dbManager.prepare('DELETE FROM requests WHERE timestamp < ?');
    const result = deleteStmt.run(cutoffTimestamp);
    
    const deleteCount = result.changes;
    
    console.log(`[Cleanup] Completed: archived ${archived}, deleted ${deleteCount}`);
    
    return { archived, deleted: deleteCount };
  }
  
  /**
   * 归档数据到压缩文件
   */
  private async archiveData(cutoffTimestamp: number): Promise<number> {
    const archiveDate = new Date().toISOString().split('T')[0];
    const archivePath = path.join(this.archiveDir, `archive-${archiveDate}.json.gz`);
    
    // 查询旧数据
    const oldData = dbManager.query(`
      SELECT 
        r.id, r.timestamp, r.provider, r.model, r.method, r.url, r.domain,
        (SELECT json_object(
          'status_code', status_code,
          'duration', duration
        ) FROM responses WHERE request_id = r.id) as response,
        (SELECT json_object(
          'total_tokens', total_tokens,
          'estimated_cost', estimated_cost,
          'duration', duration
        ) FROM metrics WHERE request_id = r.id) as metrics
      FROM requests r
      WHERE r.timestamp < ?
    `, [cutoffTimestamp]);
    
    if (oldData.length === 0) return 0;
    
    const tempPath = `${archivePath}.tmp`;
    await fs.writeFile(tempPath, JSON.stringify(oldData, null, 2));
    
    await pipelineAsync(
      createReadStream(tempPath),
      createGzip(),
      createWriteStream(archivePath)
    );
    
    await fs.unlink(tempPath);
    console.log(`[Cleanup] Data archived to: ${archivePath}`);
    
    return oldData.length;
  }
  
  /**
   * 调度每日清理（凌晨3点）
   */
  scheduleDailyCleanup(): void {
    const scheduleCleanup = () => {
      const now = new Date();
      const next3AM = new Date(now);
      next3AM.setHours(3, 0, 0, 0);
      if (next3AM <= now) {
        next3AM.setDate(next3AM.getDate() + 1);
      }
      
      const delay = next3AM.getTime() - now.getTime();
      
      console.log(`[Cleanup] Next cleanup scheduled at: ${next3AM.toISOString()}`);
      
      setTimeout(() => {
        this.cleanup().catch(console.error);
        scheduleCleanup();
      }, delay);
    };
    
    scheduleCleanup();
  }
}
