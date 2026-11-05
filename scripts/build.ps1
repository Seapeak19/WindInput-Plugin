param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent
$core = Join-Path $repo "src\WindInput-main\wind_input"
$setting = Join-Path $repo "src\wind-setting"
$release = Join-Path $repo "release"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "找不到 cargo。请先安装 Rust MSVC 工具链，并在 VS 开发者环境中运行。"
}

Write-Host "构建 core service..."
Push-Location $core
try {
    if (-not $SkipTests) {
        cargo test -p wind-ai --quiet
    }
    cargo build --release -p wind_service
    if ($LASTEXITCODE -ne 0) { throw "core 构建失败" }
    Copy-Item (Join-Path $core "target\release\wind_input.exe") $release -Force
} finally {
    Pop-Location
}

Write-Host "构建设置程序..."
Push-Location $setting
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "settings 构建失败" }
    Copy-Item (Join-Path $setting "target\release\wind_setting.exe") $release -Force
} finally {
    Pop-Location
}

Write-Host "构建完成，输出在 $release"
