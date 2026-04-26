mod api;
mod commands;
mod config;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "ccc", about = "Claude Code Config CLI")]
pub struct Cli {
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
    /// Manage API keys
    Key {
        #[command(subcommand)]
        subcmd: Option<commands::key::KeyCmd>,
    },
    /// Manage config values (base_url, model, etc.)
    Config {
        #[command(subcommand)]
        subcmd: commands::config::ConfigCmd,
    },
    /// Check for updates and install latest version
    Update,
    /// Check environment and config status
    Doctor,
    /// Test API connection with current key
    Check,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => commands::version::run(),
        Some(Commands::Init) => commands::init::run()?,
        Some(Commands::Show { target }) => commands::show::run(target.unwrap_or(ShowTarget::Global))?,
        Some(Commands::Key { subcmd }) => commands::key::run(subcmd)?,
        Some(Commands::Config { subcmd }) => commands::config::run(subcmd)?,
        Some(Commands::Update) => commands::update::run()?,
        Some(Commands::Doctor) => commands::doctor::run()?,
        Some(Commands::Check) => commands::check::run()?,
        Some(Commands::Completions { shell }) => commands::completions::run(shell),
        None => Cli::command().print_help()?,
    }

    Ok(())
}

