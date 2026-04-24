use std::sync::mpsc;
use std::thread;

use crate::api::{check_api_key, get_api_config, validate_key_format};
use crate::config::{local_settings_path, read_json_or_default, write_json, KeysStore};
use crate::utils::mask_key;

use super::state::{App, Mode, StatusEntry, StatusState};
use super::theme::ICON_CHECK;

impl App {
    pub fn load() -> Self {
        let store = KeysStore::load();
        let entries = Self::build_entries(&store);
        App { entries, selected: 0, mode: Mode::Normal, should_quit: false }
    }

    pub fn build_entries(store: &KeysStore) -> Vec<(String, String, bool)> {
        store.keys.iter()
            .map(|(name, value)| {
                let is_default = store.active.as_deref() == Some(name.as_str());
                (name.clone(), mask_key(value), is_default)
            })
            .collect()
    }

    pub fn reload(&mut self) {
        let store = KeysStore::load();
        self.entries = Self::build_entries(&store);
        if self.entries.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
    }

    pub fn selected_name(&self) -> Option<String> {
        self.entries.get(self.selected).map(|(n, _, _)| n.clone())
    }

    pub fn msg(&mut self, text: String, success: bool) {
        self.mode = Mode::Message { text, success };
    }

    // ── Operations ──

    pub fn do_add(&mut self, name: &str, value: &str) {
        if name.is_empty() {
            self.msg("Name cannot be empty.".into(), false);
            return;
        }
        if let Err(e) = validate_key_format(value) {
            self.msg(e, false);
            return;
        }
        let mut store = KeysStore::load();
        let is_first = store.active.is_none();
        store.keys.insert(name.to_string(), value.to_string());
        if is_first {
            store.active = Some(name.to_string());
        }
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        if is_first {
            self.msg(format!("{} Added '{name}' and set as default.", ICON_CHECK), true);
        } else {
            self.msg(format!("{} Added '{name}'.", ICON_CHECK), true);
        }
        self.reload();
    }

    pub fn do_default(&mut self, name: &str) {
        let mut store = KeysStore::load();
        if !store.keys.contains_key(name) {
            self.msg(format!("Key '{name}' not found."), false);
            return;
        }
        store.active = Some(name.to_string());
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        self.msg(format!("{} Set '{name}' as default.", ICON_CHECK), true);
        self.reload();
    }

    pub fn do_use(&mut self, name: &str) {
        let store = KeysStore::load();
        let key_value = match store.keys.get(name) {
            Some(v) => v.clone(),
            None => { self.msg(format!("Key '{name}' not found."), false); return; }
        };
        let local_path = local_settings_path();
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut json = if local_path.exists() {
            read_json_or_default(&local_path)
        } else {
            serde_json::json!({})
        };
        json["env"]["ANTHROPIC_API_KEY"] = serde_json::Value::String(key_value);
        if let Err(e) = write_json(&local_path, &json) {
            self.msg(format!("Failed to write config: {e}"), false);
            return;
        }
        let folder = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".into());
        self.msg(format!("{} Using '{name}' for {folder}", ICON_CHECK), true);
    }

    pub fn do_remove(&mut self, name: &str) {
        let mut store = KeysStore::load();
        store.keys.remove(name);
        if store.active.as_deref() == Some(name) {
            store.active = store.keys.keys().next().cloned();
        }
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        self.msg(format!("{} Removed '{name}'.", ICON_CHECK), true);
        self.reload();
    }

    pub fn do_rename(&mut self, old_name: &str, new_name: &str) {
        if new_name.is_empty() {
            self.msg("Name cannot be empty.".into(), false);
            return;
        }
        if new_name == old_name {
            self.mode = Mode::Normal;
            return;
        }
        let mut store = KeysStore::load();
        if store.keys.contains_key(new_name) {
            self.msg(format!("Key '{new_name}' already exists."), false);
            return;
        }
        if let Some(value) = store.keys.remove(old_name) {
            store.keys.insert(new_name.to_string(), value);
            if store.active.as_deref() == Some(old_name) {
                store.active = Some(new_name.to_string());
            }
            if let Err(e) = store.save() {
                self.msg(format!("Failed to save: {e}"), false);
                return;
            }
            self.msg(format!("{} Renamed '{old_name}' → '{new_name}'.", ICON_CHECK), true);
            self.reload();
        }
    }

    pub fn do_status(&mut self) {
        let store = KeysStore::load();
        if store.keys.is_empty() {
            self.msg("No keys to check.".into(), false);
            return;
        }

        let (api_url, model) = get_api_config();
        let total = store.keys.len();

        let results: Vec<StatusEntry> = store.keys.iter()
            .map(|(name, value)| StatusEntry {
                name: name.clone(),
                masked: mask_key(value),
                is_default: store.active.as_deref() == Some(name.as_str()),
                result: None,
            })
            .collect();

        // Spawn threads for parallel key checking
        let (tx, rx) = mpsc::channel();
        for (i, (_, value)) in store.keys.iter().enumerate() {
            let tx = tx.clone();
            let value = value.clone();
            thread::spawn(move || {
                let (ok, msg) = check_api_key(&value);
                let _ = tx.send((i, ok, if ok { "OK".into() } else { msg }));
            });
        }
        drop(tx);

        self.mode = Mode::Status(StatusState {
            api_url,
            model,
            results,
            total,
            checked: 0,
            rx: Some(rx),
        });
    }

    /// Poll for completed status checks (non-blocking)
    pub fn poll_status(&mut self) {
        if let Mode::Status(state) = &mut self.mode {
            if let Some(rx) = &state.rx {
                while let Ok((idx, ok, msg)) = rx.try_recv() {
                    if idx < state.results.len() {
                        state.results[idx].result = Some((ok, msg));
                        state.checked += 1;
                    }
                }
                if state.checked >= state.total {
                    state.rx = None;
                }
            }
        }
    }
}
