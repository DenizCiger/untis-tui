import React, { memo } from "react";
import { Box, Text } from "ink";
import { COLORS } from "../colors.ts";
import type { RenderLesson } from "./model.ts";
import { fitText } from "./text.ts";

interface LessonCellProps {
  entry: RenderLesson;
  stripeColor: string;
  isFocused: boolean;
  contentWidth: number;
  titleSuffix?: string;
}

const LessonCell = memo(function LessonCell({
  entry,
  stripeColor,
  isFocused,
  contentWidth,
  titleSuffix,
}: LessonCellProps) {
  const { lesson, continuation } = entry;
  const startsHere = continuation === "single" || continuation === "start";
  const continuesDown = continuation === "start" || continuation === "middle";
  const title = startsHere
    ? `${lesson.subject}${titleSuffix ? ` ${titleSuffix}` : ""}`
    : "";
  const meta = startsHere
    ? `${lesson.room || "?"}${lesson.teacher ? ` ${lesson.teacher}` : ""}`
    : "";

  const cellState = (lesson.cellState || "").trim().toUpperCase();
  const isSubstitutionLike =
    lesson.substitution ||
    cellState === "SUBSTITUTION" ||
    cellState === "ADDITIONAL" ||
    cellState === "ROOMSUBSTITUTION" ||
    cellState === "ROOMSUBSTITION";
  const isExam = cellState === "EXAM";
  const isCancelled = lesson.cancelled || cellState === "CANCELLED";

  const lessonType = isCancelled
    ? "cancelled"
    : isExam
      ? "exam"
      : isSubstitutionLike
        ? "substitution"
        : "default";
  const lessonColors = COLORS.lesson.byType[lessonType];

  const baseBg = lessonColors.background.base;
  const baseFg = lessonColors.text.title;
  const baseSubtextFg = lessonColors.text.subtext;
  const focusedBg = lessonColors.background.focused;
  const focusedFg = lessonColors.text.focusedTitle;
  const focusedSubtextFg = lessonColors.text.focusedSubtext;

  const mainBg = isFocused ? focusedBg : baseBg;
  const mainFg = isFocused ? focusedFg : baseFg;
  const subtextFg = isFocused ? focusedSubtextFg : baseSubtextFg;

  return (
    <Box
      flexGrow={1}
      flexBasis={0}
      height={3}
      flexDirection="column"
      justifyContent="center"
    >
      <Text
        backgroundColor={mainBg}
        color={mainFg}
        bold={startsHere}
        strikethrough={lesson.cancelled && startsHere}
      >
        <Text color={stripeColor}>▍</Text>
        {fitText(title, contentWidth)}
      </Text>
      <Text backgroundColor={mainBg} color={subtextFg}>
        <Text color={stripeColor}>▍</Text>
        {fitText(meta, contentWidth)}
      </Text>
      {continuesDown ? (
        <Text backgroundColor={baseBg} color={baseSubtextFg}>
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
