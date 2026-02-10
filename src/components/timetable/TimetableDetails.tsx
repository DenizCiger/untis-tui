import React from "react";
import { Box, Text } from "ink";
import type { ParsedLesson } from "../../utils/untis.ts";
import { COLORS, getCellStateChipColors } from "../colors.ts";
import { truncateText } from "./text.ts";

interface TimetableDetailsProps {
  bottomDividerLine: string;
  selectedLesson: ParsedLesson | null;
  selectedLessonPosition: number;
  selectedLessonCount: number;
  overlappingLessons: ParsedLesson[];
  termWidth: number;
}

export default function TimetableDetails({
  bottomDividerLine,
  selectedLesson,
  selectedLessonPosition,
  selectedLessonCount,
  overlappingLessons,
  termWidth,
}: TimetableDetailsProps) {
  const siblingLabel = truncateText(
    overlappingLessons
      .map(
        (lesson) =>
          `${lesson.subject}${lesson.room ? ` ${lesson.room}` : ""}${lesson.teacher ? ` ${lesson.teacher}` : ""}`,
      )
      .join(" | "),
    Math.max(10, termWidth - 10),
  );

  const classesLabel = truncateText(
    selectedLesson?.allClasses.join(", ") || "N/A",
    Math.max(10, termWidth - 11),
  );

  const teachersLabel = truncateText(
    selectedLesson?.allTeachers.join(", ") || selectedLesson?.teacher || "N/A",
    Math.max(10, termWidth - 12),
  );

  const cellStateLabel = (selectedLesson?.cellState || "").trim();
  const chipReservedWidth = cellStateLabel ? cellStateLabel.length + 6 : 0;
  const cellStateChipColors = getCellStateChipColors(cellStateLabel);

  return (
    <Box marginTop={0} paddingX={0} flexDirection="column" minHeight={4}>
      <Box>
        <Text dimColor>{bottomDividerLine}</Text>
      </Box>
      {selectedLesson ? (
        <Box flexDirection="column" paddingX={1}>
          <Box justifyContent="space-between">
            <Box flexDirection="row">
              <Text bold color={COLORS.brand}>
                {truncateText(
                  `${selectedLesson.subjectLongName} (${selectedLesson.subject})`,
                  Math.max(10, termWidth - 24 - chipReservedWidth),
                )}
              </Text>
              {cellStateLabel ? (
                <Box flexDirection="row" marginLeft={1}>
                  <Text
                    backgroundColor={cellStateChipColors.backgroundColor}
                    color={cellStateChipColors.color}
                    bold
                  >
                    {` ${cellStateLabel} `}
                  </Text>
                </Box>
              ) : null}
            </Box>
            <Text color={COLORS.warning}>
              {selectedLesson.startTime} - {selectedLesson.endTime}
            </Text>
          </Box>

          {selectedLessonCount > 1 && (
            <Box>
              <Text color={COLORS.warning} dimColor>
                Overlap {selectedLessonPosition}/{selectedLessonCount}
              </Text>
            </Box>
          )}

          <Box>
            <Text dimColor>
              {" "}
              Teacher{selectedLesson.allTeachers.length > 1 ? "s" : ""} ·{" "}
            </Text>
            <Text>{teachersLabel}</Text>
          </Box>

          <Box>
            <Text dimColor> Room · </Text>
            <Text>
              {truncateText(
                selectedLesson.roomLongName
                  ? `${selectedLesson.room} (${selectedLesson.roomLongName})`
                  : selectedLesson.room || "N/A",
                40,
              )}
            </Text>
          </Box>

          <Box>
            <Text dimColor>Classes · </Text>
            <Text>{classesLabel}</Text>
          </Box>

          {selectedLesson.lessonText && (
            <Box>
              <Text dimColor>Lesson text · </Text>
              <Text>
                {truncateText(
                  selectedLesson.lessonText,
                  Math.max(10, termWidth - 16),
                )}
              </Text>
            </Box>
          )}

          <Box height={2}>
            {selectedLesson.remarks ? (
              <Text color={COLORS.info} italic>
                ℹ{" "}
                {truncateText(
                  selectedLesson.remarks,
                  Math.max(10, termWidth - 8),
                )}
              </Text>
            ) : selectedLesson.cancelled ? (
              <Text color={COLORS.error} bold>
                CANCELLED
              </Text>
            ) : overlappingLessons.length > 0 ? (
              <Text dimColor>Also: {siblingLabel}</Text>
            ) : null}
          </Box>
        </Box>
      ) : (
        <Box
          justifyContent="center"
          alignItems="center"
          flexGrow={1}
          paddingX={1}
        >
          <Text dimColor>Select a lesson to see details</Text>
        </Box>
      )}
    </Box>
  );
}
