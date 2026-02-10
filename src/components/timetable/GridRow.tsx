import React from "react";
import { Box, Text } from "ink";
import type { TimeUnit } from "../../utils/untis.ts";
import LessonCell from "./LessonCell.tsx";
import {
  type DayLessonIndex,
  type DayOverlayIndex,
  EMPTY_LESSONS,
  getSubjectColor,
} from "./model.ts";
import { truncateText } from "./text.ts";

interface GridRowProps {
  period: TimeUnit;
  periodIdx: number;
  dayLessonIndex: DayLessonIndex[];
  overlayIndexByDay: DayOverlayIndex[];
  colorMap: Map<string, string>;
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  currentPeriodIdx: number;
  compact: boolean;
  timeColumnWidth: number;
  dayColumnWidth: number;
}

export default function GridRow({
  period,
  periodIdx,
  dayLessonIndex,
  overlayIndexByDay,
  colorMap,
  selectedDayIdx,
  selectedPeriodIdx,
  selectedLessonIdx,
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
        const overlay = overlayIndexByDay[dayIdx]?.get(period.startTime);
        const leftEntry = overlay?.lanes[0] ?? null;
        const rightEntry = overlay?.lanes[1] ?? null;

        const isAnchorFocused = dayIdx === selectedDayIdx && periodIdx === selectedPeriodIdx;

        const hasLessons = lessonsInPeriod.length > 0;
        const selectedEntry = lessonsInPeriod[selectedLessonIdx] ?? null;
        const displayedLessons = lessonsInPeriod;

        const contentWidth = Math.max(4, dayColumnWidth - 2);
        const showOverlapPreview = lessonsInPeriod.length > 1;
        const overlapPreviewEntry = isAnchorFocused
          ? selectedEntry ?? lessonsInPeriod[0]
          : lessonsInPeriod[0];
        const overlapPreviewLabel = overlapPreviewEntry
          ? truncateText(
              `${overlapPreviewEntry.lesson.subject} +${Math.max(0, lessonsInPeriod.length - 1)}`,
              contentWidth,
            )
          : `${lessonsInPeriod.length}x`;

        const cellWidth = Math.max(1, dayColumnWidth - 1);
        const splitGapWidth = 1;
        const leftLaneWidth = Math.floor((cellWidth - splitGapWidth) / 2);
        const rightLaneWidth = cellWidth - splitGapWidth - leftLaneWidth;
        const leftContentWidth = Math.max(2, leftLaneWidth - 1);
        const rightContentWidth = Math.max(2, rightLaneWidth - 1);

        const canRenderSplit =
          !!overlay &&
          overlay.split &&
          dayColumnWidth >= 20;

        const isLeftFocused = leftEntry
          ? isAnchorFocused &&
              selectedEntry?.lessonInstanceId === leftEntry.lessonInstanceId
          : false;
        const isRightFocused = rightEntry
          ? isAnchorFocused &&
              selectedEntry?.lessonInstanceId === rightEntry.lessonInstanceId
          : false;

        const leftSuffix =
          (leftEntry && (overlay?.hiddenCount ?? 0) > 0) ||
          (!leftEntry && rightEntry && (overlay?.hiddenCount ?? 0) > 0)
            ? `+${overlay?.hiddenCount ?? 0}`
            : undefined;

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

            <Box width={cellWidth} flexDirection="row">
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
              ) : canRenderSplit ? (
                <Box width={cellWidth} flexDirection="row">
                  <Box width={leftLaneWidth}>
                    {leftEntry ? (
                      <LessonCell
                        entry={leftEntry}
                        stripeColor={getSubjectColor(leftEntry.lesson.subject, colorMap)}
                        isFocused={isLeftFocused}
                        contentWidth={leftContentWidth}
                        titleSuffix={leftSuffix}
                      />
                    ) : (
                      <>
                        <Text dimColor>{" ".repeat(leftLaneWidth)}</Text>
                        <Text dimColor>{" ".repeat(leftLaneWidth)}</Text>
                        <Text dimColor>{" ".repeat(leftLaneWidth)}</Text>
                      </>
                    )}
                  </Box>

                  <Box width={splitGapWidth} flexDirection="column">
                    <Text dimColor> </Text>
                    <Text dimColor> </Text>
                    <Text dimColor> </Text>
                  </Box>

                  <Box width={rightLaneWidth}>
                    {rightEntry ? (
                      <LessonCell
                        entry={rightEntry}
                        stripeColor={getSubjectColor(rightEntry.lesson.subject, colorMap)}
                        isFocused={isRightFocused}
                        contentWidth={rightContentWidth}
                        titleSuffix={!leftEntry ? leftSuffix : undefined}
                      />
                    ) : (
                      <>
                        <Text dimColor>{" ".repeat(rightLaneWidth)}</Text>
                        <Text dimColor>{" ".repeat(rightLaneWidth)}</Text>
                        <Text dimColor>{" ".repeat(rightLaneWidth)}</Text>
                      </>
                    )}
                  </Box>
                </Box>
              ) : showOverlapPreview ? (
                <Box flexGrow={1} justifyContent="center" alignItems="center" paddingX={1}>
                  <Text color={isAnchorFocused ? "yellow" : "white"} dimColor={!isAnchorFocused}>
                    {overlapPreviewLabel}
                  </Text>
                </Box>
              ) : (
                displayedLessons.map((entry, index) => (
                  <LessonCell
                    key={`lesson-${dayIdx}-${periodIdx}-${entry.lessonInstanceId}-${index}`}
                    entry={entry}
                    stripeColor={getSubjectColor(entry.lesson.subject, colorMap)}
                    isFocused={isAnchorFocused}
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
