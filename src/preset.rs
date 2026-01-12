use crate::config::{Preset, load_config, save_config};
use anyhow::Result;
use std::io::{self, Write};

pub fn handle(action: crate::cli::PresetCommand) -> Result<()> {
    match action {
        crate::cli::PresetCommand::List => {
            let cfg = load_config()?;
            for key in cfg.presets.keys() {
                println!("{}", key);
            }
        }
        crate::cli::PresetCommand::Add { name } => {
            add_preset(name)?;
        }
        crate::cli::PresetCommand::Remove { name } => {
            remove_preset(name)?;
        }
        _ => unimplemented!("Command not yet implemented"),
    }
    Ok(())
}

pub fn save_preset(name: String, preset: Preset) -> Result<()> {
    let mut cfg = load_config()?;
    cfg.presets.insert(name, preset);
    save_config(&cfg)
}

pub fn load_preset(name: &str) -> Result<Preset> {
    let cfg = load_config()?;
    cfg.presets
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Preset '{}' not found", name))
}

fn add_preset(name: String) -> anyhow::Result<()> {
    println!("Enter tables to exclude (comma separated):");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let exclude_tables = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    save_preset(name, Preset { exclude_tables })
}

pub fn remove_preset(name: String) -> anyhow::Result<()> {
    let mut cfg = load_config()?;

    if cfg.presets.remove(&name).is_some() {
        save_config(&cfg)?;
        println!("Preset '{}' removed", name);
    } else {
        anyhow::bail!("Preset '{}' does not exist", name);
    }

    Ok(())
}