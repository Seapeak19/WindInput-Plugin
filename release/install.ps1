param(
    [string]$QingfengRoot = "$env:ProgramFiles\WindInput",
    [switch]$NoRestart
)

$ErrorActionPreference = "Stop"

function Find-QingfengRoot {
    param([string]$Preferred)
    if ((Test-Path (Join-Path $Preferred "wind_input.exe")) -and
        (Test-Path (Join-Path $Preferred "wind_tsf.dll"))) {
        return (Resolve-Path $Preferred).Path
    }
    $portable = Join-Path $env:LOCALAPPDATA "Programs\WindInput"
    if ((Test-Path (Join-Path $portable "wind_input.exe")) -and
        (Test-Path (Join-Path $portable "wind_tsf.dll"))) {
        return (Resolve-Path $portable).Path
    }
    throw "没有找到清风输入法。请先安装官方 0.127.0，再运行本脚本；或用 -QingfengRoot 指定安装目录。"
}

$root = Find-QingfengRoot $QingfengRoot
$src = $PSScriptRoot
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"

Write-Host "清风输入法目录: $root"
Write-Host "插件文件: $src"

$version = (Get-Item (Join-Path $root "wind_input.exe")).VersionInfo.FileVersion
Write-Host "基础版本: $version"
if ($version -and $version -notlike "0.127.*") {
    Write-Warning "本插件按 0.127.0 构建，基础版本是 $version；协议可能不兼容。"
}

function Stop-ThisInstall {
    param([string]$Root)
    $running = Get-Process -Name "wind_input" -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and (Split-Path $_.Path -Parent) -eq $Root }
    if ($running) {
        Write-Host "停止清风输入法服务..."
        $running | Stop-Process -Force
        Start-Sleep -Milliseconds 500
    }
    Get-Process -Name "wind_setting" -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and (Split-Path $_.Path -Parent) -eq $Root } |
        Stop-Process -Force
}

Stop-ThisInstall $root

foreach ($name in @("wind_input.exe", "wind_setting.exe")) {
    $target = Join-Path $root $name
    if (Test-Path $target) {
        Copy-Item $target "$target.bak-ai-$stamp" -Force
        Write-Host "已备份: $name.bak-ai-$stamp"
    }
    Copy-Item (Join-Path $src $name) $target -Force
    Write-Host "已安装: $name"
}

if (-not $NoRestart) {
    Start-Process (Join-Path $root "wind_input.exe") -WorkingDirectory $root
    Write-Host "清风输入法服务已启动。"
}

Write-Host ""
Write-Host "安装完成。打开清风输入法设置 → AI 工具，填写 API Key 后即可使用。"
Write-Host "快捷键默认: 翻译 Ctrl+Alt+T，润色 Ctrl+Alt+S。"
Write-Host "如需卸载: 运行 release\uninstall.ps1 -QingfengRoot `"$root`""
