use anyhow::{bail, Result};

use crate::api::{get_api_config, get_current_key};
use crate::ui;
use crate::utils::mask_key;

pub fn run() -> Result<()> {
    let api_key = match get_current_key() {
        Some(key) => key,
        None => bail!("API key not set. Run 'ccc key add' first."),
    };

    let (base_url, model) = get_api_config();
    let url = format!("{base_url}/v1/messages");

    // Header
    println!();
    ui::print_header(&ui::ICON_SEARCH, "API Connection Check");

    // Connection info
    ui::print_row("API", &url);
    ui::print_row("Model", &model);
    ui::print_row("Key", &mask_key(&api_key));

    // Separator
    ui::print_separator();

    // Run checks
    let mut pass = 0;
    let mut fail = 0;

    // 1. Connection check
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1,
        "messages": [{ "role": "user", "content": "hi" }]
    });
    let body_str = serde_json::to_string(&body).unwrap_or_default();

    let result = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .send(body_str.as_bytes());

    // Clear and show results

    match result {
        Ok(mut resp) => {
            ui::print_check(true, "Connection", "200 OK");
            pass += 1;

            let text = resp.body_mut().read_to_string().unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();

            // 2. Auth check
            if json["error"]["type"].as_str() == Some("authentication_error") {
                ui::print_check(false, "Auth", "invalid key");
                fail += 1;
            } else {
                ui::print_check(true, "Auth", "valid");
                pass += 1;
            }

            // 3. Model check
            if json["error"]["type"].as_str() == Some("not_found_error") {
                let msg = json["error"]["message"].as_str().unwrap_or("model not found");
                ui::print_check(false, "Model", msg);
                fail += 1;
            } else {
                ui::print_check(true, "Model", "available");
                pass += 1;
            }

            // 4. Response / billing check
            if json["content"].is_array() {
                ui::print_check(true, "Response", "OK");
                pass += 1;
            } else if json["error"].is_object() {
                let err_type = json["error"]["type"].as_str().unwrap_or("unknown");
                let err_msg = json["error"]["message"].as_str().unwrap_or("unknown error");
                if err_type == "authentication_error" {
                    // Already reported above
                } else {
                    ui::print_check(false, "Response", err_msg);
                    fail += 1;
                }
            }
        }
        Err(e) => {
            let msg = format!("{e}");
            // Check if it's an HTTP error with a response body
            if let Some(status) = extract_status(&msg) {
                if status >= 400 && status < 500 {
                    ui::print_check(true, "Connection", &format!("{status}"));
                    pass += 1;
                    ui::print_check(false, "Auth/API", &msg);
                    fail += 1;
                } else {
                    ui::print_check(false, "Connection", &msg);
                    fail += 1;
                }
            } else {
                ui::print_check(false, "Connection", &msg);
                fail += 1;
            }
        }
    }

    // Result
    ui::print_separator();
    ui::print_result_line(pass, fail);
    ui::print_footer();
    println!();

    Ok(())
}

fn extract_status(msg: &str) -> Option<u16> {
    // Try to extract HTTP status code from error message
    if msg.contains("401") { return Some(401); }
    if msg.contains("403") { return Some(403); }
    if msg.contains("404") { return Some(404); }
    if msg.contains("429") { return Some(429); }
    if msg.contains("500") { return Some(500); }
    None
}
