import type {
  DayTimetable,
  ParsedLesson,
  TimeUnit,
  WeekTimetable,
} from "../../utils/untis.ts";
import { COLORS } from "../colors.ts";

export type Continuation = "single" | "start" | "middle" | "end";

export interface RenderLesson {
  lesson: ParsedLesson;
  continuation: Continuation;
  lessonKey: string;
  lessonInstanceId: string;
  continuityKey: string;
}

export interface SelectedLessonRange {
  lesson: ParsedLesson;
  lessonKey: string;
  lessonInstanceId: string;
  startPeriodIdx: number;
  endPeriodIdx: number;
}

export type DayLessonIndex = Map<string, RenderLesson[]>;
export interface SplitLanePeriod {
  split: boolean;
  left: RenderLesson | null;
  right: RenderLesson | null;
  hiddenCount: number;
}

export type DaySplitLaneIndex = Map<string, SplitLanePeriod>;

export interface OverlayPeriod {
  split: boolean;
  lanes: Array<RenderLesson | null>;
  hiddenCount: number;
}

export type DayOverlayIndex = Map<string, OverlayPeriod>;

export const EMPTY_LESSONS: RenderLesson[] = [];

export function getSubjectColor(subject: string, colorMap: Map<string, string>): string {
  if (!colorMap.has(subject)) {
    colorMap.set(
      subject,
      COLORS.subjectStripeCycle[colorMap.size % COLORS.subjectStripeCycle.length]!,
    );
  }

  return colorMap.get(subject)!;
}

function getLessonKey(lesson: ParsedLesson): string {
  return [
    lesson.subject,
    lesson.teacher,
    lesson.room,
    lesson.cancelled ? "1" : "0",
    lesson.substitution ? "1" : "0",
  ].join("|");
}

export function indexLessonsByPeriod(
  days: DayTimetable[],
  timegrid: TimeUnit[],
): DayLessonIndex[] {
  const periodRanges = timegrid.map((period) => ({
    ...period,
    startMinutes: parseTimeToMinutes(period.startTime),
    endMinutes: parseTimeToMinutes(period.endTime),
  }));

  return days.map((day) => {
    const indexed: DayLessonIndex = new Map();
    const sortedLessons = [...day.lessons].sort(compareLessonsForDisplay);
    const emptyParsedLessons: ParsedLesson[] = [];

    const lessonsByPeriod = periodRanges.map((period) =>
      sortedLessons
        .filter((lesson) =>
          lessonIntersectsPeriod(lesson, period.startMinutes, period.endMinutes),
        )
        .sort((left, right) => compareLessonsForPeriod(left, right, period.startTime)),
    );

    const keyCountsByPeriod = lessonsByPeriod.map((lessonsInPeriod) => {
      const counts = new Map<string, number>();
      for (const lesson of lessonsInPeriod) {
        const lessonKey = getLessonKey(lesson);
        counts.set(lessonKey, (counts.get(lessonKey) ?? 0) + 1);
      }

      return counts;
    });

    for (const [periodIdx, period] of periodRanges.entries()) {
      const lessonsInPeriod = lessonsByPeriod[periodIdx] ?? emptyParsedLessons;

      if (lessonsInPeriod.length === 0) {
        continue;
      }

      const seenInPeriod = new Map<string, number>();
      const rendered = lessonsInPeriod.map<RenderLesson>((lesson) => {
        const lessonKey = getLessonKey(lesson);
        const occurrence = seenInPeriod.get(lessonKey) ?? 0;
        seenInPeriod.set(lessonKey, occurrence + 1);

        const previousCount =
          periodIdx > 0
            ? (keyCountsByPeriod[periodIdx - 1]?.get(lessonKey) ?? 0)
            : 0;
        const nextCount =
          periodIdx < periodRanges.length - 1
            ? (keyCountsByPeriod[periodIdx + 1]?.get(lessonKey) ?? 0)
            : 0;

        const hasPrevious = previousCount > occurrence;
        const hasNext = nextCount > occurrence;

        let continuation: Continuation = "single";
        if (hasPrevious && hasNext) {
          continuation = "middle";
        } else if (hasPrevious) {
          continuation = "end";
        } else if (hasNext) {
          continuation = "start";
        }

        return {
          lesson,
          continuation,
          lessonKey,
          lessonInstanceId: lesson.instanceId || getLessonKey(lesson),
          continuityKey: `${lessonKey}#${occurrence}`,
        };
      });

      indexed.set(period.startTime, rendered);
    }

    return indexed;
  });
}

