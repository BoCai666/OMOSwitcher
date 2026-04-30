# Changelog

All notable changes to this project will be documented in this file.

## [2.0.10] - 2026-04-30

### 新增功能

- **供应商品牌图标全面更新** - 7 家供应商官方 Logo 和品牌色更新
  - Kimi / Moonshot / Kimi Code：官方 K mark 黑色 logo
  - MiniMax：官方 abstract M mark，粉红→洋红渐变
  - DeepSeek：官方鲸鱼 logo，品牌色 #4d6bfe
  - 智谱 AI：官方 Z mark logo
  - OpenCode Go：官方 O 框架 logo，深炭黑 #211E1E
  - 新增三层供应商匹配逻辑（精确 → kebab-case → 关键词包含）

- **新增无问芯穹供应商元数据** - 品牌色 #7018E0，7 叶放射花瓣图标

### 优化改进

- **暗色品牌供应商主题适配** - ProviderMetadata 新增 darkColor 字段，暗色模式下暗色品牌供应商卡片使用灰色替代色，QuotaCard 新增 providerEffectiveColor 主题感知颜色
- **预设管理页当前预设标识优化** - 移除绿点标识，改用左侧 3px 主题色竖线 + 背景渐变 + 名称加粗变色，不占表格内容空间
- **OpenCode Go 设置弹窗交互优化** - 支持点击背景关闭 + 最小尺寸适配

---

## [2.0.9] - 2026-04-29

### 新增功能

- **OpenCode Go 额度查询** - 新增 OpenCode Go 供应商额度查询支持
  - 网页抓取 Dashboard HTML + SolidJS SSR 水合数据解析
  - 三维度用量展示：5 小时滚动窗口、周额度、月额度
  - 凭证存储于 `settings.json`（非 `opencode.json`），支持 workspaceId/cookie 配置
  - QuotaCard 上新增齿轮按钮，独立弹窗配置参数

### 优化改进

- **额度页渐进式刷新** - 首次加载不再预填充骨架卡片，查询成功的卡片逐个弹出，无需等待全部完成
- **DeepSeek 纯余额卡片优化** - 修复 API 返回字符串数值解析失败，卡片显示余额/100% 进度条，详情弹窗精简
- **模型配置页布局对齐** - tab-header 高度 48px 与 tab 页签对齐，max-width 缩小至 1000px 减少右侧留白
- **额度查询过滤逻辑** - 查询失败/error 卡片不再被隐藏，unsupported 供应商自动跳过

### Bug 修复

- **修复预设页刷新按钮紫色溢出** - 移除 `overflow: hidden`，暗色模式下点击刷新不再出现紫色向左溢出
- **修复预设页刷新按钮转圈动画** - 使用 `:deep()` + `@keyframes` 自定义旋转动画，保证至少 600ms 可见时长

---

## [2.0.8] - 2026-04-26

### 新增功能

- **完整系统提示词迁移** - 36 个 Agent/Category 提示词从硬编码迁移至独立 TS 文件
  - 新增 `src/prompts/` 目录，10 Agent + 8 Category 各含中英文完整版
  - 新增 `promptLoader.ts` 同步加载服务，编译时打包，消除语言切换闪烁
  - 详情页显示完整提示词（原摘要版），支持中英切换
- **详情页样式优化** - 描述改为中文，"当前模型"标题强制单行，模型外边框贴合内容

---

## [2.0.7] - 2026-04-24

### Bug 修复

- **修复 MiniMax 卡片详情误判** - MiniMax 额度卡片详情被错误识别为 Kimi Code 的问题
- **修复 Kimi Code 5小时额度显示** - 5小时额度数据缺失时正确显示 0/limit，不再错误回退为周额度

---

## [2.0.6] - 2026-04-23

### 新增功能

- **Kimi Code 额度查询** - 新增 Kimi for Coding Plan 供应商额度查询支持
  - 调用 `api.kimi.com/coding/v1/usages` 接口获取额度数据
  - 卡片展示 5 小时滚动窗口额度使用情况
  - 详情弹窗支持三级额度展示：5 小时滚动窗口、周额度、月额度
  - 修复 provider_id `kimi-for-coding` 与 `moonshot` 的匹配冲突

