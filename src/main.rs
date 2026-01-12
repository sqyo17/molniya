mod cli;
mod preset;
mod config;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Restore { backup, db, preset } => {}
        Commands::Preset { action } => {
            preset::handle(action)?;
        }
    }

    Ok(())
}
