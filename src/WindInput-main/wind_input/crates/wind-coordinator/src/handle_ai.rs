//! Global selection translation / polishing.
//!
//! Triggered by the two global hotkeys in `KeysConfig`. The heavy work runs on
//! a worker thread; the coordinator itself only starts the worker. Selection is
//! captured through the clipboard fallback (Ctrl+C) because the feature must
//! work while WindInput is not the active TSF profile.

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, info, warn};
use wind_cmdbar::KeyInjector;
use wind_keys::key_inject::SysKeys;
use wind_ui_types::{StatusTipPlacement, UiCommand};

use crate::coordinator::Coordinator;
use crate::host_services::HostServices;

#[derive(Debug, Clone, Copy)]
pub(crate) enum AiMode {
    Translate,
    Polish,
}

impl AiMode {
    fn label(self) -> &'static str {
        match self {
            Self::Translate => "翻译",
            Self::Polish => "润色",
        }
    }
}

pub(crate) fn spawn(coord: Arc<Coordinator>, mode: AiMode) {
    let _ = std::thread::Builder::new()
        .name("wind-ai-selection".to_string())
        .spawn(move || run(&coord, mode));
}

fn run(coord: &Coordinator, mode: AiMode) {
    show_loading(coord, mode);

    let host = coord.host_services().clone();
    let Some(text) = capture_selection(&host) else {
        show_error(coord, "没有读到选中文本：请先选中文字，再按快捷键");
        return;
    };
    let cfg = coord.rt().config.ai.clone();
    let max_chars = cfg.max_chars.clamp(200, 20_000);
    let mut text = text.trim().to_string();
    if text.chars().count() > max_chars {
        text = text.chars().take(max_chars).collect::<String>();
        text.push('…');
    }
    if text.is_empty() {
        show_error(coord, "选中文本为空");
        return;
    }

    // Environment fallback is useful for a portable/dev run and for the first
    // smoke test; the normal path is the DPAPI file written by settings.
    let key = match std::env::var("WIND_AI_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
    {
        Some(key) => key,
        None => match wind_ai::load_api_key() {
            Ok(Some(key)) if !key.trim().is_empty() => key,
            Ok(_) => {
                show_error(coord, "还没有配置 API Key，请到设置 → AI 工具中填写");
                return;
            }
            Err(e) => {
                show_error(coord, &format!("读取 API Key 失败：{e}"));
                return;
            }
        },
    };

    // 安全阀：用户完全可能先在设置里选中了 API Key 再按热键。剪贴板捕获会把
    // 密钥当正文发给模型，模型再原样吐回来，气泡就等于把密钥公开在屏幕上。
    // 这类内容一律拒绝，既不发请求也不显示。
    if looks_like_secret(&text, &key) {
        show_error(
            coord,
            "选区看起来是 API Key，已拒绝处理；请选中普通文本再试",
        );
        return;
    }

    // 只记长度，不记内容；排查“拿到的是旧剪贴板还是真正选区”时够用。
    info!("AI {} selection: {} chars", mode.label(), text.chars().count());
    debug!("AI {}: {} chars", mode.label(), text.chars().count());
    let result = match mode {
        AiMode::Translate => wind_ai::translate(&cfg, &key, &text),
        AiMode::Polish => wind_ai::polish(&cfg, &key, &text),
    };

    match result {
        Ok(result) => {
            // 模型可能把输入里的密钥原样带回来；无论输入侧有没有拦住，返回侧
            // 再挡一道，绝不把密钥写进剪贴板或状态气泡。
            if result.text.contains(key.trim()) || looks_like_secret(&result.text, &key) {
                show_error(coord, "返回内容疑似包含 API Key，已隐藏");
                return;
            }
            if cfg.auto_copy {
                // 气泡不可选中；自动写入剪贴板，用户可以直接 Ctrl+V 使用。
                let _ = host.clipboard_set_text(&result.text);
            }
            let display = wrap_for_tip(&result.text, 34, 20);
            show_tip(coord, &display, 20_000);
        }
        Err(e) => show_error(coord, &format!("{}失败：{e}", mode.label())),
    }
}

fn looks_like_secret(text: &str, key: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    let key = key.trim();
    if !key.is_empty() && (t == key || t.contains(key)) {
        return true;
    }
    if t.to_ascii_lowercase().starts_with("bearer ") {
        return true;
    }
    t.split_whitespace().any(|token| {
        let token = token.trim_matches(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '-' || c == '_')
        });
        token.len() >= 20
            && token.starts_with("sk-")
            && token[3..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
}

/// 写进剪贴板的哨兵。Ctrl+C 没生效时读回来的一定是它，绝不可能是上一次
/// 复制的内容；旧实现用空串清剪贴板，而空串写剪贴板是 no-op，于是会把旧
/// 剪贴板误当成选区。
const SELECTION_PROBE: &str = "\u{2063}WIND_AI_SELECTION_PROBE_8F4C";

/// 首选路径：直接问前台控件要选区（UI Automation TextPattern）。
///
/// 比“模拟 Ctrl+C + 剪贴板”稳：不污染剪贴板、不受 Alt 菜单模式影响，也能覆盖
/// 浏览器/Office 这类有 UIA 文本支持的宿主。拿不到（控件不支持、权限不够、
/// 没有选区）再走剪贴板兜底。
#[cfg(windows)]
fn capture_uia_selection() -> Option<String> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
    };

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let initialized = hr.is_ok();
        let selected = (|| {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            let focused = automation.GetFocusedElement().ok()?;
            // 密码框绝不读；即使后面还有 Key 形态守卫，这里也不该把密码带出来。
            if focused.CurrentIsPassword().ok()?.0 != 0 {
                return None;
            }
            let pattern: IUIAutomationTextPattern =
                focused.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
            let ranges = pattern.GetSelection().ok()?;
            let count = ranges.Length().ok()?;
            let mut text = String::new();
            for index in 0..count {
                if let Ok(range) = ranges.GetElement(index)
                    && let Ok(part) = range.GetText(-1)
                {
                    text.push_str(&part.to_string());
                }
            }
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        })();
        if initialized {
            CoUninitialize();
        }
        selected
    }
}

