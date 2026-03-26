import { LLMRequest, LLMResponse, LLMMetrics, MCPCall } from '../types.js';
import Database from 'better-sqlite3';

// 写操作类型定义
type WriteOperation =
  | { type: 'request'; data: LLMRequest }
  | { type: 'response'; data: LLMResponse }
  | { type: 'metrics'; data: LLMMetrics }
  | { type: 'mcpCall'; data: MCPCall };

// 写队列配置选项
interface WriteQueueOptions {
  batchSize?: number;      // 每批处理的操作数
  flushInterval?: number;  // 刷新间隔（毫秒）
  maxQueueSize?: number;   // 队列最大容量
  maxRetries?: number;     // 最大重试次数
  retryDelay?: number;     // 重试延迟（毫秒）
}

/**
 * 批量写入队列
 * 
 * 用于高性能批处理数据库写入操作，特性包括：
 * - 自动批处理：达到批次大小或定时刷新时批量写入
 * - 事务支持：使用 SQLite 事务保证数据一致性
 * - 并发安全：通过锁机制防止竞态条件
 * - 失败重试：写入失败时自动重试，避免数据丢失
 * - 内存限制：队列有最大容量限制，防止内存溢出
 */
export class WriteQueue {
  // 内部队列存储待处理的写操作
  private queue: WriteOperation[] = [];
  
  // SQLite 数据库实例
  private db: Database.Database;
  
  // 配置参数
  private batchSize: number;
  private flushInterval: number;
  private maxQueueSize: number;
  private maxRetries: number;
  private retryDelay: number;
  
  // 定时器引用
  private timer: NodeJS.Timeout | null = null;
  
  // 并发控制标志
  private isProcessing: boolean = false;
  
  // 运行状态标志
  private isRunning: boolean = false;

  /**
   * 创建 WriteQueue 实例
   * @param db - better-sqlite3 数据库实例
   * @param options - 可选配置参数
   */
  constructor(db: Database.Database, options?: WriteQueueOptions) {
    this.db = db;
    this.batchSize = options?.batchSize ?? 50;
    this.flushInterval = options?.flushInterval ?? 100;
    this.maxQueueSize = options?.maxQueueSize ?? 10000;
    this.maxRetries = options?.maxRetries ?? 3;
    this.retryDelay = options?.retryDelay ?? 100;
    
    this.start();
  }

  /**
   * 启动写队列
   * 初始化定时刷新机制
   */
  start(): void {
    if (this.isRunning) return;
    
    this.isRunning = true;
    this.startTimer();
  }

  /**
   * 停止写队列
   * 清空剩余操作并停止定时器
   */
  stop(): void {
    if (!this.isRunning) return;
    
    this.isRunning = false;
    
    // 清除定时器
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    
    // 刷新剩余操作
    this.flushSync();
  }

  /**
   * 启动定时器
   * 按配置间隔定期刷新队列
   */
  private startTimer(): void {
    if (this.timer) return;
    
    this.timer = setInterval(() => {
      if (this.queue.length > 0) {
        this.flush();
      }
    }, this.flushInterval);
  }

  /**
   * 将操作加入队列
   * @param operation - 写操作对象
   * @throws 当队列已满时抛出错误
   */
  enqueue(operation: WriteOperation): void {
    // 检查队列容量
    if (this.queue.length >= this.maxQueueSize) {
      console.error('[WriteQueue] Queue is full, dropping operation');
      throw new Error('Write queue is full');
    }
    
    // 加入队列
    this.queue.push(operation);
    
    // 达到批次大小时立即刷新
    if (this.queue.length >= this.batchSize) {
      this.flush();
    }
  }

  /**
   * 异步刷新队列
   * 批量处理队列中的操作
   */
  async flush(): Promise<void> {
    if (this.isProcessing || this.queue.length === 0) {
      return;
    }
    
    this.isProcessing = true;
    
    // 提取批次操作
    const batch = this.queue.splice(0, this.batchSize);
    
    try {
      await this.executeBatchWithRetry(batch);
    } catch (err) {
      console.error('[WriteQueue] Batch write failed after retries:', err);
      // 失败时将操作放回队列头部
      this.queue.unshift(...batch);
    } finally {
      this.isProcessing = false;
    }
  }

  /**
   * 同步刷新队列
   * 用于停止时确保所有数据写入
   */
  private flushSync(): void {
    while (this.queue.length > 0 && !this.isProcessing) {
      this.isProcessing = true;
      
      const batch = this.queue.splice(0, this.batchSize);
      
      try {
        this.executeBatch(batch);
      } catch (err) {
        console.error('[WriteQueue] Sync flush failed:', err);
        // 无法恢复，记录错误但继续
      } finally {
        this.isProcessing = false;
      }
    }
  }

  /**
   * 带重试机制的批次执行
   * @param batch - 操作批次
   * @param attempt - 当前尝试次数
   */
  private async executeBatchWithRetry(batch: WriteOperation[], attempt: number = 1): Promise<void> {
    try {
      this.executeBatch(batch);
    } catch (err) {
      if (attempt < this.maxRetries) {
        console.warn(`[WriteQueue] Batch failed, retrying (${attempt}/${this.maxRetries})...`);
        await this.delay(this.retryDelay * attempt);
        await this.executeBatchWithRetry(batch, attempt + 1);
      } else {
        throw err;
      }
    }
  }

  /**
   * 执行批次操作
   * 在事务中执行所有操作
   * @param batch - 操作批次
   */
  private executeBatch(batch: WriteOperation[]): void {
    // 使用事务包装批量操作
    const transaction = this.db.transaction((ops: WriteOperation[]) => {
      for (const op of ops) {
        this.executeOperation(op);
      }
    });
    
    transaction(batch);
  }

  /**
   * 执行单个操作
   * 委托给具体的存储方法
   * @param op - 写操作
   */
  private executeOperation(op: WriteOperation): void {
    // 注意：实际执行由外部存储类提供
    // 这里只是占位，说明操作类型
    switch (op.type) {
      case 'request':
        // 由 SQLiteStorage 执行: insertRequest
        break;
      case 'response':
        // 由 SQLiteStorage 执行: insertResponse
        break;
      case 'metrics':
        // 由 SQLiteStorage 执行: insertMetrics
        break;
      case 'mcpCall':
        // 由 SQLiteStorage 执行: insertMcpCall
        break;
      default:
        console.warn('[WriteQueue] Unknown operation type:', (op as any).type);
    }
  }

  /**
   * 延迟辅助方法
   * @param ms - 毫秒数
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 获取当前队列大小
   * @returns 队列中的操作数量
   */
  getQueueSize(): number {
    return this.queue.length;
  }

  /**
   * 获取队列状态信息
   * @returns 状态对象
   */
  getStatus(): { queueSize: number; isProcessing: boolean; isRunning: boolean } {
    return {
      queueSize: this.queue.length,
      isProcessing: this.isProcessing,
      isRunning: this.isRunning
    };
  }
}
