use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SETTINGS_FILE: &str = "settings.local.json";

fn home_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .expect("Cannot determine home directory");
    PathBuf::from(home)
}

fn ccc_home() -> PathBuf {
    home_dir().join(".ccc")
}

fn default_claude_dir() -> PathBuf {
    ccc_home().join(".claude")
}

fn default_settings_path() -> PathBuf {
    default_claude_dir().join(SETTINGS_FILE)
}

fn local_settings_path() -> PathBuf {
    Path::new(".claude").join(SETTINGS_FILE)
}

fn prompt(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn confirm(message: &str) -> bool {
    let answer = prompt(message);
    matches!(answer.to_lowercase().as_str(), "y" | "yes")
}

fn read_json(path: &Path) -> Value {
    let content = fs::read_to_string(path).expect("Failed to read settings");
    serde_json::from_str(&content).expect("Failed to parse settings")
}

fn write_json(path: &Path, value: &Value) {
    let pretty = serde_json::to_string_pretty(value).expect("Failed to serialize settings");
    fs::write(path, pretty).expect("Failed to write settings");
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
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

fn print_json_flat(value: &Value, prefix: &str) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                print_json_flat(v, &key);
            }
        }
        Value::Array(arr) => {
            let items: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
            println!("{prefix}=[{}]", items.join(", "));
        }
        _ => {
            println!("{prefix}={}", value.to_string().trim_matches('"'));
        }
    }
}

fn mask_key(key: &str) -> String {
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "****".to_string()
    }
}

#[derive(Parser)]
#[command(name = "ccc", about = "Claude Code Config CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    Init,
    Show,
    Key {
        key: Option<String>,
    },
}

fn cmd_version() {
    println!("ccc {VERSION}");
}

fn cmd_init() {
    let source = default_claude_dir();
    if !source.exists() {
        eprintln!("Default .claude folder not found at: {}", source.display());
        eprintln!("Please run the install script first.");
        std::process::exit(1);
    }

    let target = Path::new(".claude");
    let target_settings = target.join(SETTINGS_FILE);

    if target_settings.exists()
        && !confirm("settings.local.json already exists. Overwrite? (y/N): ")
    {
        println!("Cancelled.");
        return;
    }

    copy_dir_recursive(&source, target).expect("Failed to copy .claude folder");
    println!("Copied default .claude config to current directory.");
    println!("Done!");
}

fn cmd_show() {
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

fn cmd_key(key: Option<String>) {
    let api_key = key.unwrap_or_else(|| prompt("Enter ANTHROPIC_API_KEY: "));

    if api_key.is_empty() {
        eprintln!("API key cannot be empty.");
        std::process::exit(1);
    }

    let path = default_settings_path();
    if !path.exists() {
        eprintln!("Default settings not found at: {}", path.display());
        eprintln!("Please run install first.");
        std::process::exit(1);
    }

    let mut json = read_json(&path);
    json["env"]["ANTHROPIC_API_KEY"] = Value::String(api_key.clone());
    write_json(&path, &json);

    println!("Updated ANTHROPIC_API_KEY in {}", path.display());
    println!("Key: {}", mask_key(&api_key));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => cmd_version(),
        Commands::Init => cmd_init(),
        Commands::Show => cmd_show(),
        Commands::Key { key } => cmd_key(key),
    }
}
