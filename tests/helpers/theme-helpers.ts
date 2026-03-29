import { Page } from '@playwright/test'

/** 设置主题模式 */
export async function setThemeMode(page: Page, mode: 'dark' | 'light') {
  await page.evaluate((m) => {
    document.documentElement.classList.remove('dark', 'light')
    document.documentElement.classList.add(m)
  }, mode)
}

/** 获取 CSS 变量值 */
export async function getCSSVariable(page: Page, name: string): Promise<string> {
  return await page.evaluate((n) => {
    return getComputedStyle(document.documentElement).getPropertyValue(n).trim()
  }, name)
}