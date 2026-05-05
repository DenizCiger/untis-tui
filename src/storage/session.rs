use crate::storage::{StorageError, config_dir};
use std::path::PathBuf;
use tui_components::storage::json::named_file;
use tui_components::storage::session;

fn session_file() -> Result<PathBuf, StorageError> {
    Ok(named_file(config_dir()?, "session.json"))
}

pub fn auto_login_enabled() -> bool {
    session_file()
        .map(|p| session::read_session(p).auto_login)
        .unwrap_or(false)
}

pub fn set_auto_login(value: bool) {
    if let Ok(path) = session_file() {
        let _ = session::set_auto_login(path, value);
    }
}
