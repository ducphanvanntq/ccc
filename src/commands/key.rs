use serde_json::Value;

use crate::config::{default_settings_path, read_json, write_json};
use crate::utils::{mask_key, prompt};

pub fn run(key: Option<String>) {
    let api_key = key.unwrap_or_else(|| prompt("Enter ANTHROPIC_API_KEY: "));

    if api_key.is_empty() {
        eprintln!("API key cannot be empty.");
        std::process::exit(1);
    }

    let path = default_settings_path();
    if !path.exists() {
        eprintln!("Default settings not found at: {}", path.display());
        eprintln!("Please run install first.");
        std::process::exit(1);
    }

    let mut json = read_json(&path);
    json["env"]["ANTHROPIC_API_KEY"] = Value::String(api_key.clone());
    write_json(&path, &json);

    println!("Updated ANTHROPIC_API_KEY in {}", path.display());
    println!("Key: {}", mask_key(&api_key));
}
