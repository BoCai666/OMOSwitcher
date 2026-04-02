# OMOSwitcher 自动化打包脚本（方案 D：内嵌 Node.js 运行时）
# 用途：打包包含 Node.js 运行时和所有依赖的完整版本

param(
    [string]$NodeVersion = "20.11.0",
    [string]$OutputDir = "release",
    [switch]$SkipBuild = $false
)

$ErrorActionPreference = "Stop"
$PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item $PSScriptRoot).Parent.FullName

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " OMOSwitcher 打包脚本（内嵌 Node.js）" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 编译项目
if (-not $SkipBuild) {
    Write-Host "[1/6] 编译项目..." -ForegroundColor Yellow
    
    Set-Location $ProjectRoot
    
    # 编译前端
    Write-Host "  - 编译前端..."
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "前端编译失败"
    }
    
    # 编译 monitor
    Write-Host "  - 编译 monitor..."
    Set-Location "$ProjectRoot\packages\monitor"
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "Monitor 编译失败"
    }
    
    Set-Location $ProjectRoot
    Write-Host "  ✓ 编译完成" -ForegroundColor Green
} else {
    Write-Host "[1/6] 跳过编译（使用现有构建）" -ForegroundColor Gray
}

# 步骤 2: 准备 Node.js 运行时
Write-Host "[2/6] 准备 Node.js 运行时..." -ForegroundColor Yellow

$NodeDir = "$ProjectRoot\src-tauri\binaries\node"
if (Test-Path $NodeDir) {
    Remove-Item $NodeDir -Recurse -Force
}
New-Item -ItemType Directory -Path $NodeDir -Force | Out-Null

# 下载 Node.js
$NodeUrl = "https://nodejs.org/dist/v$NodeVersion/node-v$NodeVersion-win-x64.zip"
$NodeZip = "$ProjectRoot\node-temp.zip"
$NodeExtract = "$ProjectRoot\node-temp"

Write-Host "  - 下载 Node.js v$NodeVersion..."
Invoke-WebRequest -Uri $NodeUrl -OutFile $NodeZip -UseBasicParsing

Write-Host "  - 解压 Node.js..."
Expand-Archive -Path $NodeZip -DestinationPath $NodeExtract -Force

# 复制必要文件
$NodeSource = "$NodeExtract\node-v$NodeVersion-win-x64"
Copy-Item "$NodeSource\node.exe" "$NodeDir\" -Force
Copy-Item "$NodeSource\npm" "$NodeDir\" -Recurse -Force
Copy-Item "$NodeSource\npm.cmd" "$NodeDir\" -Force

# 复制 DLL 文件（如果有）
Get-ChildItem "$NodeSource\*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName "$NodeDir\" -Force
}

# 清理临时文件
Remove-Item $NodeZip -Force -ErrorAction SilentlyContinue
Remove-Item $NodeExtract -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "  ✓ Node.js 运行时准备完成" -ForegroundColor Green

# 步骤 3: 准备 monitor-package
Write-Host "[3/6] 准备 monitor-package..." -ForegroundColor Yellow

$MonitorDest = "$ProjectRoot\src-tauri\binaries\monitor-package"
if (Test-Path $MonitorDest) {
    Remove-Item $MonitorDest -Recurse -Force
}
New-Item -ItemType Directory -Path $MonitorDest -Force | Out-Null

# 复制 monitor 文件
$MonitorSource = "$ProjectRoot\packages\monitor"
Copy-Item "$MonitorSource\dist" "$MonitorDest\" -Recurse -Force
Copy-Item "$MonitorSource\package.json" "$MonitorDest\" -Force
Copy-Item "$MonitorSource\config.jsonc" "$MonitorDest\" -Force -ErrorAction SilentlyContinue

# 安装生产依赖
Write-Host "  - 安装依赖..."
Set-Location $MonitorDest
npm install --production --no-audit --no-fund
if ($LASTEXITCODE -ne 0) {
    throw "依赖安装失败"
}
Set-Location $ProjectRoot

Write-Host "  ✓ monitor-package 准备完成" -ForegroundColor Green

# 步骤 4: 构建 Tauri 应用
Write-Host "[4/6] 构建 Tauri 应用..." -ForegroundColor Yellow

Set-Location "$ProjectRoot\src-tauri"
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    throw "Tauri 构建失败"
}
Set-Location $ProjectRoot

Write-Host "  ✓ Tauri 应用构建完成" -ForegroundColor Green

# 步骤 5: 组织发布文件
Write-Host "[5/6] 组织发布文件..." -ForegroundColor Yellow

$ReleaseDir = "$ProjectRoot\$OutputDir"
if (Test-Path $ReleaseDir) {
    Remove-Item $ReleaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path $ReleaseDir -Force | Out-Null

# 复制构建产物
$BundleDir = "$ProjectRoot\src-tauri\target\release\bundle"
Copy-Item "$BundleDir\msi\*" "$ReleaseDir\" -Recurse -Force -ErrorAction SilentlyContinue
Copy-Item "$BundleDir\nsis\*" "$ReleaseDir\" -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "  ✓ 发布文件组织完成" -ForegroundColor Green

# 步骤 6: 生成信息文件
Write-Host "[6/6] 生成信息文件..." -ForegroundColor Yellow

# 读取版本号
$PackageJson = Get-Content "$ProjectRoot\package.json" | ConvertFrom-Json
$Version = $PackageJson.version

# 创建 README
$ReadmeContent = @"
# OMOSwitcher v$Version

## 版本信息
- 版本: $Version
- Node.js: v$NodeVersion
- 构建时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

## 包含内容
- OMOSwitcher 主程序
- Node.js 运行时（v$NodeVersion）
- Monitor 服务（包含所有依赖）

## 使用说明
1. 安装 OMOSwitcher
2. 直接运行，无需安装 Node.js

## 系统要求
- Windows 10/11 (64-bit)

## 文件大小
- 安装包: ~150MB
- 安装后: ~200MB
"@

Set-Content -Path "$ReleaseDir\README.txt" -Value $ReadmeContent -Encoding UTF8

Write-Host "  ✓ 信息文件生成完成" -ForegroundColor Green

# 完成
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host " 打包完成！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "输出目录: $ReleaseDir" -ForegroundColor Cyan
Write-Host ""

# 显示文件列表
Get-ChildItem $ReleaseDir -Recurse | Where-Object { -not $_.PSIsContainer } | ForEach-Object {
    $SizeInMB = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  $($_.Name) - $SizeInMB MB" -ForegroundColor Gray
}

Write-Host ""
Write-Host "提示: 请检查输出目录中的安装包" -ForegroundColor Yellow
