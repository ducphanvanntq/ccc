use dialoguer::{Select, Input, Confirm};
use dialoguer::theme::ColorfulTheme;

use crate::config::{local_settings_path, read_json, write_json, KeysStore};
use crate::utils::mask_key;
use crate::commands::check;

pub fn run(subcmd: Option<KeyCmd>) {
    match subcmd {
        Some(KeyCmd::Add { name, value }) => cmd_add(Some(name), Some(value)),
        Some(KeyCmd::List) => cmd_list(),
        Some(KeyCmd::Default { name }) => cmd_default(name),
        Some(KeyCmd::Use { name }) => cmd_use(name),
        Some(KeyCmd::Remove { name }) => cmd_remove(name),
        Some(KeyCmd::Check) => check::run(),
        None => cmd_menu(),
    }
}

#[derive(clap::Subcommand)]
pub enum KeyCmd {
    /// Add or update a named key
    Add {
        /// Key name
        name: String,
        /// API key value
        value: String,
    },
    /// List all saved keys
    List,
    /// Set default key (saved in keys.json)
    Default {
        /// Key name (optional, shows list if omitted)
        name: Option<String>,
    },
    /// Use a key for current folder (.claude/settings.local.json)
    Use {
        /// Key name (optional, shows list if omitted)
        name: Option<String>,
    },
    /// Remove a saved key
    Remove {
        /// Key name (optional, shows list if omitted)
        name: Option<String>,
    },
    /// Check if active key is valid
    Check,
}

fn cmd_menu() {
    let theme = ColorfulTheme::default();
    let items = vec![
        "Add new key",
        "List keys",
        "Set default key",
        "Use key (current folder)",
        "Remove key",
        "Check key",
        "Exit",
    ];

    let selection = Select::with_theme(&theme)
        .with_prompt("Key manager")
        .items(&items)
        .default(0)
        .interact();

    match selection {
        Ok(0) => cmd_add(None, None),
        Ok(1) => cmd_list(),
        Ok(2) => cmd_default(None),
        Ok(3) => cmd_use(None),
        Ok(4) => cmd_remove(None),
        Ok(5) => check::run(),
        _ => {}
    }
}

fn select_key(store: &KeysStore, prompt: &str) -> String {
    let theme = ColorfulTheme::default();
    let names: Vec<&str> = store.keys.keys().map(|s| s.as_str()).collect();
    let default_idx = store.active.as_ref()
        .and_then(|a| names.iter().position(|n| n == a))
        .unwrap_or(0);
    let selection = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(&names)
        .default(default_idx)
        .interact()
        .unwrap();
    names[selection].to_string()
}

fn cmd_add(name: Option<String>, value: Option<String>) {
    let theme = ColorfulTheme::default();

    let name = name.unwrap_or_else(|| {
        Input::with_theme(&theme)
            .with_prompt("Key name (e.g. work, personal)")
            .interact_text()
            .unwrap()
    });

    let value = value.unwrap_or_else(|| {
        Input::with_theme(&theme)
            .with_prompt("API key value")
            .interact_text()
            .unwrap()
    });

    if name.is_empty() || value.is_empty() {
        eprintln!("Name and value cannot be empty.");
        std::process::exit(1);
    }

    let mut store = KeysStore::load();
    store.keys.insert(name.clone(), value.clone());

    // Set as default if first key
    if store.active.is_none() {
        store.active = Some(name.clone());
        store.save();
        println!("Added '{name}' and set as default.");
    } else if store.active.as_deref() != Some(&name) {
        let set_default = Confirm::with_theme(&theme)
            .with_prompt(format!("Set '{name}' as default key?"))
            .default(false)
            .interact()
            .unwrap_or(false);
        if set_default {
            store.active = Some(name.clone());
        }
        store.save();
        if store.active.as_deref() == Some(&name) {
            println!("Added '{name}' and set as default.");
        } else {
            println!("Added '{name}'.");
        }
    } else {
        store.save();
        println!("Updated '{name}'.");
    }
}

fn cmd_list() {
    let store = KeysStore::load();

    if store.keys.is_empty() {
        println!("No keys saved. Run 'ccc key add' to add one.");
        return;
    }

    println!("Saved keys:");
    for (name, value) in &store.keys {
        let is_default = store.active.as_deref() == Some(name);
        let marker = if is_default { "*" } else { " " };
        let masked = mask_key(value);
        let tag = if is_default { " (default)" } else { "" };
        println!("  {marker} {name:<20} {masked}{tag}");
    }
}

/// Set default key in keys.json only
fn cmd_default(name: Option<String>) {
    let mut store = KeysStore::load();

    if store.keys.is_empty() {
        eprintln!("No keys saved. Run 'ccc key add' first.");
        std::process::exit(1);
    }

    let name = name.unwrap_or_else(|| select_key(&store, "Select default key"));

    if !store.keys.contains_key(&name) {
        eprintln!("Key '{name}' not found.");
        std::process::exit(1);
    }

    store.active = Some(name.clone());
    store.save();
    println!("Set '{name}' as default.");
}

/// Use key for current folder -> .claude/settings.local.json
fn cmd_use(name: Option<String>) {
    let store = KeysStore::load();

    if store.keys.is_empty() {
        eprintln!("No keys saved. Run 'ccc key add' first.");
        std::process::exit(1);
    }

    let name = name.unwrap_or_else(|| select_key(&store, "Select key for current folder"));

    if !store.keys.contains_key(&name) {
        eprintln!("Key '{name}' not found.");
        std::process::exit(1);
    }

    let key_value = store.keys.get(&name).unwrap();
    let local_path = local_settings_path();

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let mut json = if local_path.exists() {
        read_json(&local_path)
    } else {
        serde_json::json!({})
    };

    json["env"]["ANTHROPIC_API_KEY"] = serde_json::Value::String(key_value.clone());
    write_json(&local_path, &json);

    let folder = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string());
    println!("Using '{name}' for folder: {folder}");
}

fn cmd_remove(name: Option<String>) {
    let mut store = KeysStore::load();

    if store.keys.is_empty() {
        eprintln!("No keys saved.");
        std::process::exit(1);
    }

    let name = name.unwrap_or_else(|| select_key(&store, "Select key to remove"));

    if !store.keys.contains_key(&name) {
        eprintln!("Key '{name}' not found.");
        std::process::exit(1);
    }

    store.keys.remove(&name);
    if store.active.as_deref() == Some(&name) {
        store.active = store.keys.keys().next().cloned();
        if let Some(new_default) = &store.active {
            println!("Removed '{name}', default switched to '{new_default}'.");
        } else {
            println!("Removed '{name}'. No default key.");
        }
    } else {
        println!("Removed '{name}'.");
    }

    store.save();
}
