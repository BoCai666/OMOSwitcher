# OMOSwitcher 安装指南

## 环境要求

| 依赖 | 版本 |
|:----:|:----:|
| Node.js | 18+ |
| Rust | 1.70+ |

## 安装步骤

### 1. 克隆仓库

```bash
git clone https://github.com/BoCai666/OMOSwicther.git
cd OMOSwicther
```

### 2. 安装依赖

```bash
npm install
```

### 3. 运行开发模式

```bash
npm run tauri:dev
```

### 4. 构建生产版本

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
OMOSwitcher/
├── src/                    # Vue 前端源码
│   ├── views/             # 页面组件
│   ├── components/        # 公共组件
│   ├── stores/            # Pinia 状态管理
│   ├── services/          # 业务服务层
│   └── types/             # TypeScript 类型
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── lib.rs         # Tauri 应用入口
│       └── commands.rs    # Tauri 命令
└── __tests__/             # 测试文件
```

## 关键文件

| 文件 | 用途 |
|:-----|:-----|
| `AGENTS.md` | 项目知识库 |
| `src/AGENTS.md` | 前端模块说明 |
| `src-tauri/AGENTS.md` | 后端模块说明 |
| `src/views/AgentConfig.vue` | Agent 配置页面 |
| `src/views/CategoryConfig.vue` | Category 配置页面 |
| `src/services/modelStore.ts` | 模型管理服务 |
| `src-tauri/src/commands.rs` | Tauri 命令实现 |

## 代码规范

- 组合式 API (`setup()`, `defineStore`)
- 中文注释，英文标识符
- 严格 TypeScript (`strict: true`)
- 禁止 `as any`, `@ts-ignore`
- 文件命名: kebab-case
- 组件命名: PascalCase

## 常用命令

| 命令 | 说明 |
|:-----|:-----|
| `npm run dev` | Vite 开发服务器 |
| `npm run tauri:dev` | Tauri 开发模式 |
| `npm run build` | 构建前端 |
| `npm run tauri:build` | 构建生产版本 |
| `npm run test` | 运行测试 |
| `npm run type-check` | TypeScript 类型检查 |

## 配置文件路径

| 数据 | 路径 |
|:-----|:-----|
| 主配置 | `~/.config/opencode/oh-my-opencode.json` |
| 模型列表 | `~/.config/omoswitcher/models.json` |
| 预设目录 | `~/.config/omoswitcher/presets/` |
| 应用设置 | `~/.config/omoswitcher/settings.json` |
