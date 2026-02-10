import type {
  DayTimetable,
  ParsedLesson,
  TimeUnit,
  WeekTimetable,
} from "../../utils/untis.ts";

export type Continuation = "single" | "start" | "middle" | "end";

export interface RenderLesson {
  lesson: ParsedLesson;
  continuation: Continuation;
  lessonKey: string;
  occurrence: number;
}

export interface SelectedLessonRange {
  lesson: ParsedLesson;
  lessonKey: string;
  occurrence: number;
  startPeriodIdx: number;
  endPeriodIdx: number;
}

export type DayLessonIndex = Map<string, RenderLesson[]>;

export const EMPTY_LESSONS: RenderLesson[] = [];

const STRIPE_COLORS: string[] = [
  "cyan",
  "green",
  "yellow",
  "magenta",
  "blue",
  "red",
  "white",
];

export function getSubjectColor(subject: string, colorMap: Map<string, string>): string {
  if (!colorMap.has(subject)) {
    colorMap.set(subject, STRIPE_COLORS[colorMap.size % STRIPE_COLORS.length]!);
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
  const periodIndexByStart = new Map(
    timegrid.map((period, idx) => [period.startTime, idx]),
  );

  return days.map((day) => {
    const lessonsByStart = new Map<string, ParsedLesson[]>();

    for (const lesson of day.lessons) {
      const existing = lessonsByStart.get(lesson.startTime);
      if (existing) {
        existing.push(lesson);
      } else {
        lessonsByStart.set(lesson.startTime, [lesson]);
      }
    }

    const countsByPeriod = new Map<string, Map<string, number>>();
    for (const [startTime, lessons] of lessonsByStart) {
      const counts = new Map<string, number>();
      for (const lesson of lessons) {
        const key = getLessonKey(lesson);
        counts.set(key, (counts.get(key) ?? 0) + 1);
      }
      countsByPeriod.set(startTime, counts);
    }

    const indexed: DayLessonIndex = new Map();
    for (const [startTime, lessons] of lessonsByStart) {
      const periodIdx = periodIndexByStart.get(startTime);
      if (periodIdx === undefined) {
        continue;
      }

      const previousStart =
        periodIdx > 0 ? timegrid[periodIdx - 1]?.startTime : undefined;
      const nextStart =
        periodIdx < timegrid.length - 1
          ? timegrid[periodIdx + 1]?.startTime
          : undefined;

      const seenInPeriod = new Map<string, number>();
      const rendered = lessons.map<RenderLesson>((lesson) => {
        const key = getLessonKey(lesson);
        const occurrence = seenInPeriod.get(key) ?? 0;
        seenInPeriod.set(key, occurrence + 1);

        const prevCount = previousStart
          ? (countsByPeriod.get(previousStart)?.get(key) ?? 0)
          : 0;
        const nextCount = nextStart
          ? (countsByPeriod.get(nextStart)?.get(key) ?? 0)
          : 0;

        const hasPrevious = prevCount > occurrence;
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
          lessonKey: key,
          occurrence,
        };
      });

      indexed.set(startTime, rendered);
    }

    return indexed;
  });
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
        entry.lessonKey === selectedEntry.lessonKey &&
        entry.occurrence === selectedEntry.occurrence,
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
        entry.lessonKey === selectedEntry.lessonKey &&
        entry.occurrence === selectedEntry.occurrence,
    );
    if (!match) break;

    endPeriodIdx += 1;
  }

  return {
    lesson: selectedEntry.lesson,
    lessonKey: selectedEntry.lessonKey,
    occurrence: selectedEntry.occurrence,
    startPeriodIdx,
    endPeriodIdx,
  };
}
