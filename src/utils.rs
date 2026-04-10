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

pub fn print_json_flat(value: &serde_json::Value, prefix: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                print_json_flat(v, &key);
            }
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<&str> = arr.iter().filter_map(serde_json::Value::as_str).collect();
            println!("{prefix}=[{}]", items.join(", "));
        }
        _ => {
            println!("{prefix}={}", value.to_string().trim_matches('"'));
        }
    }
}
