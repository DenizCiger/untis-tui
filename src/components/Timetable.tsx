import React, { useEffect, useMemo, useState } from "react";
import { Box, Text, useApp, useStdout } from "ink";
import Spinner from "ink-spinner";
import type { Config } from "../utils/config.ts";
import GridRow from "./timetable/GridRow.tsx";
import TimetableDetails from "./timetable/TimetableDetails.tsx";
import TimetableFooter from "./timetable/TimetableFooter.tsx";
import TimetableHeader from "./timetable/TimetableHeader.tsx";
import {
  EMPTY_LESSONS,
  findCurrentPeriodIndex,
  getSelectedLessonRange,
  indexLessonsByPeriod,
} from "./timetable/model.ts";
import { buildGridDivider, centerText } from "./timetable/text.ts";
import { useTimetableData } from "./timetable/useTimetableData.ts";
import { useTimetableNavigation } from "./timetable/useTimetableNavigation.ts";

interface TimetableProps {
  config: Config;
  onLogout: () => void;
}

export default function Timetable({ config, onLogout }: TimetableProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [colorMap] = useState(() => new Map<string, string>());

  const {
    weekOffset,
    setWeekOffset,
    data,
    loading,
    isFromCache,
    error,
    refreshCurrentWeek,
    currentMonday,
    currentFriday,
  } = useTimetableData(config);

  const [now, setNow] = useState(new Date());

  const termWidth = Math.max(50, stdout?.columns ?? 120);
  const termHeight = Math.max(18, stdout?.rows ?? 24);
  const compact = termWidth < 90 || termHeight < 24;
  const timeColumnWidth = compact ? 12 : 16;
  const dayColumnWidth = Math.max(
    compact ? 10 : 14,
    Math.floor((termWidth - timeColumnWidth - 2) / 5),
  );

  const gridHeight = Math.max(5, termHeight - (compact ? 9 : 8));
  const rowsPerPage = Math.max(1, Math.floor(gridHeight / 4));

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  const dayLessonIndex = useMemo(
    () => (data ? indexLessonsByPeriod(data.days, data.timegrid) : []),
    [data],
  );

  const {
    selectedDayIdx,
    selectedPeriodIdx,
    selectedLessonIdx,
    scrollOffset,
    showHelp,
    setSelectedPeriodIdx,
  } = useTimetableNavigation({
    data,
    dayLessonIndex,
    rowsPerPage,
    setWeekOffset,
    onQuit: exit,
    onLogout,
    onRefresh: refreshCurrentWeek,
  });

  const visiblePeriods = useMemo(() => {
    if (!data) return [];
    return data.timegrid.slice(scrollOffset, scrollOffset + rowsPerPage);
  }, [data, scrollOffset, rowsPerPage]);

  useEffect(() => {
    if (!data || weekOffset !== 0) return;
    const currentIdx = findCurrentPeriodIndex(data.timegrid);
    if (currentIdx !== -1) {
      setSelectedPeriodIdx(currentIdx);
    }
  }, [data, weekOffset, setSelectedPeriodIdx]);

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
      return "[←↑↓→] [Shift+←/→] [Tab] [h] [t] [r] [q]";
    }

    return "[←↑↓→ Navigate] [Shift+←/→ Week] [Tab Overlap] [h Help] [t Today] [r Refresh] [q Quit]";
  }, [compact]);

  const dividerLine = "─".repeat(Math.max(10, timeColumnWidth + dayColumnWidth * 5));
  const headerDividerLine = buildGridDivider(timeColumnWidth, dayColumnWidth, 5, "┼");
  const bottomDividerLine = buildGridDivider(timeColumnWidth, dayColumnWidth, 5, "┴");

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight} paddingX={0}>
      <TimetableHeader
        compact={compact}
        config={config}
        termWidth={termWidth}
        isFromCache={isFromCache}
        loading={loading}
        currentMonday={currentMonday}
        currentFriday={currentFriday}
        weekOffset={weekOffset}
      />

      {data && (
        <Box flexDirection="column">
          <Box flexDirection="row">
            <Box width={timeColumnWidth} paddingLeft={1} paddingRight={1}>
              <Text bold dimColor>
                Time
              </Text>
            </Box>

            {data.days.map((day, idx) => (
              <Box key={`header-day-${idx}`} width={dayColumnWidth} flexDirection="row">
                <Box width={1}>
                  <Text dimColor>│</Text>
                </Box>
                <Box width={Math.max(1, dayColumnWidth - 1)} paddingLeft={1} paddingRight={1}>
                  <Text bold color={idx === todayIdx ? "cyan" : "white"}>
                    {compact ? day.dayName.slice(0, 2) : day.dayName.slice(0, 3)}
                  </Text>
                </Box>
              </Box>
            ))}
          </Box>
          <Box>
            <Text dimColor>{headerDividerLine}</Text>
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
            <Box flexDirection="row" height={1}>
              <Box width={timeColumnWidth} />
              {data.days.map((_, idx) => (
                <Box key={`more-top-${idx}`} width={dayColumnWidth} flexDirection="row">
                  <Box width={1}>
                    <Text dimColor>│</Text>
                  </Box>
                  <Box width={Math.max(1, dayColumnWidth - 1)}>
                    <Text dimColor>
                      {idx === 2
                        ? centerText(`▲ ${scrollOffset} more ▲`, Math.max(1, dayColumnWidth - 1))
                        : " ".repeat(Math.max(1, dayColumnWidth - 1))}
                    </Text>
                  </Box>
                </Box>
              ))}
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
            <Box flexDirection="row" height={1}>
              <Box width={timeColumnWidth} />
              {data.days.map((_, idx) => (
                <Box key={`more-bottom-${idx}`} width={dayColumnWidth} flexDirection="row">
                  <Box width={1}>
                    <Text dimColor>│</Text>
                  </Box>
                  <Box width={Math.max(1, dayColumnWidth - 1)}>
                    <Text dimColor>
                      {idx === 2
                        ? centerText(
                            `▼ ${data.timegrid.length - (scrollOffset + rowsPerPage)} more ▼`,
                            Math.max(1, dayColumnWidth - 1),
                          )
                        : " ".repeat(Math.max(1, dayColumnWidth - 1))}
                    </Text>
                  </Box>
                </Box>
              ))}
            </Box>
          )}

          <Box flexDirection="row" height={1}>
            <Box width={timeColumnWidth} />
            {data.days.map((_, idx) => (
              <Box key={`focus-row-${idx}`} width={dayColumnWidth} flexDirection="row">
                <Box width={1}>
                  <Text dimColor>│</Text>
                </Box>
                <Box width={Math.max(1, dayColumnWidth - 1)}>
                  <Text dimColor>
                    {idx === 2
                      ? centerText(
                          `Focus: ${selectedDayName} / ${selectedPeriodName}${
                            selectedLessonCount > 1
                              ? ` (${selectedLessonIdx + 1}/${selectedLessonCount})`
                              : ""
                          }`,
                          Math.max(1, dayColumnWidth - 1),
                        )
                      : " ".repeat(Math.max(1, dayColumnWidth - 1))}
                  </Text>
                </Box>
              </Box>
            ))}
          </Box>
        </Box>
      ) : null}

      <TimetableDetails
        bottomDividerLine={bottomDividerLine}
        selectedLesson={selectedLesson}
        selectedRangeTime={selectedRangeTime}
        termWidth={termWidth}
      />

      <TimetableFooter showHelp={showHelp} footerText={footerText} />
    </Box>
  );
}
