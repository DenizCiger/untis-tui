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
  maxRows: number;
}

function TimetableDetails({
  bottomDividerLine,
  selectedLesson,
  selectedLessonPosition,
  selectedLessonCount,
  overlappingLessons,
  termWidth,
  maxRows,
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

  const roomLabel = truncateText(
    selectedLesson?.roomLongName
      ? `${selectedLesson.room} (${selectedLesson.roomLongName})`
      : selectedLesson?.room || "N/A",
    Math.max(10, Math.floor(termWidth * 0.45)),
  );

  const teachersText = selectedLesson
    ? (() => {
        const combinedTeachers = selectedLesson.allTeachers
          .map((teacherShortName, idx) => {
            const teacherLongName =
              selectedLesson.allTeacherLongNames[idx] || "";
            return teacherLongName && teacherLongName !== teacherShortName
              ? `${teacherShortName} (${teacherLongName})`
              : teacherShortName;
          })
          .filter(Boolean);

        if (combinedTeachers.length > 0) {
          return combinedTeachers.join(", ");
        }

        return selectedLesson.teacherLongName &&
          selectedLesson.teacher &&
          selectedLesson.teacherLongName !== selectedLesson.teacher
          ? `${selectedLesson.teacher} (${selectedLesson.teacherLongName})`
          : selectedLesson.teacher || selectedLesson.teacherLongName || "N/A";
      })()
    : "N/A";

  const teachersLabel = truncateText(
    teachersText,
    Math.max(10, termWidth - 12),
  );

  const cellStateLabel = (selectedLesson?.cellState || "").trim();
  const chipReservedWidth = cellStateLabel ? cellStateLabel.length + 6 : 0;
  const cellStateChipColors = getCellStateChipColors(cellStateLabel);

  const footerMessage = selectedLesson?.remarks
    ? {
        text: `ℹ ${truncateText(selectedLesson.remarks, Math.max(10, termWidth - 8))}`,
        color: COLORS.info,
        bold: false,
        italic: true,
        dimColor: false,
      }
    : selectedLesson?.cancelled
      ? {
          text: "CANCELLED",
          color: COLORS.error,
          bold: true,
          italic: false,
          dimColor: false,
        }
      : overlappingLessons.length > 0
        ? {
            text: `Also: ${siblingLabel}`,
            color: undefined,
            bold: false,
            italic: false,
            dimColor: true,
          }
        : null;

  const maxContentRows = Math.max(0, maxRows - 1);

  return (
    <Box marginTop={0} paddingX={0} flexDirection="column">
      <Box>
        <Text dimColor>{bottomDividerLine}</Text>
      </Box>
      {maxContentRows > 0
        ? selectedLesson
          ? [
              <Box key="subject" justifyContent="space-between" paddingX={1}>
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
              </Box>,
              <Box key="teacher" paddingX={1}>
                <Text dimColor>
                  {" "}
                  Teacher{selectedLesson.allTeachers.length > 1 ? "s" : ""}{" "}
                  ·{" "}
                </Text>
                <Text>{teachersLabel}</Text>
              </Box>,
              <Box key="room" paddingX={1}>
                <Text dimColor>Room · </Text>
                <Text>{roomLabel}</Text>
                <Text dimColor> Classes · </Text>
                <Text>{classesLabel}</Text>
              </Box>,
              ...(selectedLessonCount > 1
                ? [
                    <Box key="overlap" paddingX={1}>
                      <Text color={COLORS.warning} dimColor>
                        Overlap {selectedLessonPosition}/{selectedLessonCount}
                      </Text>
                    </Box>,
                  ]
                : []),
              ...(selectedLesson.lessonText
                ? [
                    <Box key="lesson-text" paddingX={1}>
                      <Text dimColor>Lesson text · </Text>
                      <Text>
                        {truncateText(
                          selectedLesson.lessonText,
                          Math.max(10, termWidth - 16),
                        )}
                      </Text>
                    </Box>,
                  ]
                : []),
              ...(footerMessage
                ? [
                    <Box key="footer" paddingX={1}>
                      <Text
                        color={footerMessage.color}
                        bold={footerMessage.bold}
                        italic={footerMessage.italic}
                        dimColor={footerMessage.dimColor}
                      >
                        {footerMessage.text}
                      </Text>
                    </Box>,
                  ]
                : []),
            ].slice(0, maxContentRows)
          : [
              <Box key="empty" paddingX={1}>
                <Text dimColor>Select a lesson to see details</Text>
              </Box>,
            ]
        : null}
    </Box>
  );
}

export default React.memo(TimetableDetails);
