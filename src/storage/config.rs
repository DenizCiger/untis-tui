use crate::models::{Config, SavedConfig};
use crate::storage::{StorageError, config_dir};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

pub fn config_file() -> Result<PathBuf, StorageError> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_config() -> Option<SavedConfig> {
    let path = config_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    Some(SavedConfig {
        school: parsed.get("school")?.as_str()?.to_owned(),
        username: parsed.get("username")?.as_str()?.to_owned(),
        server: parsed.get("server")?.as_str()?.to_owned(),
    })
}

pub fn save_config(config: &Config) -> Result<(), StorageError> {
    save_saved_config(&config.saved())
}

pub fn save_saved_config(config: &SavedConfig) -> Result<(), StorageError> {
    fs::create_dir_all(config_dir()?)?;
    let payload = json!({
        "school": config.school,
        "username": config.username,
        "server": config.server,
    });
    fs::write(config_file()?, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

pub fn clear_config() -> Result<(), StorageError> {
    if let Ok(path) = config_file() {
        fs::write(path, b"{}")?;
    }
    Ok(())
}
