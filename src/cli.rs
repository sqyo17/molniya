use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Restore {
        backup: PathBuf,
        #[arg(long)]
        db: String,
        #[arg(long)]
        preset: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Preset {
        #[command(subcommand)]
        action: PresetCommand,
    },
    Doctor,
}

#[derive(Subcommand)]
pub enum PresetCommand {
    Add { name: String },
    Edit { name: String },
    List,
    Remove { name: String },
}
