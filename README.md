<p align="center">
  <img src="assets/logo.png" alt="清风输入法" width="112">
</p>

<h1 align="center">清风输入法 AI 划词插件</h1>

<p align="center">
  在清风输入法里选中文字，按一个快捷键，直接翻译或润色。
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Windows-10%2F11%20x64-0078D4" alt="Windows">
  <img src="https://img.shields.io/badge/清风输入法-0.127.0-2ea44f" alt="WindInput 0.127.0">
  <img src="https://img.shields.io/badge/license-GPL--3.0-blue" alt="License">
  <img src="https://img.shields.io/badge/AI-OpenAI%20compatible-8b5cf6" alt="OpenAI compatible">
</p>

> 这是一个**非官方扩展**，基于清风输入法 0.127.0 的源码集成。
> 它不是外挂式划词工具，而是把 AI 能力直接做进输入法服务和设置程序。

---

## 原仓库与来源

本插件不是从零实现的输入法，它直接基于以下开源项目：

| 项目 | 原仓库 | 许可证 |
|---|---|---|
| **清风输入法 WindInput**（主项目） | <https://github.com/huanfeng/WindInput> | GPL-3.0 |
| 清风输入法设置程序 | <https://github.com/huanfeng/wind-setting> | GPL-3.0 |
| windui 设置 GUI 库 | <https://github.com/huanfeng/wind-ui-rust> | MIT OR Apache-2.0 |

- 本仓库是**第三方插件/整合分支**，不是清风输入法官方仓库。
- 官方下载与文档：<https://windinput.com>
- 本仓库基于 **WindInput 0.127.0**；core 基线提交 `1771e80`，settings 基线 `529d71e`，windui 基线 `d9792f0`。
- 上游的许可证文件均保留在 `src/` 对应目录；本仓库整体按 GPL-3.0 分发。

---

## 这是什么

清风输入法原生支持五笔/拼音等输入方案，本插件在它的基础上增加了一组**划词 AI 功能**：

- **划词翻译**：选中文字，按热键，在光标旁显示译文。
- **划词润色**：选中文字，按热键，保留原意，让表达更通顺、自然、准确。
- **原生集成**：快捷键、设置、DPAPI Key 存储、状态气泡都在输入法进程内完成。
- **OpenAI 兼容接口**：默认 DeepSeek，也可以填任意兼容 `/chat/completions` 的服务。
- **结果气泡**：左键拖动，右键关闭；不会阻塞当前应用。
- **可录制快捷键**：在设置里直接按组合键，不需要手填键名。

当前版本**只做翻译和润色**，刻意没有加入总结、解释、OCR、历史记录等扩展功能，先把主链路做稳。

---

## 功能一览

| 功能 | 说明 |
|---|---|
| 划词翻译 | 全局热键，捕获当前选区，调用 OpenAI 兼容接口，气泡显示译文 |
| 划词润色 | 同一套接口，提示词只做“保留原意、改善表达”，不扩写 |
| 原生快捷键 | 默认 `Ctrl+Alt+T` / `Ctrl+Alt+S`，可在设置中修改并直接录制 |
| 结果气泡 | 跟随光标，左键拖动，右键关闭，自动隐藏 |
| Key 安全 | Windows DPAPI 加密保存，不写入 `config.toml` |
| Key 校验 | 保存和请求前校验单行 ASCII，拒绝误粘的段落/中文 |
| 日志脱敏 | API 报错和日志不会输出完整 Authorization |
| 选区兜底 | UI Automation 优先；不支持时回退到探针 + `Ctrl+C` 剪贴板 |
| 防误发 | 选区如果像 API Key / Bearer Token，直接拒绝处理 |

---

## 环境要求

- **Windows 10 / 11，64 位**
- **清风输入法 0.127.0**（必须先安装官方版本）
- 一个 OpenAI 兼容的 API Key，例如：
  - DeepSeek：`https://api.deepseek.com/v1`，模型 `deepseek-chat`
  - 其他兼容服务：填写对应的 Base URL 和模型名
- 安装/覆盖 `Program Files` 下的文件时，需要管理员权限

> 本插件只替换 `wind_input.exe` 和 `wind_setting.exe` 两个程序文件。
> `wind_tsf.dll`、词库、配置和用户数据都不动；不要替换或重新注册 TSF DLL。

---

## 快速安装（推荐）

### 1. 安装官方清风输入法 0.127.0

