import { homedir } from "os";
import { join } from "path";
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import type { DayTimetable } from "./untis.ts";

const CACHE_DIR = join(homedir(), ".config", "tui-untis");
const CACHE_FILE = join(CACHE_DIR, "cache.json");

interface CacheData {
  // Key is ISO Monday date string
  weeks: Record<string, {
    data: DayTimetable[];
    timestamp: number;
  }>;
}

function ensureCacheDir() {
  if (!existsSync(CACHE_DIR)) {
    mkdirSync(CACHE_DIR, { recursive: true });
  }
}

export function getCachedWeek(mondayStr: string): DayTimetable[] | null {
  try {
    if (!existsSync(CACHE_FILE)) return null;
    const raw = readFileSync(CACHE_FILE, "utf-8");
    const cache: CacheData = JSON.parse(raw);
    
    const week = cache.weeks[mondayStr];
    if (!week) return null;

    // Convert string dates back to Date objects
    return week.data.map(day => ({
      ...day,
      date: new Date(day.date)
    }));
  } catch {
    return null;
  }
}

export function saveWeekToCache(mondayStr: string, data: DayTimetable[]): void {
  try {
    ensureCacheDir();
    let cache: CacheData = { weeks: {} };
    if (existsSync(CACHE_FILE)) {
      cache = JSON.parse(readFileSync(CACHE_FILE, "utf-8"));
    }
    
    cache.weeks[mondayStr] = {
      data,
      timestamp: Date.now()
    };

    writeFileSync(CACHE_FILE, JSON.stringify(cache, null, 2));
  } catch {
    // Ignore cache write errors
  }
}

export function clearCache(): void {
  try {
    if (existsSync(CACHE_FILE)) {
      writeFileSync(CACHE_FILE, JSON.stringify({ weeks: {} }));
    }
  } catch {
    // Ignore
  }
}
