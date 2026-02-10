import { WebUntis } from "webuntis";
import { loadConfig } from "../src/utils/config.ts";

interface CliOptions {
  targetDate: Date;
  asJson: boolean;
}

function parseCliOptions(args: string[]): CliOptions {
  const asJson = args.includes("--json");
  const dateArg = args.find((arg) => !arg.startsWith("--"));

  return {
    targetDate: parseDateArg(dateArg),
    asJson,
  };
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
        Number.parseInt(normalized[1]!, 10),
        Number.parseInt(normalized[2]!, 10) - 1,
        Number.parseInt(normalized[3]!, 10),
      )
    : new Date(value);

  if (Number.isNaN(parsed.getTime())) {
    throw new Error(`Invalid date '${value}'. Use YYYY-MM-DD or YYYY/MM/DD.`);
  }

  parsed.setHours(0, 0, 0, 0);
  return parsed;
}

function getMonday(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  d.setHours(0, 0, 0, 0);
  return d;
}

function formatLocalDate(date: Date): string {
  const y = date.getFullYear();
  const m = `${date.getMonth() + 1}`.padStart(2, "0");
  const d = `${date.getDate()}`.padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function toDateNum(date: Date): number {
  return Number.parseInt(formatLocalDate(date).replaceAll("-", ""), 10);
}

async function main() {
  const options = parseCliOptions(process.argv.slice(2));
  const config = loadConfig();

  if (!config) {
    throw new Error("No config found. Login once in the app first.");
  }

  const untis = new WebUntis(
    config.school,
    config.username,
    config.password,
    config.server,
    "tui-untis",
  );

  await untis.login();

  try {
    const monday = getMonday(options.targetDate);
    const targetDateNum = toDateNum(options.targetDate);
    const [rawWeekEntries, timegrid] = await Promise.all([
      untis.getOwnTimetableForWeek(monday, 1),
      untis.getTimegrid(),
    ]);

    const dayEntries = rawWeekEntries
      .filter((entry) => entry.date === targetDateNum)
      .sort((a, b) => {
        if (a.startTime !== b.startTime) return a.startTime - b.startTime;
        if (a.endTime !== b.endTime) return a.endTime - b.endTime;
        return ((a as any).id ?? 0) - ((b as any).id ?? 0);
      });

    const payload = {
      weekMonday: formatLocalDate(monday),
      targetDate: formatLocalDate(options.targetDate),
      targetDateNum,
      weekEntryCount: rawWeekEntries.length,
      dayEntryCount: dayEntries.length,
      dayEntries,
      rawTimegrid: timegrid,
    };

    if (options.asJson) {
      console.log(JSON.stringify(payload, null, 2));
      return;
    }

    console.log(`Week Monday : ${payload.weekMonday}`);
    console.log(`Target date : ${payload.targetDate} (${payload.targetDateNum})`);
    console.log(`Day entries : ${payload.dayEntryCount}`);
    console.log("\nRaw day entries:\n");
    console.log(JSON.stringify(dayEntries, null, 2));
    console.log("\nRaw timegrid:\n");
    console.log(JSON.stringify(timegrid, null, 2));
  } finally {
    await untis.logout();
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
