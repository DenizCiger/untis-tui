import { execFileSync } from "child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";
import type { SavedConfig } from "./config.ts";

const SECRET_SERVICE = "tui-untis";
const CONFIG_DIR = join(homedir(), ".config", "tui-untis");
const WINDOWS_SECRET_FILE = join(CONFIG_DIR, "secrets.json");

interface SecretFileData {
  entries: Record<string, string>;
}

interface SecretStorageDiagnostic {
  available: boolean;
  message: string;
}

function getAccountKey(config: SavedConfig): string {
  return `${config.server}|${config.school}|${config.username}`;
}

function runCommand(
  command: string,
  args: string[],
  options?: { input?: string },
): string {
  return execFileSync(command, args, {
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
    input: options?.input,
  }).trim();
}

function runPowerShell(script: string, env: Record<string, string>): string {
  const shell = getWindowsShellCommand();
  return execFileSync(
    shell,
    ["-NoProfile", "-NonInteractive", "-Command", script],
    {
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, ...env },
    },
  ).trim();
}

function commandExists(command: string): boolean {
  try {
    if (process.platform === "win32") {
      execFileSync("where", [command], { stdio: "ignore" });
      return true;
    }

    execFileSync("which", [command], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function getWindowsShellCommand(): string {
  const candidates = ["powershell.exe", "powershell", "pwsh.exe", "pwsh"];
  for (const candidate of candidates) {
    if (commandExists(candidate)) {
      return candidate;
    }
  }

  throw new Error("No PowerShell executable found");
}

function ensureConfigDir(): void {
  if (!existsSync(CONFIG_DIR)) {
    mkdirSync(CONFIG_DIR, { recursive: true });
  }
}

function readWindowsSecretFile(): SecretFileData {
  if (!existsSync(WINDOWS_SECRET_FILE)) {
    return { entries: {} };
  }

  try {
    const parsed = JSON.parse(readFileSync(WINDOWS_SECRET_FILE, "utf-8")) as SecretFileData;
    if (!parsed || typeof parsed !== "object" || typeof parsed.entries !== "object") {
      return { entries: {} };
    }
    return parsed;
  } catch {
    return { entries: {} };
  }
}

function writeWindowsSecretFile(data: SecretFileData): void {
  ensureConfigDir();
  writeFileSync(WINDOWS_SECRET_FILE, JSON.stringify(data, null, 2), { mode: 0o600 });
}

function encryptDpapi(plaintext: string): string {
  return runPowerShell(
    "Add-Type -AssemblyName System.Security;$bytes=[System.Text.Encoding]::UTF8.GetBytes($env:TUI_UNTIS_SECRET);$enc=[System.Security.Cryptography.ProtectedData]::Protect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[Convert]::ToBase64String($enc)",
    { TUI_UNTIS_SECRET: plaintext },
  );
}

function decryptDpapi(ciphertextB64: string): string {
  return runPowerShell(
    "Add-Type -AssemblyName System.Security;$bytes=[Convert]::FromBase64String($env:TUI_UNTIS_SECRET_B64);$dec=[System.Security.Cryptography.ProtectedData]::Unprotect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[System.Text.Encoding]::UTF8.GetString($dec)",
    { TUI_UNTIS_SECRET_B64: ciphertextB64 },
  );
}

export function getSecureStorageDiagnostic(): SecretStorageDiagnostic {
  if (process.platform === "darwin") {
    if (!commandExists("security")) {
      return {
        available: false,
        message: "macOS Keychain CLI not found; auto-login password storage is unavailable.",
      };
    }

    return { available: true, message: "" };
  }

  if (process.platform === "linux") {
    if (!commandExists("secret-tool")) {
      return {
        available: false,
        message: "Install 'secret-tool' (libsecret) to enable secure password storage and auto-login.",
      };
    }

    return { available: true, message: "" };
  }

  if (process.platform === "win32") {
    const hasShell = ["powershell.exe", "powershell", "pwsh.exe", "pwsh"].some(
      (candidate) => commandExists(candidate),
    );
    if (!hasShell) {
      return {
        available: false,
        message:
          "PowerShell (powershell.exe or pwsh) is required for secure password storage and auto-login.",
      };
    }

    try {
      runPowerShell("Add-Type -AssemblyName System.Security; 'ok'", {});
    } catch {
      return {
        available: false,
        message:
          "Windows secure storage initialization failed (System.Security unavailable in PowerShell).",
      };
    }

    return { available: true, message: "" };
  }

  return {
    available: false,
    message: `Secure password storage is not supported on platform '${process.platform}'.`,
  };
}

export async function savePassword(config: SavedConfig, password: string): Promise<void> {
  const accountKey = getAccountKey(config);

  if (process.platform === "darwin") {
    runCommand("security", [
      "add-generic-password",
      "-a",
      accountKey,
      "-s",
      SECRET_SERVICE,
      "-w",
      password,
      "-U",
    ]);
    return;
  }

  if (process.platform === "linux") {
    runCommand(
      "secret-tool",
      ["store", "--label", "tui-untis", "service", SECRET_SERVICE, "account", accountKey],
      { input: password },
    );
    return;
  }

  if (process.platform === "win32") {
    const encrypted = encryptDpapi(password);
    const store = readWindowsSecretFile();
    store.entries[accountKey] = encrypted;
    writeWindowsSecretFile(store);
    return;
  }

  throw new Error(`Unsupported platform '${process.platform}' for secure password storage`);
}

export async function loadPassword(config: SavedConfig): Promise<string | null> {
  const accountKey = getAccountKey(config);

  try {
    if (process.platform === "darwin") {
      const password = runCommand("security", [
        "find-generic-password",
        "-a",
        accountKey,
        "-s",
        SECRET_SERVICE,
        "-w",
      ]);
      return password || null;
    }

    if (process.platform === "linux") {
      const password = runCommand("secret-tool", [
        "lookup",
        "service",
        SECRET_SERVICE,
        "account",
        accountKey,
      ]);
      return password || null;
    }

    if (process.platform === "win32") {
      const store = readWindowsSecretFile();
      const encrypted = store.entries[accountKey];
      if (!encrypted) return null;
      const password = decryptDpapi(encrypted);
      return password || null;
    }
  } catch {
    return null;
  }

  return null;
}

export async function clearPassword(config: SavedConfig): Promise<void> {
  const accountKey = getAccountKey(config);

  try {
    if (process.platform === "darwin") {
      runCommand("security", [
        "delete-generic-password",
        "-a",
        accountKey,
        "-s",
        SECRET_SERVICE,
      ]);
      return;
    }

    if (process.platform === "linux") {
      runCommand("secret-tool", ["clear", "service", SECRET_SERVICE, "account", accountKey]);
      return;
    }

    if (process.platform === "win32") {
      const store = readWindowsSecretFile();
      if (accountKey in store.entries) {
        delete store.entries[accountKey];
        writeWindowsSecretFile(store);
      }
    }
  } catch {
    // ignore
  }
}
