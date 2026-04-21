/**
 * 截取 OMOSwitcher 桌面应用窗口
 * 用法: node scripts/capture-exe.mjs [输出文件名]
 * 例:   node scripts/capture-exe.mjs exe-home
 */
import screenshot from 'screenshot-desktop'
import activeWin from 'active-win'
import sharp from 'sharp'
import { writeFileSync, mkdirSync, existsSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { execSync } from 'child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')
const outDir = join(root, 'docs', 'screenshots')
const name = process.argv[2] || 'capture'

async function main() {
  if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true })

  // 激活 OMOSwitcher 窗口到前台
  console.log('激活 OMOSwitcher 窗口...')
  try {
    execSync(
      `powershell -Command "` +
      `$p = Get-Process omoswitcher -EA 0 | Where-Object {$_.MainWindowTitle -ne ''} | Select-Object -First 1;` +
      `if ($p) { [void][System.Windows.Forms.Screen]; Add-Type -MemberDefinition '[DllImport(\\"user32.dll\\")]public static extern bool SetForegroundWindow(IntPtr h);[DllImport(\\"user32.dll\\")]public static extern bool ShowWindow(IntPtr h,int n);' -Name U32 -Namespace W -PassThru | Out-Null; [W.U32]::ShowWindow($p.MainWindowHandle,9)|Out-Null; Start-Sleep -Ms 300; [W.U32]::SetForegroundWindow($p.MainWindowHandle)|Out-Null; Start-Sleep -Ms 500 }"`,
      { stdio: 'pipe' }
    )
  } catch {}

  // 截取全屏
  console.log('截取屏幕...')
  const imgBuffer = await screenshot({ format: 'png' })

  // 获取 OMOSwitcher 窗口信息 - 通过 powershell 获取
  console.log('获取窗口位置...')
  let bounds = null
  try {
    const result = execSync(
      `powershell -Command "Add-Type -MemberDefinition '[DllImport(\\"user32.dll\\")]public static extern bool GetWindowRect(IntPtr h,out RECT r);[StructLayout(LayoutKind.Sequential)]public struct RECT{public int L,T,R,B;}' -Name U2 -Namespace W2 -PassThru | Out-Null; $p = Get-Process omoswitcher -EA 0 | Where-Object {$_.MainWindowTitle -ne ''} | Select-Object -First 1; if ($p) { $r = New-Object W2.U2+RECT; [W2.U2]::GetWindowRect($p.MainWindowHandle,[ref]$r)|Out-Null; Write-Output \\"$($r.L),$($r.T),$($r.R),$($r.B)\\" }"`,
      { encoding: 'utf-8' }
    ).trim()
    const [left, top, right, bottom] = result.split(',').map(Number)
    if (right > left && bottom > top) {
      bounds = { left, top, width: right - left, height: bottom - top }
    }
  } catch (e) {
    console.warn('获取窗口位置失败:', e.message)
  }

  if (!bounds) {
    // 回退: 直接保存全屏截图
    const outPath = join(outDir, `${name}.png`)
    writeFileSync(outPath, imgBuffer)
    console.log(`已保存全屏截图: ${outPath}`)
    return
  }

  // 去掉窗口阴影/边框 (Windows 10/11 窗口阴影约 7-10px)
  const PAD = 8
  const cropLeft = Math.max(0, bounds.left + PAD)
  const cropTop = Math.max(0, bounds.top + PAD)
  const cropW = Math.max(1, bounds.width - PAD * 2)
  const cropH = Math.max(1, bounds.height - PAD * 2)
  console.log(`裁剪区域: left=${cropLeft} top=${cropTop} ${cropW}x${cropH}`)

  // 裁剪到窗口区域
  const cropped = await sharp(imgBuffer)
    .extract({
      left: cropLeft,
      top: cropTop,
      width: cropW,
      height: cropH,
    })
    .png()
    .toBuffer()

  const outPath = join(outDir, `${name}.png`)
  writeFileSync(outPath, cropped)
  const sizeKB = Math.round(cropped.length / 1024)
  console.log(`已保存: ${outPath} (${sizeKB} KB)`)
}

main().catch(err => {
  console.error('截图失败:', err)
  process.exit(1)
})
