use dialoguer::{Select, Input, Confirm};
use dialoguer::theme::ColorfulTheme;

use crate::config::{default_settings_path, read_json, write_json, KeysStore};
use crate::utils::mask_key;

pub fn run(subcmd: Option<KeyCmd>) {
    match subcmd {
        Some(KeyCmd::Add { name, value }) => cmd_add(Some(name), Some(value)),
        Some(KeyCmd::List) => cmd_list(),
        Some(KeyCmd::Use { name }) => cmd_use(Some(name)),
        Some(KeyCmd::Remove { name }) => cmd_remove(Some(name)),
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
    /// Switch to a saved key
    Use {
        /// Key name
        name: String,
    },
    /// Remove a saved key
    Remove {
        /// Key name
        name: String,
    },
}

fn cmd_menu() {
    let theme = ColorfulTheme::default();
    let items = vec!["Add new key", "List keys", "Switch active key", "Remove key", "Exit"];

    let selection = Select::with_theme(&theme)
        .with_prompt("Key manager")
        .items(&items)
        .default(0)
        .interact();

    match selection {
        Ok(0) => cmd_add(None, None),
        Ok(1) => cmd_list(),
        Ok(2) => cmd_use(None),
        Ok(3) => cmd_remove(None),
        _ => {}
    }
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

    // Set as active if first key or confirm
    if store.active.is_none() {
        store.active = Some(name.clone());
    } else if store.active.as_deref() != Some(&name) {
        let set_active = Confirm::with_theme(&theme)
            .with_prompt(format!("Set '{name}' as active key?"))
            .default(false)
            .interact()
            .unwrap_or(false);
        if set_active {
            store.active = Some(name.clone());
        }
    }

    store.save();
    if store.active.as_deref() == Some(&name) {
        store.apply_active();
        println!("Added '{name}' and set as active.");
    } else {
        println!("Added '{name}'.");
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
        let active = store.active.as_deref() == Some(name);
        let marker = if active { "*" } else { " " };
        let masked = mask_key(value);
        let tag = if active { " (active)" } else { "" };
        println!("  {marker} {name:<20} {masked}{tag}");
    }
}

fn cmd_use(name: Option<String>) {
    let store = KeysStore::load();

    if store.keys.is_empty() {
        eprintln!("No keys saved. Run 'ccc key add' first.");
        std::process::exit(1);
    }

    let name = name.unwrap_or_else(|| {
        let theme = ColorfulTheme::default();
        let names: Vec<&str> = store.keys.keys().map(|s| s.as_str()).collect();
        let selection = Select::with_theme(&theme)
            .with_prompt("Select key to use")
            .items(&names)
            .default(0)
            .interact()
            .unwrap();
        names[selection].to_string()
    });

    if !store.keys.contains_key(&name) {
        eprintln!("Key '{name}' not found.");
        std::process::exit(1);
    }

    let mut store = store;
    store.active = Some(name.clone());
    store.save();
    store.apply_active();
    println!("Switched to '{name}'.");
}

fn cmd_remove(name: Option<String>) {
    let mut store = KeysStore::load();

    if store.keys.is_empty() {
        eprintln!("No keys saved.");
        std::process::exit(1);
    }

    let name = name.unwrap_or_else(|| {
        let theme = ColorfulTheme::default();
        let names: Vec<&str> = store.keys.keys().map(|s| s.as_str()).collect();
        let selection = Select::with_theme(&theme)
            .with_prompt("Select key to remove")
            .items(&names)
            .default(0)
            .interact()
            .unwrap();
        names[selection].to_string()
    });

    if !store.keys.contains_key(&name) {
        eprintln!("Key '{name}' not found.");
        std::process::exit(1);
    }

    store.keys.remove(&name);
    if store.active.as_deref() == Some(&name) {
        store.active = store.keys.keys().next().cloned();
        store.apply_active();
        if let Some(new_active) = &store.active {
            println!("Removed '{name}', switched to '{new_active}'.");
        } else {
            println!("Removed '{name}'. No active key.");
            // Clear key from settings
            let settings_path = default_settings_path();
            if settings_path.exists() {
                let mut json = read_json(&settings_path);
                json["env"]["ANTHROPIC_API_KEY"] = serde_json::Value::String(String::new());
                write_json(&settings_path, &json);
            }
        }
    } else {
        println!("Removed '{name}'.");
    }

    store.save();
}
