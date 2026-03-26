# Vue 前端模块

## 概述

Vue 3 + TypeScript + Vite 前端，使用 Element Plus UI 组件库。

## 目录结构

```
src/
├── main.ts           # 应用入口
├── App.vue           # 根组件
├── views/            # 页面组件 (7个)
├── components/       # 公共组件 (6个)
│   └── layout/       # 布局组件
├── stores/           # Pinia 状态管理 (4个)
├── services/         # 业务服务层 (6个)
├── composables/      # 组合式函数
├── types/            # 类型定义
└── utils/            # 工具函数
```

## 快速定位

| 任务 | 文件 |
|------|------|
| 添加新页面 | `src/views/`, `src/router/index.ts` |
| 添加新组件 | `src/components/`, `src/components/index.ts` |
| 添加新 Store | `src/stores/`, `src/stores/index.ts` |
| 添加新服务 | `src/services/`, `src/services/index.ts` |
| 添加新类型 | `src/types/config.ts`, `src/types/index.ts` |

## 约定

### 组件规范
- 页面组件放在 `views/`
- 公共组件放在 `components/`
- 使用 `<script setup lang="ts">`
- 组件命名: PascalCase

### 状态管理
- 使用 Pinia `defineStore` 组合式 API
- Store 命名: `use{Feature}Store`
- 文件命名: kebab-case (`config.ts`, `preset.ts`)

### 服务层
- 服务命名: camelCase (`configReader`, `presetStore`)
- 每个服务独立文件
- 通过 `index.ts` 统一导出

### 类型定义
- 类型集中在 `types/` 目录
- 使用 `export type` 和 `export interface`
- 常量使用 `export const`

## 测试

测试文件位于 `__tests__/` 目录，使用 Vitest 框架。

```bash
npm run test        # 监听模式
npm run test:run    # 单次运行
```

## 路由

| 路径 | 页面 | 说明 |
|------|------|------|
| `/` | Home.vue | 仪表盘 |
| `/agents` | AgentConfig.vue | Agent 配置 |
| `/categories` | CategoryConfig.vue | Category 配置 |
| `/models` | ModelManage.vue | 模型管理 |
| `/presets` | PresetManage.vue | 预设管理 |

## 核心类型

```typescript
// Agent 名称 (10个)
type AgentName = 'sisyphus' | 'oracle' | 'librarian' | 'explore' | ...

// Category 名称 (8个)
type CategoryName = 'visual-engineering' | 'ultrabrain' | 'deep' | ...

// 配置结构
interface OhMyOpenCodeConfig {
  agents: Record<AgentName, AgentConfig>
  categories: Record<CategoryName, CategoryConfig>
}
```
