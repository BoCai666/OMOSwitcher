/**
 * 存储管理器
 * 
 * 用于在运行时切换和访问实际的存储实例
 * 解决 routes.ts 无法访问 index.ts 中局部 storage 变量的问题
 */

import { StorageInterface } from './interface.js';
import { memoryStore } from './memory-store.js';

// 当前使用的存储实例
let currentStorage: StorageInterface = memoryStore;

/**
 * 设置当前存储实例
 * @param storage 存储实例
 */
export function setStorage(storage: StorageInterface): void {
  currentStorage = storage;
}

/**
 * 获取当前存储实例
 * @returns 当前存储实例
 */
export function getStorage(): StorageInterface {
  return currentStorage;
}

/**
 * 重置为内存存储（用于测试）
 */
export function resetStorage(): void {
  currentStorage = memoryStore;
}
