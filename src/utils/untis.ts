import { WebUntis } from "webuntis";
import type { WebAPITimetable } from "webuntis";
import type { Config } from "./config.ts";

export interface ParsedLesson {
  subject: string;
  teacher: string;
  room: string;
  startTime: string;
  endTime: string;
  cancelled: boolean;
  substitution: boolean;
}

export interface DayTimetable {
  date: Date;
  dayName: string;
  lessons: ParsedLesson[];
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

function parseTimetableEntry(entry: WebAPITimetable): ParsedLesson {
  const subject =
    entry.subjects?.[0]?.element?.longName ||
    entry.subjects?.[0]?.element?.name ||
    "Unknown";
  const teacher =
    entry.teachers?.[0]?.element?.name ||
    entry.teachers?.[0]?.element?.longName ||
    "";
  const room =
    entry.rooms?.[0]?.element?.name ||
    entry.rooms?.[0]?.element?.longName ||
    "";

  return {
    subject,
    teacher,
    room,
    startTime: formatUntisTime(entry.startTime),
    endTime: formatUntisTime(entry.endTime),
    cancelled: entry.is?.standard === false && entry.cellState === "SUBSTITUTION" || entry.lessonCode === "cancelled",
    substitution: entry.is?.substitution === true,
  };
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
): Promise<DayTimetable[]> {
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis"
  );

  await untis.login();

  try {
    const raw = await untis.getOwnTimetableForWeek(weekDate, 1);

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

      const entries = byDate.get(dateNum) || [];
      // Sort by start time
      entries.sort((a, b) => a.startTime - b.startTime);

      days.push({
        date: dayDate,
        dayName: DAY_NAMES[dayDate.getDay()] || "Unknown",
        lessons: entries.map(parseTimetableEntry),
      });
    }

    return days;
  } finally {
    await untis.logout();
  }
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
