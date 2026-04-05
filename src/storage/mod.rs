pub mod cache;
pub mod config;
pub mod secret;

use directories::BaseDirs;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn config_dir() -> Result<PathBuf, StorageError> {
    let base_dirs = BaseDirs::new()
        .ok_or_else(|| StorageError::Message("Failed to determine home directory".to_owned()))?;
    Ok(base_dirs.home_dir().join(".config").join("tui-untis"))
}
