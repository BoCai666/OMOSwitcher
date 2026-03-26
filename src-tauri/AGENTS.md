# Tauri 后端模块

## 概述

Tauri 2 (Rust) 桌面应用后端，提供配置文件读写和 OpenCode 启动功能。

## 目录结构

```
src-tauri/
├── Cargo.toml          # Rust 依赖配置
├── tauri.conf.json     # Tauri 应用配置
├── capabilities/       # 权限配置
├── icons/              # 应用图标
└── src/
    ├── main.rs         # 程序入口
    ├── lib.rs          # 库入口，Tauri Builder 配置
    └── commands.rs     # Tauri 命令实现
```

## 命令列表

| 命令 | 功能 |
|------|------|
| `read_config` | 读取主配置文件 |
| `write_config` | 写入主配置文件 |
| `list_presets` | 列出所有预设 |
| `read_preset` | 读取指定预设 |
| `save_preset` | 保存预设 |
| `delete_preset` | 删除预设 |
| `read_models` | 读取模型列表 |
| `write_models` | 写入模型列表 |
| `read_settings` | 读取应用设置 |
| `write_settings` | 写入应用设置 |
| `launch_opencode` | 启动 opencode CLI |

## 存储路径

```rust
// 主配置文件（与 OhMyOpenCode 共享）
fn get_opencode_dir() -> ~/.config/opencode/
fn get_config_path()   -> ~/.config/opencode/oh-my-opencode.json

// OMOSwitcher 数据（独立存储）
fn get_omoswitcher_dir() -> ~/.config/omoswitcher/
fn get_presets_dir()     -> ~/.config/omoswitcher/presets/
fn get_models_path()     -> ~/.config/omoswitcher/models.json
fn get_settings_path()   -> ~/.config/omoswitcher/settings.json
```

使用 `dirs::home_dir()` 动态获取用户主目录，支持 Windows/Linux/macOS。

## 开发命令

```bash
# Tauri 开发模式
npm run tauri:dev

# Tauri 生产构建
npm run tauri:build

# 直接运行 Cargo
cd src-tauri && cargo build
```

## 约定

### 命令定义
- 使用 `#[tauri::command]` 宏
- 返回 `Result<T, String>` 类型
- 错误消息使用中文

### 依赖
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## 注意事项

1. **跨平台路径**: 使用 `dirs::home_dir()` 动态获取用户主目录
2. **错误处理**: 使用 `map_err` 转换为用户友好的中文错误消息
3. **权限配置**: `capabilities/default.json` 定义应用权限
4. **数据分离**: 主配置与 OhMyOpenCode 共享，其他数据独立存储
