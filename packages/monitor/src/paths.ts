/**
 * Monitor 程序统一存储路径定义
 * 
 * 所有配置、数据库、证书等文件统一存储在 ~/.config/omoswitcher/monitor/ 目录下
 */

import { homedir } from 'os';
import { join } from 'path';

// ============ 基础路径 ============

/**
 * Monitor 程序根目录
 * ~/.config/omoswitcher/monitor/
 */
export const MONITOR_ROOT = join(homedir(), '.config', 'omoswitcher', 'monitor');

// ============ 子目录路径 ============

/**
 * 证书目录
 * ~/.config/omoswitcher/monitor/certs/
 */
export const CERTS_DIR = join(MONITOR_ROOT, 'certs');

/**
 * 备份目录
 * ~/.config/omoswitcher/monitor/backups/
 */
export const BACKUPS_DIR = join(MONITOR_ROOT, 'backups');

/**
 * 日志目录
 * ~/.config/omoswitcher/monitor/logs/
 */
export const LOGS_DIR = join(MONITOR_ROOT, 'logs');

// ============ 文件路径 ============

/**
 * 主配置文件
 * ~/.config/omoswitcher/monitor/config.jsonc
 */
export const CONFIG_FILE = join(MONITOR_ROOT, 'config.jsonc');

/**
 * 数据库文件
 * ~/.config/omoswitcher/monitor/data.db
 */
export const DATABASE_FILE = join(MONITOR_ROOT, 'data.db');

/**
 * CA 证书文件
 * ~/.config/omoswitcher/monitor/certs/ca.crt
 */
export const CA_CERT_FILE = join(CERTS_DIR, 'ca.crt');

/**
 * CA 私钥文件
 * ~/.config/omoswitcher/monitor/certs/ca.key
 */
export const CA_KEY_FILE = join(CERTS_DIR, 'ca.key');

/**
 * 日志文件
 * ~/.config/omoswitcher/monitor/logs/monitor.log
 */
export const LOG_FILE = join(LOGS_DIR, 'monitor.log');
