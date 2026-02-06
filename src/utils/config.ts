import { homedir } from "os";
import { join } from "path";
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "fs";

export interface Config {
  school: string;
  username: string;
  password: string;
  server: string; // e.g. "mese.webuntis.com"
}

const CONFIG_DIR = join(homedir(), ".config", "tui-untis");
const CONFIG_FILE = join(CONFIG_DIR, "config.json");

export function loadConfig(): Config | null {
  try {
    if (!existsSync(CONFIG_FILE)) return null;
    const raw = readFileSync(CONFIG_FILE, "utf-8");
    const parsed = JSON.parse(raw);
    if (parsed.school && parsed.username && parsed.password && parsed.server) {
      return parsed as Config;
    }
    return null;
  } catch {
    return null;
  }
}

export function saveConfig(config: Config): void {
  mkdirSync(CONFIG_DIR, { recursive: true });
  writeFileSync(CONFIG_FILE, JSON.stringify(config, null, 2), {
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
