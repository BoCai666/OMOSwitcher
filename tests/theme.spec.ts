import { test, expect } from '@playwright/test'

test.describe('主题系统', () => {
  test('页面加载成功', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('#app')).toBeVisible()
  })
})