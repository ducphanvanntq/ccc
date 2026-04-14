use std::path::Path;
use std::process::Command;

use crate::config::{ccc_home, default_settings_path, local_settings_path, read_json, SETTINGS_FILE};

fn check(label: &str, ok: bool, detail: &str) {
    if ok {
        println!("  [OK] {label}: {detail}");
    } else {
        println!("  [!!] {label}: {detail}");
    }
}

pub fn run() {
    println!("ccc doctor\n");

    // 1. Check Claude Code installed
    let claude_found = find_claude();
    match &claude_found {
        Some(path) => check("Claude Code", true, path),
        None => check("Claude Code", false, "not found. Install from https://claude.ai/download"),
    }

    // 2. Check ccc home
    let home = ccc_home();
    check("ccc home", home.exists(), &home.display().to_string());

    // 3. Check global default config
    let global_path = default_settings_path();
    let global_ok = global_path.exists();
    if global_ok {
        check("Global config", true, &global_path.display().to_string());

        // Check API key in global
        let json = read_json(&global_path);
        let has_key = json["env"]["ANTHROPIC_API_KEY"]
            .as_str()
            .map(|k| !k.is_empty())
            .unwrap_or(false);
        check("API key (global)", has_key, if has_key { "set" } else { "not set. Run 'ccc key' to set" });
    } else {
        check("Global config", false, "not found. Run install script first");
    }

    // 4. Check local .claude folder
    let local_dir = Path::new(".claude");
    check(".claude folder (local)", local_dir.exists(), &local_dir.display().to_string());

    // 5. Check local settings
    let local_path = local_settings_path();
    let local_ok = local_path.exists();
    if local_ok {
        check(SETTINGS_FILE, true, &local_path.display().to_string());

        let json = read_json(&local_path);
        let has_key = json["env"]["ANTHROPIC_API_KEY"]
            .as_str()
            .map(|k| !k.is_empty())
            .unwrap_or(false);
        check("API key (local)", has_key, if has_key { "set" } else { "not set" });

        let has_url = json["env"]["ANTHROPIC_BASE_URL"]
            .as_str()
            .map(|u| !u.is_empty())
            .unwrap_or(false);
        check("Base URL (local)", has_url, json["env"]["ANTHROPIC_BASE_URL"].as_str().unwrap_or("not set"));
    } else {
        check(SETTINGS_FILE, false, "not found. Run 'ccc init' to create");
    }

    println!();
}

fn find_claude() -> Option<String> {
    // Try `where` on Windows, `which` on Unix
    let cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    if let Ok(output) = Command::new(cmd).arg("claude").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().lines().next().unwrap_or("").to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    // Fallback: check common paths
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();

    let candidates = [
        format!("{home}/.local/bin/claude"),
        format!("{home}/.local/bin/claude.exe"),
    ];

    for path in &candidates {
        if Path::new(path).exists() {
            return Some(path.clone());
        }
    }

    None
}
