use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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