### 优化改进

- 清理额度查询模块调试日志，移除不必要的 `tracing::info!` 输出

---

## [2.0.5] - 2026-04-20

### 新增功能

- **供应商额度查询模块** - 全新的 LLM 供应商余额/配额查询功能
  - 支持 8 个供应商：智谱、MiniMax、DeepSeek、Moonshot、OpenRouter、SiliconFlow、Infini
  - 并发查询，统一返回格式
  - 智谱额度详情弹窗支持今日/近 7 天 tab 切换展示

- **额度管理页面** - QuotaView.vue
  - 供应商余额卡片展示
  - 额度格式化组合式函数 (useQuotaFormatter)
  - 供应商品牌色/图标元数据 (providerMetadata)

- **Commands 模块拆分** - commands.rs 拆分为 7 个子模块
  - config / launch / model / monitor_service / preset / settings
  - pub use re-export 保持调用路径不变

### Bug 修复

- **修复请求详情弹窗双滚动条** - Monitor.vue 弹窗外层 overflow 修正
- **修复消息列表样式不一致** - MessageList.vue 补全四套主题样式
- **修复远程同步后预设不生效** - sync.ts 新增 reapplyCurrentPreset，下载/同步/冲突解决后自动重新应用当前预设
- **移除 MCP 误判逻辑** - 删除 OpenAI Function Calling 格式的 MCP 检测，仅保留 JSON-RPC 2.0 和 URL 路径匹配

### 构建改进

- **OAuth Secret 编译时强制检查** - OAUTH_CLIENT_SECRET 环境变量未设置时编译失败
- **CI 环境变量重命名** - 避免 GitHub 保留的 GITHUB_ 前缀冲突
- **额度详情弹窗 tab 样式美化** - 修复弹窗滚动条缺失

---

## [2.0.4] - 2026-04-19

### 新增功能

- **智能热重载** - 保存配置后自动等待活跃会话空闲再推送
  - 检测 OpenCode Server 活跃会话，轮询等待（最长 60 秒）
  - 空闲后自动执行 dispose + rebuild，插件重新加载配置
  - 超时跳过并提示用户，避免中断工作中的会话
  - TUI Toast 提示重载状态（等待/进行中）

- **统一日志系统** - Rust 后端全面接入 tracing 框架
  - 本地时间格式，精确到秒
  - 彩色日志级别，去掉 crate 名前缀
  - Debug/Release 分级输出

- **额度管理改进**
  - 智谱额度详情弹窗支持 7 日用量图表
  - 额度管理页面主题适配和供应商图标美化

- **模型推理能力写入** - 推理模型自动写入 thinking 配置

- **预设缓存刷新** - 切换预设后自动刷新缓存

### Bug 修复

- **修复热重载不生效** - 用 dispose + lazy rebuild 替代 PATCH /config，绕过上游 bug
- **修复 Windows 启动 opencode 失败** - cmd /C start 引号解析导致 `\\` 文件找不到
  - 代理模式改用 cmd /C 子进程启动，环境变量通过 API 注入
  - 直连模式使用 PowerShell -EncodedCommand (Base64) 避免引号转义
- **修复启动黑屏** - 优化应用启动流程
- **修复同步元数据** - last_sync_at 改用 Gist 远端更新时间
- **修复智谱额度详情** - 7 日用量不显示的问题

### 优化改进

- 代理模式启动的 opencode 作为子进程管理，关闭 OMOSwitcher 时自动清理
- 欢迎卡片齿轮图标对比度优化，多主题适配
- 热重载提示文案精简，去掉多余的 Server 模式说明

---

## [2.0.3] - 2026-04-09

### 新增功能

- **自定义供应商管理** - 全新的自定义 LLM 供应商配置功能
  - 添加/删除自定义供应商
  - 完整的变体参数自定义（推理参数、输入模态等）
  - 推理模型自动添加 `options.thinking` 配置，输入模态支持视频
  - 变体区域可折叠，默认折叠
  - 变体参数选项中英文双语显示

