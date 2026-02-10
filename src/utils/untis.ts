import { WebUntis } from "webuntis";
import type { WebAPITimetable } from "webuntis";
import type { Config } from "./config.ts";
import { getCachedWeek, saveWeekToCache } from "./cache.ts";

export interface ParsedLesson {
  instanceId: string;
  subject: string;
  subjectLongName: string;
  lessonText: string;
  cellState: string;
  teacher: string;
  teacherLongName: string;
  allTeachers: string[];
  room: string;
  roomLongName: string;
  allClasses: string[];
  startTime: string;
  endTime: string;
  cancelled: boolean;
  substitution: boolean;
  remarks: string;
}

export interface TimeUnit {
  name: string;
  startTime: string;
  endTime: string;
}

export interface DayTimetable {
  date: Date;
  dayName: string;
  lessons: ParsedLesson[];
}

export interface WeekTimetable {
  days: DayTimetable[];
  timegrid: TimeUnit[];
}

function formatUntisTime(time: number): string {
  const str = time.toString().padStart(4, "0");
  return `${str.slice(0, 2)}:${str.slice(2)}`;
}

function parseUntisDate(dateNum: number): Date {
  const str = dateNum.toString();
  const year = parseInt(str.slice(0, 4));
  const month = parseInt(str.slice(4, 6)) - 1;
  const day = parseInt(str.slice(6, 8));
  return new Date(year, month, day);
}

const DAY_NAMES = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

function parseTimetableEntry(
  entry: WebAPITimetable,
  indexInDay: number,
  dateNum: number,
): ParsedLesson {
  const entryAny = entry as any;
  const subject = entry.subjects?.[0]?.element?.name || "Unknown";
  const subjectLongName = entry.subjects?.[0]?.element?.longName || subject;
  
  const lessonText = entryAny.lessonText || "";

  const teacher = entry.teachers?.[0]?.element?.name || "";
  const teacherLongName = entry.teachers?.[0]?.element?.longName || teacher;
  const allTeachers = (entry.teachers ?? [])
    .map((teacherEntry) => teacherEntry.element?.name || "")
    .filter(Boolean);

  const room = entry.rooms?.[0]?.element?.name || "";
  const roomLongName = entry.rooms?.[0]?.element?.longName || room;

  const classesRaw = (entryAny.classes ?? entryAny.klassen ?? []) as Array<{
    name?: string;
    element?: { name?: string };
  }>;
  const allClasses = classesRaw
    .map((classEntry) => classEntry.element?.name || classEntry.name || "")
    .filter(Boolean);

  const instanceId =
    String(
      entryAny.id ??
        entryAny.lessonId ??
        entryAny.lstid ??
        `${dateNum}-${entry.startTime}-${entry.endTime}-${subject}-${teacher}-${room}-${indexInDay}`,
    );

  return {
    instanceId,
    subject,
    subjectLongName,
    lessonText,
    cellState: entry.cellState || "",
    teacher,
    teacherLongName,
    allTeachers,
    room,
    roomLongName,
    allClasses,
    startTime: formatUntisTime(entry.startTime),
    endTime: formatUntisTime(entry.endTime),
    cancelled: (entry.is?.standard === false && entry.cellState === "SUBSTITUTION") || entry.lessonCode === "cancelled",
    substitution: entry.is?.substitution === true,
    remarks: entryAny.info || entryAny.substitutionText || "",
  };
}

function getElementIds(
  elements:
    | Array<{ id?: number; element?: { id?: number } }>
    | { id?: number; element?: { id?: number } }
    | undefined,
): string {
  if (!elements) return "";
  const list = Array.isArray(elements) ? elements : [elements];
  if (list.length === 0) return "";

  return list
    .map((item) => String(item.id ?? item.element?.id ?? ""))
    .filter(Boolean)
    .sort()
    .join(",");
}

