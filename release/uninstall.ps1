param(
    [string]$QingfengRoot = "$env:ProgramFiles\WindInput",
    [switch]$Restart
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path (Join-Path $QingfengRoot "wind_input.exe"))) {
    $portable = Join-Path $env:LOCALAPPDATA "Programs\WindInput"
    if (Test-Path (Join-Path $portable "wind_input.exe")) {
        $QingfengRoot = $portable
    } else {
        throw "没有找到清风输入法目录，请用 -QingfengRoot 指定。"
    }
}
$root = (Resolve-Path $QingfengRoot).Path

$wasRunning = Get-Process -Name "wind_input" -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and (Split-Path $_.Path -Parent) -eq $root }
$wasRunning | Stop-Process -Force
Get-Process -Name "wind_setting" -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and (Split-Path $_.Path -Parent) -eq $root } |
    Stop-Process -Force
Start-Sleep -Milliseconds 500

$found = $false
foreach ($name in @("wind_input.exe", "wind_setting.exe")) {
    $backup = Get-ChildItem (Join-Path $root "$name.bak-ai-*") -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if ($backup) {
        Copy-Item $backup.FullName (Join-Path $root $name) -Force
        Write-Host "已恢复: $name (来自 $($backup.Name))"
        $found = $true
    }
}

if (-not $found) {
    throw "没有找到本插件生成的 .bak-ai-* 备份。请重新安装官方 0.127.0 覆盖。"
}

if ($wasRunning -or $Restart) {
    Start-Process (Join-Path $root "wind_input.exe") -WorkingDirectory $root
    Write-Host "已恢复官方文件并启动服务。"
} else {
    Write-Host "已恢复官方文件。目标服务原本未运行，未自动启动。"
}
