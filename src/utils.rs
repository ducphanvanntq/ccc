use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::config::default_settings_path;

pub fn prompt(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

pub fn confirm(message: &str) -> bool {
    let answer = prompt(message);
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
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "****".to_string()
    }
}

pub fn print_json_pretty(value: &serde_json::Value) {
    match colored_json::to_colored_json(value, colored_json::ColorMode::Auto(colored_json::Output::StdOut)) {
        Ok(colored) => println!("{colored}"),
        Err(_) => {
            let json_str = serde_json::to_string_pretty(value).expect("Failed to serialize JSON");
            println!("{json_str}");
        }
    }
}

/// Get API config from global settings
pub fn get_api_config() -> (String, String) {
    let path = default_settings_path();
    if path.exists() {
        if let Ok(json) = try_read_json(&path) {
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

/// Try to read JSON file, returns Result instead of panicking
pub fn try_read_json(path: &Path) -> Result<serde_json::Value, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))
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
