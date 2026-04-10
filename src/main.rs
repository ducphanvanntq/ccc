mod commands;
mod config;
mod utils;

use clap::{Parser, Subcommand};

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
    /// Copy default .claude config to current directory
    Init,
    /// Show current local config
    Show,
    /// Show global default config
    Global,
    /// Set ANTHROPIC_API_KEY in default config
    Key {
        /// API key value (if omitted, will prompt for input)
        key: Option<String>,
    },
    /// Check for updates and install latest version
    Update,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => commands::version::run(),
        Commands::Init => commands::init::run(),
        Commands::Show => commands::show::run(),
        Commands::Global => commands::global::run(),
        Commands::Key { key } => commands::key::run(key),
        Commands::Update => commands::update::run(),
    }
}
