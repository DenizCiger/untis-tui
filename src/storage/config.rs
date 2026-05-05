use crate::models::{Config, SavedConfig};
use crate::storage::{StorageError, config_dir};
use serde_json::{Value, json};
use std::path::PathBuf;
use tui_components::storage::json::{clear_json_object, named_file, read_json, write_json_pretty};

pub fn config_file() -> Result<PathBuf, StorageError> {
    Ok(named_file(config_dir()?, "config.json"))
}

pub fn load_config() -> Option<SavedConfig> {
    let parsed: Value = read_json(config_file().ok()?)?;
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
    let payload = json!({
        "school": config.school,
        "username": config.username,
        "server": config.server,
    });
    write_json_pretty(config_file()?, &payload)
}

pub fn clear_config() -> Result<(), StorageError> {
    if let Ok(path) = config_file() {
        clear_json_object(path)?;
    }
    Ok(())
}
