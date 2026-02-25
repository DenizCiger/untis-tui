import { homedir } from "os";
import { join } from "path";
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import type { ParsedLesson, WeekTimetable } from "./untis.ts";

const CACHE_DIR = join(homedir(), ".config", "tui-untis");
const CACHE_FILE = join(CACHE_DIR, "cache.json");
const MAX_CACHED_WEEKS = 12;
const CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 21;

interface CacheData {
  // Key is "{targetKey}:{mondayISO}" (legacy: "mondayISO" for own timetable)
  weeks: Record<
    string,
    {
      data: WeekTimetable;
      timestamp: number;
    }
  >;
}

function ensureCacheDir() {
  if (!existsSync(CACHE_DIR)) {
    mkdirSync(CACHE_DIR, { recursive: true });
  }
}

export function buildWeekCacheKey(
  mondayStr: string,
  targetKey: string = "own",
): string {
  const normalizedTargetKey = targetKey.trim() || "own";
  return `${normalizedTargetKey}:${mondayStr}`;
}

export function getWeekLookupKeys(
  mondayStr: string,
  targetKey: string = "own",
): string[] {
  const normalizedTargetKey = targetKey.trim() || "own";
  if (normalizedTargetKey === "own") {
    return [buildWeekCacheKey(mondayStr, normalizedTargetKey), mondayStr];
  }

  return [buildWeekCacheKey(mondayStr, normalizedTargetKey)];
}

export function getCachedWeek(
  mondayStr: string,
  targetKey: string = "own",
): WeekTimetable | null {
  try {
    if (!existsSync(CACHE_FILE)) return null;
    const raw = readFileSync(CACHE_FILE, "utf-8");
    const cache: CacheData = JSON.parse(raw);

    const lookupKeys = getWeekLookupKeys(mondayStr, targetKey);
    let week: CacheData["weeks"][string] | undefined;
    let storageKey = "";

    for (const lookupKey of lookupKeys) {
      const existing = cache.weeks[lookupKey];
      if (!existing) continue;
      week = existing;
      storageKey = lookupKey;
      break;
    }

    if (!week) return null;

    if (Date.now() - week.timestamp > CACHE_TTL_MS) {
      if (storageKey) {
        delete cache.weeks[storageKey];
      }
      writeFileSync(CACHE_FILE, JSON.stringify(cache, null, 2));
      return null;
    }

    // Convert string dates back to Date objects
    return {
      ...week.data,
      days: week.data.days.map((day) => ({
        ...day,
        date: new Date(day.date),
        lessons: day.lessons.map((lesson, index) =>
          ensureLessonInstanceId(lesson, day.date, index),
        ),
      })),
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
  const datePart = new Date(dayDate).toISOString().slice(0, 10);
  return {
    ...lesson,
    lessonText: lesson.lessonText || "",
    cellState: lesson.cellState || "",
    allTeachers: lesson.allTeachers ?? (lesson.teacher ? [lesson.teacher] : []),
    allClasses: lesson.allClasses ?? [],
    instanceId:
      lesson.instanceId ||
      `${datePart}-${lesson.startTime}-${lesson.endTime}-${lesson.subject}-${lesson.teacher}-${lesson.room}-${indexInDay}`,
  };
}

export function saveWeekToCache(
  mondayStr: string,
  data: WeekTimetable,
  targetKey: string = "own",
): void {
  try {
    ensureCacheDir();
    let cache: CacheData = { weeks: {} };
    if (existsSync(CACHE_FILE)) {
      cache = JSON.parse(readFileSync(CACHE_FILE, "utf-8"));
    }

    const storageKey = buildWeekCacheKey(mondayStr, targetKey);

    cache.weeks[storageKey] = {
      data,
      timestamp: Date.now(),
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
