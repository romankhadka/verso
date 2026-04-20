use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "verso", version, about = "Terminal EPUB reader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Re-scan the library folder.
    Scan,
    /// Export highlights for a book (path or title) to Markdown.
    Export { target: String },
    /// Permanently remove soft-deleted books and their highlights (asks first).
    PurgeOrphans,
    /// Print the effective config.
    Config,
}
