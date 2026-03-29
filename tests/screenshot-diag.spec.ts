import { test } from '@playwright/test'

test.use({ viewport: { width: 1280, height: 800 } })

test('截图诊断 - 所有页面', async ({ page }) => {
  // 赛博朋克主题 - 首页
  await page.goto('/')
  await page.evaluate(() => {
    document.documentElement.className = 'cyberpunk accent-cyan'
  })
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'diag-cyberpunk-home.png', fullPage: true })
  
  // 玻璃拟态主题 - 首页
  await page.evaluate(() => {
    document.documentElement.className = 'glassmorphism'
  })
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'diag-glassmorphism-home.png', fullPage: true })
  
  // 赛博朋克 - 配置页
  await page.goto('/config')
  await page.evaluate(() => {
    document.documentElement.className = 'cyberpunk accent-cyan'
  })
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'diag-cyberpunk-config.png', fullPage: true })
  
  // 玻璃拟态 - 配置页
  await page.evaluate(() => {
    document.documentElement.className = 'glassmorphism'
  })
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'diag-glassmorphism-config.png', fullPage: true })
  
  console.log('Screenshots saved!')
})
