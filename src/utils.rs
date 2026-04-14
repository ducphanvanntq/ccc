use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn prompt(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().unwrap();
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
