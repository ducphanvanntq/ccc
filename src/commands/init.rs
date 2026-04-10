use std::path::Path;

use crate::config::{default_claude_dir, SETTINGS_FILE};
use crate::utils::{confirm, copy_dir_recursive};

pub fn run() {
    let source = default_claude_dir();
    if !source.exists() {
        eprintln!("Default .claude folder not found at: {}", source.display());
        eprintln!("Please run the install script first.");
        std::process::exit(1);
    }

    let target = Path::new(".claude");
    let target_settings = target.join(SETTINGS_FILE);

    if target_settings.exists()
        && !confirm("settings.local.json already exists. Overwrite? (y/N): ")
    {
        println!("Cancelled.");
        return;
    }

    copy_dir_recursive(&source, target).expect("Failed to copy .claude folder");
    println!("Copied default .claude config to current directory.");
    println!("Done!");
}
