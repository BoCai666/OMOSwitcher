# OMOSwitcher 项目知识库

**生成时间:** 2026-03-25
**项目类型:** Tauri 2 桌面应用 (Vue 3 + Rust)

## 概述

OhMyOpenCode 模型配置管理工具，用于管理 Agent/Category 模型配置、预设和模型列表。前端 Vue 3 + Element Plus，后端 Tauri 2 (Rust)。

## 目录结构

```
OMOSwitcher/
├── src/                    # Vue 前端源码
│   ├── views/             # 页面组件
│   ├── components/        # 公共组件
│   ├── stores/            # Pinia 状态管理
│   ├── services/          # 业务服务层
│   ├── composables/       # 组合式函数
│   ├── types/             # 类型定义
│   └── utils/             # 工具函数
├── src-tauri/              # Rust 后端
│   └── src/               # Rust 源码
│       ├── lib.rs         # Tauri 应用入口
│       └── commands.rs    # Tauri 命令
└── __tests__/             # 测试文件
```

## 快速定位

| 任务 | 位置 |
|------|------|
| 修改 Agent 配置页面 | `src/views/AgentConfig.vue` |
| 修改 Category 配置页面 | `src/views/CategoryConfig.vue` |
| 修改预设管理 | `src/views/PresetManage.vue`, `src/services/presetStore.ts` |
| 修改模型管理 | `src/views/ModelManage.vue`, `src/services/modelStore.ts` |
| 修改配置存储逻辑 | `src/stores/config.ts` |
| 添加 Tauri 命令 | `src-tauri/src/commands.rs` |
| 修改窗口配置 | `src-tauri/tauri.conf.json` |

## 命令

```bash
# 开发
npm run dev              # Vite 开发服务器 (端口 1420)
npm run tauri:dev        # Tauri 开发模式

# 构建
npm run build            # 前端构建
npm run tauri:build      # Tauri 生产构建

# 测试
npm run test             # Vitest 监听模式
npm run test:run         # Vitest 单次运行
npm run type-check       # TypeScript 类型检查
```

## 约定

### 命名
- **文件**: kebab-case (`config-reader.ts`, `PresetDialog.vue`)
- **组件/类型**: PascalCase
- **变量/函数**: camelCase
- **常量**: UPPER_SNAKE_CASE
- **路径别名**: `@/` 指向 `src/`

### 代码风格
- 组合式 API (`setup()`, `defineStore`)
- 中文注释，英文标识符
- 严格 TypeScript (`strict: true`, `noUnusedLocals`)

### 配置文件路径

| 数据类型 | 路径 |
|---------|------|
| 主配置 | `~/.config/opencode/oh-my-opencode.json` |
| 模型列表 | `~/.config/omoswitcher/models.json` |
| 预设目录 | `~/.config/omoswitcher/presets/` |
| 应用设置 | `~/.config/omoswitcher/settings.json` |

> 主配置与 OhMyOpenCode 共享，其他数据存储在独立的 omoswitcher 目录

## 反模式

| 禁止 | 原因 |
|------|------|
| `as any`, `@ts-ignore` | 破坏类型安全 |
| 空catch块 | 隐藏错误 |
| 删除失败测试 | 掩盖问题 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite |
| 状态 | Pinia |
| 路由 | Vue Router 4 |
| UI | Element Plus |
| 桌面 | Tauri 2 |
| 后端 | Rust |
| 测试 | Vitest |

## 注意事项

1. **无 CI/CD**: 项目当前无 GitHub Actions 或其他自动化流水线
2. **无 ESLint/Prettier**: 代码风格依赖手动维护
3. **非 Git 仓库**: 当前目录未初始化 Git
4. **跨平台路径**: 使用 `dirs::home_dir()` 动态获取用户主目录，支持 Windows/Linux/macOS
