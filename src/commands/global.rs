use crate::config::{default_settings_path, read_json};
use crate::utils::print_json_flat;

pub fn run() {
    let path = default_settings_path();

    if !path.exists() {
        eprintln!("Global settings not found at: {}", path.display());
        eprintln!("Please run the install script first.");
        std::process::exit(1);
    }

    let json = read_json(&path);
    println!("Global config: {}", path.display());
    println!("---");
    print_json_flat(&json, "");
}
