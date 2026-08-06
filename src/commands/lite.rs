use anyhow::{bail, Context, Result};
use std::path::Path;

use crate::config::{default_claude_dir, read_json, write_json, SETTINGS_FILE};
use crate::utils::{confirm, copy_dir_recursive, prompt};

const LITE_BASE_URL: &str = "https://litellm-proxy-ep-cncyfugmcnadc6g4.a02.azurefd.net";
const LITE_MODEL: &str = "claude-sonnet-5[1m]";
const LITE_SMALL_FAST_MODEL: &str = "claude-sonnet-5";
const LITE_DEFAULT_SONNET_MODEL: &str = "claude-sonnet-4.6[1m]";
const LITE_DEFAULT_OPUS_MODEL: &str = "claude-opus-4.7[1m]";
const LITE_DEFAULT_HAIKU_MODEL: &str = "claude-haiku-4.5";

pub fn run() -> Result<()> {
    let source = default_claude_dir()?;
    if !source.exists() {
        bail!("Default .claude folder not found at: {}. Please run the install script first.", source.display());
    }

    let target = Path::new(".claude");
    let target_settings = target.join(SETTINGS_FILE);

    if target_settings.exists()
        && !confirm("settings.local.json already exists. Overwrite? (y/N): ")
    {
        println!("Cancelled.");
        return Ok(());
    }

    // Ask for the auth token before writing anything.
    let auth_token = prompt("Enter ANTHROPIC_AUTH_TOKEN: ")?;
    if auth_token.is_empty() {
        bail!("ANTHROPIC_AUTH_TOKEN is required.");
    }

    copy_dir_recursive(&source, target).context("Failed to copy .claude folder")?;

    // Build the lite env config, preserving any existing settings.
    let mut json = read_json(&target_settings).unwrap_or_else(|_| serde_json::json!({}));
    let env = &mut json["env"];
    env["ANTHROPIC_BASE_URL"] = serde_json::Value::String(LITE_BASE_URL.to_string());
    env["ANTHROPIC_AUTH_TOKEN"] = serde_json::Value::String(auth_token);
    env["ANTHROPIC_MODEL"] = serde_json::Value::String(LITE_MODEL.to_string());
    env["ANTHROPIC_SMALL_FAST_MODEL"] = serde_json::Value::String(LITE_SMALL_FAST_MODEL.to_string());
    env["ANTHROPIC_DEFAULT_SONNET_MODEL"] = serde_json::Value::String(LITE_DEFAULT_SONNET_MODEL.to_string());
    env["ANTHROPIC_DEFAULT_OPUS_MODEL"] = serde_json::Value::String(LITE_DEFAULT_OPUS_MODEL.to_string());
    env["ANTHROPIC_DEFAULT_HAIKU_MODEL"] = serde_json::Value::String(LITE_DEFAULT_HAIKU_MODEL.to_string());
    write_json(&target_settings, &json)?;

    println!("Copied default .claude config to current directory.");
    println!("Applied lite config (base_url: {LITE_BASE_URL}).");
    println!("Done!");
    Ok(())
}
