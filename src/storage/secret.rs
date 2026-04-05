use crate::models::SavedConfig;
use crate::storage::{StorageError, config_dir};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const SECRET_SERVICE: &str = "tui-untis";

#[derive(Debug, Clone)]
pub struct SecretStorageDiagnostic {
    pub available: bool,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct SecretFileData {
    #[serde(default)]
    entries: HashMap<String, String>,
}

fn account_key(config: &SavedConfig) -> String {
    format!("{}|{}|{}", config.server, config.school, config.username)
}

fn secret_file() -> Result<PathBuf, StorageError> {
    Ok(config_dir()?.join("secrets.json"))
}

fn command_exists(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    Command::new(checker)
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn windows_shell_command() -> Result<String, StorageError> {
    for candidate in ["powershell.exe", "powershell", "pwsh.exe", "pwsh"] {
        if command_exists(candidate) {
            return Ok(candidate.to_owned());
        }
    }
    Err(StorageError::Message(
        "No PowerShell executable found".to_owned(),
    ))
}

fn run_command(command: &str, args: &[&str], input: Option<&str>) -> Result<String, StorageError> {
    let mut process = Command::new(command);
    process.args(args);
    if input.is_some() {
        process.stdin(Stdio::piped());
    }
    process.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = process.spawn()?;
    if let Some(input) = input {
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input.as_bytes())?;
        }
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(StorageError::Message(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn run_powershell(script: &str, envs: &[(&str, &str)]) -> Result<String, StorageError> {
    let shell = windows_shell_command()?;
    let mut command = Command::new(shell);
    command
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(StorageError::Message(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn read_windows_secret_file() -> SecretFileData {
    let path = match secret_file() {
        Ok(path) => path,
        Err(_) => return SecretFileData::default(),
    };
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => SecretFileData::default(),
    }
}

fn write_windows_secret_file(data: &SecretFileData) -> Result<(), StorageError> {
    fs::create_dir_all(config_dir()?)?;
    fs::write(secret_file()?, serde_json::to_vec_pretty(data)?)?;
    Ok(())
}

fn encrypt_dpapi(plaintext: &str) -> Result<String, StorageError> {
    run_powershell(
        "Add-Type -AssemblyName System.Security;$bytes=[System.Text.Encoding]::UTF8.GetBytes($env:TUI_UNTIS_SECRET);$enc=[System.Security.Cryptography.ProtectedData]::Protect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[Convert]::ToBase64String($enc)",
        &[("TUI_UNTIS_SECRET", plaintext)],
    )
}

fn decrypt_dpapi(ciphertext_b64: &str) -> Result<String, StorageError> {
    run_powershell(
        "Add-Type -AssemblyName System.Security;$bytes=[Convert]::FromBase64String($env:TUI_UNTIS_SECRET_B64);$dec=[System.Security.Cryptography.ProtectedData]::Unprotect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[System.Text.Encoding]::UTF8.GetString($dec)",
        &[("TUI_UNTIS_SECRET_B64", ciphertext_b64)],
    )
}

pub fn get_secure_storage_diagnostic() -> SecretStorageDiagnostic {
    if cfg!(target_os = "macos") {
        if !command_exists("security") {
            return SecretStorageDiagnostic {
                available: false,
                message:
                    "macOS Keychain CLI not found; auto-login password storage is unavailable."
                        .to_owned(),
            };
        }
        return SecretStorageDiagnostic {
            available: true,
            message: String::new(),
        };
    }

    if cfg!(target_os = "linux") {
        if !command_exists("secret-tool") {
            return SecretStorageDiagnostic {
                available: false,
                message: "Install 'secret-tool' (libsecret) to enable secure password storage and auto-login.".to_owned(),
            };
        }
        return SecretStorageDiagnostic {
            available: true,
            message: String::new(),
        };
    }

    if cfg!(windows) {
        if windows_shell_command().is_err() {
            return SecretStorageDiagnostic {
                available: false,
                message: "PowerShell (powershell.exe or pwsh) is required for secure password storage and auto-login.".to_owned(),
            };
        }
        if run_powershell("Add-Type -AssemblyName System.Security; 'ok'", &[]).is_err() {
            return SecretStorageDiagnostic {
                available: false,
                message: "Windows secure storage initialization failed (System.Security unavailable in PowerShell).".to_owned(),
            };
        }
        return SecretStorageDiagnostic {
            available: true,
            message: String::new(),
        };
    }

    SecretStorageDiagnostic {
        available: false,
        message: format!(
            "Secure password storage is not supported on platform '{}'.",
            std::env::consts::OS
        ),
    }
}

pub fn save_password(config: &SavedConfig, password: &str) -> Result<(), StorageError> {
    let account_key = account_key(config);

    if cfg!(target_os = "macos") {
        run_command(
            "security",
            &[
                "add-generic-password",
                "-a",
                &account_key,
                "-s",
                SECRET_SERVICE,
                "-w",
                password,
                "-U",
            ],
            None,
        )?;
        return Ok(());
    }

    if cfg!(target_os = "linux") {
        run_command(
            "secret-tool",
            &[
                "store",
                "--label",
                "tui-untis",
                "service",
                SECRET_SERVICE,
                "account",
                &account_key,
            ],
            Some(password),
        )?;
        return Ok(());
    }

    if cfg!(windows) {
        let encrypted = encrypt_dpapi(password)?;
        let mut store = read_windows_secret_file();
        store.entries.insert(account_key, encrypted);
        write_windows_secret_file(&store)?;
        return Ok(());
    }

    Err(StorageError::Message(format!(
        "Unsupported platform '{}' for secure password storage",
        std::env::consts::OS
    )))
}

pub fn load_password(config: &SavedConfig) -> Result<Option<String>, StorageError> {
    let account_key = account_key(config);

    if cfg!(target_os = "macos") {
        return run_command(
            "security",
            &[
                "find-generic-password",
                "-a",
                &account_key,
                "-s",
                SECRET_SERVICE,
                "-w",
            ],
            None,
        )
        .map(Some)
        .or(Ok(None));
    }

    if cfg!(target_os = "linux") {
        return run_command(
            "secret-tool",
            &["lookup", "service", SECRET_SERVICE, "account", &account_key],
            None,
        )
        .map(Some)
        .or(Ok(None));
    }

    if cfg!(windows) {
        let store = read_windows_secret_file();
        let encrypted = match store.entries.get(&account_key) {
            Some(value) => value,
            None => return Ok(None),
        };
        return decrypt_dpapi(encrypted).map(Some).or(Ok(None));
    }

    Ok(None)
}

pub fn clear_password(config: &SavedConfig) -> Result<(), StorageError> {
    let account_key = account_key(config);

    if cfg!(target_os = "macos") {
        let _ = run_command(
            "security",
            &[
                "delete-generic-password",
                "-a",
                &account_key,
                "-s",
                SECRET_SERVICE,
            ],
            None,
        );
        return Ok(());
    }

    if cfg!(target_os = "linux") {
        let _ = run_command(
            "secret-tool",
            &["clear", "service", SECRET_SERVICE, "account", &account_key],
            None,
        );
        return Ok(());
    }

    if cfg!(windows) {
        let mut store = read_windows_secret_file();
        store.entries.remove(&account_key);
        write_windows_secret_file(&store)?;
    }

    Ok(())
}
