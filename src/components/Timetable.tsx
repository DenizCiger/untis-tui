import React, { useState, useEffect } from "react";
import { Box, Text, useInput, useApp, useStdout } from "ink";
import Spinner from "ink-spinner";
import type { Config } from "../utils/config.ts";
import {
  fetchWeekTimetable,
  getWeekTimetableWithCache,
  getMonday,
  addDays,
  formatDate,
  type WeekTimetable,
  type DayTimetable,
  type ParsedLesson,
  type TimeUnit,
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
  colorMap: Map<string, string>,
): string {
  if (!colorMap.has(subject)) {
    colorMap.set(
      subject,
      SUBJECT_COLORS[colorMap.size % SUBJECT_COLORS.length]!,
    );
  }
  return colorMap.get(subject)!;
}

function LessonCell({
  lesson,
  color,
  isFocused,
  isCurrent,
}: {
  lesson: ParsedLesson;
  color: string;
  isFocused?: boolean;
  isCurrent?: boolean;
}) {
  return (
    <Box
      flexGrow={1}
      flexBasis={0}
      minHeight={3}
      flexDirection="column"
      paddingX={1}
    >
      <Text
        color={
          isFocused
            ? "yellow"
            : lesson.cancelled
              ? "gray"
              : isCurrent
                ? "cyan"
                : color
        }
        bold={isFocused || isCurrent}
        strikethrough={lesson.cancelled}
      >
        {lesson.subject}
      </Text>
      <Box>
        <Text
          color={isFocused ? "white" : isCurrent ? "cyan" : undefined}
          dimColor={!isFocused && !isCurrent}
        >
          {lesson.room || "?"}
        </Text>
        {lesson.teacher && (
          <Text
            color={isFocused ? "white" : isCurrent ? "cyan" : undefined}
            dimColor={!isFocused && !isCurrent}
          >
            {" "}
            {lesson.teacher}
          </Text>
        )}
      </Box>
    </Box>
  );
}

function GridRow({
  period,
  periodIdx,
  days,
  colorMap,
  todayIdx,
  selectedDayIdx,
  selectedPeriodIdx,
  selectedLessonIdx,
  currentPeriodIdx,
}: {
  period: TimeUnit;
  periodIdx: number;
  days: DayTimetable[];
  colorMap: Map<string, string>;
  todayIdx: number;
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  currentPeriodIdx: number;
}) {
  return (
    <Box flexDirection="row">
      {/* Time Label */}
      <Box
        width={15}
        paddingX={1}
        justifyContent="center"
        flexDirection="column"
        borderStyle="single"
      >
        <Text bold color={periodIdx === currentPeriodIdx ? "cyan" : "yellow"}>
          {period.name}
        </Text>
        <Text dimColor>{period.startTime}</Text>
      </Box>

      {/* Day Cells */}
      {days.map((day, dayIdx) => {
        const lessonsInPeriod = day.lessons.filter(
          (l) => l.startTime === period.startTime,
        );

        const isCellFocused =
          dayIdx === selectedDayIdx && periodIdx === selectedPeriodIdx;
        const isToday = dayIdx === todayIdx;
        const isCurrentPeriod = periodIdx === currentPeriodIdx && isToday;
        const hasLessons = lessonsInPeriod.length > 0;

        return (
          <Box
            key={dayIdx}
            flexGrow={1}
            flexBasis={0}
            borderStyle="single"
            borderColor={
              isCellFocused
                ? "yellow"
                : isCurrentPeriod
                  ? "cyan"
                  : hasLessons
                    ? (isToday ? "white" : "gray")
                    : "black"
            }
            minHeight={3}
            flexDirection="row"
          >
            {!hasLessons ? (
              <Box flexGrow={1}>
                <Box flexGrow={1} justifyContent="center" alignItems="center">
                  <Text color="gray" dimColor>
                    {" · "}
                  </Text>
                </Box>
              </Box>
            ) : (
              lessonsInPeriod.map((lesson, i) => (
                <LessonCell
                  key={i}
                  lesson={lesson}
                  color={getSubjectColor(lesson.subject, colorMap)}
                  isFocused={isCellFocused && i === selectedLessonIdx}
                  isCurrent={isCurrentPeriod}
                />
              ))
            )}
          </Box>
        );
      })}
    </Box>
  );
}

