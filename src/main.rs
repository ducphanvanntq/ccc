use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_ccc_home() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .expect("Cannot determine home directory");
    Path::new(&home).join(".ccc")
}

fn get_default_claude_dir() -> PathBuf {
    get_ccc_home().join(".claude")
}

fn get_settings_path() -> PathBuf {
    get_default_claude_dir().join("settings.local.json")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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

#[derive(Parser)]
#[command(name = "ccc", about = "Claude Code Config CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the current version
    Version,
    /// Copy default .claude folder to current directory
    Init,
    /// Set ANTHROPIC_API_KEY in default config
    Key {
        /// API key value (if omitted, will prompt for input)
        key: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("ccc {VERSION}");
        }
        Commands::Init => {
            let default_dir = get_default_claude_dir();

            if !default_dir.exists() {
                eprintln!("Default .claude folder not found at: {}", default_dir.display());
                eprintln!("Please run install.bat first to set up the default config.");
                std::process::exit(1);
            }

            let target_dir = Path::new(".claude");

            if target_dir.exists() {
                println!(".claude folder already exists, overwriting...");
            }

            copy_dir_recursive(&default_dir, target_dir)
                .expect("Failed to copy .claude folder");

            println!("Copied default .claude config to current directory.");
            println!("Done!");
        }
        Commands::Key { key } => {
            let api_key = match key {
                Some(k) => k,
                None => {
                    print!("Enter ANTHROPIC_API_KEY: ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Failed to read input");
                    input.trim().to_string()
                }
            };

            if api_key.is_empty() {
                eprintln!("API key cannot be empty.");
                std::process::exit(1);
            }

            let settings_path = get_settings_path();
            if !settings_path.exists() {
                eprintln!("Default settings not found at: {}", settings_path.display());
                eprintln!("Please run install first.");
                std::process::exit(1);
            }

            let content = fs::read_to_string(&settings_path).expect("Failed to read settings");
            let mut json: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse settings");

            json["env"]["ANTHROPIC_API_KEY"] = serde_json::Value::String(api_key.clone());

            let updated = serde_json::to_string_pretty(&json).expect("Failed to serialize settings");
            fs::write(&settings_path, updated).expect("Failed to write settings");

            println!("Updated ANTHROPIC_API_KEY in {}", settings_path.display());

            // Mask key for display
            let masked = if api_key.len() > 8 {
                format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
            } else {
                "****".to_string()
            };
            println!("Key: {masked}");
        }
    }
}
