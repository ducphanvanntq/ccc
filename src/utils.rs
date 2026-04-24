use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::config::{default_settings_path, local_settings_path, KeysStore};

pub fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).context("Failed to read input")?;
    Ok(input.trim().to_string())
}

pub fn confirm(message: &str) -> bool {
    let answer = prompt(message).unwrap_or_default();
    matches!(answer.to_lowercase().as_str(), "y" | "yes")
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() > 8 {
        let first: String = chars[..4].iter().collect();
        let last: String = chars[chars.len() - 4..].iter().collect();
        format!("{first}...{last}")
    } else {
        "****".to_string()
    }
}

pub fn print_json_pretty(value: &serde_json::Value) {
    match colored_json::to_colored_json(value, colored_json::ColorMode::Auto(colored_json::Output::StdOut)) {
        Ok(colored) => println!("{colored}"),
        Err(_) => {
            if let Ok(json_str) = serde_json::to_string_pretty(value) {
                println!("{json_str}");
            }
        }
    }
}

/// Get API config from global settings
pub fn get_api_config() -> (String, String) {
    let path = match default_settings_path() {
        Ok(p) => p,
        Err(_) => return default_api_config(),
    };
    if path.exists() {
        if let Ok(json) = crate::config::read_json(&path) {
            let base_url = json["env"]["ANTHROPIC_BASE_URL"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("https://api.anthropic.com")
                .to_string();
            let model = json["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("claude-sonnet-4-20250514")
                .to_string();
            return (base_url, model);
        }
    }
    default_api_config()
}

fn default_api_config() -> (String, String) {
    ("https://api.anthropic.com".to_string(), "claude-sonnet-4-20250514".to_string())
}

/// Check if an API key is valid by calling the API
pub fn check_api_key(api_key: &str) -> (bool, String) {
    let (base_url, model) = get_api_config();
    let url = format!("{base_url}/v1/messages");
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1,
        "messages": [{ "role": "user", "content": "hi" }]
    });
    let body_str = serde_json::to_string(&body).unwrap_or_default();

    let result = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send(body_str.as_bytes());

    match result {
        Ok(mut resp) => {
            let text = resp.body_mut().read_to_string().unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            if json["content"].is_array() {
                (true, "OK".to_string())
            } else if json["error"].is_object() {
                let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
                (false, msg.to_string())
            } else {
                (false, "unexpected response".to_string())
            }
        }
        Err(e) => (false, format!("{e}")),
    }
}

/// Validate API key format
pub fn validate_key_format(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Key cannot be empty.".to_string());
    }
    if key.len() < 10 {
        return Err("Key is too short.".to_string());
    }
    if key.contains(' ') {
        return Err("Key cannot contain spaces.".to_string());
    }
    Ok(())
}

/// Resolve current API key: local config → keys.json default → global config
pub fn get_current_key() -> Option<String> {
    // 1. Try local .claude/settings.local.json
    let local = local_settings_path();
    if local.exists() {
        if let Ok(json) = crate::config::read_json(&local) {
            if let Some(key) = json["env"]["ANTHROPIC_API_KEY"].as_str() {
                if !key.is_empty() {
                    return Some(key.to_string());
                }
            }
        }
    }

    // 2. Try default key from keys.json
    let store = KeysStore::load();
    if let Some(key) = store.get_active_key() {
        return Some(key.clone());
    }

    // 3. Try global settings
    if let Ok(global) = default_settings_path() {
        if global.exists() {
            if let Ok(json) = crate::config::read_json(&global) {
                if let Some(key) = json["env"]["ANTHROPIC_API_KEY"].as_str() {
                    if !key.is_empty() {
                        return Some(key.to_string());
                    }
                }
            }
        }
    }

    None
}
