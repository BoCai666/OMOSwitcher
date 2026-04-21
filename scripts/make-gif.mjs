/**
 * 将截图合成为展示 GIF
 * 使用 sharp 处理图片 + gifenc 编码 GIF
 */
import gifenc from 'gifenc'
const { GIFEncoder, quantize, applyPalette } = gifenc
import sharp from 'sharp'
import { readFileSync, mkdirSync, existsSync, writeFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')

// 截图文件列表（使用 exe 真实截图）
const screenshots = [
  'docs/screenshots/exe-01-home.png',
  'docs/screenshots/exe-02-config.png',
  'docs/screenshots/exe-03-models.png',
  'docs/screenshots/exe-04-presets.png',
  'docs/screenshots/exe-05-quota.png',
  'docs/screenshots/exe-06-monitor.png',
]

// GIF 配置
const FRAME_DELAY = 1800  // 每帧显示 1.8 秒
const OUTPUT_PATH = 'docs/omoswitcher-demo.gif'

async function processImage(filePath) {
  const buffer = readFileSync(join(root, filePath))
  // 不缩放，保持原图尺寸
  const { data, info } = await sharp(buffer)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true })

  return { data: new Uint8Array(data), width: info.width, height: info.height }
}

async function main() {
  console.log('开始生成 GIF...')

  // 处理所有截图
  const frames = []
  for (const screenshot of screenshots) {
    const fullPath = join(root, screenshot)
    if (!existsSync(fullPath)) {
      console.warn(`跳过不存在的截图: ${screenshot}`)
      continue
    }
    console.log(`处理: ${screenshot}`)
    const frame = await processImage(screenshot)
    frames.push(frame)
  }

  if (frames.length === 0) {
    console.error('没有找到可用的截图')
    process.exit(1)
  }

  console.log(`共 ${frames.length} 帧，开始编码 GIF...`)

  // 创建 GIF 编码器
  const gif = GIFEncoder()

  for (let i = 0; i < frames.length; i++) {
    const frame = frames[i]
    // 量化为 256 色调色板（返回 Uint8Array 平铺的 RGB 调色板）
    const palette = quantize(frame.data, 256)
    // 将 RGBA 像素映射为调色板索引
    const indexed = applyPalette(frame.data, palette)

    // 第一帧和最后一帧显示久一点
    const delay = (i === 0 || i === frames.length - 1) ? FRAME_DELAY + 500 : FRAME_DELAY
    gif.writeFrame(indexed, frame.width, frame.height, {
      palette,
      delay,
      dispose: 2,
    })
    console.log(`  帧 ${i + 1}/${frames.length} 已写入 (delay=${delay}ms)`)
  }

  gif.finish()

  // 输出文件
  const outputPath = join(root, OUTPUT_PATH)
  const outputDir = dirname(outputPath)
  if (!existsSync(outputDir)) {
    mkdirSync(outputDir, { recursive: true })
  }

  const outputBuffer = gif.bytes()
  writeFileSync(outputPath, Buffer.from(outputBuffer))

  const sizeKB = Math.round(outputBuffer.length / 1024)
  console.log(`\nGIF 已生成: ${OUTPUT_PATH} (${sizeKB} KB)`)
}

main().catch(err => {
  console.error('生成 GIF 失败:', err)
  process.exit(1)
})
