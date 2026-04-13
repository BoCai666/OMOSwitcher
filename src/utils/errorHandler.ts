/**
 * 全局错误处理模块
 * 使用 Element Plus Message 和 MessageBox 显示错误信息
 */

import { ElMessage, ElMessageBox } from 'element-plus'

/**
 * 应用错误类
 * 用于封装带有错误码和重试功能的错误
 */
export class AppError extends Error {
  constructor(
    message: string,
    public code?: string,
    public retry?: () => void
  ) {
    super(message)
    this.name = 'AppError'
  }
}

/**
 * 错误类型枚举
 */
export enum ErrorCode {
  // 配置相关错误
  CONFIG_READ_FAILED = 'CONFIG_READ_FAILED',
  CONFIG_WRITE_FAILED = 'CONFIG_WRITE_FAILED',
  CONFIG_PARSE_FAILED = 'CONFIG_PARSE_FAILED',
  CONFIG_NOT_FOUND = 'CONFIG_NOT_FOUND',

  // 预设相关错误
  PRESET_SAVE_FAILED = 'PRESET_SAVE_FAILED',
  PRESET_LOAD_FAILED = 'PRESET_LOAD_FAILED',
  PRESET_DELETE_FAILED = 'PRESET_DELETE_FAILED',
  PRESET_NOT_FOUND = 'PRESET_NOT_FOUND',

  // 模型相关错误
  MODEL_ADD_FAILED = 'MODEL_ADD_FAILED',
  MODEL_UPDATE_FAILED = 'MODEL_UPDATE_FAILED',
  MODEL_DELETE_FAILED = 'MODEL_DELETE_FAILED',
  MODEL_NOT_FOUND = 'MODEL_NOT_FOUND',
  MODEL_DUPLICATE = 'MODEL_DUPLICATE',

  // 文件操作错误
  FILE_READ_FAILED = 'FILE_READ_FAILED',
  FILE_WRITE_FAILED = 'FILE_WRITE_FAILED',

  // 网络错误
  NETWORK_ERROR = 'NETWORK_ERROR',
  TIMEOUT_ERROR = 'TIMEOUT_ERROR',

  // 同步相关错误
  SYNC_NOT_CONFIGURED = 'SYNC_NOT_CONFIGURED',
  SYNC_AUTH_FAILED = 'SYNC_AUTH_FAILED',
  SYNC_TOKEN_EXPIRED = 'SYNC_TOKEN_EXPIRED',
  SYNC_UPLOAD_FAILED = 'SYNC_UPLOAD_FAILED',
  SYNC_DOWNLOAD_FAILED = 'SYNC_DOWNLOAD_FAILED',
  SYNC_CONFLICT_DETECTED = 'SYNC_CONFLICT_DETECTED',
  SYNC_NETWORK_ERROR = 'SYNC_NETWORK_ERROR',
  SYNC_GIST_NOT_FOUND = 'SYNC_GIST_NOT_FOUND',
  SYNC_RATE_LIMITED = 'SYNC_RATE_LIMITED',

  // 未知错误
  UNKNOWN_ERROR = 'UNKNOWN_ERROR'
}

/**
 * 错误消息映射
 */
const ERROR_MESSAGES: Record<string, string> = {
  [ErrorCode.CONFIG_READ_FAILED]: '配置文件读取失败',
  [ErrorCode.CONFIG_WRITE_FAILED]: '配置文件写入失败',
  [ErrorCode.CONFIG_PARSE_FAILED]: '配置文件格式错误',
  [ErrorCode.CONFIG_NOT_FOUND]: '配置文件不存在',

  [ErrorCode.PRESET_SAVE_FAILED]: '预设保存失败',
  [ErrorCode.PRESET_LOAD_FAILED]: '预设加载失败',
  [ErrorCode.PRESET_DELETE_FAILED]: '预设删除失败',
  [ErrorCode.PRESET_NOT_FOUND]: '预设不存在',

  [ErrorCode.MODEL_ADD_FAILED]: '模型添加失败',
  [ErrorCode.MODEL_UPDATE_FAILED]: '模型更新失败',
  [ErrorCode.MODEL_DELETE_FAILED]: '模型删除失败',
  [ErrorCode.MODEL_NOT_FOUND]: '模型不存在',
  [ErrorCode.MODEL_DUPLICATE]: '该模型 ID 已存在',

  [ErrorCode.FILE_READ_FAILED]: '文件读取失败',
  [ErrorCode.FILE_WRITE_FAILED]: '文件写入失败',

  [ErrorCode.NETWORK_ERROR]: '网络连接失败',
  [ErrorCode.TIMEOUT_ERROR]: '请求超时',

  [ErrorCode.SYNC_NOT_CONFIGURED]: '同步功能未配置',
  [ErrorCode.SYNC_AUTH_FAILED]: 'GitHub 认证失败',
  [ErrorCode.SYNC_TOKEN_EXPIRED]: 'GitHub Token 已过期，请重新登录',
  [ErrorCode.SYNC_UPLOAD_FAILED]: '同步上传失败',
  [ErrorCode.SYNC_DOWNLOAD_FAILED]: '同步下载失败',
  [ErrorCode.SYNC_CONFLICT_DETECTED]: '检测到同步冲突',
  [ErrorCode.SYNC_NETWORK_ERROR]: '同步网络连接失败',
  [ErrorCode.SYNC_GIST_NOT_FOUND]: '同步 Gist 未找到',
  [ErrorCode.SYNC_RATE_LIMITED]: 'GitHub API 请求频率超限',

  [ErrorCode.UNKNOWN_ERROR]: '未知错误'
}

