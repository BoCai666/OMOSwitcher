# src-tauri 目录占位

此目录用于存放 Tauri 后端代码。

## 说明

由于当前系统未安装 Rust，暂时无法创建完整的 Tauri 项目。

待 Rust 安装后，将执行以下步骤：

1. 安装 Tauri CLI: `cargo install tauri-cli`
2. 初始化 Tauri: `cargo tauri init`
3. 配置 tauri.conf.json
4. 实现 Rust 后端逻辑

## 预期目录结构

```
src-tauri/
├── src/
│   ├── main.rs        # 主入口
│   └── lib.rs         # 库文件
├── Cargo.toml         # Rust 依赖配置
├── tauri.conf.json    # Tauri 配置
└── icons/             # 应用图标
```
