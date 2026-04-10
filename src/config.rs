use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SETTINGS_FILE: &str = "settings.local.json";
pub const REPO: &str = "ducphanvanntq/ccc";

pub fn ccc_home() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .expect("Cannot determine home directory");
    Path::new(&home).join(".ccc")
}

pub fn default_claude_dir() -> PathBuf {
    ccc_home().join(".claude")
}

pub fn default_settings_path() -> PathBuf {
    default_claude_dir().join(SETTINGS_FILE)
}

pub fn local_settings_path() -> PathBuf {
    Path::new(".claude").join(SETTINGS_FILE)
}

pub fn read_json(path: &Path) -> Value {
    let content = fs::read_to_string(path).expect("Failed to read settings");
    serde_json::from_str(&content).expect("Failed to parse settings")
}

pub fn write_json(path: &Path, value: &Value) {
    let pretty = serde_json::to_string_pretty(value).expect("Failed to serialize settings");
    fs::write(path, pretty).expect("Failed to write settings");
}