export default function Timetable({ config, onLogout }: TimetableProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [weekOffset, setWeekOffset] = useState(0);
  const [data, setData] = useState<WeekTimetable | null>(null);
  const [loading, setLoading] = useState(true);
  const [isFromCache, setIsFromCache] = useState(false);
  const [error, setError] = useState("");
  const [colorMap] = useState(() => new Map<string, string>());

  const [selectedDayIdx, setSelectedDayIdx] = useState(() => {
    const d = new Date().getDay();
    return d >= 1 && d <= 5 ? d - 1 : 0;
  });
  const [selectedPeriodIdx, setSelectedPeriodIdx] = useState(0);
  const [selectedLessonIdx, setSelectedLessonIdx] = useState(0);
  const [now, setNow] = useState(new Date());

  const termWidth = (stdout?.columns || 120) - 4;

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  const currentMonday = getMonday(addDays(new Date(), weekOffset * 7));
  const currentFriday = addDays(currentMonday, 4);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setData(null);
      setError("");
      const targetDate = addDays(new Date(), weekOffset * 7);

      try {
        const { data: cachedData, fromCache } = await getWeekTimetableWithCache(
          config,
          targetDate,
        );
        if (!cancelled) {
          if (fromCache) {
            setData(cachedData);
            setIsFromCache(true);
            setLoading(false);

            // Set initial period focus if today
            const nowTime = new Date();
            const timeStr = `${nowTime.getHours().toString().padStart(2, "0")}:${nowTime.getMinutes().toString().padStart(2, "0")}`;
            const pIdx = cachedData.timegrid.findIndex(
              (p) => timeStr >= p.startTime && timeStr <= p.endTime,
            );
            if (pIdx !== -1 && weekOffset === 0) {
              setSelectedPeriodIdx(pIdx);
            }

            // Background refresh
            fetchWeekTimetable(config, targetDate)
              .then((freshData) => {
                if (!cancelled) {
                  setData(freshData);
                  setIsFromCache(false);
                }
              })
              .catch(() => {});
          } else {
            setData(cachedData);
            setIsFromCache(false);
            setLoading(false);

            // Set initial period focus if today
            const nowTime = new Date();
            const timeStr = `${nowTime.getHours().toString().padStart(2, "0")}:${nowTime.getMinutes().toString().padStart(2, "0")}`;
            const pIdx = cachedData.timegrid.findIndex(
              (p) => timeStr >= p.startTime && timeStr <= p.endTime,
            );
            if (pIdx !== -1 && weekOffset === 0) {
              setSelectedPeriodIdx(pIdx);
            }
          }
        }
      } catch (err: any) {
        if (!cancelled) {
          setError(err?.message || "Failed to fetch timetable");
          setLoading(false);
        }
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, [weekOffset, config]);

  useInput((input, key) => {
    if (input === "q") exit();
    if (input === "l") onLogout();
    if (key.leftArrow) {
      if (key.shift) setWeekOffset((prev) => prev - 1);
      else setSelectedDayIdx((prev) => Math.max(0, prev - 1));
    }
    if (key.rightArrow) {
      if (key.shift) setWeekOffset((prev) => prev + 1);
      else setSelectedDayIdx((prev) => Math.min(4, prev + 1));
    }
    if (key.upArrow) {
      setSelectedPeriodIdx((prev) => Math.max(0, prev - 1));
      setSelectedLessonIdx(0);
    }
    if (key.downArrow) {
      setSelectedPeriodIdx((prev) =>
        Math.min((data?.timegrid.length || 1) - 1, prev + 1),
      );
      setSelectedLessonIdx(0);
    }
    if (key.tab) {
      const day = data?.days[selectedDayIdx];
      const period = data?.timegrid[selectedPeriodIdx];
      if (day && period) {
        const lessons = day.lessons.filter(
          (l) => l.startTime === period.startTime,
        );
        if (lessons.length > 1) {
          setSelectedLessonIdx((prev) => (prev + 1) % lessons.length);
        }
      }
    }
    if (input === "t") {
      setWeekOffset(0);
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const idx = data?.days.findIndex(
        (d) => new Date(d.date).setHours(0, 0, 0, 0) === today.getTime(),
      );
      if (idx !== undefined && idx !== -1) setSelectedDayIdx(idx);
    }
    if (input === "r") {
      setLoading(true);
      setError("");
      fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7))
        .then((fresh) => {
          setData(fresh);
          setIsFromCache(false);
        })
        .catch((err) => setError(err?.message || "Refresh failed"))
        .finally(() => setLoading(false));
    }
  });

  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const todayIdx =
    data?.days.findIndex(
      (d) => new Date(d.date).setHours(0, 0, 0, 0) === today.getTime(),
    ) ?? -1;

  const currentTimeStr = `${now.getHours().toString().padStart(2, "0")}:${now
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;

  const currentPeriodIdx =
    data?.timegrid.findIndex((p) => {
      return currentTimeStr >= p.startTime && currentTimeStr <= p.endTime;
    }) ?? -1;

  const selectedLesson = (() => {
    if (!data) return null;
    const day = data.days[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return null;
    const lessons = day.lessons.filter((l) => l.startTime === period.startTime);
    return lessons[selectedLessonIdx] || null;
  })();

  return (
    <Box flexDirection="column" width={termWidth} paddingX={1}>
      {/* Header */}
      <Box marginBottom={1} justifyContent="space-between">
        <Text bold color="cyan">
          WebUntis TUI
        </Text>
        <Box>
          <Text dimColor>
            {config.username}@{config.school}
          </Text>
          {isFromCache && !loading && (
            <Text color="yellow" dimColor>
              {" "}
              (cached)
            </Text>
          )}
        </Box>
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

      {/* Grid Header */}
      {data && (
        <Box flexDirection="row">
          <Box width={15} paddingX={1} borderStyle="single">
            <Text bold>Time</Text>
          </Box>
          {data.days.map((day, i) => (
            <Box
              key={i}
              flexGrow={1}
              flexBasis={0}
              justifyContent="center"
              borderStyle="single"
              borderColor={i === todayIdx ? "cyan" : "gray"}
            >
              <Text bold color={i === todayIdx ? "cyan" : "white"}>
                {day.dayName.slice(0, 3)}{" "}
                {day.date.toLocaleDateString("en-US", {
                  month: "short",
                  day: "numeric",
                })}
              </Text>
            </Box>
          ))}
        </Box>
      )}

      {/* Grid Rows */}
      {loading ? (
        <Box
          justifyContent="center"
          marginTop={2}
          flexDirection="column"
          alignItems="center"
        >
          <Box>
            <Text color="yellow">
              <Spinner type="dots" />
            </Text>
            <Text color="yellow"> Loading timetable...</Text>
          </Box>
        </Box>
      ) : error ? (
        <Box marginTop={1}>
          <Text color="red">Error: {error}</Text>
        </Box>
      ) : data ? (
        <Box flexDirection="column">
          {data.timegrid.map((period, i) => (
            <GridRow
              key={i}
              period={period}
              periodIdx={i}
              days={data.days}
              colorMap={colorMap}
              todayIdx={todayIdx}
              selectedDayIdx={selectedDayIdx}
              selectedPeriodIdx={selectedPeriodIdx}
              selectedLessonIdx={selectedLessonIdx}
              currentPeriodIdx={currentPeriodIdx}
            />
          ))}
        </Box>
      ) : null}

      {/* Lesson Details */}
      <Box
        marginTop={1}
        paddingX={1}
        borderStyle="round"
        borderColor="blue"
        flexDirection="column"
        minHeight={5}
      >
        {selectedLesson ? (
          <Box flexDirection="column">
            <Box justifyContent="space-between">
              <Text bold color="cyan">
                {selectedLesson.subjectLongName} ({selectedLesson.subject})
              </Text>
              <Text color="yellow">
                {selectedLesson.startTime} - {selectedLesson.endTime}
              </Text>
            </Box>
            <Box>
              <Text dimColor>Teacher: </Text>
              <Text>
                {selectedLesson.teacherLongName ||
                  selectedLesson.teacher ||
                  "N/A"}
              </Text>
              <Text dimColor> Room: </Text>
              <Text>{selectedLesson.room || selectedLesson.room || "N/A"}</Text>
            </Box>
            {selectedLesson.remarks && (
              <Box marginTop={1}>
                <Text color="magenta" italic>
                  ℹ {selectedLesson.remarks}
                </Text>
              </Box>
            )}
            {selectedLesson.cancelled && (
              <Text color="red" bold>
                CANCELLED
              </Text>
            )}
          </Box>
        ) : (
          <Box justifyContent="center" alignItems="center" flexGrow={1}>
            <Text dimColor>Select a lesson to see details</Text>
          </Box>
        )}
      </Box>

      {/* Footer */}
      <Box marginTop={1} justifyContent="center">
        <Text dimColor>
          [<Text color="white">Arrows</Text> Nav] [
          <Text color="white">Shift+Arrows</Text> Week] [
          <Text color="white">Tab</Text> Overlap] [<Text color="white">t</Text>{" "}
          Today] [<Text color="white">r</Text> Refresh] [
          <Text color="white">q</Text> Quit]
        </Text>
      </Box>
    </Box>
  );
}
