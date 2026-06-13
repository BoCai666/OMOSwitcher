// 悬浮球入口文件 - 轻量 Vue 应用（无 Router/ElementPlus）
// 注意：Tauri 2 的两个 webview 各自有独立 JS runtime，
//       Pinia store 不能跨 webview 共享；这里只挂 Pinia 是为了 useQuotaStore 能工作
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

// 导入全局样式变量（顺序重要）
import '@/styles/variables.css'

const app = createApp(App)
app.use(createPinia())
app.mount('#bubble-app')
