import { loadConfig } from "../src/utils/config.ts";
import {
  fetchWeekTimetable,
  getMonday,
  type ParsedLesson,
  type TimeUnit,
} from "../src/utils/untis.ts";
import { buildOverlayIndex, indexLessonsByPeriod } from "../src/components/timetable/model.ts";

interface CliOptions {
  targetDate: Date;
  json: boolean;
  overlay: boolean;
  lanes: number;
}

function parseCliOptions(args: string[]): CliOptions {
  const json = args.includes("--json");
  const overlay = args.includes("--overlay");
  const lanes = parseLaneCount(args);
  const dateArg = args.find((arg) => !arg.startsWith("--"));
  const targetDate = parseDateArg(dateArg);

  return {
    targetDate,
    json,
    overlay,
    lanes,
  };
}

function parseLaneCount(args: string[]): number {
  const flag = args.find((arg) => arg.startsWith("--lanes="));
  if (!flag) return 2;

  const value = Number.parseInt(flag.slice("--lanes=".length), 10);
  if (!Number.isFinite(value) || value < 1) {
    throw new Error(`Invalid lanes value '${flag}'. Example: --lanes=2`);
  }

  return value;
}

function parseDateArg(value: string | undefined): Date {
  if (!value) {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    return today;
  }

  const isoMatch = value.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  const slashMatch = value.match(/^(\d{4})\/(\d{1,2})\/(\d{1,2})$/);
  const normalized = isoMatch ?? slashMatch;
  const parsed = normalized
    ? new Date(
        Number.parseInt(normalized![1]!, 10),
        Number.parseInt(normalized![2]!, 10) - 1,
        Number.parseInt(normalized![3]!, 10),
      )
    : new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    throw new Error(`Invalid date '${value}'. Use ISO format like 2026-02-10.`);
  }

  parsed.setHours(0, 0, 0, 0);
  return parsed;
}

function formatLesson(lesson: ParsedLesson): string {
  const parts = [lesson.subject, lesson.teacher, lesson.room].filter(Boolean);
  return `${lesson.instanceId} ${lesson.startTime}-${lesson.endTime} ${parts.join("/")}`;
}

function intersects(lesson: ParsedLesson, period: TimeUnit): boolean {
  return lesson.startTime < period.endTime && lesson.endTime > period.startTime;
}

function sameDate(left: Date, right: Date): boolean {
  return (
    left.getFullYear() === right.getFullYear() &&
    left.getMonth() === right.getMonth() &&
    left.getDate() === right.getDate()
  );
}

function formatLocalDate(date: Date): string {
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, "0");
  const day = date.getDate().toString().padStart(2, "0");
  return `${year}-${month}-${day}`;
}

async function main() {
  const options = parseCliOptions(process.argv.slice(2));
  const config = loadConfig();

  if (!config) {
    throw new Error("No config found. Login once in the app first.");
  }

  const weekData = await fetchWeekTimetable(config, options.targetDate);
  const monday = getMonday(options.targetDate);
  const targetDayIndex = weekData.days.findIndex((day) => sameDate(day.date, options.targetDate));

  if (targetDayIndex === -1) {
    throw new Error("Target date is outside fetched timetable days (Mon-Fri).");
  }

  const targetDay = weekData.days[targetDayIndex]!;
  const dayIndex = indexLessonsByPeriod([targetDay], weekData.timegrid)[0]!;
  const overlayIndex = buildOverlayIndex(dayIndex, weekData.timegrid, options.lanes);

  if (options.json) {
    const payload = {
      weekMonday: formatLocalDate(monday),
      targetDate: formatLocalDate(targetDay.date),
      dayName: targetDay.dayName,
      rawLessons: targetDay.lessons,
      occupancyByPeriod: weekData.timegrid
        .map((period) => ({
          periodName: period.name,
          startTime: period.startTime,
          endTime: period.endTime,
          entries: dayIndex.get(period.startTime) ?? [],
        }))
        .filter((row) => row.entries.length > 0),
      overlayByPeriod: weekData.timegrid
        .map((period) => ({
          periodName: period.name,
          startTime: period.startTime,
          endTime: period.endTime,
          overlay: overlayIndex.get(period.startTime) ?? null,
        }))
        .filter((row) => row.overlay?.split),
    };

    console.log(JSON.stringify(payload, null, 2));
    return;
  }

  console.log(`Week Monday: ${formatLocalDate(monday)}`);
  console.log(`Date      : ${formatLocalDate(targetDay.date)} (${targetDay.dayName})`);
  console.log(`Lessons   : ${targetDay.lessons.length}`);

  console.log("\nRaw lessons:");
  for (const lesson of targetDay.lessons) {
    console.log(`- ${formatLesson(lesson)}`);
  }

  console.log("\nPer period occupancy:");
  for (const period of weekData.timegrid) {
    const entries = dayIndex.get(period.startTime) ?? [];
    if (entries.length === 0) continue;

    console.log(
      `\n${period.name} ${period.startTime}-${period.endTime}${entries.length > 1 ? " overlap" : ""}`,
    );

    for (const entry of entries) {
      console.log(
        `  - ${entry.lessonInstanceId} [${entry.continuation}] ${entry.lesson.subject} ${entry.lesson.startTime}-${entry.lesson.endTime}`,
      );
    }

    const rawIntersecting = targetDay.lessons.filter((lesson) => intersects(lesson, period));
    if (rawIntersecting.length !== entries.length) {
      console.log(
        `  ! mismatch raw(${rawIntersecting.length}) vs indexed(${entries.length})`,
      );
    }

    if (options.overlay) {
      const overlay = overlayIndex.get(period.startTime);
      if (overlay?.split) {
        const laneLabel = overlay.lanes
          .map((entry, laneIdx) =>
            `${laneIdx}:${entry ? `${entry.lesson.subject}(${entry.continuityKey})` : "-"}`,
          )
          .join(" | ");
        console.log(`  · lanes ${laneLabel}${overlay.hiddenCount > 0 ? ` | +${overlay.hiddenCount}` : ""}`);
      }
    }
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
