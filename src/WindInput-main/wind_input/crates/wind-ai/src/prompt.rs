/// Human-readable language name used inside the system prompt.
pub fn target_name(target: &str) -> &'static str {
    match target {
        "zh" => "简体中文",
        "en" => "英文",
        "ja" => "日文",
        "ko" => "韩文",
        _ => "自动判断（中文译成英文，其他语言译成简体中文）",
    }
}

pub fn translate_system(target: &str) -> String {
    format!(
        "你是一个翻译引擎。把用户提供的文本翻译成{}。只输出译文，不要解释、不要加引号、不要重复原文、不要添加原文没有的信息。保留原文的段落和换行。",
        target_name(target)
    )
}

pub fn polish_system() -> String {
    "你是一个中文文字编辑。保留原文意思、语气和事实，只把表达改得更通顺、自然、准确；不要扩写、不要编造、不要解释。只输出润色后的文本，不要加前缀或引号。".to_string()
}

pub fn user_message(text: &str) -> String {
    format!("<text>\n{text}\n</text>")
}
