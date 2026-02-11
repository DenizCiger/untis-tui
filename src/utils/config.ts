import { homedir } from "os";
import { join } from "path";
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "fs";

export interface Config {
  school: string;
  username: string;
  password: string;
  server: string; // e.g. "mese.webuntis.com"
}

export interface SavedConfig {
  school: string;
  username: string;
  server: string;
}

const CONFIG_DIR = join(homedir(), ".config", "tui-untis");
const CONFIG_FILE = join(CONFIG_DIR, "config.json");

export function loadConfig(): SavedConfig | null {
  try {
    if (!existsSync(CONFIG_FILE)) return null;
    const raw = readFileSync(CONFIG_FILE, "utf-8");
    const parsed = JSON.parse(raw);
    if (parsed.school && parsed.username && parsed.server) {
      return {
        school: String(parsed.school),
        username: String(parsed.username),
        server: String(parsed.server),
      };
    }
    return null;
  } catch {
    return null;
  }
}

export function saveConfig(config: Config | SavedConfig): void {
  const persistedConfig: SavedConfig = {
    school: config.school,
    username: config.username,
    server: config.server,
  };

  mkdirSync(CONFIG_DIR, { recursive: true });
  writeFileSync(CONFIG_FILE, JSON.stringify(persistedConfig, null, 2), {
    mode: 0o600,
  });
}

export function clearConfig(): void {
  try {
    if (existsSync(CONFIG_FILE)) {
      writeFileSync(CONFIG_FILE, "{}", { mode: 0o600 });
    }
  } catch {
    // ignore
  }
}