- **模型管理页重构**
  - 卡片网格布局，支持自定义供应商筛选
  - 内置模型筛选功能
  - 模型选择弹窗重构为双栏布局，数据源改为模型管理页可用模型

- **OpenCode 热重载功能** - 配置保存后自动通知运行中的 OpenCode
  - 端口自动发现（4096-4098）
  - 活跃会话探测，空闲时推送配置

### 优化改进

- 统一通知为 ElMessage Toast，移除 SaveIndicator 组件
- 暗色主题下 loading 遮罩改为深色背景
- 自定义徽章移至已配置徽章后面
- 同步 Agent 和 Category 描述与官方仓库一致
- 同步模型配置与官方仓库一致

### Bug 修复

- 修复变体无法勾选的问题
- 修复变体显示格式，补全 OpenAI 兼容格式的所有级别
- 修复自定义供应商合并逻辑

### 技术改进

- 重构 monitor sidecar 内嵌打包资源，切换 SQLite 存储实现
- 移除软重载功能，简化为静态配置保存
- 启用 `serde_json` preserve_order feature 保持字段顺序

---

## [2.0.2] - 2026-04-01

### 构建优化

- **CI/CD 改进**
  - 添加 monitor sidecar 自动构建步骤
  - 完善构建流程，确保发布包包含最新 monitor

- **构建脚本**
  - 新增 `build:all` 一键打包命令
  - 新增 `build:monitor:prod` 生产环境构建命令
  - 区分 monitor 开发版和生产版构建配置

### 资源优化

- **图标文件清理**
  - 删除不需要的 Windows Store 图标（Square*Logo.png、StoreLogo.png）
  - 移除不存在的 icon.icns 引用
  - 精简图标资源约 290KB

### 文档更新

- README 补充 LLM API 监控功能说明

---

## [2.0.1] - 2026-04-01

### 新增功能

- **Agent/Category 配置页面** - 独立的配置管理界面
  - Agent 模型配置
  - Category 模型配置
  - 更直观的配置管理体验

### 优化改进

- **UI/UX 优化**
  - 整体 UI 样式优化
  - 模型配置交互体验优化
  - 启动体验优化

### Bug 修复

- 修复 ModelConfig 页面导航空白问题
- 恢复 ModelConfig 页面功能

---

## [2.0.0] - 2026-03-30

### 新增功能

- **大模型请求监控功能** - 全新的 LLM API 调用监控模块
  - HTTPS 代理服务器拦截 LLM 请求和响应
  - SQLite 数据库持久化存储请求历史
  - Token 使用量和费用估算
  - MCP 工具调用检测和记录
  - 支持自定义 CA 证书

- **监控管理页面**
  - 服务启动/停止控制
  - 今日/本周/本月统计数据展示
  - 请求列表按时间倒序显示
  - 请求详情弹窗查看（请求体、响应体、MCP 调用）
  - 自动刷新功能

- **多主题支持**
  - 暗色主题 (Dark)
  - 明色主题 (Light)
  - 赛博朋克主题 (Cyberpunk)
  - 玻璃拟态主题 (Glassmorphism)

- **弹窗优化**
  - 所有弹窗居中显示
  - 遮罩覆盖整个页面（包括侧边栏）

- **监控端口可配置化** - 支持自定义代理端口和 Web 端口

### 优化改进

- **性能优化**
  - 异步文件操作
  - IPC 调用合并优化

- **UI/UX 改进**
  - 暗色模式对比度优化
  - 明色模式样式适配
  - 请求列表表格滚动条限制
  - 404 错误静默处理

### Bug 修复

- 修复请求详情无法正确显示 Buffer 类型的请求/响应体
- 修复预设描述丢失问题
- 修复重启后当前预设未恢复的问题
- 修复 SSL 证书验证相关问题

### 技术改进

- 项目结构重构：Monitor 作为独立子包
- Tauri 2.0 升级
- 数据库自动备份功能
- 定时数据清理任务

---

## [0.1.0] - 2026-03-26

### 新增功能

- Agent/Category 模型配置管理
- 预设管理功能
- 模型列表管理
- OpenCode CLI 启动集成
- 无边框透明窗口
