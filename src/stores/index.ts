import { createPinia } from 'pinia'

// 创建 Pinia 实例
const pinia = createPinia()

export default pinia

// 导出所有 stores
export * from './config'

// 统一导出 store hooks
export { useConfigStore } from './config'
