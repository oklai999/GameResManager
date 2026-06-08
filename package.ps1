#Requires -Version 5.1
<#
.SYNOPSIS
    打包脚本 - 游戏资源管理器 Tauri 应用
.DESCRIPTION
    构建前端 + Tauri 后端，收集安装包到 releases/ 目录
#>

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProductName = "游戏资源管理器"
$TauriConfigPath = Join-Path $ProjectRoot "src-tauri" "tauri.conf.json"
$TauriConfig = Get-Content -LiteralPath $TauriConfigPath -Raw | ConvertFrom-Json
$Version = [string]$TauriConfig.version

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Test-Command {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Error "未找到 $Name，请确认已安装并加入 PATH"
    }
}

# 1. 检查环境
Write-Step "检查构建环境..."
Test-Command "npm"
Test-Command "cargo"

$RustVersion = (rustc --version) 2>$null
if (-not $RustVersion) {
    Write-Error "未找到 rustc"
}
Write-Host "Rust: $RustVersion"

# 2. 检查旧构建目录
Write-Step "检查构建环境..."
$DistDir = Join-Path $ProjectRoot "dist"
$ReleaseDir = Join-Path $ProjectRoot "releases"
$BundleDir = Join-Path $ProjectRoot "src-tauri" "target" "release" "bundle"

if (Test-Path $DistDir) {
    Write-Host "检测到已有 dist 目录；npm run build 会覆盖前端产物。脚本不会递归删除该目录。"
}
if (-not (Test-Path $ReleaseDir)) {
    New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
}

# 3. 构建前端
Write-Step "构建前端 (npm run build)..."
Push-Location $ProjectRoot
try {
    npm run build
    if ($LASTEXITCODE -ne 0) { throw "前端构建失败" }
} finally {
    Pop-Location
}

# 4. 构建 Tauri
Write-Step "构建 Tauri 应用 (npx tauri build)..."
Push-Location $ProjectRoot
try {
    npx tauri build
    if ($LASTEXITCODE -ne 0) { throw "Tauri 构建失败" }
} finally {
    Pop-Location
}

# 5. 收集产物
Write-Step "收集打包产物..."
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$VersionDir = Join-Path $ReleaseDir "v$Version-$Timestamp"
New-Item -ItemType Directory -Force -Path $VersionDir | Out-Null

$Artifacts = @()

# MSI (with version verification)
$Msi = Get-ChildItem -Path $BundleDir -Filter "*.msi" -Recurse -ErrorAction SilentlyContinue
if ($Msi) {
    $MsiMatched = $Msi | Where-Object { $_.Name -like "*$Version*" } | Select-Object -First 1
    if (-not $MsiMatched) {
        throw "找到 MSI 文件但不匹配版本 $Version，请检查 bundle 目录是否存在旧构建产物。MSI: $($Msi.FullName)"
    }
    $Dest = Join-Path $VersionDir $MsiMatched.Name
    Copy-Item $MsiMatched.FullName $Dest
    $Artifacts += $Dest
}

# NSIS (with version verification)
$Nsis = Get-ChildItem -Path $BundleDir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue
if ($Nsis) {
    $NsisInstaller = $Nsis | Where-Object { $_.FullName -like "*nsis*" -and $_.Name -like "*$Version*" } | Select-Object -First 1
    if (-not $NsisInstaller) {
        throw "未找到匹配版本 $Version 的 NSIS 安装包。请确认 bundle/nsis 目录包含正确版本的安装包，或清理旧构建产物。"
    }
    $Dest = Join-Path $VersionDir $NsisInstaller.Name
    Copy-Item $NsisInstaller.FullName $Dest
    $Artifacts += $Dest
}

# 便携版 (如果启用了)
$Portable = Get-ChildItem -Path (Join-Path $ProjectRoot "src-tauri" "target" "release") -Filter "*.exe" -ErrorAction SilentlyContinue
$MainExe = $Portable | Where-Object { $_.Name -notmatch "^(build-script|deps|incremental|\.fingerprint)" -and $_.Name -ne "game_resource_manager_lib.dll" } | Select-Object -First 1
if ($MainExe) {
    $Dest = Join-Path $VersionDir $MainExe.Name
    Copy-Item $MainExe.FullName $Dest
    $Artifacts += $Dest
}

# 6. 确认找到了版本匹配的产物
if ($Artifacts.Count -eq 0) {
    throw "未找到任何版本 $Version 的打包产物。请确认 tauri build 已成功执行。"
}

# 7. 输出结果
Write-Step "打包完成"
Write-Host "产物目录: $VersionDir" -ForegroundColor Green
foreach ($a in $Artifacts) {
    $Size = (Get-Item $a).Length / 1MB
    Write-Host "  - $(Split-Path -Leaf $a) ($([math]::Round($Size, 2)) MB)"
}
