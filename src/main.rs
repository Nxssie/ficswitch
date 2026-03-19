mod commands;
mod config;
mod core;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ficswitch",
    about = "CLI tool for switching Satisfactory between stable/experimental Steam branches",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current status (installation, branch, saves, profiles)
    Status,

    /// Switch to a different branch (stable or experimental)
    Switch {
        /// Target branch: stable or experimental
        branch: String,

        /// Skip automatic backup before switching
        #[arg(long)]
        no_backup: bool,
    },

    /// Manage save backups
    Backup {
        #[command(subcommand)]
        action: BackupAction,
    },

    /// Manage SMM profile-branch associations
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Manage local branch game file cache (avoids Steam re-downloads)
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand)]
enum BackupAction {
    /// Create a new backup
    Create {
        /// Optional label for the backup
        #[arg(long)]
        label: Option<String>,
    },

    /// List all backups
    List,

    /// Restore a backup by ID
    Restore {
        /// Backup ID (timestamp format: YYYYMMDD_HHMMSS)
        id: String,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List available SMM profiles
    List,

    /// Link a SMM profile to a branch
    Link {
        /// Profile name (must exist in SMM)
        name: String,

        /// Branch to link to: stable or experimental
        branch: String,
    },

    /// Show current profile-branch mappings
    Show,
}

#[derive(Subcommand)]
enum CacheAction {
    /// Cache the current branch's game files using hardlinks
    Create,

    /// Show cached branches
    Status,

    /// Clear the cache for a branch
    Clear {
        /// Branch to clear: stable or experimental
        branch: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => commands::status::run(),
        Commands::Switch { branch, no_backup } => commands::switch::run(&branch, no_backup),
        Commands::Backup { action } => match action {
            BackupAction::Create { label } => {
                commands::backup::create(label.as_deref())
            }
            BackupAction::List => commands::backup::list(),
            BackupAction::Restore { id } => commands::backup::restore(&id),
        },
        Commands::Profile { action } => match action {
            ProfileAction::List => commands::profile::list(),
            ProfileAction::Link { name, branch } => commands::profile::link(&name, &branch),
            ProfileAction::Show => commands::profile::show(),
        },
        Commands::Cache { action } => match action {
            CacheAction::Create => commands::cache::create(),
            CacheAction::Status => commands::cache::status(),
            CacheAction::Clear { branch } => commands::cache::clear(&branch),
        },
    }
}