function buildDuplicateEntryKey(entry: WebAPITimetable): string {
  const entryAny = entry as any;
  const isFlags = entry.is
    ? Object.entries(entry.is)
        .filter(([, value]) => value === true)
        .map(([key]) => key)
        .sort()
        .join(",")
    : "";

  return [
    String(entry.date),
    String(entry.startTime),
    String(entry.endTime),
    String(entryAny.lessonId ?? entryAny.lstid ?? ""),
    String(entry.lessonCode ?? ""),
    String(entry.cellState ?? ""),
    getElementIds(entry.subjects as any),
    getElementIds(entry.teachers as any),
    getElementIds(entry.rooms as any),
    getElementIds((entry as any).klassen),
    getElementIds((entry as any).studentGroup),
    isFlags,
  ].join("|");
}

function dedupeDayEntries(entries: WebAPITimetable[]): WebAPITimetable[] {
  const seen = new Set<string>();
  const deduped: WebAPITimetable[] = [];

  for (const entry of entries) {
    const key = buildDuplicateEntryKey(entry);
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(entry);
  }

  return deduped;
}

export function getMonday(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  d.setHours(0, 0, 0, 0);
  return d;
}

export function addDays(date: Date, days: number): Date {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d;
}

export function formatDate(date: Date): string {
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export async function fetchWeekTimetable(
  config: Config,
  weekDate: Date
): Promise<WeekTimetable> {
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis"
  );

  await untis.login();

  try {
    const [raw, timegridRaw] = await Promise.all([
      untis.getOwnTimetableForWeek(weekDate, 1),
      untis.getTimegrid(),
    ]);

    // Parse timegrid
    // WebUntis returns timegrids per day, but usually they are the same.
    // We take the first one that has time units.
    const firstGrid = timegridRaw.find((g) => g.timeUnits.length > 0);
    const timegrid: TimeUnit[] = (firstGrid?.timeUnits || []).map((u) => ({
      name: u.name,
      startTime: formatUntisTime(u.startTime),
      endTime: formatUntisTime(u.endTime),
    }));

    // Group by date
    const byDate = new Map<number, WebAPITimetable[]>();
    for (const entry of raw) {
      const existing = byDate.get(entry.date) || [];
      existing.push(entry);
      byDate.set(entry.date, existing);
    }

    // Build monday-friday structure
    const monday = getMonday(weekDate);
    const days: DayTimetable[] = [];

    for (let i = 0; i < 5; i++) {
      const dayDate = addDays(monday, i);
      const dateNum = parseInt(
        `${dayDate.getFullYear()}${(dayDate.getMonth() + 1).toString().padStart(2, "0")}${dayDate.getDate().toString().padStart(2, "0")}`
      );

      const entries = dedupeDayEntries(byDate.get(dateNum) || []);
      // Sort by start time
      entries.sort((a, b) => a.startTime - b.startTime);

      days.push({
        date: dayDate,
        dayName: DAY_NAMES[dayDate.getDay()] || "Unknown",
        lessons: entries.map((entry, idx) => parseTimetableEntry(entry, idx, dateNum)),
      });
    }

    const result: WeekTimetable = { days, timegrid };
    const mondayStr = getMonday(weekDate).toISOString().split("T")[0];
    if (mondayStr) {
      saveWeekToCache(mondayStr, result);
    }

    return result;
  } finally {
    await untis.logout();
  }
}

export async function getWeekTimetableWithCache(
  config: Config,
  weekDate: Date,
  forceRefresh: boolean = false
): Promise<{ data: WeekTimetable; fromCache: boolean }> {
  const mondayStr = getMonday(weekDate).toISOString().split("T")[0]!;

  if (!forceRefresh) {
    const cached = getCachedWeek(mondayStr);
    if (cached) {
      return { data: cached, fromCache: true };
    }
  }

  const data = await fetchWeekTimetable(config, weekDate);
  return { data, fromCache: false };
}

export async function testCredentials(config: Config): Promise<boolean> {
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis"
  );

  try {
    await untis.login();
    await untis.logout();
    return true;
  } catch {
    return false;
  }
}