export function buildSplitLaneIndex(
  dayIndex: DayLessonIndex,
  timegrid: TimeUnit[],
): DaySplitLaneIndex {
  const overlay = buildOverlayIndex(dayIndex, timegrid, 2);
  const split: DaySplitLaneIndex = new Map();

  for (const period of timegrid) {
    const row = overlay.get(period.startTime);
    split.set(period.startTime, {
      split: row?.split ?? false,
      left: row?.lanes[0] ?? null,
      right: row?.lanes[1] ?? null,
      hiddenCount: row?.hiddenCount ?? 0,
    });
  }

  return split;
}

export function buildOverlayIndex(
  dayIndex: DayLessonIndex,
  timegrid: TimeUnit[],
  laneCount: number,
): DayOverlayIndex {
  const overlay: DayOverlayIndex = new Map();
  const lanes = Math.max(1, laneCount);
  let previousLaneKeys: Array<string | null> = Array.from({ length: lanes }, () => null);

  for (const [periodIdx, period] of timegrid.entries()) {
    const entries = dayIndex.get(period.startTime) ?? EMPTY_LESSONS;
    const shouldSplit =
      entries.length > 1 || shouldReserveSplitForSingle(dayIndex, timegrid, periodIdx, entries);

    if (!shouldSplit) {
      overlay.set(period.startTime, {
        split: false,
        lanes: Array.from({ length: lanes }, () => null),
        hiddenCount: 0,
      });
      previousLaneKeys = Array.from({ length: lanes }, () => null);
      continue;
    }

    const laneEntries: Array<RenderLesson | null> = Array.from({ length: lanes }, () => null);
    const remaining = [...entries];

    for (let laneIdx = 0; laneIdx < lanes; laneIdx += 1) {
      const previousKey = previousLaneKeys[laneIdx];
      if (!previousKey) {
        continue;
      }

      const matchIdx = remaining.findIndex((entry) => entry.continuityKey === previousKey);
      if (matchIdx !== -1) {
        const [matched] = remaining.splice(matchIdx, 1);
        laneEntries[laneIdx] = matched ?? null;
      }
    }

    if (lanes >= 1 && !laneEntries[0]) {
      const candidate = pickLeftLaneCandidate(remaining);
      if (candidate) {
        laneEntries[0] = candidate;
        removeFromRemainingByContinuityKey(remaining, candidate.continuityKey);
      }
    }

    if (lanes >= 2 && !laneEntries[1]) {
      const candidate = pickRightLaneCandidate(remaining);
      if (candidate) {
        laneEntries[1] = candidate;
        removeFromRemainingByContinuityKey(remaining, candidate.continuityKey);
      }
    }

    for (let laneIdx = 2; laneIdx < lanes; laneIdx += 1) {
      if (!laneEntries[laneIdx] && remaining.length > 0) {
        laneEntries[laneIdx] = remaining.shift() ?? null;
      }
    }

    overlay.set(period.startTime, {
      split: true,
      lanes: laneEntries,
      hiddenCount: remaining.length,
    });

    previousLaneKeys = laneEntries.map((entry) => entry?.continuityKey ?? null);
  }

  return overlay;
}

function parseTimeToMinutes(value: string): number {
  const [hours, minutes] = value.split(":").map((part) => Number.parseInt(part, 10));
  return (hours ?? 0) * 60 + (minutes ?? 0);
}

function lessonIntersectsPeriod(
  lesson: ParsedLesson,
  periodStartMinutes: number,
  periodEndMinutes: number,
): boolean {
  const lessonStartMinutes = parseTimeToMinutes(lesson.startTime);
  const lessonEndMinutes = parseTimeToMinutes(lesson.endTime);

  return lessonStartMinutes < periodEndMinutes && lessonEndMinutes > periodStartMinutes;
}

function compareLessonsForDisplay(left: ParsedLesson, right: ParsedLesson): number {
  const byStart = left.startTime.localeCompare(right.startTime);
  if (byStart !== 0) return byStart;

  const byEnd = left.endTime.localeCompare(right.endTime);
  if (byEnd !== 0) return byEnd;

  const bySubject = left.subject.localeCompare(right.subject);
  if (bySubject !== 0) return bySubject;

  const byTeacher = left.teacher.localeCompare(right.teacher);
  if (byTeacher !== 0) return byTeacher;

  const byRoom = left.room.localeCompare(right.room);
  if (byRoom !== 0) return byRoom;

  return (left.instanceId || "").localeCompare(right.instanceId || "");
}

function compareLessonsForPeriod(
  left: ParsedLesson,
  right: ParsedLesson,
  periodStart: string,
): number {
  const leftStartsHere = left.startTime === periodStart;
  const rightStartsHere = right.startTime === periodStart;
  if (leftStartsHere !== rightStartsHere) {
    return leftStartsHere ? -1 : 1;
  }

  return compareLessonsForDisplay(left, right);
}

