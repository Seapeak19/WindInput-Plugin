use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::prompt;

#[derive(Debug, Clone)]
pub struct AiResult {
    pub text: String,
    pub model: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("还没有配置 API Key")]
    MissingKey,
    #[error("API Key 格式无效：{0}")]
    InvalidKey(String),
    #[error("接口地址为空")]
    MissingBaseUrl,
    #[error("模型名为空")]
    MissingModel,
    #[error("网络请求失败：{0}")]
    Network(String),
    #[error("接口返回错误（HTTP {status}）：{body}")]
    Http { status: u16, body: String },
    #[error("接口返回内容为空")]
    Empty,
    #[error("接口返回格式无法识别：{0}")]
    InvalidResponse(String),
}

pub fn translate(
    config: &wind_config::config::AiConfig,
    api_key: &str,
    text: &str,
) -> Result<AiResult, AiError> {
    call(
        config,
        api_key,
        &prompt::translate_system(&config.target),
        text,
    )
}

pub fn polish(
    config: &wind_config::config::AiConfig,
    api_key: &str,
    text: &str,
) -> Result<AiResult, AiError> {
    call(config, api_key, &prompt::polish_system(), text)
}

fn endpoint(base: &str) -> Result<String, AiError> {
    let base = base.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err(AiError::MissingBaseUrl);
    }
    if base.ends_with("/chat/completions") {
        return Ok(base.to_string());
    }
    if base.ends_with("/v1") {
        return Ok(format!("{base}/chat/completions"));
    }
    Ok(format!("{base}/v1/chat/completions"))
}

fn call(
    config: &wind_config::config::AiConfig,
    api_key: &str,
    system: &str,
    text: &str,
) -> Result<AiResult, AiError> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(AiError::MissingKey);
    }
    crate::secret::validate_api_key(api_key)
        .map_err(|e| AiError::InvalidKey(e.to_string()))?;
    if config.model.trim().is_empty() {
        return Err(AiError::MissingModel);
    }
    let url = endpoint(&config.base_url)?;
    let timeout = Duration::from_secs(config.timeout_seconds.clamp(5, 120));
    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let body = json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt::user_message(text)}
        ],
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "stream": false
    });

    let response = agent
        .post(&url)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| match e {
            ureq::Error::Status(status, response) => {
                let body = redact(&response.into_string().unwrap_or_default(), api_key);
                AiError::Http {
                    status,
                    body: body.chars().take(300).collect(),
                }
            }
            // ureq 的 Bad Header 错误会把整个 Authorization 值写进 Display；
            // 用错内容当 Key 时（例如误粘一段译文）绝不能把这段内容写进日志/气泡。
            ureq::Error::Transport(e) => AiError::Network(redact(&e.to_string(), api_key)),
        })?;

    let value: Value = response
        .into_json()
        .map_err(|e| AiError::InvalidResponse(e.to_string()))?;
    let content = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(AiError::Empty)?;
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(&config.model)
        .to_string();

    Ok(AiResult {
        text: content.to_string(),
        model,
    })
}

fn redact(message: &str, key: &str) -> String {
    if key.is_empty() {
        return message.to_string();
    }
    message
        .replace(&format!("Bearer {key}"), "Bearer <redacted>")
        .replace(key, "<redacted>")
}

/// Wrap http errors into anyhow at the boundary for callers that want a
/// one-line message.
pub fn error_message(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

pub fn ensure_non_empty(text: &str) -> Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("没有可处理的内容");
    }
    Ok(())
}

pub fn one_line(err: &anyhow::Error) -> String {
    error_message(err)
}

// Keep Context in scope for callers that wrap network errors.
#[allow(dead_code)]
fn _context_marker() -> Result<()> {
    Err(anyhow::anyhow!("unused")).context("unused")
}
