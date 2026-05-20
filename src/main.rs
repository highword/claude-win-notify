mod assets;
mod cli;
mod error;
mod hook;
mod log;
mod notification;
mod toast;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Hook => {
            use std::io::IsTerminal;
            if std::io::stdin().is_terminal() {
                eprintln!("Error: claude-win-notify hook expects JSON input via stdin.");
                eprintln!("This command is meant to be invoked by Claude Code hooks.");
                std::process::exit(1);
            }
            if let Err(e) = hook::handle_hook() {
                log::log_error(&format!("Hook error: {}", e));
            }
        }
        Commands::Focus { uri: _ } => {
            eprintln!("Focus subcommand not yet implemented");
            std::process::exit(1);
        }
    }
}
