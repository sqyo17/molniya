use serde::{Serialize, Deserialize};
use std::{collections::HashMap, path::PathBuf, fs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preset {
    pub exclude_tables: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RestoreConfig {
    pub presets: HashMap<String, Preset>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("molniya")
        .join("config.json")
}

pub fn load_config() -> anyhow::Result<RestoreConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(RestoreConfig::default());
    }
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn save_config(cfg: &RestoreConfig) -> anyhow::Result<()> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn ensure_config_writable() -> anyhow::Result<()> {
    let path = config_path();

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    Ok(())
}