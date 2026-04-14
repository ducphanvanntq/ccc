mod commands;
mod config;
mod utils;

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccc", about = "Claude Code Config CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
    /// Test API connection with current key
    Check,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => commands::version::run(),
        Some(Commands::Init) => commands::init::run(),
        Some(Commands::Show { target }) => commands::show::run(target.unwrap_or(ShowTarget::Global)),
        Some(Commands::Key { key }) => commands::key::run(key),
        Some(Commands::Update) => commands::update::run(),
        Some(Commands::Doctor) => commands::doctor::run(),
        Some(Commands::Check) => commands::check::run(),
        None => Cli::command().print_help().unwrap(),
    }
}
