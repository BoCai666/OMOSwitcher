import { createPinia } from 'pinia'

// 创建 Pinia 实例
const pinia = createPinia()

export default pinia

// 导出所有 stores
export * from './config'
export * from './preset'
export * from './model'

// 统一导出 store hooks
export { useConfigStore } from './config'
export { usePresetStore } from './preset'
export { useModelStore } from './model'
