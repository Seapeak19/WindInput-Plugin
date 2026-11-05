$ErrorActionPreference = "Stop"

$repo = Split-Path $PSScriptRoot -Parent
$coreExe = Join-Path $repo "release\wind_input.exe"
$settingExe = Join-Path $repo "release\wind_setting.exe"
$out = Join-Path $repo "release\QingfengAI-Setup.exe"
$csc = Join-Path $env:WINDIR "Microsoft.NET\Framework64\v4.0.30319\csc.exe"

foreach ($f in @($coreExe, $settingExe, $csc)) {
    if (-not (Test-Path $f)) {
        throw "缺少文件: $f"
    }
}

& $csc /nologo /target:winexe /platform:x64 /optimize+ `
    "/out:$out" `
    "/win32manifest:$PSScriptRoot\Setup.manifest" `
    "/resource:$coreExe,wind_input.exe" `
    "/resource:$settingExe,wind_setting.exe" `
    "/r:System.dll" `
    "/r:System.Drawing.dll" `
    "/r:System.Windows.Forms.dll" `
    "$PSScriptRoot\Setup.cs"

if ($LASTEXITCODE -ne 0) {
    throw "安装程序编译失败"
}

Write-Host "安装程序已生成: $out"
