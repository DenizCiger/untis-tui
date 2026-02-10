import { homedir } from "os";
import { join } from "path";
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import type { ParsedLesson, WeekTimetable } from "./untis.ts";

const CACHE_DIR = join(homedir(), ".config", "tui-untis");
const CACHE_FILE = join(CACHE_DIR, "cache.json");
const MAX_CACHED_WEEKS = 12;
const CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 21;

interface CacheData {
  // Key is ISO Monday date string
  weeks: Record<string, {
    data: WeekTimetable;
    timestamp: number;
  }>;
}

function ensureCacheDir() {
  if (!existsSync(CACHE_DIR)) {
    mkdirSync(CACHE_DIR, { recursive: true });
  }
}

export function getCachedWeek(mondayStr: string): WeekTimetable | null {
  try {
    if (!existsSync(CACHE_FILE)) return null;
    const raw = readFileSync(CACHE_FILE, "utf-8");
    const cache: CacheData = JSON.parse(raw);
    
    const week = cache.weeks[mondayStr];
    if (!week) return null;
    if (Date.now() - week.timestamp > CACHE_TTL_MS) {
      delete cache.weeks[mondayStr];
      writeFileSync(CACHE_FILE, JSON.stringify(cache, null, 2));
      return null;
    }

    // Convert string dates back to Date objects
    return {
      ...week.data,
      days: week.data.days.map(day => ({
        ...day,
        date: new Date(day.date),
        lessons: day.lessons.map((lesson, index) =>
          ensureLessonInstanceId(lesson, day.date, index),
        ),
      }))
    };
  } catch {
    return null;
  }
}

function ensureLessonInstanceId(
  lesson: ParsedLesson,
  dayDate: Date,
  indexInDay: number,
): ParsedLesson {
  if (lesson.instanceId) {
    return lesson;
  }

  const datePart = new Date(dayDate).toISOString().slice(0, 10);
  return {
    ...lesson,
    instanceId: `${datePart}-${lesson.startTime}-${lesson.endTime}-${lesson.subject}-${lesson.teacher}-${lesson.room}-${indexInDay}`,
  };
}

export function saveWeekToCache(mondayStr: string, data: WeekTimetable): void {
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

    const entries = Object.entries(cache.weeks)
      .sort((a, b) => b[1].timestamp - a[1].timestamp)
      .slice(0, MAX_CACHED_WEEKS);

    cache.weeks = Object.fromEntries(entries);

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
