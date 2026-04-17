use serde_json::Value;
use std::collections::BTreeMap;
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

pub fn keys_path() -> PathBuf {
    ccc_home().join("keys.json")
}

pub fn read_json(path: &Path) -> Value {
    let content = fs::read_to_string(path).expect("Failed to read settings");
    serde_json::from_str(&content).expect("Failed to parse settings")
}

pub fn write_json(path: &Path, value: &Value) {
    let pretty = serde_json::to_string_pretty(value).expect("Failed to serialize settings");
    fs::write(path, pretty).expect("Failed to write settings");
}

// Keys store: { active: "name", keys: { "name": "sk-..." } }
pub struct KeysStore {
    pub active: Option<String>,
    pub keys: BTreeMap<String, String>,
}

impl KeysStore {
    pub fn load() -> Self {
        let path = keys_path();
        if !path.exists() {
            return KeysStore { active: None, keys: BTreeMap::new() };
        }
        let json = read_json(&path);
        let active = json["active"].as_str().map(String::from);
        let keys = json["keys"]
            .as_object()
            .map(|map| {
                map.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        KeysStore { active, keys }
    }

    pub fn save(&self) {
        let path = keys_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut map = serde_json::Map::new();
        if let Some(active) = &self.active {
            map.insert("active".into(), Value::String(active.clone()));
        }
        let keys_obj: serde_json::Map<String, Value> = self.keys
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        map.insert("keys".into(), Value::Object(keys_obj));
        write_json(&path, &Value::Object(map));
    }

    pub fn get_active_key(&self) -> Option<&String> {
        self.active.as_ref().and_then(|a| self.keys.get(a))
    }
}
