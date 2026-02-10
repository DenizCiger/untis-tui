import React from "react";
import { Box, Text } from "ink";
import type { TimeUnit } from "../../utils/untis.ts";
import LessonCell from "./LessonCell.tsx";
import {
  type DayLessonIndex,
  EMPTY_LESSONS,
  getSubjectColor,
  type SelectedLessonRange,
} from "./model.ts";
import { truncateText } from "./text.ts";

interface GridRowProps {
  period: TimeUnit;
  periodIdx: number;
  dayLessonIndex: DayLessonIndex[];
  colorMap: Map<string, string>;
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  selectedRange: SelectedLessonRange | null;
  currentPeriodIdx: number;
  compact: boolean;
  timeColumnWidth: number;
  dayColumnWidth: number;
}

export default function GridRow({
  period,
  periodIdx,
  dayLessonIndex,
  colorMap,
  selectedDayIdx,
  selectedPeriodIdx,
  selectedLessonIdx,
  selectedRange,
  currentPeriodIdx,
  compact,
  timeColumnWidth,
  dayColumnWidth,
}: GridRowProps) {
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
        const contentWidth = Math.max(4, dayColumnWidth - 2);

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
            <Box width={1} flexDirection="column">
              <Text dimColor>│</Text>
              <Text dimColor>│</Text>
              <Text dimColor>│</Text>
            </Box>
            <Box width={Math.max(1, dayColumnWidth - 1)} flexDirection="row">
              {!hasLessons ? (
                isAnchorFocused ? (
                  <Box flexGrow={1} flexDirection="column" justifyContent="center">
                    <Text backgroundColor="white" color="black">
                      {" ".repeat(contentWidth + 1)}
                    </Text>
                    <Text backgroundColor="white" color="black">
                      {" ".repeat(contentWidth + 1)}
                    </Text>
                    <Text>{" ".repeat(contentWidth + 1)}</Text>
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
                    contentWidth={contentWidth}
                  />
                ))
              )}
            </Box>
          </Box>
        );
      })}
    </Box>
  );
}
