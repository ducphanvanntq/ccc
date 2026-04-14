use serde_json::json;

use crate::config::{default_settings_path, read_json};

pub fn run() {
    let path = default_settings_path();
    if !path.exists() {
        eprintln!("Global settings not found. Please run install first.");
        std::process::exit(1);
    }

    let settings = read_json(&path);

    let api_key = settings["env"]["ANTHROPIC_API_KEY"]
        .as_str()
        .unwrap_or("");
    if api_key.is_empty() {
        eprintln!("API key not set. Run 'ccc key' first.");
        std::process::exit(1);
    }

    let base_url = settings["env"]["ANTHROPIC_BASE_URL"]
        .as_str()
        .unwrap_or("https://api.anthropic.com");

    let model = settings["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"]
        .as_str()
        .unwrap_or("claude-sonnet-4-20250514");

    let url = format!("{base_url}/v1/messages");

    println!("Checking API connection...");
    println!("  URL:   {url}");
    println!("  Model: {model}");
    println!();

    let body = json!({
        "model": model,
        "max_tokens": 64,
        "messages": [
            {
                "role": "user",
                "content": "Say OK"
            }
        ]
    });

    let body_str = serde_json::to_string(&body).expect("Failed to serialize body");

    let result = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send(body_str.as_bytes());

    match result {
        Ok(mut resp) => {
            let text: String = resp.body_mut().read_to_string().unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();

            if json["content"].is_array() {
                let reply = json["content"][0]["text"].as_str().unwrap_or("");
                println!("  [OK] API key is valid!");
                println!("  Response: {reply}");
            } else if json["error"].is_object() {
                let msg = json["error"]["message"].as_str().unwrap_or("unknown error");
                println!("  [!!] API returned error: {msg}");
            } else {
                println!("  [!!] Unexpected response: {text}");
            }
        }
        Err(e) => {
            println!("  [!!] Cannot connect to API: {e}");
            println!("  Key has been set but connection failed.");
        }
    }

    println!();
}
