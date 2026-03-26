import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  
  server: {
    port: 1420, // Tauri 默认端口
    strictPort: true,
    host: 'localhost'
  },
  
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true
  }
})
