import React from "react";
import { Box, Text } from "ink";
import type { Config } from "../../utils/config.ts";
import { COLORS } from "../colors.ts";
import { formatDate } from "../../utils/untis.ts";
import { truncateText } from "./text.ts";

interface TimetableHeaderProps {
  compact: boolean;
  config: Config;
  termWidth: number;
  isFromCache: boolean;
  loading: boolean;
  currentMonday: Date;
  currentFriday: Date;
  weekOffset: number;
  targetLabel: string;
}

export default function TimetableHeader({
  compact,
  config,
  termWidth,
  isFromCache,
  loading,
  currentMonday,
  currentFriday,
  weekOffset,
  targetLabel,
}: TimetableHeaderProps) {
  return (
    <>
      <Box justifyContent="space-between">
        <Text bold color={COLORS.brand}>
          {compact ? "WebUntis" : "WebUntis TUI"}
        </Text>

        <Box>
          {isFromCache && !loading && (
            <Text color={COLORS.warning} dimColor>
              (cached){" "}
            </Text>
          )}
          {!compact && (
            <Text dimColor>
              {truncateText(
                `${config.username}@${config.school}`,
                Math.max(10, termWidth - 22),
              )}
            </Text>
          )}
        </Box>
      </Box>

      <Box justifyContent="center">
        <Text dimColor>{"‹ "}</Text>
        <Text bold>
          {formatDate(currentMonday)} - {formatDate(currentFriday)}
        </Text>
        <Text dimColor>{" ›"}</Text>
        {weekOffset === 0 && !compact && (
          <Text color={COLORS.brand} bold>
            {"  • This week"}
          </Text>
        )}
      </Box>

      <Box justifyContent="center">
        <Text dimColor>{truncateText(targetLabel, Math.max(14, termWidth - 2))}</Text>
      </Box>
    </>
  );
}