从 [windinput.com/download](https://windinput.com/download) 安装 Windows 版，确认输入法能正常打字。

> 本插件按 **0.127.0** 构建。其他主版本可能协议不兼容。

### 2. 退出正在运行的清风输入法

托盘右键退出，或：

```powershell
Stop-Process -Name wind_input -Force
Stop-Process -Name wind_setting -Force -ErrorAction SilentlyContinue
```

### 3. 运行安装脚本

在本仓库的 `release` 目录打开 PowerShell：

```powershell
# 默认安装目录：C:\Program Files\WindInput
.\install.ps1

# 便携版/自定义目录：
.\install.ps1 -QingfengRoot "D:\Apps\WindInput"
```

如果提示权限不足，请用**管理员 PowerShell** 重新运行。

脚本会自动：

1. 检查官方 `wind_input.exe` 和 `wind_tsf.dll`；
2. 停止服务；
3. 备份原来的 `wind_input.exe` / `wind_setting.exe` 为 `.bak-ai-时间戳`；
4. 复制本插件的两个程序；
5. 重新启动服务。

### 4. 配置 API

打开 **清风输入法设置 → AI 工具**：

- **接口地址**：例如 `https://api.deepseek.com/v1`
- **模型**：例如 `deepseek-chat`
- **API Key**：粘贴真实 Key，按 Enter 或点“应用设置”
- 需要时调整翻译目标、选区上限、自动复制、超时

Key 会用当前 Windows 用户的 DPAPI 加密，保存到用户数据目录的 `ai_key.bin`，不会出现在 `config.toml`。

### 5. 使用

在任意支持选中文本的应用里：

1. 选中一段文字；
2. 按 **Ctrl+Alt+T** 翻译，或 **Ctrl+Alt+S** 润色；
3. 结果出现在光标旁；
4. **左键按住可拖动**，**右键直接关闭**。

> 如果你之前改过快捷键，以设置页显示的为准。默认值是 `Ctrl+Alt+T` / `Ctrl+Alt+S`。

---

## 手动安装（不用脚本）

1. 安装官方清风输入法 0.127.0，并退出服务。
2. 备份安装目录里的：
   - `wind_input.exe`
   - `wind_setting.exe`
3. 用本仓库 `release` 里的同名文件覆盖。
4. 启动 `wind_input.exe`。
5. 打开设置 → AI 工具，填写 Key。

如果只替换 `wind_input.exe` 而不替换 `wind_setting.exe`，翻译/润色可能能用，但设置页不会有 AI Key 和快捷键配置，**不建议**。

---

## 从源码构建

源码已经按可构建的目录整合：

```text
src/
  WindInput-main/    # 清风输入法 core（含 AI 服务、TSF、数据）
  wind-setting/      # 设置程序（AI 工具页、快捷键录制）
  wind-ui-rust/      # 设置程序使用的 windui GUI 库
```

`wind-setting` 的 `Cargo.toml` 使用相对路径依赖 `../wind-ui-rust` 和 `../WindInput-main/wind_input`，所以这三个目录必须保持同级。

### 前置工具

- Visual Studio 2022 Build Tools（Desktop development with C++ + Windows SDK）
- Rust stable MSVC 工具链
- Git / PowerShell 7+（或 Windows PowerShell 5.1）
- CMake（部分依赖会用到）

在 “x64 Native Tools Command Prompt for VS 2022” 中执行，或先运行 `vcvars64.bat`。

### 一键构建

```powershell
.\scripts\build.ps1
```

脚本会构建 core service 和 settings，并把两个 exe 复制到 `release/`。

### 手动构建

```powershell
cd src\WindInput-main\wind_input
cargo build --release -p wind_service
copy target\release\wind_input.exe ..\..\..\release\

cd ..\..\wind-setting
cargo build --release
copy target\release\wind_setting.exe ..\..\release\
```

> 运行时仍然依赖官方清风输入法的 `wind_tsf.dll`、词库和安装目录结构。
> 本仓库不包含词库数据，也不替代官方安装器。

更完整的构建、签名和排错说明见 [docs/BUILD.md](docs/BUILD.md)。

---

## 配置说明

### AI 工具页

| 配置 | 说明 |
|---|---|
| API Key | DPAPI 加密；只在当前 Windows 用户下可解密 |
| 接口地址 | 兼容 OpenAI 的 Base URL，可直接填到 `/v1` 或完整 `/chat/completions` |
| 模型 | 服务商模型名，如 `deepseek-chat` |
| 翻译目标 | `auto` / 中文 / 英文 / 日文 / 韩文 |
| 选区上限 | 超过后截断，避免超大请求 |
| 结果自动复制 | 完成后把结果写入剪贴板 |
| 请求超时 | 5–120 秒 |
| 快捷键 | 划词翻译 / 划词润色，支持“录制快捷键” |

### 录制快捷键

**AI 工具 → 快捷键 → 设置 → 录制快捷键**，按下组合键，确认后保存。

建议使用带修饰键的组合，例如 `Ctrl+Alt+F9`，避免和浏览器/系统的常用键冲突。

---

## 工作原理

```mermaid
flowchart LR
    A[选中文字 + 全局热键] --> B[UI Automation TextPattern]
    B -->|有选区| D[OpenAI 兼容接口]
    B -->|不支持/无选区| C[剪贴板探针 + Ctrl+C]
    C -->|拿到选区| D
    C -->|复制失败| E[提示没有读到选中文本]
    D --> F[结果气泡]
    F --> G[左键拖动 / 右键关闭]
```

更具体地说：

1. 全局热键触发 AI worker 线程；
2. 优先用 **UI Automation TextPattern** 读取当前焦点控件的选区；
3. 不支持 UIA 的应用，使用“写入不可混淆的探针 → Ctrl+C → 轮询剪贴板”的兜底；
4. 如果检测到 Alt 菜单模式，先退出菜单再复制；
5. 选区像 API Key 时拒绝处理；
6. 调用配置的 OpenAI 兼容接口；
7. 结果通过原生状态气泡显示，左键可拖动，右键关闭。

---

## 隐私与安全

- 选中的文本会发送到**你自己配置的 API 服务**；本插件没有中转服务器，也没有遥测。
- API Key 使用 Windows DPAPI 加密，不写入 `config.toml`，不会提交到本仓库。
- 日志只记录字符数，不记录选区正文；API Key 会在错误信息中脱敏。
- 选区如果匹配 `sk-...`、`Bearer ...` 或当前保存的 Key，会直接拒绝，不发送请求。
- 请只使用你信任的 API 服务商；不要把 Key 发到不可信代理。

> 这个仓库里**不包含任何真实 API Key**。`release` 里的程序是编译产物，首次使用需要你自己填写 Key。

---

## 常见问题

### 按了快捷键提示“没有读到选中文本”

- 确认先在目标应用中真正选中了文字；
- 某些游戏、远程桌面、管理员权限窗口不提供 UIA，也不响应模拟 Ctrl+C；
- 可先在记事本/浏览器里测试，确认功能正常；
- 如果目标程序以管理员身份运行，尝试也用管理员身份启动清风输入法（不建议长期这样做）。

### 提示 401 / 403

Key 不对、过期或没有该模型权限。重新在 AI 工具页粘贴 Key。

### 提示 400 / 404

通常是模型名或接口地址不对：

- DeepSeek：`https://api.deepseek.com/v1`，`deepseek-chat`
- 其他服务：确认是否需要 `/v1`，以及模型名是否准确

### 快捷键没反应

- 检查是否被浏览器/系统/其他软件占用；
- 在 AI 工具页把快捷键改成一个冷门组合；
- `Ctrl+T` 这类组合会抢占浏览器“新建标签页”，不建议使用。

### 结果气泡看不到

- 独占全屏游戏/播放器上，输入法浮层可能被系统抑制；
- 窗口化或无边框窗口通常正常；
- 结果气泡会在约 20 秒后自动隐藏。

### 设置页没有 AI 工具

说明只替换了 `wind_input.exe`，没有替换 `wind_setting.exe`。按本仓库的安装步骤重新覆盖设置程序。

---

## 卸载

### 用脚本恢复

```powershell
.\release\uninstall.ps1 -QingfengRoot "C:\Program Files\WindInput"
```

脚本会寻找最近的 `.bak-ai-*` 备份并恢复；如果目标服务原本在运行，会重新启动。

### 手动恢复

1. 退出清风输入法；
2. 把 `wind_input.exe.bak-ai-*` 和 `wind_setting.exe.bak-ai-*` 改回原名；
3. 重新启动。

如果没有备份，重新安装官方 0.127.0 覆盖即可；用户词库和配置不会被本插件修改。

---

## 仓库结构

```text
qingfeng-ai-plugin/
├─ README.md
├─ LICENSE                     # GPL-3.0（继承清风输入法）
├─ NOTICE.md                   # 上游项目与许可证说明
├─ docs/
│  ├─ INSTALL.md               # 更详细的安装/升级/排错
│  └─ BUILD.md                 # 从源码构建
├─ assets/logo.png
├─ release/
│  ├─ wind_input.exe
│  ├─ wind_setting.exe
│  ├─ install.ps1
│  ├─ uninstall.ps1
│  └─ SHA256SUMS.txt
├─ scripts/build.ps1
└─ src/
   ├─ WindInput-main/
   ├─ wind-setting/
   └─ wind-ui-rust/
```

---

## 版本与兼容性

- 插件版本：`0.127.0-ai.1`
- 基础版本：清风输入法 `0.127.0`
- 平台：Windows 10/11 x64
- 源码基线：
  - core：`1771e80`（含 UIA 选区、剪贴板探针、AI 安全校验、结果气泡交互）
  - settings：`529d71e`（AI 工具页、快捷键录制）
  - windui：`d9792f0`

因为插件修改了 core 和 settings 的内部协议，**不要把它和不同版本的 `wind_tsf.dll` 混用**。

---

## 许可证与致谢

本仓库包含/修改了以下项目：

- [清风输入法 WindInput](https://github.com/huanfeng/WindInput) — GPL-3.0
- [wind-setting](https://github.com/huanfeng/wind-setting) — GPL-3.0
- [wind-ui-rust](https://github.com/huanfeng/wind-ui-rust) — MIT OR Apache-2.0

因此本整合仓库按 **GPL-3.0** 分发。各组件的原始许可证文件保留在 `src/` 对应目录中；详见 [NOTICE.md](NOTICE.md)。

本项目与清风输入法官方无隶属关系；请遵守上游项目的品牌和许可证要求。