function shouldReserveSplitForSingle(
  dayIndex: DayLessonIndex,
  timegrid: TimeUnit[],
  periodIdx: number,
  entries: RenderLesson[],
): boolean {
  if (entries.length !== 1) {
    return false;
  }

  const [entry] = entries;
  if (!entry || entry.continuation === "single") {
    return false;
  }

  const previousStart = periodIdx > 0 ? timegrid[periodIdx - 1]?.startTime : undefined;
  const nextStart = periodIdx < timegrid.length - 1 ? timegrid[periodIdx + 1]?.startTime : undefined;

  const previousEntries = previousStart ? dayIndex.get(previousStart) ?? EMPTY_LESSONS : EMPTY_LESSONS;
  const nextEntries = nextStart ? dayIndex.get(nextStart) ?? EMPTY_LESSONS : EMPTY_LESSONS;

  const continuityKey = entry.continuityKey;
  const lessonId = entry.lessonInstanceId;
  const previousHasOverlap =
    previousEntries.length > 1 &&
    previousEntries.some(
      (other) =>
        other.continuityKey === continuityKey ||
        other.lessonInstanceId === lessonId,
    );
  const nextHasOverlap =
    nextEntries.length > 1 &&
    nextEntries.some(
      (other) =>
        other.continuityKey === continuityKey ||
        other.lessonInstanceId === lessonId,
    );

  return previousHasOverlap || nextHasOverlap;
}

function pickLeftLaneCandidate(entries: RenderLesson[]): RenderLesson | null {
  const continuing = entries.find(
    (entry) => entry.continuation === "middle" || entry.continuation === "end",
  );
  if (continuing) {
    return continuing;
  }

  return entries[0] ?? null;
}

function pickRightLaneCandidate(entries: RenderLesson[]): RenderLesson | null {
  const startsHere = entries.find(
    (entry) => entry.continuation === "start" || entry.continuation === "single",
  );
  if (startsHere) {
    return startsHere;
  }

  return entries[0] ?? null;
}

function removeFromRemainingByContinuityKey(
  entries: RenderLesson[],
  continuityKey: string,
): void {
  const index = entries.findIndex((entry) => entry.continuityKey === continuityKey);
  if (index !== -1) {
    entries.splice(index, 1);
  }
}

export function findCurrentPeriodIndex(timegrid: TimeUnit[]): number {
  const now = new Date();
  const currentTime = `${now.getHours().toString().padStart(2, "0")}:${now
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;

  return timegrid.findIndex(
    (period) => currentTime >= period.startTime && currentTime <= period.endTime,
  );
}

export function getSelectedLessonRange(
  data: WeekTimetable,
  dayLessonIndex: DayLessonIndex[],
  selectedDayIdx: number,
  selectedPeriodIdx: number,
  selectedLessonIdx: number,
): SelectedLessonRange | null {
  const dayIndex = dayLessonIndex[selectedDayIdx];
  const period = data.timegrid[selectedPeriodIdx];
  if (!dayIndex || !period) return null;

  const entries = dayIndex.get(period.startTime) ?? EMPTY_LESSONS;
  const selectedEntry = entries[selectedLessonIdx];
  if (!selectedEntry) return null;

  let startPeriodIdx = selectedPeriodIdx;
  let endPeriodIdx = selectedPeriodIdx;

  while (startPeriodIdx > 0) {
    const prevPeriod = data.timegrid[startPeriodIdx - 1];
    if (!prevPeriod) break;

    const prevEntries = dayIndex.get(prevPeriod.startTime) ?? EMPTY_LESSONS;
    const match = prevEntries.find(
      (entry) =>
        entry.lessonInstanceId === selectedEntry.lessonInstanceId,
    );
    if (!match) break;

    startPeriodIdx -= 1;
  }

  while (endPeriodIdx < data.timegrid.length - 1) {
    const nextPeriod = data.timegrid[endPeriodIdx + 1];
    if (!nextPeriod) break;

    const nextEntries = dayIndex.get(nextPeriod.startTime) ?? EMPTY_LESSONS;
    const match = nextEntries.find(
      (entry) =>
        entry.lessonInstanceId === selectedEntry.lessonInstanceId,
    );
    if (!match) break;

    endPeriodIdx += 1;
  }

  return {
    lesson: selectedEntry.lesson,
    lessonKey: selectedEntry.lessonKey,
    lessonInstanceId: selectedEntry.lessonInstanceId,
    startPeriodIdx,
    endPeriodIdx,
  };
}
