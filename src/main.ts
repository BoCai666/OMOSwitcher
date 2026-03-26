import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import 'element-plus/dist/index.css'
import App from './App.vue'
import router from './router'
import { globalErrorHandler, setupGlobalErrorHandling } from './utils/errorHandler'

// 设置全局错误处理（捕获未处理的 Promise 拒绝和全局错误）
setupGlobalErrorHandling()

// 创建 Vue 应用实例
const app = createApp(App)

// 配置 Vue 全局错误处理器
app.config.errorHandler = globalErrorHandler

// 配置 Vue 警告处理器（仅在开发环境）
if (import.meta.env.DEV) {
  app.config.warnHandler = (msg, instance, trace) => {
    console.warn('Vue 警告:', msg)
    console.warn('组件实例:', instance)
    console.warn('追踪:', trace)
  }
}

// 使用 Pinia 状态管理
app.use(createPinia())

// 使用 Vue Router
app.use(router)

// 使用 Element Plus UI 组件库（中文语言包）
app.use(ElementPlus, { locale: zhCn })

// 挂载应用
app.mount('#app')
