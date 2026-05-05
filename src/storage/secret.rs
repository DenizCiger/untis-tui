use crate::models::SavedConfig;
use crate::storage::{StorageError, config_dir};
pub use tui_components::storage::secret::SecretStorageDiagnostic;
use tui_components::storage::secret::SecretStore;

const SECRET_SERVICE: &str = "tui-untis";

fn account_key(config: &SavedConfig) -> String {
    format!("{}|{}|{}", config.server, config.school, config.username)
}

fn store() -> Result<SecretStore, StorageError> {
    Ok(SecretStore::new(
        SECRET_SERVICE,
        "tui-untis",
        "TUI_UNTIS",
        config_dir()?,
    ))
}

pub fn get_secure_storage_diagnostic() -> SecretStorageDiagnostic {
    tui_components::storage::secret::get_secure_storage_diagnostic()
}

pub fn save_password(config: &SavedConfig, password: &str) -> Result<(), StorageError> {
    store()?.save(&account_key(config), password)
}

pub fn load_password(config: &SavedConfig) -> Result<Option<String>, StorageError> {
    store()?.load(&account_key(config))
}

pub fn clear_password(config: &SavedConfig) -> Result<(), StorageError> {
    store()?.clear(&account_key(config))
}
