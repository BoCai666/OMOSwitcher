# OMOSwitcher 安装指南

## 环境要求

| 依赖 | 版本 |
|:----:|:----:|
| Node.js | 18+ |
| Rust | 1.70+ |

## 安装步骤

### 1. 克隆仓库

```bash
git clone https://github.com/BoCai666/OMOSwitcher.git
cd OMOSwitcher
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
