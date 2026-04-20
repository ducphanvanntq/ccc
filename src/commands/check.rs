use crate::config::{default_settings_path, KeysStore};
use crate::utils::{check_api_key, get_api_config, mask_key, try_read_json};

pub fn run() {
    // Try to get key from local config first, then keys.json default
    let api_key = get_current_key();

    if api_key.is_empty() {
        eprintln!("API key not set. Run 'ccc key add' first.");
        return;
    }

    let (base_url, model) = get_api_config();

    println!("Checking API connection...");
    println!("  URL:   {base_url}/v1/messages");
    println!("  Model: {model}");
    println!("  Key:   {}", mask_key(&api_key));
    println!();

    let (ok, msg) = check_api_key(&api_key);
    if ok {
        println!("  [OK] API key is valid!");
    } else {
        println!("  [!!] API check failed: {msg}");
    }

    println!();
}

fn get_current_key() -> String {
    // 1. Try local .claude/settings.local.json
    let local = crate::config::local_settings_path();
    if local.exists() {
        if let Ok(json) = try_read_json(&local) {
            if let Some(key) = json["env"]["ANTHROPIC_API_KEY"].as_str() {
                if !key.is_empty() {
                    return key.to_string();
                }
            }
        }
    }

    // 2. Try default key from keys.json
    let store = KeysStore::load();
    if let Some(key) = store.get_active_key() {
        return key.clone();
    }

    // 3. Try global settings
    let global = default_settings_path();
    if global.exists() {
        if let Ok(json) = try_read_json(&global) {
            if let Some(key) = json["env"]["ANTHROPIC_API_KEY"].as_str() {
                if !key.is_empty() {
                    return key.to_string();
                }
            }
        }
    }

    String::new()
}
