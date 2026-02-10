import React from "react";
import { Box, Text } from "ink";
import type { ParsedLesson } from "../../utils/untis.ts";
import { truncateText } from "./text.ts";

interface TimetableDetailsProps {
  bottomDividerLine: string;
  selectedLesson: ParsedLesson | null;
  selectedRangeTime: string | null;
  termWidth: number;
}

export default function TimetableDetails({
  bottomDividerLine,
  selectedLesson,
  selectedRangeTime,
  termWidth,
}: TimetableDetailsProps) {
  return (
    <Box marginTop={0} paddingX={0} flexDirection="column" minHeight={4}>
      <Box>
        <Text dimColor>{bottomDividerLine}</Text>
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
            <Text dimColor>Teacher · </Text>
            <Text>
              {truncateText(
                selectedLesson.teacherLongName || selectedLesson.teacher || "N/A",
                Math.max(10, termWidth - 24),
              )}
            </Text>
            <Text dimColor>  Room · </Text>
            <Text>{truncateText(selectedLesson.room || "N/A", 10)}</Text>
          </Box>

          <Box height={2}>
            {selectedLesson.remarks ? (
              <Text color="magenta" italic>
                ℹ {truncateText(selectedLesson.remarks, Math.max(10, termWidth - 8))}
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
  );
}
