use std::path::PathBuf;

use anyhow::{Context, Result};

fn key_path() -> Result<PathBuf> {
    let dir = wind_config::Config::user_config_dir()
        .context("找不到用户配置目录，无法保存 API Key")?;
    Ok(dir.join("ai_key.bin"))
}

pub fn has_api_key() -> bool {
    load_api_key().ok().flatten().is_some_and(|k| !k.is_empty())
}

/// API Key 必须是单行 ASCII 可见字符。
///
/// 这不是过度严格：HTTP `Authorization` 头不能含中文、空格、换行或控制字符。
/// 把其他内容（例如一段译文）误粘进 Key 输入框时，ureq 会在**发请求前**报
/// `Bad Header`，旧实现还会把整段内容写进错误日志。这里提前挡住并给出可操作的
/// 提示，避免把“粘贴错了”误报成“网络失败”。
pub fn validate_api_key(key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("API Key 为空");
    }
    if key.len() > 512 {
        anyhow::bail!("API Key 过长（超过 512 字符）");
    }
    if !key.is_ascii() {
        anyhow::bail!("API Key 含中文/全角字符，请重新粘贴纯英文 Key");
    }
    if !key.bytes().all(|b| b.is_ascii_graphic()) {
        anyhow::bail!("API Key 含空格或换行，请只保留 Key 本身");
    }
    Ok(())
}

/// Save an API key encrypted to the current Windows user. An empty string
/// removes the stored key.
pub fn save_api_key(key: &str) -> Result<()> {
    let path = key_path()?;
    let key = key.trim();
    if key.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path).with_context(|| format!("删除 {}", path.display()))?;
        }
        return Ok(());
    }
    validate_api_key(key)?;
    let protected = protect(key.as_bytes())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, protected).with_context(|| format!("写入 {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| format!("替换 {}", path.display()))?;
    Ok(())
}

pub fn load_api_key() -> Result<Option<String>> {
    let path = key_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).with_context(|| format!("读取 {}", path.display()))?;
    if bytes.is_empty() {
        return Ok(None);
    }
    let plain = unprotect(&bytes)?;
    let key = String::from_utf8(plain).context("API Key 不是有效 UTF-8")?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Ok(None);
    }
    validate_api_key(&key).context("已保存的 API Key 格式无效")?;
    Ok(Some(key))
}

pub fn clear_api_key() -> Result<()> {
    save_api_key("")
}

#[cfg(windows)]
fn protect(data: &[u8]) -> Result<Vec<u8>> {
    use windows::core::w;
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
    };
    use windows::Win32::Foundation::LocalFree;

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len().try_into().context("API Key 过长")?,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &input,
            w!("WindInput AI API key"),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .context("Windows DPAPI 加密失败")?;
    }
    let result = unsafe {
        std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
    };
    unsafe {
        let _ = LocalFree(HLOCAL(output.pbData as _));
    }
    Ok(result)
}

#[cfg(windows)]
fn unprotect(data: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
    };
    use windows::Win32::Foundation::LocalFree;

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len().try_into().context("加密的 API Key 过长")?,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .context("Windows DPAPI 解密失败（Key 可能由其他 Windows 用户保存）")?;
    }
    let result = unsafe {
        std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
    };
    unsafe {
        let _ = LocalFree(HLOCAL(output.pbData as _));
    }
    Ok(result)
}

#[cfg(not(windows))]
fn protect(_data: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("划词 AI 的 Key 存储目前只支持 Windows")
}

#[cfg(not(windows))]
fn unprotect(_data: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("划词 AI 的 Key 存储目前只支持 Windows")
}

#[cfg(test)]
mod tests {
    use super::validate_api_key;

    #[test]
    fn accepts_normal_bearer_keys() {
        for key in ["sk-abc123", "sk-proj_AbC-123.456", "a+b/c="] {
            validate_api_key(key).expect("常见 API Key 应通过");
        }
    }

    #[test]
    fn rejects_pasted_prose_and_whitespace() {
        assert!(validate_api_key("这是润色后的文本。").is_err());
        assert!(validate_api_key("sk-abc 123").is_err());
        assert!(validate_api_key("sk-abc\n123").is_err());
        assert!(validate_api_key("").is_err());
    }
}

