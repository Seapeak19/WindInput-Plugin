# 从源码构建

本仓库把三个上游工程放在同一级目录，保持 `wind-setting` 的相对路径依赖可用。

```text
src/
  WindInput-main/    # core workspace
  wind-setting/      # settings app
  wind-ui-rust/      # windui GUI library
```

---

## 1. 工具链

- Windows 10/11 x64
- Visual Studio 2022 Build Tools
  - Desktop development with C++
  - Windows 10/11 SDK
- Rust stable，`x86_64-pc-windows-msvc`
- Git
- CMake（部分依赖）
- PowerShell 7+ 或 Windows PowerShell 5.1

检查：

```powershell
rustc -V
cargo -V
```

如果 `cargo build` 报 `LINK : fatal error LNK1181: 无法打开输入文件“kernel32.lib”`，说明没有在 MSVC 环境中运行。先执行：

```bat
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

路径按本机实际安装调整。

---

## 2. 一键构建

在仓库根目录：

```powershell
.\scripts\build.ps1
```

脚本会：

1. 运行 `wind-ai` 单元测试；
2. 构建 `wind_service`；
3. 构建设置程序；
4. 把两个 exe 复制到 `release/`。

跳过测试：

```powershell
.\scripts\build.ps1 -SkipTests
```

---

## 3. 手动构建

### 3.1 core service

```powershell
cd src\WindInput-main\wind_input
cargo build --release -p wind_service
```

产物：

```text
src\WindInput-main\wind_input\target\release\wind_input.exe
```

### 3.2 settings

```powershell
cd ..\..\wind-setting
cargo build --release
```

产物：

```text
src\wind-setting\target\release\wind_setting.exe
```

`wind-setting` 会通过相对路径自动编译 `../wind-ui-rust` 和
`../WindInput-main/wind_input/crates/*`，不要单独移动目录。

### 3.3 复制到 release

```powershell
copy ..\WindInput-main\wind_input\target\release\wind_input.exe ..\..\release\
copy target\release\wind_setting.exe ..\..\release\
```

---

## 4. 测试

```powershell
cd src\WindInput-main\wind_input
cargo test -p wind-ai
cargo test -p wind-coordinator handle_ai::tests
```

设置程序：

```powershell
cd src\wind-setting
cargo test manifest::tests
```

完整 workspace 测试耗时较长，发布前至少跑上述与 AI/快捷键相关的测试。

---

## 5. 运行时资源

构建只生成 exe；运行时仍需要官方清风输入法的：

- `wind_tsf.dll`
- `data/` 词库、方案、主题
- 用户数据目录

因此**不要**把 `release/` 当成完整输入法安装包。它是官方安装之上的覆盖包。

---

## 6. 打包发布

1. 构建两个 exe；
2. 更新 `release/SHA256SUMS.txt`；
3. 更新 README 的版本/兼容性；
4. 提交并打 tag，例如：

```powershell
git tag v0.127.0-ai.1
git push origin main --tags
```

如果发布 GitHub Release，建议附带：

- `wind-ai-plugin-0.127.0-ai.1.zip`
- `SHA256SUMS.txt`
- 更新说明

---

## 7. 常见构建错误

### `kernel32.lib` 找不到

没有在 `vcvars64.bat` 环境里运行。先调用它，再运行 cargo。

### `failed to load manifest for dependency windui`

`src/wind-setting` 与 `src/wind-ui-rust` 不是同级目录，或目录名被改动。恢复仓库结构。

### `wind-ipc` / `wind-config` 路径找不到

`src/WindInput-main/wind_input` 被移动。`wind-setting` 依赖它的相对路径。

### 设置程序缺字体/DirectWrite 错误

安装 Windows SDK 与 Visual Studio C++ 工具链；不要用 GNU 工具链构建。

### 链接很慢 / 磁盘占用大

Rust debug 构建体积较大，可在发布时只保留 `target/release` 产物；仓库本身不提交 `target/`。
