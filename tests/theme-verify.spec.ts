import { test, expect } from '@playwright/test'

test.use({ viewport: { width: 1280, height: 800 } })

test.describe('主题 CSS 变量验证', () => {
  test('赛博朋克主题 CSS 变量', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
    
    // 设置赛博朋克主题
    await page.evaluate(() => {
      document.documentElement.className = 'cyberpunk accent-cyan'
    })
    
    await page.waitForTimeout(300)
    
    // 获取关键 CSS 变量
    const variables = await page.evaluate(() => {
      const style = getComputedStyle(document.documentElement)
      return {
        primary: style.getPropertyValue('--app-color-primary').trim(),
        bgBase: style.getPropertyValue('--app-bg-base').trim(),
        textPrimary: style.getPropertyValue('--app-text-primary').trim(),
        borderActive: style.getPropertyValue('--app-border-active').trim(),
        shadowGlow: style.getPropertyValue('--app-shadow-glow-primary').trim(),
        elPrimary: style.getPropertyValue('--el-color-primary').trim()
      }
    })
    
    console.log('\n=== Cyberpunk Theme Variables ===')
    console.log('Primary Color:', variables.primary)
    console.log('Background:', variables.bgBase)
    console.log('Text Primary:', variables.textPrimary)
    console.log('Border Active:', variables.borderActive)
    console.log('Shadow Glow:', variables.shadowGlow)
    console.log('Element Plus Primary:', variables.elPrimary)
    
    // 验证关键变量
    expect(variables.primary).toBe('#00ffff')
    expect(variables.textPrimary).toBe('#e0e0ff')
    
    // 截图
    await page.screenshot({ path: 'test-cyberpunk.png', fullPage: true })
  })
  
  test('玻璃拟态主题 CSS 变量', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
    
    // 设置玻璃拟态主题
    await page.evaluate(() => {
      document.documentElement.className = 'glassmorphism'
    })
    
    await page.waitForTimeout(300)
    
    // 获取关键 CSS 变量
    const variables = await page.evaluate(() => {
      const style = getComputedStyle(document.documentElement)
      return {
        primary: style.getPropertyValue('--app-color-primary').trim(),
        bgBase: style.getPropertyValue('--app-bg-base').trim(),
        textPrimary: style.getPropertyValue('--app-text-primary').trim(),
        borderActive: style.getPropertyValue('--app-border-active').trim(),
        elPrimary: style.getPropertyValue('--el-color-primary').trim(),
        glassBg: style.getPropertyValue('--app-glass-bg').trim()
      }
    })
    
    console.log('\n=== Glassmorphism Theme Variables ===')
    console.log('Primary Color:', variables.primary)
    console.log('Background:', variables.bgBase)
    console.log('Text Primary:', variables.textPrimary)
    console.log('Border Active:', variables.borderActive)
    console.log('Element Plus Primary:', variables.elPrimary)
    console.log('Glass Background:', variables.glassBg)
    
    // 验证关键变量
    expect(variables.primary).toBe('#2563eb')
    expect(variables.textPrimary).toBe('#1e293b')
    
    // 截图
    await page.screenshot({ path: 'test-glassmorphism.png', fullPage: true })
  })
  
  test('主题色切换效果', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
    
    // 测试所有主题色
    const accents = ['cyan', 'magenta', 'purple', 'gold']
    const expectedColors = {
      cyan: '#00ffff',
      magenta: '#ff00ff',
      purple: '#a855f7',
      gold: '#fbbf24'
    }
    
    for (const accent of accents) {
      await page.evaluate((a) => {
        document.documentElement.className = `cyberpunk accent-${a}`
      }, accent)
      
      await page.waitForTimeout(200)
      
      const accentPrimary = await page.evaluate(() => {
        return getComputedStyle(document.documentElement)
          .getPropertyValue('--app-accent-primary').trim()
      })
      
      console.log(`Accent ${accent}: ${accentPrimary}`)
      expect(accentPrimary).toBe(expectedColors[accent])
    }
  })
})
