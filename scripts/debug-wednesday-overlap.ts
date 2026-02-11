import { loadConfig, type Config } from "../src/utils/config.ts";
import { loadPassword } from "../src/utils/secret.ts";
import {
  addDays,
  fetchWeekTimetable,
  getMonday,
  type ParsedLesson,
  type TimeUnit,
} from "../src/utils/untis.ts";
import { indexLessonsByPeriod } from "../src/components/timetable/model.ts";

function formatLesson(lesson: ParsedLesson): string {
  const who = [lesson.subject, lesson.teacher, lesson.room].filter(Boolean).join("/");
  return `${lesson.instanceId} ${lesson.startTime}-${lesson.endTime} ${who}`;
}

function intersects(lesson: ParsedLesson, period: TimeUnit): boolean {
  return lesson.startTime < period.endTime && lesson.endTime > period.startTime;
}

async function main() {
  const savedConfig = loadConfig();
  if (!savedConfig) {
    console.error("No saved profile found. Please login once with the app first.");
    process.exit(1);
  }

  const password = (await loadPassword(savedConfig)) || process.env.UNTIS_PASSWORD;
  if (!password) {
    console.error(
      "No password in secure store. Log in once with the app or set UNTIS_PASSWORD.",
    );
    process.exit(1);
  }

  const config: Config = { ...savedConfig, password };

  const monday = getMonday(new Date());
  const wednesday = addDays(monday, 2);
  const data = await fetchWeekTimetable(config, wednesday);

  const wednesdayDay = data.days.find((day) => day.date.getDay() === 3);
  if (!wednesdayDay) {
    console.error("Could not find Wednesday in fetched week data.");
    process.exit(1);
  }

  console.log(`Week Monday: ${monday.toISOString().slice(0, 10)}`);
  console.log(`Wednesday : ${wednesdayDay.date.toISOString().slice(0, 10)}`);
  console.log(`Lessons   : ${wednesdayDay.lessons.length}`);
  console.log("\nRaw Wednesday lessons:");
  for (const lesson of wednesdayDay.lessons) {
    console.log(`- ${formatLesson(lesson)}`);
  }

  const dayIndex = indexLessonsByPeriod([wednesdayDay], data.timegrid)[0]!;
  console.log("\nPer period occupancy:");

  for (const period of data.timegrid) {
    const entries = dayIndex.get(period.startTime) ?? [];
    if (entries.length === 0) continue;

    const overlaps = entries.length > 1 ? " overlap" : "";
    console.log(`\n${period.name} ${period.startTime}-${period.endTime}${overlaps}`);
    for (const entry of entries) {
      console.log(
        `  - ${entry.lessonInstanceId} [${entry.continuation}] ${entry.lesson.subject} ${entry.lesson.startTime}-${entry.lesson.endTime}`,
      );
    }

    const intersectingRaw = wednesdayDay.lessons.filter((lesson) => intersects(lesson, period));
    if (intersectingRaw.length !== entries.length) {
      console.log(
        `  ! mismatch raw(${intersectingRaw.length}) vs indexed(${entries.length}) for ${period.startTime}`,
      );
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
