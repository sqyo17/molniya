mod cli;
mod restore;
mod preset;
mod config;
mod db;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Commands::Restore {
            db,
            preset,
            backup,
            dry_run,
            yes,
        } => {
            restore::handle(db, preset, backup, dry_run, yes)?;
        }
        Commands::Preset { action } => {
            preset::handle(action)?;
        }
        Commands::Doctor => {
            doctor()?;
        }
    }
    Ok(())
}

fn doctor() -> anyhow::Result<()> {
    println!("🔍 Running molniya doctor…");

    db::test_connection()?;
    println!("✅ MySQL connection OK");

    config::ensure_config_writable()?;
    println!("✅ Config directory writable");

    println!("🎉 Environment looks healthy");
    Ok(())
}