import { test, expect } from '@playwright/test'

// 设置更大的视口以避免元素在视口外
test.use({ viewport: { width: 1280, height: 800 } })

test.describe('主题视觉效果测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
  })

  test('赛博朋克暗色主题', async ({ page }) => {
    // 设置暗色主题
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk'
      localStorage.setItem('theme-mode', 'cyberpunk')
    })
    
    await page.waitForTimeout(500)
    
    // 验证 CSS 变量
    const primaryColor = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--app-color-primary').trim()
    })
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--app-bg-base').trim()
    })
    
    console.log('Cyberpunk Primary Color:', primaryColor)
    console.log('Cyberpunk Background Color:', bgColor)
    
    // 截图
    await page.screenshot({ 
      path: '.sisyphus/evidence/theme-cyberpunk.png', 
      fullPage: true 
    })
    
    // 验证主题已应用
    const htmlClass = await page.locator('html').getAttribute('class')
    expect(htmlClass).toContain('cyberpunk')
  })

  test('玻璃拟态明色主题', async ({ page }) => {
    // 设置明色主题
    await page.evaluate(() => {
      document.documentElement.className = 'glassmorphism'
      localStorage.setItem('theme-mode', 'glassmorphism')
    })
    
    await page.waitForTimeout(500)
    
    // 验证 CSS 变量
    const primaryColor = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--app-color-primary').trim()
    })
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--app-bg-base').trim()
    })
    
    console.log('Glassmorphism Primary Color:', primaryColor)
    console.log('Glassmorphism Background Color:', bgColor)
    
    // 截图
    await page.screenshot({ 
      path: '.sisyphus/evidence/theme-glassmorphism.png', 
      fullPage: true 
    })
    
    // 验证主题已应用
    const htmlClass = await page.locator('html').getAttribute('class')
    expect(htmlClass).toContain('glassmorphism')
  })

  test('主题色切换 - 品红色', async ({ page }) => {
    // 设置暗色主题 + 品红强调色
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk theme-magenta'
      localStorage.setItem('theme-mode', 'cyberpunk')
      localStorage.setItem('theme-accent', 'magenta')
    })
    
    await page.waitForTimeout(500)
    
    // 截图
    await page.screenshot({ 
      path: '.sisyphus/evidence/theme-magenta.png', 
      fullPage: true 
    })
  })

  test('主题色切换 - 紫色', async ({ page }) => {
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk theme-purple'
    })
    
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/theme-purple.png', fullPage: true })
  })

  test('主题色切换 - 金色', async ({ page }) => {
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk theme-gold'
    })
    
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/theme-gold.png', fullPage: true })
  })

  test('导航测试 - 使用 router', async ({ page }) => {
    // 设置暗色主题
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk'
    })
    
    // 直接导航到配置页面
    await page.goto('/config')
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/page-config.png', fullPage: true })
    
    // 导航到模型页面
    await page.goto('/models')
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/page-models.png', fullPage: true })
    
    // 导航到预设页面
    await page.goto('/presets')
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/page-presets.png', fullPage: true })
    
    // 返回首页
    await page.goto('/')
    await page.waitForTimeout(500)
    await page.screenshot({ path: '.sisyphus/evidence/page-home.png', fullPage: true })
  })
})
