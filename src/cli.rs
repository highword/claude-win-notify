use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-win-notify", version, about = "Windows notifications for Claude Code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read Claude Code hook stdin and show Toast notification
    Hook,
    /// Handle protocol activation (claude-notify:// URI)
    Focus {
        /// The protocol URI (claude-notify://...)
        uri: String,
    },
}
