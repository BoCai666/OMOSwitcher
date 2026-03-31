/**
 * Monitor 日志工具
 * 支持同时输出到控制台和文件
 */

import fs from 'fs';
import path from 'path';
import { LOGS_DIR, LOG_FILE } from './paths.js';

// 最大日志文件大小（5MB）
const MAX_LOG_SIZE = 5 * 1024 * 1024;
// 保留的备份日志数量
const MAX_BACKUP_COUNT = 3;

/**
 * 确保日志目录存在
 */
function ensureLogDir(): void {
  try {
    if (!fs.existsSync(LOGS_DIR)) {
      fs.mkdirSync(LOGS_DIR, { recursive: true });
    }
  } catch (err) {
    // 如果创建目录失败，仅输出到控制台
    console.error('[Logger] 无法创建日志目录:', err);
  }
}

/**
 * 检查并轮转日志文件
 * 当日志文件超过最大大小时，创建备份并开始新日志
 */
function rotateLogIfNeeded(): void {
  try {
    if (!fs.existsSync(LOG_FILE)) {
      return;
    }

    const stats = fs.statSync(LOG_FILE);
    if (stats.size >= MAX_LOG_SIZE) {
      // 删除最旧的备份
      const oldestBackup = path.join(LOGS_DIR, `monitor.log.${MAX_BACKUP_COUNT}`);
      if (fs.existsSync(oldestBackup)) {
        fs.unlinkSync(oldestBackup);
      }

      // 轮转备份文件
      for (let i = MAX_BACKUP_COUNT - 1; i >= 1; i--) {
        const currentBackup = path.join(LOGS_DIR, `monitor.log.${i}`);
        const nextBackup = path.join(LOGS_DIR, `monitor.log.${i + 1}`);
        if (fs.existsSync(currentBackup)) {
          fs.renameSync(currentBackup, nextBackup);
        }
      }

      // 将当前日志文件重命名为 .1
      fs.renameSync(LOG_FILE, path.join(LOGS_DIR, 'monitor.log.1'));
    }
  } catch (err) {
    console.error('[Logger] 日志轮转失败:', err);
  }
}

/**
 * 写入日志到文件
 */
function writeToFile(message: string): void {
  try {
    ensureLogDir();
    rotateLogIfNeeded();
    fs.appendFileSync(LOG_FILE, message + '\n', 'utf-8');
  } catch (err) {
    // 写入失败时仅输出到控制台
    console.error('[Logger] 写入日志文件失败:', err);
  }
}

/**
 * 格式化日志消息
 */
function formatMessage(level: string, message: string, ...args: unknown[]): string {
  const timestamp = new Date().toISOString();
  const argsStr = args.length > 0 ? ' ' + args.map(a => 
    typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)
  ).join(' ') : '';
  return `[${timestamp}] [${level}] ${message}${argsStr}`;
}

/**
 * 日志级别
 */
export const logger = {
  /**
   * 信息日志
   */
  info(message: string, ...args: unknown[]): void {
    const formatted = formatMessage('INFO', message, ...args);
    console.log(formatted);
    writeToFile(formatted);
  },

  /**
   * 警告日志
   */
  warn(message: string, ...args: unknown[]): void {
    const formatted = formatMessage('WARN', message, ...args);
    console.warn(formatted);
    writeToFile(formatted);
  },

  /**
   * 错误日志
   */
  error(message: string, ...args: unknown[]): void {
    const formatted = formatMessage('ERROR', message, ...args);
    console.error(formatted);
    writeToFile(formatted);
  },

  /**
   * 调试日志（仅在开发模式）
   */
  debug(message: string, ...args: unknown[]): void {
    if (process.env.NODE_ENV === 'development' || process.env.DEBUG) {
      const formatted = formatMessage('DEBUG', message, ...args);
      console.log(formatted);
      writeToFile(formatted);
    }
  },

  /**
   * 获取日志文件路径
   */
  getLogFilePath(): string {
    return LOG_FILE;
  },

  /**
   * 清空日志文件
   */
  clear(): void {
    try {
      ensureLogDir();
      fs.writeFileSync(LOG_FILE, '', 'utf-8');
    } catch (err) {
      console.error('[Logger] 清空日志文件失败:', err);
    }
  },
};

// 默认导出
export default logger;
