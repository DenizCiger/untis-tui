import { WebUntis, WebUntisElementType } from "webuntis";
import type { Klasse, Room, Teacher, WebAPITimetable } from "webuntis";
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

export interface ParsedAbsence {
  id: number;
  studentName: string;
  reason: string;
  text: string;
  excuseStatus: string;
  isExcused: boolean;
  startDate: Date;
  endDate: Date;
  startTime: string;
  endTime: string;
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

export type TimetableTargetType = "own" | "class" | "room" | "teacher";
export type TimetableSearchTargetType = Exclude<TimetableTargetType, "own">;

export type TimetableTarget =
  | { type: "own" }
  | {
      type: TimetableSearchTargetType;
      id: number;
      name: string;
      longName: string;
    };

export interface TimetableSearchItem {
  type: TimetableSearchTargetType;
  id: number;
  name: string;
  longName: string;
  searchText: string;
}

interface OwnTimetableRequest {
  mode: "own";
}

interface TargetTimetableRequest {
  mode: "target";
  id: number;
  type: number;
}

const TARGET_TYPE_LABEL: Record<TimetableSearchTargetType, string> = {
  class: "Class",
  room: "Room",
  teacher: "Teacher",
};

export function getDefaultTimetableTarget(): TimetableTarget {
  return { type: "own" };
}

function normalizeTimetableTarget(target?: TimetableTarget): TimetableTarget {
  if (!target || target.type === "own") {
    return { type: "own" };
  }

  return target;
}

export function targetToCacheKey(target?: TimetableTarget): string {
  const normalized = normalizeTimetableTarget(target);
  if (normalized.type === "own") return "own";
  return `${normalized.type}:${normalized.id}`;
}

export function formatTimetableTargetLabel(target?: TimetableTarget): string {
  const normalized = normalizeTimetableTarget(target);
  if (normalized.type === "own") return "My timetable";
  return `${TARGET_TYPE_LABEL[normalized.type]}: ${normalized.name}`;
}

function mapTargetTypeToWebUntisElementType(type: TimetableSearchTargetType): number {
  if (type === "class") return WebUntisElementType.CLASS;
  if (type === "room") return WebUntisElementType.ROOM;
  return WebUntisElementType.TEACHER;
}

export function resolveTimetableForWeekRequest(
  target?: TimetableTarget,
): OwnTimetableRequest | TargetTimetableRequest {
  const normalized = normalizeTimetableTarget(target);
  if (normalized.type === "own") {
    return { mode: "own" };
  }

  return {
    mode: "target",
    id: normalized.id,
    type: mapTargetTypeToWebUntisElementType(normalized.type),
  };
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
  const isFlags = (entry.is ?? {}) as Record<string, boolean | undefined>;
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
    substitution:
      isFlags.substitution === true ||
      isFlags.roomSubstitution === true ||
      isFlags.roomSubstition === true ||
      isFlags.roomsubstition === true,
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

function buildSearchText(...parts: Array<string | undefined>): string {
  return parts
    .filter((part): part is string => Boolean(part && part.trim()))
    .join(" ")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
}

function normalizeSearchItems(items: TimetableSearchItem[]): TimetableSearchItem[] {
  const deduped = new Map<string, TimetableSearchItem>();

  for (const item of items) {
    const key = `${item.type}:${item.id}`;
    if (deduped.has(key)) continue;
    deduped.set(key, item);
  }

  return Array.from(deduped.values()).sort((left, right) => {
    const byType = left.type.localeCompare(right.type);
    if (byType !== 0) return byType;

    const byName = left.name.localeCompare(right.name);
    if (byName !== 0) return byName;

    return left.id - right.id;
  });
}

async function fetchClassesForSearch(untis: WebUntis): Promise<Klasse[]> {
  try {
    const schoolYear = await untis.getCurrentSchoolyear();
    return await untis.getClasses(true, schoolYear.id);
  } catch {
    const classes = await (untis as any).getClasses(true);
    return Array.isArray(classes) ? (classes as Klasse[]) : [];
  }
}

function mapTeachersToSearchItems(teachers: Teacher[]): TimetableSearchItem[] {
  return teachers.map((teacher) => {
    const teacherShortName = teacher.name?.trim() || "";
    const teacherSurname = teacher.longName?.trim() || "";
    const teacherForename = teacher.foreName?.trim() || "";

    const combinedFullName = `${teacherForename} ${teacherSurname}`.trim();
    const displayName =
      combinedFullName ||
      teacherSurname ||
      teacherShortName ||
      String(teacher.id);
    const secondaryName =
      teacherShortName && teacherShortName !== displayName
        ? teacherShortName
        : teacherSurname && teacherSurname !== displayName
          ? teacherSurname
          : displayName;

    return {
      type: "teacher",
      id: teacher.id,
      name: displayName,
      longName: secondaryName,
      searchText: buildSearchText(
        displayName,
        secondaryName,
        teacherShortName,
        teacherSurname,
        teacherForename,
      ),
    };
  });
}

function mapRoomsToSearchItems(rooms: Room[]): TimetableSearchItem[] {
  return rooms.map((room) => ({
    type: "room",
    id: room.id,
    name: room.name || room.longName || String(room.id),
    longName: room.longName || room.name || String(room.id),
    searchText: buildSearchText(room.name, room.longName, room.alternateName),
  }));
}

function mapClassesToSearchItems(classes: Klasse[]): TimetableSearchItem[] {
  return classes.map((klasse) => ({
    type: "class",
    id: klasse.id,
    name: klasse.name || klasse.longName || String(klasse.id),
    longName: klasse.longName || klasse.name || String(klasse.id),
    searchText: buildSearchText(klasse.name, klasse.longName),
  }));
}

export async function fetchTimetableSearchIndex(
  config: Config,
): Promise<TimetableSearchItem[]> {
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis",
  );

  await untis.login();

  try {
    const [teachers, rooms, classes] = await Promise.all([
      untis.getTeachers(),
      untis.getRooms(),
      fetchClassesForSearch(untis),
    ]);

    return normalizeSearchItems([
      ...mapClassesToSearchItems(classes),
      ...mapRoomsToSearchItems(rooms),
      ...mapTeachersToSearchItems(teachers),
    ]);
  } finally {
    await untis.logout();
  }
}

export async function fetchWeekTimetable(
  config: Config,
  weekDate: Date,
  target?: TimetableTarget,
): Promise<WeekTimetable> {
  const normalizedTarget = normalizeTimetableTarget(target);
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis",
  );

