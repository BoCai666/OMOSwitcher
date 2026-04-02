# Monitor 日志清理脚本
# 用途：统一使用 logger 接口，删除冗余日志

$monitorSrc = "E:\AI\Programs\OMOSwitcher\packages\monitor\src"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " 开始清理日志" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 清理策略：保留关键日志，删除冗余日志
# 关键日志：error, warn, 启动/停止, 初始化成功/失败
# 冗余日志：每个请求的详细日志、调试日志、过于频繁的日志

# 处理 db/index.ts
Write-Host "`n处理 db/index.ts..." -ForegroundColor Yellow
$file = "$monitorSrc\db\index.ts"
$content = Get-Content $file -Raw

# 添加 logger 导入
if ($content -notmatch 'import logger from') {
    $content = "import logger from '../logger.js';`n$content"
}

# 统一使用 logger
$content = $content -replace 'console\.log\(', 'logger.info('
$content = $content -replace 'console\.error\(', 'logger.error('
$content = $content -replace 'console\.warn\(', 'logger.warn('

# 删除冗余日志（每个操作的详细日志）
$lines = $content -split "`n"
$newLines = @()
$skipPatterns = @(
    '\[Database\] Saved to',
    '\[Database\] Loading SQL',
    '\[Database\] Configuration applied',
    '\[Database\] Statement'
)

foreach ($line in $lines) {
    $shouldSkip = $false
    foreach ($pattern in $skipPatterns) {
        if ($line -match $pattern) {
            $shouldSkip = $true
            break
        }
    }
    if (-not $shouldSkip) {
        $newLines += $line
    }
}

$content = $newLines -join "`n"
Set-Content -Path $file -Value $content -NoNewline
Write-Host "✓ db/index.ts 清理完成" -ForegroundColor Green

# 处理 db/backup.ts
Write-Host "`n处理 db/backup.ts..." -ForegroundColor Yellow
$file = "$monitorSrc\db\backup.ts"
$content = Get-Content $file -Raw

if ($content -notmatch 'import logger from') {
    $content = "import logger from '../logger.js';`n$content"
}

$content = $content -replace 'console\.log\(', 'logger.info('
$content = $content -replace 'console\.error\(', 'logger.error('
$content = $content -replace 'console\.warn\(', 'logger.warn('

# 删除冗余日志
$lines = $content -split "`n"
$newLines = @()
$skipPatterns = @(
    '\[Backup\] Backup directory:',
    '\[Backup\] Deleting old backup'
)

foreach ($line in $lines) {
    $shouldSkip = $false
    foreach ($pattern in $skipPatterns) {
        if ($line -match $pattern) {
            $shouldSkip = $true
            break
        }
    }
    if (-not $shouldSkip) {
        $newLines += $line
    }
}

$content = $newLines -join "`n"
Set-Content -Path $file -Value $content -NoNewline
Write-Host "✓ db/backup.ts 清理完成" -ForegroundColor Green

Write-Host "`n数据库模块日志清理完成！" -ForegroundColor Green
