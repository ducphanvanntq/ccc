use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::{ccc_home, default_settings_path, local_settings_path, read_json_or_default, SETTINGS_FILE, VERSION};
use crate::ui;

pub fn run() -> Result<()> {
    println!();
    ui::print_header(&ui::ICON_DOC, &format!("ccc doctor v{VERSION}"));

    let mut pass = 0;
    let mut fail = 0;

    // 1. Claude Code installed
    let claude_found = find_claude();
    match &claude_found {
        Some(path) => { ui::print_check(true, "Claude Code", path); pass += 1; }
        None => { ui::print_check(false, "Claude Code", "not found"); fail += 1; }
    }

    // 2. ccc home
    let home = ccc_home()?;
    if home.exists() {
        ui::print_check(true, "ccc home", &home.display().to_string());
        pass += 1;
    } else {
        ui::print_check(false, "ccc home", "not found");
        fail += 1;
    }

    ui::print_separator();

    // 3. Global config
    let global_path = default_settings_path()?;
    if global_path.exists() {
        ui::print_check(true, "Global config", &global_path.display().to_string());
        pass += 1;

        let json = read_json_or_default(&global_path);
        let has_key = json["env"]["ANTHROPIC_API_KEY"]
            .as_str()
            .is_some_and(|k| !k.is_empty());
        if has_key {
            ui::print_check(true, "API key", "set (global)");
            pass += 1;
        } else {
            ui::print_check(false, "API key", "not set. Run 'ccc key'");
            fail += 1;
        }
    } else {
        ui::print_check(false, "Global config", "not found. Run install script");
        fail += 1;
    }

    ui::print_separator();

    // 4. Local .claude folder
    let local_dir = Path::new(".claude");
    if local_dir.exists() {
        ui::print_check(true, ".claude/", "found");
        pass += 1;
    } else {
        ui::print_check(false, ".claude/", "not found. Run 'ccc init'");
        fail += 1;
    }

    // 5. Local settings
    let local_path = local_settings_path();
    if local_path.exists() {
        ui::print_check(true, SETTINGS_FILE, &local_path.display().to_string());
        pass += 1;

        let json = read_json_or_default(&local_path);

        let has_key = json["env"]["ANTHROPIC_API_KEY"]
            .as_str()
            .is_some_and(|k| !k.is_empty());
        if has_key {
            ui::print_check(true, "API key", "set (local)");
            pass += 1;
        } else {
            ui::print_check(false, "API key", "not set (local)");
            fail += 1;
        }

        let base_url = json["env"]["ANTHROPIC_BASE_URL"]
            .as_str()
            .filter(|u| !u.is_empty());
        match base_url {
            Some(url) => { ui::print_check(true, "Base URL", url); pass += 1; }
            None => { ui::print_check(false, "Base URL", "not set"); fail += 1; }
        }
    } else {
        ui::print_check(false, SETTINGS_FILE, "not found. Run 'ccc init'");
        fail += 1;
    }

    // Result
    ui::print_separator();
    ui::print_result_line(pass, fail);
    ui::print_footer();
    println!();

    Ok(())
}

fn find_claude() -> Option<String> {
    let cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    if let Ok(output) = Command::new(cmd).arg("claude").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().lines().next().unwrap_or("").to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

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
