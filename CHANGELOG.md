# Changelog

## 0.127.0-ai.1 — 2026-11-05

首个整合版。

### 新增

- 划词翻译（全局热键）
- 划词润色（全局热键）
- AI 工具设置页：接口地址、模型、API Key、翻译目标、选区上限、自动复制、超时
- 可录制快捷键
- 结果气泡：左键拖动、右键关闭
- OpenAI 兼容接口，默认 DeepSeek
- Windows DPAPI 加密保存 API Key

### 稳定性

- 优先使用 UI Automation 读取选区
- UIA 不可用时回退到探针 + `Ctrl+C` 剪贴板
- 修复空字符串写剪贴板是 no-op、导致读到旧剪贴板的问题
- 修复 Alt 热键进入菜单模式后复制失败的问题
- 拒绝把 API Key / Bearer Token 当正文发送
- API 错误和日志对 Key 脱敏
- 保存/加载 Key 时校验单行 ASCII

### 兼容性

- 清风输入法 0.127.0
- Windows 10/11 x64
