# 安装、升级与卸载

本文是 [README](../README.md) 的详细安装说明。适用于 Windows 10/11 x64。

---

## 0. 先确认基础版本

本插件按 **清风输入法 0.127.0** 构建。

检查方法：

1. 打开清风输入法设置 → 关于；
2. 或查看安装目录 `wind_input.exe` 的文件版本；
3. 如果不是 0.127.x，先安装官方 0.127.0。

```powershell
(Get-Item "C:\Program Files\WindInput\wind_input.exe").VersionInfo.FileVersion
```

不要只复制 `wind_input.exe` 到不同版本的安装目录；TSF、RPC 协议和配置结构可能不兼容。

---

## 1. 安装官方清风输入法

从 <https://windinput.com/download> 下载安装。

安装完成后确认：

- 能用 `Win + Space` 切换到清风输入法；
- 能正常打字；
- 安装目录里有：
  - `wind_input.exe`
  - `wind_setting.exe`
  - `wind_tsf.dll`
  - `data\` 等运行资源。

本插件**不提供** TSF DLL、词库和安装器，必须依赖官方基础安装。

---

## 2. 选择安装方式

### A. 脚本安装（推荐）

1. 退出清风输入法（托盘退出，或结束 `wind_input` / `wind_setting` 进程）。
2. 在仓库 `release` 目录打开 PowerShell。
3. 运行：

```powershell
.\install.ps1
```

默认目录是 `C:\Program Files\WindInput`。便携版或其他目录：

```powershell
.\install.ps1 -QingfengRoot "D:\Apps\WindInput"
```

`Program Files` 需要管理员权限；否则复制会失败。

脚本做这些事：

- 校验 `wind_input.exe` 和 `wind_tsf.dll` 是否存在；
- 停止服务；
- 生成 `.bak-ai-yyyyMMdd-HHmmss` 备份；
- 覆盖两个 exe；
- 重新启动服务。

### B. 手动覆盖

1. 退出清风输入法；
2. 备份安装目录里的 `wind_input.exe`、`wind_setting.exe`；
3. 用本仓库 `release` 下的同名文件覆盖；
4. 启动 `wind_input.exe`。

只替换 `wind_input.exe` 时，翻译/润色可能能用，但设置页缺少 AI 工具和 Key 输入，**不建议**。

### C. 从源码构建后安装

见 [BUILD.md](BUILD.md)。构建产物同样只有两个 exe，安装方式与 A/B 相同。

---

## 3. 校验文件

`release/SHA256SUMS.txt` 包含两个 exe 的 SHA-256。

```powershell
Get-FileHash .\release\wind_input.exe -Algorithm SHA256
Get-FileHash .\release\wind_setting.exe -Algorithm SHA256
```

与 `SHA256SUMS.txt` 对比即可。

---

## 4. 首次配置

1. 启动清风输入法；
2. 打开设置 → **AI 工具**；
3. 填写：

| 字段 | DeepSeek 示例 |
|---|---|
| 接口地址 | `https://api.deepseek.com/v1` |
| 模型 | `deepseek-chat` |
| API Key | 你的真实 Key |

4. 按 Enter 或点“应用设置”；
5. 在浏览器/记事本里选中一段文字；
6. 按默认 `Ctrl+Alt+T`（翻译）或 `Ctrl+Alt+S`（润色）。

Key 使用 Windows DPAPI 加密，保存在用户数据目录的 `ai_key.bin`，不写入 `config.toml`。

### 修改快捷键

**AI 工具 → 快捷键 → 设置 → 录制快捷键**：

1. 点“录制快捷键”；
2. 按下新的组合键；
3. 点“确认”；
4. 点底部“应用设置”。

如果组合键已被其他程序占用，设置页会提示冲突。

---

## 5. 升级插件

1. 备份当前安装目录；
2. 退出清风输入法；
3. 用新版本 `release` 里的两个 exe 覆盖；
4. 启动服务。

新版本的 `install.ps1` 会额外生成一份新的 `.bak-ai-*`，不会删除旧备份。

如果基础清风输入法也升级了主版本（例如 0.128），必须使用对应版本重新构建/获取插件，不能直接覆盖。

---

## 6. 卸载

### 用脚本

```powershell
.\release\uninstall.ps1 -QingfengRoot "C:\Program Files\WindInput"
```

它会找每个 exe 最近的 `.bak-ai-*` 备份并恢复。

### 手动

1. 退出服务；
2. 将 `wind_input.exe.bak-ai-*` 改回 `wind_input.exe`；
3. 将 `wind_setting.exe.bak-ai-*` 改回 `wind_setting.exe`；
4. 启动。

没有备份时，重新安装官方 0.127.0 覆盖即可。

用户词库、配置和 `ai_key.bin` 不会被卸载脚本主动删除。若不再使用 AI 功能，可在设置里清空 API Key，或手动删除用户数据目录下的 `ai_key.bin`。

---

## 7. 文件职责

| 文件 | 是否替换 | 说明 |
|---|---|---|
| `wind_input.exe` | 是 | 输入法服务 + AI 请求 + 结果气泡 |
| `wind_setting.exe` | 是 | 设置界面 + AI 工具页 + 快捷键录制 |
| `wind_tsf.dll` | **否** | 官方 TSF 文本服务，保持原样 |
| `data/` | **否** | 词库、方案、主题等运行资源 |
| 用户 `config.toml` | **否** | 插件只读取/写入 AI 相关字段 |
| `ai_key.bin` | 新增 | DPAPI 加密的 API Key |

---

## 8. 安装失败排查

### “没有找到清风输入法”

说明默认路径没有官方安装。用：

```powershell
.\install.ps1 -QingfengRoot "你的实际安装目录"
```

### “拒绝访问” / 复制失败

用管理员 PowerShell，或把清风输入法装在用户可写目录（便携版）。

### 覆盖后设置页打不开

确认 `wind_setting.exe` 也替换了，并且和 `wind_input.exe` 来自同一版本。

### 覆盖后输入法不能打字

通常是 TSF 或版本不匹配：

1. 恢复 `.bak-ai-*`；
2. 重新安装官方 0.127.0；
3. 确认没有替换 `wind_tsf.dll`。

### 想完全回到官方

重新运行官方安装器覆盖安装，最稳妥。