  await untis.login();

  try {
    const request = resolveTimetableForWeekRequest(normalizedTarget);
    const timetableRequest =
      request.mode === "own"
        ? untis.getOwnTimetableForWeek(weekDate, 1)
        : untis.getTimetableForWeek(weekDate, request.id, request.type, 1);

    const [raw, timegridRaw] = await Promise.all([
      timetableRequest,
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
      saveWeekToCache(mondayStr, result, targetToCacheKey(normalizedTarget));
    }

    return result;
  } finally {
    await untis.logout();
  }
}

export async function getWeekTimetableWithCache(
  config: Config,
  weekDate: Date,
  forceRefresh: boolean = false,
  target?: TimetableTarget,
): Promise<{ data: WeekTimetable; fromCache: boolean }> {
  const mondayStr = getMonday(weekDate).toISOString().split("T")[0]!;
  const targetKey = targetToCacheKey(target);

  if (!forceRefresh) {
    const cached = getCachedWeek(mondayStr, targetKey);
    if (cached) {
      return { data: cached, fromCache: true };
    }
  }

  const data = await fetchWeekTimetable(config, weekDate, target);
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

export async function fetchAbsencesForRange(
  config: Config,
  rangeStart: Date,
  rangeEnd: Date,
): Promise<ParsedAbsence[]> {
  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis",
  );

  await untis.login();

  try {
    const result = await untis.getAbsentLesson(rangeStart, rangeEnd);
    const absences = result.absences ?? [];

    return absences
      .map((absence) => ({
        id: absence.id,
        studentName: absence.studentName || config.username,
        reason: absence.reason || "",
        text: absence.text || "",
        excuseStatus: absence.excuseStatus || "",
        isExcused: absence.isExcused,
        startDate: parseUntisDate(absence.startDate),
        endDate: parseUntisDate(absence.endDate),
        startTime: formatUntisTime(absence.startTime),
        endTime: formatUntisTime(absence.endTime),
      }))
      .sort((left, right) => {
        const byStartDate = right.startDate.getTime() - left.startDate.getTime();
        if (byStartDate !== 0) return byStartDate;

        const byStartTime = right.startTime.localeCompare(left.startTime);
        if (byStartTime !== 0) return byStartTime;

        return right.id - left.id;
      });
  } finally {
    await untis.logout();
  }
}
