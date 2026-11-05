# 交给 Agent 安装

本文说明如何把本仓库“一键”交给一个**能执行 Windows 命令**的 Agent（Claude Code、Cursor、Codex、DeepSeek Harness 等）安装。

先给结论：

- **可以**：Agent 有本机 shell，且能写清风输入法目录（管理员，或目录对当前用户可写）。
- **不可以全自动**：如果清风输入法装在 `Program Files` 且 Agent 没有管理员权限，Windows 的 UAC 必须由用户本人确认，Agent 不能代替点击。
- **不包含基础输入法**：必须先安装官方清风输入法 **0.127.0**。本仓库只是覆盖两个 exe，不提供 TSF DLL、词库和官方安装器。

---

## 1. 给 Agent 的完整提示词

把下面这段直接发给 Agent：

```text
请在当前 Windows 机器上安装清风输入法 AI 划词插件：
https://github.com/Seapeak19/WindInput-Plugin

前提与约束：
1. 这是 Windows 插件，基础输入法必须是官方清风输入法 0.127.0。
2. 只允许替换 wind_input.exe 和 wind_setting.exe；
   绝对不要替换、注册或修改 wind_tsf.dll、词库、用户 config.toml。
3. 先 clone 仓库到临时目录，检查 release/agent-install.ps1，再执行。

步骤：
1. git clone https://github.com/Seapeak19/WindInput-Plugin.git
2. cd WindInput-Plugin
3. powershell -ExecutionPolicy Bypass -File release\agent-install.ps1 -Check
4. 如果输出 QINGFENG_ROOT 为空：停止，请用户先安装官方 0.127.0，
   或让用户提供清风输入法安装目录，再用 -QingfengRoot 指定。
5. 如果 ADMIN=False 且 WRITABLE=False：停止，请用户用管理员权限运行，
   或改用 release\QingfengAI-Setup.exe。
6. 执行：
   powershell -ExecutionPolicy Bypass -File release\agent-install.ps1
   自定义目录时：
   powershell -ExecutionPolicy Bypass -File release\agent-install.ps1 -QingfengRoot "<目录>"
7. 确认输出包含 OK，然后检查 wind_input 进程和两个 exe 的 SHA256。
8. 告诉用户：打开 清风输入法设置 -> AI 工具，填写 API Key。

不要使用 irm ... | iex 这类远程直接执行脚本的方式。
不要自动修改用户 API Key 或 config.toml。
如果遇到 UAC 弹窗且无法操作，停止并让用户手动确认。
```

---

## 2. 检查环境

```powershell
git clone https://github.com/Seapeak19/WindInput-Plugin.git
cd WindInput-Plugin

powershell -ExecutionPolicy Bypass -File release\agent-install.ps1 -Check
```

输出示例（已安装、可写目录）：

```text
QINGFENG_ROOT=D:\Apps\WindInput
ADMIN=False
WRITABLE=True
```

输出示例（标准 Program Files）：

```text
QINGFENG_ROOT=C:\Program Files\WindInput
ADMIN=True
WRITABLE=False
```

判断：

| 结果 | 能否安装 |
|---|---|
| `ADMIN=True` | 可以 |
| `ADMIN=False` + `WRITABLE=True` | 通常可以（服务停止权限仍需满足） |
| 两者都 `False` | 不可以，需用户用管理员运行 |
| `QINGFENG_ROOT` 为空 | 先安装官方 0.127.0，或传 `-QingfengRoot` |

`-Check` 不会写文件、不会停服务。

---

## 3. 执行安装

自动检测目录：

```powershell
powershell -ExecutionPolicy Bypass -File release\agent-install.ps1
```

指定目录：

```powershell
powershell -ExecutionPolicy Bypass -File release\agent-install.ps1 -QingfengRoot "D:\Apps\WindInput"
```

成功输出类似：

```text
INSTALLED=wind_input.exe
INSTALLED=wind_setting.exe
START_REQUESTED=wind_input.exe
QINGFENG_ROOT=D:\Apps\WindInput
BACKUP_SUFFIX=.bak-ai-20261105-183333
OK
```

脚本行为：

1. 校验 `wind_tsf.dll` 和 `wind_input.exe`；
2. 只停止目标目录下的 `wind_input` / `wind_setting`；
3. 备份两个 exe 为 `.bak-ai-时间戳`；
4. 从 `release/` 复制新 exe；
5. 尝试通过 Explorer 以普通用户权限启动服务（避免管理员权限继承）；
6. 输出可解析的结果。

如果 Agent 是管理员，**不要**让它直接 `Start-Process wind_input.exe`；那会把输入法服务拉成高完整性进程，普通应用可能无法正常注入。脚本已经用 Explorer 代启动。

---

## 4. 验证

```powershell
Get-Process wind_input -ErrorAction SilentlyContinue |
    Select-Object Id, Path

Get-FileHash release\wind_input.exe -Algorithm SHA256
Get-FileHash release\wind_setting.exe -Algorithm SHA256
Get-Content release\SHA256SUMS.txt
```

`release/SHA256SUMS.txt` 中的哈希应与构建产物一致。

然后让用户：

1. 打开清风输入法设置；
2. 进入 **AI 工具**；
3. 填写 API Key；
4. 选中文字，按 `Ctrl+Alt+T` / `Ctrl+Alt+S` 测试。

---

## 5. 限制与边界

### 不能代替 UAC

Agent 不能点击 Windows 的 UAC 弹窗。若目录不可写且不是管理员，必须由用户确认，或改用 `QingfengAI-Setup.exe` 手动安装。

### 不能安装基础输入法

本仓库不包含官方 TSF DLL/词库。基础输入法未安装时，Agent 应停止并让用户从 <https://windinput.com/download> 安装 0.127.0。

### 启动权限

如果服务原本以管理员身份运行，脚本停止/启动它可能需要更高权限；此时让用户手动退出并以普通用户启动。

### 回滚

找到目标目录下最新的：

```text
wind_input.exe.bak-ai-*
wind_setting.exe.bak-ai-*
```

改回原名，或运行：

```powershell
powershell -ExecutionPolicy Bypass -File release\uninstall.ps1 -QingfengRoot "<目录>"
```

### 不要用远程 `iex`

下面这种写法**不要用**：

```powershell
# 不要这样做
irm https://raw.githubusercontent.com/.../agent-install.ps1 | iex
```

正确方式是 clone 仓库、检查脚本、再执行。

---

## 6. 一句话总结

**Agent 能装，但前提是基础清风输入法已安装，且 Agent 有写权限或管理员权限；UAC 不能由 Agent 代点。**
