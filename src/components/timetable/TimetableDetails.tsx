import React from "react";
import { Box, Text } from "ink";
import type { ParsedLesson } from "../../utils/untis.ts";
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
              <Text bold color="cyan">
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
            <Text color="yellow">
              {selectedLesson.startTime} - {selectedLesson.endTime}
            </Text>
          </Box>

          {selectedLessonCount > 1 && (
            <Box>
              <Text color="yellow" dimColor>
                Overlap {selectedLessonPosition}/{selectedLessonCount}
              </Text>
            </Box>
          )}

          <Box>
            <Text dimColor>Teacher · </Text>
            <Text>
              {truncateText(
                selectedLesson.teacherLongName ||
                  selectedLesson.teacher ||
                  "N/A",
                Math.max(10, termWidth - 24),
              )}
            </Text>
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
            <Text dimColor>All teachers · </Text>
            <Text>{teachersLabel}</Text>
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
              <Text color="magenta" italic>
                ℹ{" "}
                {truncateText(
                  selectedLesson.remarks,
                  Math.max(10, termWidth - 8),
                )}
              </Text>
            ) : selectedLesson.cancelled ? (
              <Text color="red" bold>
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

function getCellStateChipColors(cellState: string): {
  backgroundColor: "red" | "yellow" | "green" | "greenBright" | "blue" | "gray";
  color: "black" | "white";
} {
  const normalized = cellState.toUpperCase();

  switch (normalized) {
    case "EXAM":
      return { backgroundColor: "yellow", color: "white" };
    case "CONFIRMED":
      return { backgroundColor: "green", color: "white" };
    case "SUBSTITUTION":
    case "ADDITIONAL":
      return { backgroundColor: "greenBright", color: "white" };
    case "CANCELLED":
      return { backgroundColor: "red", color: "white" };
    default:
      return { backgroundColor: "gray", color: "white" };
  }
}
