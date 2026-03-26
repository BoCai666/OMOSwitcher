# OMOSwitcher

<p align="center">
  <b>OhMyOpenCode 模型配置管理工具</b>
</p>

<p align="center">
  一款用于管理 OpenCode Agent/Category 模型配置、预设和模型列表的桌面应用
</p>

---

## 功能特性

- **Agent 配置管理** - 为每个 Agent（Sisyphus、Oracle、Librarian 等）配置专属模型
- **Category 配置管理** - 为不同任务类别（Deep、Quick、Visual Engineering 等）配置模型
- **预设管理** - 保存、加载、切换配置预设，快速适应不同工作场景
- **模型列表管理** - 自定义可用模型列表，自动从 `opencode.json` 的 Provider 字段读取默认模型
- **一键启动** - 直接启动 OpenCode CLI 并指定工作目录

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 构建工具 | Vite 5 |
| 状态管理 | Pinia |
| 路由 | Vue Router 4 |
| UI 组件 | Element Plus |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust |
| 测试框架 | Vitest |

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm / npm / yarn

### 安装依赖

```bash
# 安装 Node 依赖
npm install

# 安装 Rust 依赖（首次运行时自动安装）
cd src-tauri && cargo build
```

### 开发模式

```bash
# 启动 Tauri 开发模式（前端 + 后端热重载）
npm run tauri:dev

# 仅启动前端开发服务器
npm run dev
```

### 构建发布

```bash
# 构建生产版本
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
OMOSwitcher/
├── src/                      # Vue 前端源码
│   ├── views/               # 页面组件
│   │   ├── AgentConfig.vue  # Agent 配置页
│   │   ├── CategoryConfig.vue # Category 配置页
│   │   ├── ModelManage.vue  # 模型管理页
│   │   └── PresetManage.vue # 预设管理页
│   ├── components/          # 公共组件
│   ├── stores/              # Pinia 状态管理
│   ├── services/            # 业务服务层
│   ├── types/               # TypeScript 类型定义
│   └── utils/               # 工具函数
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── lib.rs           # Tauri 应用入口
│   │   └── commands.rs      # Tauri 命令实现
│   ├── Cargo.toml           # Rust 依赖配置
│   └── tauri.conf.json      # Tauri 应用配置
├── __tests__/               # 测试文件
└── AGENTS.md                # 项目知识库（AI 辅助开发）
```

## 配置文件

应用数据存储在用户配置目录：

| 数据类型 | 路径 |
|---------|------|
| 主配置 | `~/.config/opencode/oh-my-opencode.json` |
| 模型列表 | `~/.config/omoswitcher/models.json` |
| 预设目录 | `~/.config/omoswitcher/presets/` |
| 应用设置 | `~/.config/omoswitcher/settings.json` |

> 主配置文件与 OhMyOpenCode 共享，其他数据存储在独立的 `omoswitcher` 目录。

## 支持的 Agent

| Agent | 说明 |
|-------|------|
| Sisyphus | 默认主编排器，计划、委托并执行复杂任务 |
| Hephaestus | 自主深度工作者 |
| Oracle | 架构决策、代码审查、调试 |
| Librarian | 多仓库分析、文档查找 |
| Explore | 快速代码库探索 |
| Prometheus | 战略规划代理 |
| Metis | 计划顾问 |
| Momus | 计划审查代理 |
| Atlas | 执行 Prometheus 计划 |

## 支持的 Category

| Category | 说明 |
|----------|------|
| Visual Engineering | 前端、UI/UX、设计、样式、动画 |
| Ultra Brain | 深度逻辑推理、复杂架构决策 |
| Deep | 目标导向的自主问题解决 |
| Artistry | 高度创意/艺术性任务 |
| Quick | 琐碎任务 - 单文件更改 |
| Writing | 文档、散文、技术写作 |

## 开发命令

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

## 相关项目

- [OhMyOpenCode](https://github.com/code-yeongyu/oh-my-opencode) - AI 编程助手核心项目
- [OpenCode](https://github.com/opencode-ai/opencode) - OpenCode CLI

## License

[MIT](LICENSE)

---

<p align="center">
  Made with ❤️ for OpenCode users
</p>
