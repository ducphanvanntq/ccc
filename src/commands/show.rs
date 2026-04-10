use crate::config::{default_settings_path, local_settings_path, read_json};
use crate::utils::print_json_flat;

pub fn run() {
    let local = local_settings_path();
    let path = if local.exists() {
        local
    } else {
        default_settings_path()
    };

    if !path.exists() {
        eprintln!("No settings found. Run 'ccc init' or 'ccc key' first.");
        std::process::exit(1);
    }

    let json = read_json(&path);
    println!("Config: {}", path.display());
    println!("---");
    print_json_flat(&json, "");
}
