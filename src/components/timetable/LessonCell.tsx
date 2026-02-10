import React, { memo } from "react";
import { Box, Text } from "ink";
import type { RenderLesson } from "./model.ts";
import { fitText } from "./text.ts";

interface LessonCellProps {
  entry: RenderLesson;
  stripeColor: string;
  isFocused: boolean;
  contentWidth: number;
}

const LessonCell = memo(function LessonCell({
  entry,
  stripeColor,
  isFocused,
  contentWidth,
}: LessonCellProps) {
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

export default LessonCell;
