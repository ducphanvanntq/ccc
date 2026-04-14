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
pub enum ShowTarget {
    /// Show local config (fallback to global)
    Config,
    /// Show global default config
    Global,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the current version
    Version,
    /// Copy default .claude config to current directory
    Init,
    /// Show config (default: global)
    Show {
        #[command(subcommand)]
        target: Option<ShowTarget>,
    },
    /// Set ANTHROPIC_API_KEY in default config
    Key {
        /// API key value (if omitted, will prompt for input)
        key: Option<String>,
    },
    /// Check for updates and install latest version
    Update,
    /// Check environment and config status
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => commands::version::run(),
        Commands::Init => commands::init::run(),
        Commands::Show { target } => commands::show::run(target.unwrap_or(ShowTarget::Global)),
        Commands::Key { key } => commands::key::run(key),
        Commands::Update => commands::update::run(),
        Commands::Doctor => commands::doctor::run(),
    }
}
