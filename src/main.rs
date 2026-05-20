mod cli;
mod error;
mod log;

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
            // Placeholder — implemented in Plan 02
            std::process::exit(0);
        }
        Commands::Focus { uri: _ } => {
            // Placeholder — implemented in Phase 4
            eprintln!("Focus subcommand not yet implemented");
            std::process::exit(1);
        }
    }
}
