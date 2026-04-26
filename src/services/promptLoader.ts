/**
 * 系统提示词加载服务
 * 所有提示词以 TypeScript 常量形式存储在 src/prompts/ 目录下
 * 编译时打包进 bundle，运行时同步加载
 */

export type PromptLang = 'zh' | 'en'

// Agent 提示词导入
import { SYSTEM_PROMPT as sisyphusZh } from '../prompts/agents/sisyphus.zh'
import { SYSTEM_PROMPT as sisyphusEn } from '../prompts/agents/sisyphus.en'
import { SYSTEM_PROMPT as hephaestusZh } from '../prompts/agents/hephaestus.zh'
import { SYSTEM_PROMPT as hephaestusEn } from '../prompts/agents/hephaestus.en'
import { SYSTEM_PROMPT as oracleZh } from '../prompts/agents/oracle.zh'
import { SYSTEM_PROMPT as oracleEn } from '../prompts/agents/oracle.en'
import { SYSTEM_PROMPT as librarianZh } from '../prompts/agents/librarian.zh'
import { SYSTEM_PROMPT as librarianEn } from '../prompts/agents/librarian.en'
import { SYSTEM_PROMPT as exploreZh } from '../prompts/agents/explore.zh'
import { SYSTEM_PROMPT as exploreEn } from '../prompts/agents/explore.en'
import { SYSTEM_PROMPT as metisZh } from '../prompts/agents/metis.zh'
import { SYSTEM_PROMPT as metisEn } from '../prompts/agents/metis.en'
import { SYSTEM_PROMPT as momusZh } from '../prompts/agents/momus.zh'
import { SYSTEM_PROMPT as momusEn } from '../prompts/agents/momus.en'
import { SYSTEM_PROMPT as atlasZh } from '../prompts/agents/atlas.zh'
import { SYSTEM_PROMPT as atlasEn } from '../prompts/agents/atlas.en'
import { SYSTEM_PROMPT as prometheusZh } from '../prompts/agents/prometheus.zh'
import { SYSTEM_PROMPT as prometheusEn } from '../prompts/agents/prometheus.en'
import { SYSTEM_PROMPT as multimodalLookerZh } from '../prompts/agents/multimodal-looker.zh'
import { SYSTEM_PROMPT as multimodalLookerEn } from '../prompts/agents/multimodal-looker.en'

// Category 提示词导入
import { SYSTEM_PROMPT as visualEngineeringZh } from '../prompts/categories/visual-engineering.zh'
import { SYSTEM_PROMPT as visualEngineeringEn } from '../prompts/categories/visual-engineering.en'
import { SYSTEM_PROMPT as ultrabrainZh } from '../prompts/categories/ultrabrain.zh'
import { SYSTEM_PROMPT as ultrabrainEn } from '../prompts/categories/ultrabrain.en'
import { SYSTEM_PROMPT as deepZh } from '../prompts/categories/deep.zh'
import { SYSTEM_PROMPT as deepEn } from '../prompts/categories/deep.en'
import { SYSTEM_PROMPT as artistryZh } from '../prompts/categories/artistry.zh'
import { SYSTEM_PROMPT as artistryEn } from '../prompts/categories/artistry.en'
import { SYSTEM_PROMPT as quickZh } from '../prompts/categories/quick.zh'
import { SYSTEM_PROMPT as quickEn } from '../prompts/categories/quick.en'
import { SYSTEM_PROMPT as unspecifiedLowZh } from '../prompts/categories/unspecified-low.zh'
import { SYSTEM_PROMPT as unspecifiedLowEn } from '../prompts/categories/unspecified-low.en'
import { SYSTEM_PROMPT as unspecifiedHighZh } from '../prompts/categories/unspecified-high.zh'
import { SYSTEM_PROMPT as unspecifiedHighEn } from '../prompts/categories/unspecified-high.en'
import { SYSTEM_PROMPT as writingZh } from '../prompts/categories/writing.zh'
import { SYSTEM_PROMPT as writingEn } from '../prompts/categories/writing.en'

const agentPrompts: Record<string, Record<string, string>> = {
  sisyphus: { zh: sisyphusZh, en: sisyphusEn },
  hephaestus: { zh: hephaestusZh, en: hephaestusEn },
  oracle: { zh: oracleZh, en: oracleEn },
  librarian: { zh: librarianZh, en: librarianEn },
  explore: { zh: exploreZh, en: exploreEn },
  metis: { zh: metisZh, en: metisEn },
  momus: { zh: momusZh, en: momusEn },
  atlas: { zh: atlasZh, en: atlasEn },
  prometheus: { zh: prometheusZh, en: prometheusEn },
  'multimodal-looker': { zh: multimodalLookerZh, en: multimodalLookerEn },
}

const categoryPrompts: Record<string, Record<string, string>> = {
  'visual-engineering': { zh: visualEngineeringZh, en: visualEngineeringEn },
  ultrabrain: { zh: ultrabrainZh, en: ultrabrainEn },
  deep: { zh: deepZh, en: deepEn },
  artistry: { zh: artistryZh, en: artistryEn },
  quick: { zh: quickZh, en: quickEn },
  'unspecified-low': { zh: unspecifiedLowZh, en: unspecifiedLowEn },
  'unspecified-high': { zh: unspecifiedHighZh, en: unspecifiedHighEn },
  writing: { zh: writingZh, en: writingEn },
}

/**
 * 加载系统提示词
 * @param type 'agent' | 'category'
 * @param name Agent 或 Category 名称
 * @param lang 语言: 'zh' | 'en'
 * @returns 提示词文本
 */
export function loadSystemPrompt(
  type: 'agent' | 'category',
  name: string,
  lang: PromptLang = 'zh'
): string {
  const prompts = type === 'agent' ? agentPrompts : categoryPrompts
  const prompt = prompts[name]?.[lang]

  if (prompt === undefined) {
    const error = `提示词未找到: ${type}/${name}.${lang}`
    console.error(error)
    return `// ${error}`
  }

  return prompt
}

/**
 * 预加载所有提示词（实际上已经通过 import 预加载了）
 */
export function preloadAllPrompts(): void {
  console.log(
    `已预加载 ${Object.keys(agentPrompts).length} 个 Agent 提示词和 ${Object.keys(categoryPrompts).length} 个 Category 提示词`
  )
}

/**
 * 获取可用的提示词列表
 */
export function getAvailablePrompts(): {
  agents: string[]
  categories: string[]
} {
  return {
    agents: Object.keys(agentPrompts),
    categories: Object.keys(categoryPrompts),
  }
}
