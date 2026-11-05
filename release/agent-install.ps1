param(
    [string]$QingfengRoot,
    [switch]$NoRestart,
    [switch]$Check
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot

function Test-QingfengRoot {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    try {
        $Path = $Path.Trim().TrimEnd('\')
        return (Test-Path (Join-Path $Path "wind_input.exe")) -and
               (Test-Path (Join-Path $Path "wind_tsf.dll"))
    } catch {
        return $false
    }
}

function Find-QingfengRoot {
    $candidates = New-Object System.Collections.Generic.List[string]

    Get-Process -Name "wind_input" -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            if ($_.Path) { $candidates.Add((Split-Path $_.Path -Parent)) }
        } catch { }
    }

    $registryPaths = @(
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )
    foreach ($regPath in $registryPaths) {
        Get-ItemProperty $regPath -ErrorAction SilentlyContinue | ForEach-Object {
            $name = [string]$_.DisplayName
            if ($name -notmatch "清风|WindInput|Wind Input") { return }
            $location = [string]$_.InstallLocation
            if ($location) { $candidates.Add($location.TrimEnd('\')) }

            $command = [string]$_.UninstallString
            if ($command) {
                if ($command.StartsWith('"')) {
                    $end = $command.IndexOf('"', 1)
                    if ($end -gt 1) { $command = $command.Substring(1, $end - 1) }
                } else {
                    $exe = $command.IndexOf(".exe", [StringComparison]::OrdinalIgnoreCase)
                    if ($exe -gt 0) { $command = $command.Substring(0, $exe + 4) }
                }
                try {
                    $dir = Split-Path $command -Parent
                    if ($dir) { $candidates.Add($dir) }
                } catch { }
            }
        }
    }

    $candidates.Add((Join-Path $env:ProgramFiles "WindInput"))
    if (${env:ProgramFiles(x86)}) {
        $candidates.Add((Join-Path ${env:ProgramFiles(x86)} "WindInput"))
    }
    $candidates.Add((Join-Path $env:LOCALAPPDATA "Programs\WindInput"))
    $candidates.Add((Join-Path $env:LOCALAPPDATA "WindInput"))

    foreach ($candidate in $candidates) {
        if (Test-QingfengRoot $candidate) {
            return (Resolve-Path $candidate).Path
        }
    }
    return $null
}

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-WriteAccess {
    param([string]$Path)
    try {
        $probe = Join-Path $Path (".qfai-write-test-" + [Guid]::NewGuid().ToString("N"))
        [IO.File]::WriteAllText($probe, "")
        Remove-Item $probe -Force
        return $true
    } catch {
        return $false
    }
}

function Stop-ThisInstall {
    param([string]$Root)
    foreach ($name in @("wind_input", "wind_setting")) {
        Get-Process -Name $name -ErrorAction SilentlyContinue | ForEach-Object {
            try {
                if ($_.Path -and (Split-Path $_.Path -Parent) -eq $Root) {
                    Stop-Process -Id $_.Id -Force
                }
            } catch { }
        }
    }
    Start-Sleep -Milliseconds 500
}

if (-not $QingfengRoot) {
    $QingfengRoot = Find-QingfengRoot
}
if (-not $QingfengRoot) {
    throw "没有找到清风输入法 0.127.0。请先安装官方版本，或传 -QingfengRoot。"
}
$root = (Resolve-Path $QingfengRoot).Path
if (-not (Test-QingfengRoot $root)) {
    throw "目录无效（缺少 wind_input.exe 或 wind_tsf.dll）: $root"
}

if ($Check) {
    Write-Output "QINGFENG_ROOT=$root"
    Write-Output "ADMIN=$(Test-Admin)"
    Write-Output "WRITABLE=$(Test-WriteAccess $root)"
    exit 0
}

$admin = Test-Admin
$writable = Test-WriteAccess $root
if (-not $admin -and -not $writable) {
    throw "当前进程没有管理员权限，且安装目录不可写。请让 Agent 以管理员身份运行，或由用户批准 UAC。"
}

$version = (Get-Item (Join-Path $root "wind_input.exe")).VersionInfo.FileVersion
if ($version -and $version -notlike "0.127.*") {
    Write-Warning "基础版本是 $version，本插件按 0.127.0 构建，协议可能不兼容。"
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
Stop-ThisInstall $root

foreach ($name in @("wind_input.exe", "wind_setting.exe")) {
    $target = Join-Path $root $name
    if (Test-Path $target) {
        Copy-Item $target "$target.bak-ai-$stamp" -Force
    }
    Copy-Item (Join-Path $here $name) $target -Force
    Write-Output "INSTALLED=$name"
}

if (-not $NoRestart) {
    # 如果 Agent 本身是管理员，直接启动会把输入法拉成高完整性进程。
    # 交给现有 Explorer 启动，尽量保持普通用户权限。
    Start-Process -FilePath "explorer.exe" -ArgumentList ('"{0}"' -f (Join-Path $root "wind_input.exe"))
    Write-Output "START_REQUESTED=wind_input.exe"
}

Write-Output "QINGFENG_ROOT=$root"
Write-Output "BACKUP_SUFFIX=.bak-ai-$stamp"
Write-Output "OK"