/**
 * 获取错误消息
 * @param error 错误对象或错误码
 * @returns 用户友好的错误消息
 */
export function getErrorMessage(error: unknown): string {
  // 如果是 AppError
  if (error instanceof AppError) {
    // 如果有错误码，尝试获取映射的消息
    if (error.code && ERROR_MESSAGES[error.code]) {
      return ERROR_MESSAGES[error.code]
    }
    return error.message
  }

  // 如果是普通 Error
  if (error instanceof Error) {
    return error.message || '操作失败'
  }

  // 如果是字符串错误码
  if (typeof error === 'string') {
    return ERROR_MESSAGES[error] || error
  }

  return '未知错误'
}

/**
 * 显示错误提示
 * @param error 错误对象
 * @param retry 可选的重试回调函数
 */
export function showError(error: unknown, retry?: () => void): void {
  const message = getErrorMessage(error)
  const appError = error instanceof AppError ? error : null

  // 如果有重试函数或错误自带重试函数，显示确认框
  const retryFn = retry || appError?.retry
  if (retryFn) {
    ElMessageBox.confirm(message, '错误', {
      confirmButtonText: '重试',
      cancelButtonText: '取消',
      type: 'error',
      customClass: 'error-message-box'
    })
      .then(() => {
        retryFn()
      })
      .catch(() => {
        // 用户取消，不做处理
      })
  } else {
    // 普通错误，显示消息提示
    ElMessage.error(message)
  }
}

/**
 * 显示成功提示
 * @param message 成功消息
 */
export function showSuccess(message: string): void {
  ElMessage.success(message)
}

/**
 * 显示警告提示
 * @param message 警告消息
 */
export function showWarning(message: string): void {
  ElMessage.warning(message)
}

/**
 * 显示信息提示
 * @param message 信息消息
 */
export function showInfo(message: string): void {
  ElMessage.info(message)
}

/**
 * 确认对话框
 * @param message 确认消息
 * @param title 标题
 * @returns Promise<boolean> 用户确认返回 true
 */
export async function confirm(
  message: string,
  title: string = '确认'
): Promise<boolean> {
  try {
    await ElMessageBox.confirm(message, title, {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    return true
  } catch {
    return false
  }
}

/**
 * 异步操作包装器
 * 自动捕获错误并显示提示
 * @param fn 要执行的异步函数
 * @param errorMessage 自定义错误消息
 * @param retry 重试函数
 * @returns 函数结果或 null（错误时）
 */
export async function withErrorHandling<T>(
  fn: () => Promise<T>,
  errorMessage?: string,
  retry?: () => void
): Promise<T | null> {
  try {
    return await fn()
  } catch (error) {
    if (errorMessage) {
      showError(errorMessage, retry)
    } else {
      showError(error, retry)
    }
    return null
  }
}

/**
 * 同步操作包装器
 * 自动捕获错误并显示提示
 * @param fn 要执行的同步函数
 * @param errorMessage 自定义错误消息
 * @param retry 重试函数
 * @returns 函数结果或 null（错误时）
 */
export function withSyncErrorHandling<T>(
  fn: () => T,
  errorMessage?: string,
  retry?: () => void
): T | null {
  try {
    return fn()
  } catch (error) {
    if (errorMessage) {
      showError(errorMessage, retry)
    } else {
      showError(error, retry)
    }
    return null
  }
}

/**
 * 创建带有错误码的 AppError
 * @param code 错误码
 * @param message 错误消息（可选，默认使用映射的消息）
 * @param retry 重试函数（可选）
 * @returns AppError 实例
 */
export function createError(
  code: ErrorCode,
  message?: string,
  retry?: () => void
): AppError {
  const msg = message || ERROR_MESSAGES[code] || '操作失败'
  return new AppError(msg, code, retry)
}

/**
 * Vue 全局错误处理器
 * 用于捕获 Vue 组件中的未处理错误
 * @param error 错误对象
 * @param instance Vue 组件实例
 * @param info 错误信息
 */
export function globalErrorHandler(
  error: unknown,
  instance?: any,
  info?: string
): void {
  console.error('全局错误:', error)
  console.error('错误信息:', info)
  console.error('错误堆栈:', error instanceof Error ? error.stack : '无堆栈信息')
  
  // 如果是开发环境，打印更多调试信息
  if (import.meta.env.DEV) {
    console.error('组件实例:', instance)
  }

  // 显示错误提示
  showError(error)
}

/**
 * 设置全局错误处理
 * 在应用初始化时调用
 */
export function setupGlobalErrorHandling(): void {
  // 捕获未处理的 Promise 拒绝
  window.addEventListener('unhandledrejection', (event) => {
    console.error('未处理的 Promise 拒绝:')
    console.error('原因:', event.reason)
    console.error('堆栈:', event.reason?.stack)
    event.preventDefault()
    showError(event.reason)
  })

  // 捕获全局错误
  window.addEventListener('error', (event) => {
    // 忽略 ResizeObserver 的良性警告
    if (event.message?.includes('ResizeObserver loop completed with undelivered notifications')) {
      event.preventDefault()
      return
    }

    console.error('全局错误:')
    console.error('消息:', event.message)
    console.error('文件:', event.filename)
    console.error('行号:', event.lineno)
    console.error('列号:', event.colno)
    console.error('错误对象:', event.error)
    console.error('堆栈:', event.error?.stack)
    // 阻止默认错误处理（如控制台显示错误）
    event.preventDefault()
    showError(event.error)
  })
}
