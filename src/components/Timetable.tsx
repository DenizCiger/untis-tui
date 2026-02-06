import React, { useState, useEffect } from "react";
import { Box, Text, useInput, useApp, useStdout } from "ink";
import Spinner from "ink-spinner";
import type { Config } from "../utils/config.ts";
import {
  fetchWeekTimetable,
  getMonday,
  addDays,
  formatDate,
  type DayTimetable,
  type ParsedLesson,
} from "../utils/untis.ts";

interface TimetableProps {
  config: Config;
  onLogout: () => void;
}

const SUBJECT_COLORS: string[] = [
  "cyan",
  "green",
  "yellow",
  "magenta",
  "blue",
  "red",
  "white",
];

function getSubjectColor(
  subject: string,
  colorMap: Map<string, string>
): string {
  if (!colorMap.has(subject)) {
    colorMap.set(
      subject,
      SUBJECT_COLORS[colorMap.size % SUBJECT_COLORS.length]!
    );
  }
  return colorMap.get(subject)!;
}

function LessonBlock({
  lesson,
  color,
}: {
  lesson: ParsedLesson;
  color: string;
}) {
  if (lesson.cancelled) {
    return (
      <Box flexDirection="column">
        <Text strikethrough dimColor>
          {lesson.startTime}-{lesson.endTime} {lesson.subject}
        </Text>
        <Text dimColor italic>
          {"  "}CANCELLED
        </Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column">
      <Text color={color} bold>
        {lesson.startTime}-{lesson.endTime} {lesson.subject}
      </Text>
      <Box>
        <Text dimColor>{"  "}</Text>
        {lesson.room ? <Text color="white">{lesson.room}</Text> : null}
        {lesson.room && lesson.teacher ? <Text dimColor> | </Text> : null}
        {lesson.teacher ? <Text dimColor>{lesson.teacher}</Text> : null}
      </Box>
    </Box>
  );
}

function DayColumn({
  day,
  isToday,
  colorMap,
}: {
  day: DayTimetable;
  isToday: boolean;
  colorMap: Map<string, string>;
}) {
  const dateStr = day.date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
  const header = `${day.dayName.slice(0, 3)} ${dateStr}`;

  return (
    <Box
      flexDirection="column"
      flexGrow={1}
      flexBasis={0}
      borderStyle="single"
      borderColor={isToday ? "cyan" : "gray"}
      paddingX={1}
    >
      <Box justifyContent="center" marginBottom={1}>
        <Text bold color={isToday ? "cyan" : "white"} underline={isToday}>
          {header}
        </Text>
      </Box>

      {day.lessons.length === 0 ? (
        <Text dimColor italic>
          No lessons
        </Text>
      ) : (
        day.lessons.map((lesson, i) => (
          <Box key={i} marginBottom={i < day.lessons.length - 1 ? 1 : 0}>
            <LessonBlock
              lesson={lesson}
              color={getSubjectColor(lesson.subject, colorMap)}
            />
          </Box>
        ))
      )}
    </Box>
  );
}

export default function Timetable({ config, onLogout }: TimetableProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [weekOffset, setWeekOffset] = useState(0);
  const [days, setDays] = useState<DayTimetable[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [colorMap] = useState(() => new Map<string, string>());

  const termWidth = stdout?.columns || 120;

  const currentMonday = getMonday(addDays(new Date(), weekOffset * 7));
  const currentFriday = addDays(currentMonday, 4);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setError("");
      try {
        const targetDate = addDays(new Date(), weekOffset * 7);
        const result = await fetchWeekTimetable(config, targetDate);
        if (!cancelled) {
          setDays(result);
        }
      } catch (err: any) {
        if (!cancelled) {
          setError(err?.message || "Failed to fetch timetable");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, [weekOffset]);

  useInput((input, key) => {
    if (input === "q") {
      exit();
    }
    if (input === "l") {
      onLogout();
    }
    if (key.leftArrow) {
      setWeekOffset((prev) => prev - 1);
    }
    if (key.rightArrow) {
      setWeekOffset((prev) => prev + 1);
    }
    if (input === "t") {
      setWeekOffset(0);
    }
    if (input === "r") {
      setDays(null);
      setLoading(true);
      fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7))
        .then(setDays)
        .catch((err) => setError(err?.message || "Refresh failed"))
        .finally(() => setLoading(false));
    }
  });

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  return (
    <Box flexDirection="column" width={termWidth}>
      {/* Header */}
      <Box marginBottom={1} justifyContent="space-between" paddingX={1}>
        <Text bold color="cyan">
          WebUntis TUI
        </Text>
        <Text dimColor>
          {config.username}@{config.school}
        </Text>
      </Box>

      {/* Week navigation */}
      <Box marginBottom={1} justifyContent="center">
        <Text dimColor>{"<-- "}</Text>
        <Text bold>
          {formatDate(currentMonday)} - {formatDate(currentFriday)}
        </Text>
        <Text dimColor>{" -->"}</Text>
        {weekOffset === 0 && (
          <Text color="cyan" bold>
            {" "}
            (This week)
          </Text>
        )}
      </Box>

      {/* Timetable grid */}
      {loading ? (
        <Box justifyContent="center" marginTop={2}>
          <Text color="yellow">
            <Spinner type="dots" />
          </Text>
          <Text color="yellow"> Loading timetable...</Text>
        </Box>
      ) : error ? (
        <Box marginTop={1} paddingX={1}>
          <Text color="red">Error: {error}</Text>
        </Box>
      ) : days ? (
        <Box flexDirection="row" width="100%">
          {days.map((day, i) => {
            const dayDate = new Date(day.date);
            dayDate.setHours(0, 0, 0, 0);
            const isToday = dayDate.getTime() === today.getTime();
            return (
              <DayColumn
                key={i}
                day={day}
                isToday={isToday}
                colorMap={colorMap}
              />
            );
          })}
        </Box>
      ) : null}

      {/* Footer / controls */}
      <Box marginTop={1} justifyContent="center" paddingX={1}>
        <Text dimColor>
          {"  "}
          <Text color="white" bold>
            {"<-/->"}
          </Text>
          {" Week  "}
          <Text color="white" bold>
            t
          </Text>
          {" Today  "}
          <Text color="white" bold>
            r
          </Text>
          {" Refresh  "}
          <Text color="white" bold>
            l
          </Text>
          {" Logout  "}
          <Text color="white" bold>
            q
          </Text>
          {" Quit"}
        </Text>
      </Box>
    </Box>
  );
}
