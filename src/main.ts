import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
// Element Plus 样式异步加载，不阻塞渲染
import 'element-plus/dist/index.css'

import App from './App.vue'
import router from './router'
import { globalErrorHandler, setupGlobalErrorHandling } from './utils/errorHandler'

// 设计系统样式引入（顺序重要）
import '@/styles/variables.css'
import '@/styles/element-override.css'
// 新主题系统
import '@/styles/themes/cyberpunk.css'
import '@/styles/themes/glassmorphism.css'
// 主题色预设
import '@/styles/theme-colors/cyan.css'
import '@/styles/theme-colors/magenta.css'
import '@/styles/theme-colors/purple.css'
import '@/styles/theme-colors/gold.css'
// 特效系统
import '@/styles/effects/neon.css'
import '@/styles/effects/glitch.css'
import '@/styles/effects/scanlines.css'
import '@/styles/effects/grid.css'
// 全局样式
import '@/styles/global.css'

// 主题初始化 - 必须在 CSS 加载后、应用挂载前导入
// 这会触发 useTheme.ts 中的 IIFE，立即设置 html 元素的 light/dark class
import '@/composables/useTheme'

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

// 全局注册 Element Plus 图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

// 挂载应用
app.mount('#app')

// 立即移除预加载动画
const preloadEl = document.getElementById('preload-loading')
if (preloadEl) {
  preloadEl.style.opacity = '0'
  preloadEl.style.transition = 'opacity 0.3s ease'
  setTimeout(() => {
    preloadEl.remove()
  }, 300)
}
