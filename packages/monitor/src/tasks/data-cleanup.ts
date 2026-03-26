import Database from 'better-sqlite3';
import fs from 'fs/promises';
import path from 'path';
import { createGzip } from 'zlib';
import { promisify } from 'util';
import { pipeline } from 'stream';
import { createReadStream, createWriteStream } from 'fs';
import { config } from '../config.js';

const pipelineAsync = promisify(pipeline);

export class DataCleanupTask {
  private db: Database.Database;
  private archiveDir: string;
  
  constructor(db: Database.Database) {
    this.db = db;
    this.archiveDir = path.join(process.cwd(), 'archives');
  }
  
  async initialize(): Promise<void> {
    await fs.mkdir(this.archiveDir, { recursive: true });
  }
  
  async cleanup(): Promise<{ archived: number; deleted: number }> {
    if (!config.retention.enabled) {
      return { archived: 0, deleted: 0 };
    }
    
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - config.retention.days);
    const cutoffTimestamp = cutoffDate.getTime();
    
    console.log(`[Cleanup] 清理 ${config.retention.days} 天前的数据 (截止: ${cutoffDate.toISOString()})`);
    
    // 查询将要删除的数据
    const oldRequests = this.db.prepare(
      'SELECT id FROM requests WHERE timestamp < ?'
    ).all(cutoffTimestamp) as { id: string }[];
    
    if (oldRequests.length === 0) {
      console.log('[Cleanup] 没有需要清理的数据');
      return { archived: 0, deleted: 0 };
    }
    
    console.log(`[Cleanup] 发现 ${oldRequests.length} 条过期记录`);
    
    // 归档数据
    let archived = 0;
    if (config.retention.archiveBeforeDelete) {
      archived = await this.archiveData(cutoffTimestamp);
    }
    
    // 删除数据
    const deleteCount = this.db.transaction(() => {
      const result = this.db.prepare(
        'DELETE FROM requests WHERE timestamp < ?'
      ).run(cutoffTimestamp);
      return result.changes;
    })();
    
    // 压缩数据库
    this.db.exec('VACUUM');
    
    console.log(`[Cleanup] 完成: 归档 ${archived} 条, 删除 ${deleteCount} 条`);
    
    return { archived, deleted: deleteCount };
  }
  
  private async archiveData(cutoffTimestamp: number): Promise<number> {
    const archiveDate = new Date().toISOString().split('T')[0];
    const archivePath = path.join(this.archiveDir, `archive-${archiveDate}.json.gz`);
    
    const oldData = this.db.prepare(`
      SELECT 
        r.*,
        json_object(
          'response', (SELECT json_object(*) FROM responses WHERE request_id = r.id),
          'metrics', (SELECT json_object(*) FROM metrics WHERE request_id = r.id),
          'mcp_calls', (SELECT json_group_array(json_object(*)) FROM mcp_calls WHERE request_id = r.id)
        ) as related_data
      FROM requests r
      WHERE r.timestamp < ?
    `).all(cutoffTimestamp);
    
    if (oldData.length === 0) return 0;
    
    const tempPath = `${archivePath}.tmp`;
    await fs.writeFile(tempPath, JSON.stringify(oldData, null, 2));
    
    await pipelineAsync(
      createReadStream(tempPath),
      createGzip(),
      createWriteStream(archivePath)
    );
    
    await fs.unlink(tempPath);
    console.log(`[Cleanup] 数据已归档到: ${archivePath}`);
    
    return oldData.length;
  }
  
  scheduleDailyCleanup(): void {
    const scheduleCleanup = () => {
      const now = new Date();
      const next3AM = new Date(now);
      next3AM.setHours(3, 0, 0, 0);
      if (next3AM <= now) {
        next3AM.setDate(next3AM.getDate() + 1);
      }
      
      const delay = next3AM.getTime() - now.getTime();
      
      setTimeout(() => {
        this.cleanup().catch(console.error);
        scheduleCleanup();
      }, delay);
    };
    
    scheduleCleanup();
  }
}