#[cfg(not(windows))]
fn capture_uia_selection() -> Option<String> {
    None
}

fn capture_selection(host: &Arc<dyn HostServices>) -> Option<String> {
    // RegisterHotKey does not necessarily release the modifiers before WM_HOTKEY.
    // Release them first, otherwise Ctrl+Alt+T is seen as Ctrl+Alt+C by the host.
    let keys = SysKeys;
    let _ = keys.release("Ctrl+Alt");
    std::thread::sleep(Duration::from_millis(35));
    close_menu_if_open(&keys);

    if let Some(text) = capture_uia_selection() {
        debug!("AI selection via UIA: {} chars", text.chars().count());
        return Some(text);
    }

    let saved = host.clipboard_get_text().ok();
    if host.clipboard_set_text(SELECTION_PROBE).is_err() {
        restore_clipboard(host, saved.as_deref());
        return None;
    }
    // 确认探测串真的进了剪贴板；打不开剪贴板时不要拿旧值凑数。
    std::thread::sleep(Duration::from_millis(25));
    if host.clipboard_get_text().ok().as_deref() != Some(SELECTION_PROBE) {
        restore_clipboard(host, saved.as_deref());
        return None;
    }

    if keys.tap("Ctrl+C").is_err() {
        restore_clipboard(host, saved.as_deref());
        return None;
    }

    // 复制是异步的（浏览器/Office 处理 WM_COPY 的时间差很大）。轮询到剪贴板
    // 离开哨兵为止；超时仍等于哨兵就当作“没有选区”，而不是返回旧内容。
    let deadline = std::time::Instant::now() + Duration::from_millis(700);
    let mut selected = None;
    while std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
        match host.clipboard_get_text() {
            Ok(text) if text != SELECTION_PROBE && !text.trim().is_empty() => {
                selected = Some(text);
                break;
            }
            _ => {}
        }
    }
    restore_clipboard(host, saved.as_deref());
    selected
}

