清风输入法 AI 划词插件 0.127.0-ai.1
=====================================

安装前：
1. 先安装官方清风输入法 0.127.0。
2. 退出正在运行的清风输入法。

默认安装（C:\Program Files\WindInput）：
  .\install.ps1

自定义目录：
  .\install.ps1 -QingfengRoot "D:\Apps\WindInput"

安装后：
  打开设置 -> AI 工具，填写 API Key。

默认快捷键：
  翻译 Ctrl+Alt+T
  润色 Ctrl+Alt+S

本包只替换 wind_input.exe 和 wind_setting.exe。
不要替换 wind_tsf.dll。

卸载：
  .\uninstall.ps1 -QingfengRoot "你的安装目录"

详细说明见 ../README.md 与 ../docs/INSTALL.md。
