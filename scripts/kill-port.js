// 启动前清理指定端口
// 使用方法: node scripts/kill-port.js 1420

import { exec } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)
const port = process.argv[2] || 1420

async function killPortWindows(port) {
  try {
    // 先查找占用端口的进程
    const { stdout } = await execAsync(`netstat -ano | findstr :${port}`)
    
    if (!stdout) return

    // 解析 PID
    const lines = stdout.split('\n')
    const pids = new Set()

    lines.forEach(line => {
      const match = line.match(/\s+(\d+)\s*$/)
      if (match && match[1] !== '0') {
        pids.add(match[1])
      }
    })

    if (pids.size === 0) return

    // 终止所有占用端口的进程
    for (const pid of pids) {
      try {
        await execAsync(`taskkill /PID ${pid} /F`)
        console.log(`已终止进程 PID: ${pid}`)
      } catch {
        // 忽略终止失败的进程
      }
    }
  } catch {
    // 没有找到占用端口的进程，忽略
  }
}

async function killPortUnix(port) {
  try {
    await execAsync(`lsof -ti:${port} | xargs kill -9 2>/dev/null || true`)
    console.log(`已清理端口 ${port}`)
  } catch {
    // 忽略错误
  }
}

async function main() {
  const isWindows = process.platform === 'win32'

  if (isWindows) {
    await killPortWindows(port)
  } else {
    await killPortUnix(port)
  }

  console.log(`端口 ${port} 已准备就绪`)
}

main()
