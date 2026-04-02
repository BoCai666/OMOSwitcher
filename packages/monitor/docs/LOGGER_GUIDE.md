# Monitor 日志统一指南

## 现状

当前 monitor 中存在两种日志调用方式：

1. **统一日志接口**（推荐）- 使用 `logger` 模块
2. **直接 console 调用**（不推荐）

### 已使用 logger 的文件

✅ 以下文件已正确使用 logger 接口：

- `src/index.ts` - 主入口
- `src/config.ts` - 配置模块
- `src/config-manager.ts` - 配置管理器
- `src/proxy/response-capture.ts` - 响应捕获

### 仍使用 console 的文件

⚠️ 以下文件需要迁移到 logger 接口：

| 文件 | console 调用次数 |
|------|-----------------|
| `src/db/index.ts` | 21 次 |
| `src/db/backup.ts` | 12 次 |
| `src/proxy/server.ts` | 23 次 |
| `src/server/routes.ts` | 16 次 |
| `src/interceptor/response-handler.ts` | 12 次 |
| `src/proxy/cert-manager.ts` | 12 次 |
| `src/proxy/cert-manager-openssl.ts` | 11 次 |
| `src/tasks/data-cleanup.ts` | 8 次 |
| `src/interceptor/stream-handler.ts` | 7 次 |
| 其他文件 | < 5 次 |

---

## Logger 模块说明

### 位置
`src/logger.ts`

### 功能
- ✅ 同时输出到控制台和文件
- ✅ 自动日志轮转（最大 5MB）
- ✅ 保留最近 3 个备份
- ✅ 统一格式：`[时间戳] [级别] 消息`

### API

```typescript
import logger from '../logger.js';

logger.info('信息消息');
logger.warn('警告消息');
logger.error('错误消息');
logger.debug('调试消息'); // 仅开发模式
```

---

## 迁移步骤

### 1. 添加 logger 导入

```typescript
// 在文件顶部添加
import logger from '../logger.js';
```

**注意：根据文件深度调整路径：**
- `src/db/*.ts` → `import logger from '../logger.js';`
- `src/proxy/*.ts` → `import logger from '../logger.js';`
- `src/server/*.ts` → `import logger from '../logger.js';`
- `src/interceptor/*.ts` → `import logger from '../logger.js';`

### 2. 替换 console 调用

**替换规则：**

| 原调用 | 新调用 |
|--------|--------|
| `console.log(...)` | `logger.info(...)` |
| `console.error(...)` | `logger.error(...)` |
| `console.warn(...)` | `logger.warn(...)` |
| `console.info(...)` | `logger.info(...)` |

### 3. 示例

**修改前：**
```typescript
console.log('[Database] Loading database...');
console.error('[Database] Error:', err);
console.warn('[Database] Warning: data might be incomplete');
```

**修改后：**
```typescript
import logger from '../logger.js';

logger.info('[Database] Loading database...');
logger.error('[Database] Error:', err);
logger.warn('[Database] Warning: data might be incomplete');
```

---

## 批量替换命令（慎用）

### 使用 sed (Linux/macOS)

```bash
# 添加 logger 导入
sed -i "1i import logger from '../logger.js';" src/db/backup.ts

# 替换 console 调用
sed -i 's/console\.log(/logger.info(/g' src/db/backup.ts
sed -i 's/console\.error(/logger.error(/g' src/db/backup.ts
sed -i 's/console\.warn(/logger.warn(/g' src/db/backup.ts
```

### 使用 PowerShell (Windows)

```powershell
# 批量替换示例
$file = "src/db/backup.ts"
$content = Get-Content $file -Raw

# 添加导入（如果需要）
if ($content -notmatch 'import logger from') {
    $content = "import logger from '../logger.js';`n$content"
}

# 替换 console
$content = $content -replace 'console\.log\(', 'logger.info('
$content = $content -replace 'console\.error\(', 'logger.error('
$content = $content -replace 'console\.warn\(', 'logger.warn('

Set-Content -Path $file -Value $content
```

---

## 迁移优先级

### 高优先级（核心模块）
1. ✅ `src/db/index.ts` - 数据库核心
2. ✅ `src/db/backup.ts` - 数据库备份
3. ✅ `src/proxy/server.ts` - 代理服务器
4. ✅ `src/server/routes.ts` - API 路由

### 中优先级（重要功能）
5. `src/proxy/cert-manager.ts` - 证书管理
6. `src/proxy/cert-manager-openssl.ts` - OpenSSL 证书
7. `src/tasks/data-cleanup.ts` - 数据清理

### 低优先级（辅助功能）
8. `src/interceptor/response-handler.ts`
9. `src/interceptor/stream-handler.ts`
10. 其他文件

---

## 验证

### 编译检查

```bash
npm run build
```

确保没有编译错误。

### 运行时检查

启动 monitor 后，检查日志文件：

```bash
cat ~/.config/omoswitcher/monitor/logs/monitor.log
```

确认所有日志都写入文件。

---

## 注意事项

### ⚠️ 重要

1. **不要重复导入**
   ```typescript
   // ❌ 错误 - 重复导入
   import logger from '../logger.js';
   import logger from '../logger.js';
   
   // ✅ 正确 - 只导入一次
   import logger from '../logger.js';
   ```

2. **注意相对路径**
   - 不同目录深度的文件需要调整导入路径
   - 参考 `src/index.ts` 中已有导入

3. **logger.ts 中的 console 调用**
   - `logger.ts` 本身可以使用 `console.*`
   - 这是日志系统的核心，需要直接输出

### ✅ 建议

1. **逐个文件迁移**
   - 不要批量修改所有文件
   - 每个文件修改后立即编译测试

2. **保留原有标签**
   ```typescript
   // ✅ 保留 [Database] 等标签
   logger.info('[Database] Loading...');
   
   // ❌ 不要去掉标签
   logger.info('Loading...');
   ```

3. **统一消息格式**
   ```typescript
   // ✅ 推荐
   logger.info('[Module] Action: detail');
   
   // ❌ 不推荐
   logger.info('Action: detail');
   ```

---

## 完成标准

当所有文件迁移完成后：

- [ ] 所有 `.ts` 文件使用 `logger` 接口
- [ ] 编译无错误
- [ ] 运行时日志正常写入文件
- [ ] 控制台输出正常
- [ ] 日志文件路径：`~/.config/omoswitcher/monitor/logs/monitor.log`

---

## 相关文件

- Logger 实现：`src/logger.ts`
- 日志路径定义：`src/paths.ts`
- 示例文件：`src/index.ts`（已使用 logger）

---

**更新时间：** 2026-04-02  
**状态：** 待迁移
