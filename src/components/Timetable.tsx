import React, { memo, useEffect, useMemo, useState } from "react";
import { Box, Text, useApp, useInput, useStdout } from "ink";
import Spinner from "ink-spinner";
import type { Config } from "../utils/config.ts";
import {
  addDays,
  fetchWeekTimetable,
  formatDate,
  getMonday,
  getWeekTimetableWithCache,
  type DayTimetable,
  type ParsedLesson,
  type TimeUnit,
  type WeekTimetable,
} from "../utils/untis.ts";

interface TimetableProps {
  config: Config;
  onLogout: () => void;
}

type Continuation = "single" | "start" | "middle" | "end";

interface RenderLesson {
  lesson: ParsedLesson;
  continuation: Continuation;
  lessonKey: string;
  occurrence: number;
}

interface SelectedLessonRange {
  lesson: ParsedLesson;
  lessonKey: string;
  occurrence: number;
  startPeriodIdx: number;
  endPeriodIdx: number;
}

type DayLessonIndex = Map<string, RenderLesson[]>;

const EMPTY_LESSONS: RenderLesson[] = [];

const STRIPE_COLORS: string[] = [
  "cyan",
  "green",
  "yellow",
  "magenta",
  "blue",
  "red",
  "white",
];

function getSubjectColor(subject: string, colorMap: Map<string, string>): string {
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

function indexLessonsByPeriod(
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

function truncateText(value: string, maxWidth: number): string {
  if (maxWidth <= 0) return "";
  if (value.length <= maxWidth) return value;
  if (maxWidth <= 3) return value.slice(0, maxWidth);
  return `${value.slice(0, maxWidth - 3)}...`;
}

function fitText(value: string, width: number): string {
  return truncateText(value, width).padEnd(Math.max(0, width), " ");
}

const LessonCell = memo(function LessonCell({
  entry,
  stripeColor,
  isFocused,
  compact,
  contentWidth,
}: {
  entry: RenderLesson;
  stripeColor: string;
  isFocused: boolean;
  compact: boolean;
  contentWidth: number;
}) {
  const { lesson, continuation } = entry;
  const startsHere = continuation === "single" || continuation === "start";
  const continuesDown = continuation === "start" || continuation === "middle";
  const title = startsHere ? lesson.subject : "";
  const meta = startsHere
    ? `${lesson.room || "?"}${lesson.teacher ? ` ${lesson.teacher}` : ""}`
    : "";

  const bg = isFocused ? "white" : "blackBright";
  const fg = isFocused ? "black" : "white";

  return (
    <Box
      flexGrow={1}
      flexBasis={0}
      height={3}
      flexDirection="column"
      justifyContent="center"
    >
      <Text
        backgroundColor={bg}
        color={fg}
        bold={startsHere}
        strikethrough={lesson.cancelled && startsHere}
      >
        <Text color={stripeColor}>▍</Text>
        {fitText(title, contentWidth)}
      </Text>
      <Text backgroundColor={bg} color={fg}>
        <Text color={stripeColor}>▍</Text>
        {fitText(meta, contentWidth)}
      </Text>
      {continuesDown ? (
        <Text backgroundColor={bg} color={fg}>
          <Text color={stripeColor}>▍</Text>
          {" ".repeat(contentWidth)}
        </Text>
      ) : (
        <Text>
          <Text color={stripeColor}> </Text>
          {" ".repeat(contentWidth)}
        </Text>
      )}
    </Box>
  );
});

function findCurrentPeriodIndex(timegrid: TimeUnit[]): number {
  const now = new Date();
  const currentTime = `${now.getHours().toString().padStart(2, "0")}:${now
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;

  return timegrid.findIndex(
    (period) => currentTime >= period.startTime && currentTime <= period.endTime,
  );
}

function getSelectedLessonRange(
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

function GridRow({
  period,
  periodIdx,
  dayLessonIndex,
  colorMap,
  todayIdx,
  selectedDayIdx,
  selectedPeriodIdx,
  selectedLessonIdx,
  selectedRange,
  currentPeriodIdx,
  compact,
  timeColumnWidth,
  dayColumnWidth,
}: {
  period: TimeUnit;
  periodIdx: number;
  dayLessonIndex: DayLessonIndex[];
  colorMap: Map<string, string>;
  todayIdx: number;
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  selectedRange: SelectedLessonRange | null;
  currentPeriodIdx: number;
  compact: boolean;
  timeColumnWidth: number;
  dayColumnWidth: number;
}) {
  return (
    <Box flexDirection="row">
      <Box
        width={timeColumnWidth}
        paddingLeft={1}
        paddingRight={1}
        justifyContent="center"
        flexDirection="column"
        height={3}
      >
        <Text bold color={periodIdx === currentPeriodIdx ? "cyan" : "yellow"}>
          {truncateText(period.name, compact ? 8 : 12)}
        </Text>
        <Text dimColor>{compact ? period.startTime.slice(0, 5) : period.startTime}</Text>
      </Box>

      {dayLessonIndex.map((dayIndex, dayIdx) => {
        const lessonsInPeriod = dayIndex.get(period.startTime) ?? EMPTY_LESSONS;
        const isAnchorFocused =
          dayIdx === selectedDayIdx && periodIdx === selectedPeriodIdx;
        const isRangeFocused =
          dayIdx === selectedDayIdx &&
          !!selectedRange &&
          periodIdx >= selectedRange.startPeriodIdx &&
          periodIdx <= selectedRange.endPeriodIdx;
        const contentWidth = Math.max(4, dayColumnWidth - 1);

        const hasLessons = lessonsInPeriod.length > 0;
        const rangeEntry =
          isRangeFocused && selectedRange
            ? lessonsInPeriod.find(
                (entry) =>
                  entry.lessonKey === selectedRange.lessonKey &&
                  entry.occurrence === selectedRange.occurrence,
              ) ?? null
            : null;
        const selectedEntry = lessonsInPeriod[selectedLessonIdx] ?? null;
        const activeEntry = rangeEntry ?? selectedEntry;
        const showOverlapCount = lessonsInPeriod.length > 1 && !isRangeFocused;
        const displayedLessons =
          isRangeFocused && activeEntry ? [activeEntry] : lessonsInPeriod;

        return (
          <Box
            key={`day-${periodIdx}-${dayIdx}`}
            width={dayColumnWidth}
            height={3}
            flexDirection="row"
          >
            {!hasLessons ? (
              isAnchorFocused ? (
                <Box flexGrow={1} flexDirection="column" justifyContent="center">
                  <Text backgroundColor="white" color="black">
                    {" ".repeat(contentWidth + 1)}
                  </Text>
                  <Text backgroundColor="white" color="black">
                    {" ".repeat(contentWidth + 1)}
                  </Text>
                  <Text backgroundColor="white" color="black">
                    {" ".repeat(contentWidth + 1)}
                  </Text>
                </Box>
              ) : (
                <Box flexGrow={1} justifyContent="center" alignItems="center" paddingX={1}>
                  <Text color="gray" dimColor>
                    .
                  </Text>
                </Box>
              )
            ) : showOverlapCount ? (
              <Box flexGrow={1} justifyContent="center" alignItems="center" paddingX={1}>
                <Text color={isAnchorFocused ? "yellow" : "white"} dimColor={!isAnchorFocused}>
                  {lessonsInPeriod.length}x
                </Text>
              </Box>
            ) : (
              displayedLessons.map((entry, index) => (
                <LessonCell
                  key={`lesson-${dayIdx}-${periodIdx}-${index}`}
                  entry={entry}
                  stripeColor={getSubjectColor(entry.lesson.subject, colorMap)}
                  isFocused={isRangeFocused}
                  compact={compact}
                  contentWidth={contentWidth}
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
  const [colorMap] = useState(() => new Map<string, string>());

  const [weekOffset, setWeekOffset] = useState(0);
  const [data, setData] = useState<WeekTimetable | null>(null);
  const [loading, setLoading] = useState(true);
  const [isFromCache, setIsFromCache] = useState(false);
  const [error, setError] = useState("");

  const [selectedDayIdx, setSelectedDayIdx] = useState(() => {
    const day = new Date().getDay();
    return day >= 1 && day <= 5 ? day - 1 : 0;
  });
  const [selectedPeriodIdx, setSelectedPeriodIdx] = useState(0);
  const [selectedLessonIdx, setSelectedLessonIdx] = useState(0);
  const [now, setNow] = useState(new Date());

  const [scrollOffset, setScrollOffset] = useState(0);
  const [showHelp, setShowHelp] = useState(false);

  const termWidth = Math.max(50, stdout?.columns ?? 120);
  const termHeight = Math.max(18, stdout?.rows ?? 24);
  const compact = termWidth < 90 || termHeight < 24;
  const timeColumnWidth = compact ? 12 : 16;
  const dayColumnWidth = Math.max(
    compact ? 10 : 14,
    Math.floor((termWidth - timeColumnWidth - 2) / 5),
  );

  const currentMonday = getMonday(addDays(new Date(), weekOffset * 7));
  const currentFriday = addDays(currentMonday, 4);

  const gridHeight = Math.max(5, termHeight - (compact ? 9 : 8));
  const rowsPerPage = Math.max(1, Math.floor(gridHeight / 4));

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    if (selectedPeriodIdx < scrollOffset) {
      setScrollOffset(selectedPeriodIdx);
    } else if (selectedPeriodIdx >= scrollOffset + rowsPerPage) {
      setScrollOffset(selectedPeriodIdx - rowsPerPage + 1);
    }
  }, [selectedPeriodIdx, scrollOffset, rowsPerPage]);

  useEffect(() => {
    let cancelled = false;

    const setCurrentPeriodFocus = (weekData: WeekTimetable) => {
      if (weekOffset !== 0) return;
      const currentIdx = findCurrentPeriodIndex(weekData.timegrid);
      if (currentIdx !== -1) {
        setSelectedPeriodIdx(currentIdx);
      }
    };

    async function load() {
      setLoading(true);
      setError("");
      setData(null);

      const targetDate = addDays(new Date(), weekOffset * 7);

      try {
        const { data: cachedData, fromCache } = await getWeekTimetableWithCache(
          config,
          targetDate,
        );

        if (cancelled) return;

        setData(cachedData);
        setIsFromCache(fromCache);
        setLoading(false);
        setCurrentPeriodFocus(cachedData);

        if (fromCache) {
          fetchWeekTimetable(config, targetDate)
            .then((freshData) => {
              if (cancelled) return;
              setData(freshData);
              setIsFromCache(false);
            })
            .catch(() => {});
        }
      } catch (err: any) {
        if (cancelled) return;
        setError(err?.message || "Failed to fetch timetable");
        setLoading(false);
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [config, weekOffset]);

  const dayLessonIndex = useMemo(
    () => (data ? indexLessonsByPeriod(data.days, data.timegrid) : []),
    [data],
  );

  const visiblePeriods = useMemo(() => {
    if (!data) return [];
    return data.timegrid.slice(scrollOffset, scrollOffset + rowsPerPage);
  }, [data, scrollOffset, rowsPerPage]);

  useInput(
    (input, key) => {
      if (input === "q") {
        exit();
        return;
      }

      if (input === "l") {
        onLogout();
        return;
      }

      if (key.leftArrow && key.shift) {
        setWeekOffset((prev) => prev - 1);
        setSelectedPeriodIdx(0);
        setSelectedLessonIdx(0);
        return;
      }

      if (key.rightArrow && key.shift) {
        setWeekOffset((prev) => prev + 1);
        setSelectedPeriodIdx(0);
        setSelectedLessonIdx(0);
        return;
      }

      if (key.leftArrow) {
        setSelectedDayIdx((prev) => Math.max(0, prev - 1));
        setSelectedLessonIdx(0);
        return;
      }

      if (key.rightArrow) {
        setSelectedDayIdx((prev) => Math.min(4, prev + 1));
        setSelectedLessonIdx(0);
        return;
      }

      if (key.upArrow) {
        if (data) {
          const range = getSelectedLessonRange(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            selectedLessonIdx,
          );
          if (range) {
            setSelectedPeriodIdx(Math.max(0, range.startPeriodIdx - 1));
          } else {
            setSelectedPeriodIdx((prev) => Math.max(0, prev - 1));
          }
        } else {
          setSelectedPeriodIdx((prev) => Math.max(0, prev - 1));
        }
        setSelectedLessonIdx(0);
        return;
      }

      if (key.downArrow) {
        const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
        if (data) {
          const range = getSelectedLessonRange(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            selectedLessonIdx,
          );
          if (range) {
            setSelectedPeriodIdx(Math.min(maxPeriod, range.endPeriodIdx + 1));
          } else {
            setSelectedPeriodIdx((prev) => Math.min(maxPeriod, prev + 1));
          }
        } else {
          setSelectedPeriodIdx((prev) => Math.min(maxPeriod, prev + 1));
        }
        setSelectedLessonIdx(0);
        return;
      }

      if (key.tab) {
        const day = dayLessonIndex[selectedDayIdx];
        const period = data?.timegrid[selectedPeriodIdx];
        if (!day || !period) return;

        const lessons = day.get(period.startTime) ?? EMPTY_LESSONS;
        if (lessons.length > 1) {
          setSelectedLessonIdx((prev) => (prev + 1) % lessons.length);
        }
        return;
      }

      if (input === "h") {
        setShowHelp((prev) => !prev);
        return;
      }

      if (input === "t") {
        setWeekOffset(0);
        setSelectedLessonIdx(0);

        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const index = data?.days.findIndex(
          (day) => new Date(day.date).setHours(0, 0, 0, 0) === today.getTime(),
        );

        if (index !== undefined && index !== -1) {
          setSelectedDayIdx(index);
        }
        return;
      }

      if (input === "r") {
        setLoading(true);
        setError("");

        fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7))
          .then((freshData) => {
            setData(freshData);
            setIsFromCache(false);
          })
          .catch((err: any) => {
            setError(err?.message || "Refresh failed");
          })
          .finally(() => setLoading(false));
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const todayIdx =
    data?.days.findIndex(
      (day) => new Date(day.date).setHours(0, 0, 0, 0) === today.getTime(),
    ) ?? -1;

  const currentTime = `${now.getHours().toString().padStart(2, "0")}:${now
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;

  const currentPeriodIdx =
    data?.timegrid.findIndex(
      (period) =>
        currentTime >= period.startTime && currentTime <= period.endTime,
    ) ?? -1;

  const selectedRange = useMemo(() => {
    if (!data) return null;
    return getSelectedLessonRange(
      data,
      dayLessonIndex,
      selectedDayIdx,
      selectedPeriodIdx,
      selectedLessonIdx,
    );
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx, selectedLessonIdx]);

  const selectedLesson = selectedRange?.lesson ?? null;

  const selectedRangeTime = useMemo(() => {
    if (!data || !selectedRange) return null;
    const start = data.timegrid[selectedRange.startPeriodIdx]?.startTime;
    const end = data.timegrid[selectedRange.endPeriodIdx]?.endTime;
    if (!start || !end) return null;
    return `${start} - ${end}`;
  }, [data, selectedRange]);

  const selectedLessonCount = useMemo(() => {
    if (!data) return 0;

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return 0;

    return (day.get(period.startTime) ?? EMPTY_LESSONS).length;
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx]);

  const selectedDayName = data?.days[selectedDayIdx]?.dayName ?? "-";
  const selectedPeriodName = data?.timegrid[selectedPeriodIdx]?.name ?? "-";

  const footerText = useMemo(() => {
    if (compact) {
      return "[Arrows] [Shift+<-/->] [Tab] [h] [t] [r] [q]";
    }

    return "[Arrows Nav] [Shift+<-/-> Week] [Tab Overlap] [h Help] [t Today] [r Refresh] [q Quit]";
  }, [compact]);

  const dividerLine = "-".repeat(
    Math.max(10, timeColumnWidth + dayColumnWidth * 5),
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight} paddingX={0}>
      <Box justifyContent="space-between">
        <Text bold color="cyan">
          {compact ? "WebUntis" : "WebUntis TUI"}
        </Text>

        <Box>
          {!compact && (
            <Text dimColor>
              {truncateText(
                `${config.username}@${config.school}`,
                Math.max(10, termWidth - 22),
              )}
            </Text>
          )}

          {isFromCache && !loading && (
            <Text color="yellow" dimColor>
              {" "}
              (cached)
            </Text>
          )}
        </Box>
      </Box>

      <Box justifyContent="center">
        <Text dimColor>{"<-- "}</Text>
        <Text bold>
          {formatDate(currentMonday)} - {formatDate(currentFriday)}
        </Text>
        <Text dimColor>{" -->"}</Text>
        {weekOffset === 0 && !compact && (
          <Text color="cyan" bold>
            {" "}
            (This week)
          </Text>
        )}
      </Box>

      {data && (
        <Box flexDirection="column">
          <Box flexDirection="row">
            <Box width={timeColumnWidth} paddingLeft={1} paddingRight={1}>
              <Text bold dimColor>
                Time
              </Text>
            </Box>

            {data.days.map((day, idx) => (
              <Box
                key={`header-day-${idx}`}
                width={dayColumnWidth}
                paddingLeft={1}
                paddingRight={1}
              >
                <Text bold color={idx === todayIdx ? "cyan" : "white"}>
                  {idx === 0 ? "" : "| "}
                  {compact ? day.dayName.slice(0, 2) : day.dayName.slice(0, 3)}
                </Text>
              </Box>
            ))}
          </Box>
          <Box paddingX={1}>
            <Text dimColor>{dividerLine}</Text>
          </Box>
        </Box>
      )}

      {loading ? (
        <Box justifyContent="center" marginTop={1} alignItems="center">
          <Text color="yellow">
            <Spinner type="dots" /> Loading timetable...
          </Text>
        </Box>
      ) : error ? (
        <Box justifyContent="center">
          <Text color="red">Error: {error} (press r to retry)</Text>
        </Box>
      ) : data ? (
        <Box flexDirection="column">
          {scrollOffset > 0 && (
            <Box justifyContent="center" height={1}>
              <Text dimColor>^ ({scrollOffset} more periods) ^</Text>
            </Box>
          )}

          {visiblePeriods.map((period, idx) => {
            const actualIndex = idx + scrollOffset;
            return (
              <GridRow
                key={`period-${actualIndex}-${period.startTime}`}
                period={period}
                periodIdx={actualIndex}
                dayLessonIndex={dayLessonIndex}
                colorMap={colorMap}
                todayIdx={todayIdx}
                selectedDayIdx={selectedDayIdx}
                selectedPeriodIdx={selectedPeriodIdx}
                selectedLessonIdx={selectedLessonIdx}
                selectedRange={selectedRange}
                currentPeriodIdx={currentPeriodIdx}
                compact={compact}
                timeColumnWidth={timeColumnWidth}
                dayColumnWidth={dayColumnWidth}
              />
            );
          })}

          {scrollOffset + rowsPerPage < data.timegrid.length && (
            <Box justifyContent="center" height={1}>
              <Text dimColor>
                v ({data.timegrid.length - (scrollOffset + rowsPerPage)} more periods) v
              </Text>
            </Box>
          )}

          <Box justifyContent="center">
            <Text dimColor>
              Focus: {selectedDayName} / {selectedPeriodName}
              {selectedLessonCount > 1
                ? ` (${selectedLessonIdx + 1}/${selectedLessonCount})`
                : ""}
            </Text>
          </Box>
        </Box>
      ) : null}

      <Box marginTop={0} paddingX={1} flexDirection="column" minHeight={4}>
        <Box paddingX={1}>
          <Text dimColor>{dividerLine}</Text>
        </Box>
        {selectedLesson ? (
          <Box flexDirection="column" paddingX={1}>
            <Box justifyContent="space-between">
              <Text bold color="cyan">
                {truncateText(
                  `${selectedLesson.subjectLongName} (${selectedLesson.subject})`,
                  Math.max(10, termWidth - 24),
                )}
              </Text>
              <Text color="yellow">
                {selectedRangeTime || `${selectedLesson.startTime} - ${selectedLesson.endTime}`}
              </Text>
            </Box>

            <Box>
              <Text dimColor>Teacher: </Text>
              <Text>
                {truncateText(
                  selectedLesson.teacherLongName ||
                    selectedLesson.teacher ||
                    "N/A",
                  Math.max(10, termWidth - 24),
                )}
              </Text>
              <Text dimColor> Room: </Text>
              <Text>{truncateText(selectedLesson.room || "N/A", 10)}</Text>
            </Box>

            <Box height={2}>
              {selectedLesson.remarks ? (
                <Text color="magenta" italic>
                  i {truncateText(selectedLesson.remarks, Math.max(10, termWidth - 8))}
                </Text>
              ) : selectedLesson.cancelled ? (
                <Text color="red" bold>
                  CANCELLED
                </Text>
              ) : null}
            </Box>
          </Box>
        ) : (
          <Box justifyContent="center" alignItems="center" flexGrow={1} paddingX={1}>
            <Text dimColor>Select a lesson to see details</Text>
          </Box>
        )}
      </Box>

      {showHelp && (
        <Box justifyContent="center">
          <Text dimColor>
            l logout | tab cycle overlapping lessons | h hide help
          </Text>
        </Box>
      )}

      <Box justifyContent="center">
        <Text dimColor>{footerText}</Text>
      </Box>
    </Box>
  );
}
