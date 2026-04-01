<div align="center">

# OMOSwitcher

**OhMyOpenCode 模型配置管理工具**

一款优雅的桌面应用，用于管理 OpenCode 的 Agent/Category 模型配置、预设和模型列表

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.4-4FC08D?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.3-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

[功能特性](#-功能特性) • [快速开始](#-快速开始) • [配置说明](#-配置说明) • [开发指南](#-开发指南)

</div>

---

## ✨ 功能特性

<table>
<tr>
<td width="50%">

### 🤖 Agent 配置管理

为每个 Agent 配置专属模型：
- Sisyphus - 主编排器
- Oracle - 架构决策
- Librarian - 文档查找
- Explore - 代码探索
- 更多...

</td>
<td width="50%">

### 📂 Category 配置管理

为不同任务类别配置模型：
- Visual Engineering - 前端/UI
- Deep - 深度问题解决
- Quick - 琐碎任务
- Writing - 文档写作
- 更多...

</td>
</tr>
<tr>
<td width="50%">

### 🎯 模型列表管理

- 自定义可用模型
- 自动读取 `opencode.json` Provider 字段
- 支持 provider/model-name 格式

</td>
<td width="50%">

### 📊 LLM API 监控

- 实时监控 API 调用
- Token 消耗统计
- 费用估算
- 请求历史记录

</td>
</tr>
</table>

---

## 🚀 快速开始

### 👨‍💻 For Humans

**环境要求**

| 依赖 | 版本 |
|:----:|:----:|
| Node.js | 18+ |
| Rust | 1.70+ |

**安装运行**

```bash
# 克隆仓库
git clone https://github.com/BoCai666/OMOSwitcher.git
cd OMOSwitcher

# 安装依赖
npm install

# 开发模式
npm run tauri:dev

# 构建发布版
npm run tauri:build
```

### 🤖 For LLM Agents

```
Install and configure OMOSwitcher by following the instructions here:
https://raw.githubusercontent.com/BoCai666/OMOSwitcher/main/docs/installation.md
```

---

## ⚙️ 配置说明

应用数据存储位置：

| 数据 | 路径 |
|:-----|:-----|
| 主配置 | `~/.config/opencode/oh-my-opencode.json` |
| 模型列表 | `~/.config/omoswitcher/models.json` |
| 预设目录 | `~/.config/omoswitcher/presets/` |
| 应用设置 | `~/.config/omoswitcher/settings.json` |

> 💡 主配置文件与 OhMyOpenCode 共享，其他数据存储在独立的 `omoswitcher` 目录

---

## 🛠️ 开发指南

### 命令一览

| 命令 | 说明 |
|:-----|:-----|
| `npm run tauri:dev` | Tauri 开发模式（热重载） |
| `npm run tauri:build` | 构建生产版本 |
| `npm run build` | 构建前端 |
| `npm run type-check` | TypeScript 类型检查 |
| `npm run kill-port` | 清理 1420 端口（Windows） |

### 技术栈

<table>
<tr>
<th align="center">层级</th>
<th align="center">技术</th>
</tr>
<tr>
<td align="center">前端框架</td>
<td align="center">

![Vue](https://img.shields.io/badge/Vue-3.4-4FC08D?style=flat-square&logo=vue.js&logoColor=white)

</td>
</tr>
<tr>
<td align="center">构建工具</td>
<td align="center">

![Vite](https://img.shields.io/badge/Vite-5.0-646CFF?style=flat-square&logo=vite&logoColor=white)

</td>
</tr>
<tr>
<td align="center">状态管理</td>
<td align="center">

![Pinia](https://img.shields.io/badge/Pinia-2.1-FFD859?style=flat-square)

</td>
</tr>
<tr>
<td align="center">UI 组件</td>
<td align="center">

![Element Plus](https://img.shields.io/badge/Element_Plus-2.5-409EFF?style=flat-square)

</td>
</tr>
<tr>
<td align="center">桌面框架</td>
<td align="center">

![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8?style=flat-square&logo=tauri&logoColor=white)

</td>
</tr>
<tr>
<td align="center">后端语言</td>
<td align="center">

![Rust](https://img.shields.io/badge/Rust-1.70+-DEA584?style=flat-square&logo=rust&logoColor=white)

</td>
</tr>
</table>

---

## 🤝 相关项目

<table>
<tr>
<td align="center" width="50%">

### [OhMyOpenCode](https://github.com/code-yeongyu/oh-my-opencode)

AI 编程助手核心项目

</td>
<td align="center" width="50%">

### [OpenCode](https://github.com/opencode-ai/opencode)

OpenCode CLI 工具

</td>
</tr>
</table>

---

<div align="center">

## 📄 License

[MIT](LICENSE) © 2025

---

**Made with ❤️ for OpenCode users**

</div>
