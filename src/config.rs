use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SETTINGS_FILE: &str = "settings.local.json";
pub const REPO: &str = "ducphanvanntq/ccc";

pub fn ccc_home() -> Result<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("Cannot determine home directory (USERPROFILE or HOME not set)")?;
    Ok(Path::new(&home).join(".ccc"))
}

pub fn default_claude_dir() -> Result<PathBuf> {
    Ok(ccc_home()?.join(".claude"))
}

pub fn default_settings_path() -> Result<PathBuf> {
    Ok(default_claude_dir()?.join(SETTINGS_FILE))
}

pub fn local_settings_path() -> PathBuf {
    Path::new(".claude").join(SETTINGS_FILE)
}

pub fn keys_path() -> Result<PathBuf> {
    Ok(ccc_home()?.join("keys.json"))
}

pub fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in {}", path.display()))
}

/// Read JSON, returning empty object on any error (for non-critical reads)
pub fn read_json_or_default(path: &Path) -> Value {
    read_json(path).unwrap_or_else(|_| Value::Object(serde_json::Map::new()))
}

pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    let pretty = serde_json::to_string_pretty(value)
        .context("Failed to serialize JSON")?;
    fs::write(path, pretty)
        .with_context(|| format!("Failed to write {}", path.display()))
}

// Keys store: { active: "name", keys: { "name": "sk-..." } }
#[derive(Serialize, Deserialize, Default)]
pub struct KeysStore {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
}

impl KeysStore {
    pub fn load() -> Self {
        let path = match keys_path() {
            Ok(p) => p,
            Err(_) => return KeysStore::default(),
        };
        if !path.exists() {
            return KeysStore::default();
        }
        match read_json(&path) {
            Ok(json) => serde_json::from_value(json).unwrap_or_default(),
            Err(_) => KeysStore::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = keys_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let value = serde_json::to_value(self)
            .context("Failed to serialize keys")?;
        write_json(&path, &value)
    }

    pub fn get_active_key(&self) -> Option<&String> {
        self.active.as_ref().and_then(|a| self.keys.get(a))
    }
}
