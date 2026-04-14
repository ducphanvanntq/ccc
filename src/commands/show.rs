use crate::config::{default_settings_path, local_settings_path, read_json};
use crate::utils::print_json_pretty;
use crate::ShowTarget;

pub fn run(target: ShowTarget) {
    match target {
        ShowTarget::Config => show_config(),
        ShowTarget::Global => show_global(),
    }
}

fn show_config() {
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
    print_json_pretty(&json);
}

fn show_global() {
    let path = default_settings_path();

    if !path.exists() {
        eprintln!("Global settings not found at: {}", path.display());
        eprintln!("Please run the install script first.");
        std::process::exit(1);
    }

    let json = read_json(&path);
    println!("Global config: {}", path.display());
    println!("---");
    print_json_pretty(&json);
}
