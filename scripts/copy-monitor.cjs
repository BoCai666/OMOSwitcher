// 复制 monitor.exe 到 Tauri binaries 目录
const fs = require('fs')
const path = require('path')

const src = path.join(__dirname, '../packages/monitor/dist/monitor.exe')
const dest = path.join(__dirname, '../src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe')

fs.copyFileSync(src, dest)
console.log('✅ Copied monitor.exe to Tauri binaries')
