清风输入法 AI 划词插件 0.127.0-ai.2
=====================================

最简单的安装：
  0. 先安装官方清风输入法 0.127.0。
  1. 双击 QingfengAI-Setup.exe。
  2. UAC 选“是”。
  3. 确认目录，点“安装插件”。
  4. 打开设置 -> AI 工具，填写 API Key。

安装程序会备份旧文件，只替换：
  wind_input.exe
  wind_setting.exe
不会动 wind_tsf.dll、词库和用户配置。

默认快捷键：
  翻译 Ctrl+Alt+T
  润色 Ctrl+Alt+S

脚本安装（高级）：
  .\install.ps1
  .\install.ps1 -QingfengRoot "D:\Apps\WindInput"

交给 Agent 安装：
  见 ../docs/AGENT_INSTALL.md
  或：
  powershell -ExecutionPolicy Bypass -File .\agent-install.ps1 -Check

卸载：
  .\uninstall.ps1 -QingfengRoot "你的安装目录"
  或双击 Setup.exe 后点“卸载 / 恢复”。

详细说明见 ../README.md 与 ../docs/INSTALL.md。
