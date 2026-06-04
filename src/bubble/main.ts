// 悬浮球入口文件 - 轻量 Vue 应用，无 Pinia/Router/ElementPlus
import { createApp } from 'vue'
import App from './App.vue'

// 导入全局样式变量（顺序重要）
import '@/styles/variables.css'

const app = createApp(App)
app.mount('#bubble-app')