/// Alt+T 这类热键在 RegisterHotKey 吞掉 T 后，Windows 仍可能因为 Alt 的
/// 按下/抬起进入菜单模式；此时 Ctrl+C 发给了菜单而不是正文，裁剪失败。
/// 只在系统报告确实处于菜单模式时补一个 Esc，避免普通应用里误清选区。
fn close_menu_if_open(keys: &SysKeys) {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            GUI_INMENUMODE, GUI_POPUPMENUMODE, GUITHREADINFO, GetForegroundWindow,
            GetGUIThreadInfo, GetWindowThreadProcessId,
        };
        let fg = unsafe { GetForegroundWindow() };
        if fg.is_invalid() {
            return;
        }
        let tid = unsafe { GetWindowThreadProcessId(fg, None) };
        if tid == 0 {
            return;
        }
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if unsafe { GetGUIThreadInfo(tid, &mut info) }.is_ok() {
            let menu_bits = GUI_INMENUMODE.0 | GUI_POPUPMENUMODE.0;
            if info.flags.0 & menu_bits != 0 {
                let _ = keys.tap("Esc");
                std::thread::sleep(Duration::from_millis(60));
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = keys;
    }
}

fn restore_clipboard(host: &Arc<dyn HostServices>, saved: Option<&str>) {
    match saved {
        Some(text) => {
            let _ = host.clipboard_set_text(text);
        }
        None => {
            // 原来的剪贴板没有文本（可能是图片）；至少不要把探测串/选区留在里面。
            let _ = host.clipboard_set_text("");
        }
    }
}

fn show_loading(coord: &Coordinator, mode: AiMode) {
    show_tip(coord, &format!("正在读取选区，准备{}…", mode.label()), 3_000);
}

fn show_error(coord: &Coordinator, message: &str) {
    warn!("AI tool: {}", message);
    // 错误也走气泡的硬换行路径，避免长句被单行窗口裁掉。
    show_tip(coord, &wrap_for_tip(message, 30, 6), 9_000);
}

fn show_tip(coord: &Coordinator, text: &str, duration_ms: u64) {
    let (x, y) = cursor_position();
    let _ = coord.ui_tx.send(UiCommand::ShowAiResult {
        text: text.to_string(),
        x,
        y,
        caret_height: 20,
        duration_ms,
        placement: StatusTipPlacement::Caret {
            offset_x: 0,
            offset_y: 0,
        },
    });
}

fn cursor_position() -> (i32, i32) {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut point = POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_ok() {
            return (point.x, point.y);
        }
    }
    (0, 0)
}

/// StatusTip's text layout is a single leaf; hard newlines become real lines,
/// but it does not wrap on its own. Keep the card narrow and bounded.
#[cfg(test)]
mod tests {
    use super::looks_like_secret;

    const FAKE: &str = "sk-0123456789abcdef0123456789abcdef";

    #[test]
    fn detects_api_keys_and_bearer_tokens() {
        assert!(looks_like_secret(FAKE, FAKE));
        assert!(looks_like_secret(&format!("前缀 {FAKE} 后缀"), FAKE));
        assert!(looks_like_secret(
            "Bearer 0123456789abcdef0123456789abcdef",
            FAKE
        ));
    }

    #[test]
    fn leaves_normal_text_alone() {
        assert!(!looks_like_secret("请把这句话润色一下", FAKE));
        assert!(!looks_like_secret("sku-1234 is not a key", FAKE));
    }
}

fn wrap_for_tip(text: &str, max_units: usize, max_lines: usize) -> String {
    // 状态气泡没有自动换行。按「半角 1、全角 2」估算显示宽度，给 44 个单位，
    // 大约 22 个汉字或 44 个拉丁字符；比按字符数硬切更不容易横向溢出。
    let max_units = max_units.max(16);
    let mut lines: Vec<String> = Vec::new();
    for source_line in text.lines() {
        let mut current = String::new();
        let mut units = 0usize;
        for ch in source_line.chars() {
            let w = if ch.is_ascii() { 1 } else { 2 };
            if units + w > max_units && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                units = 0;
                if lines.len() >= max_lines {
                    break;
                }
            }
            current.push(ch);
            units += w;
        }
        if lines.len() >= max_lines {
            break;
        }
        lines.push(current);
        if lines.len() >= max_lines {
            break;
        }
    }
    if lines.len() >= max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            if !last.ends_with('…') {
                last.push('…');
            }
        }
    }
    lines.join("\n")
}
