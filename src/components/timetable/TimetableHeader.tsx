import React from "react";
import { Box, Text } from "ink";
import type { Config } from "../../utils/config.ts";
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
}: TimetableHeaderProps) {
  return (
    <>
      <Box justifyContent="space-between">
        <Text bold color="cyan">
          {compact ? "WebUntis" : "WebUntis TUI"}
        </Text>

        <Box>
          {!compact && (
            <Text dimColor>
              {truncateText(
                `${config.username}@${config.school}`,
                Math.max(10, termWidth - 22),
              )}
            </Text>
          )}

          {isFromCache && !loading && (
            <Text color="yellow" dimColor>
              {" "}
              (cached)
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
          <Text color="cyan" bold>
            {"  • This week"}
          </Text>
        )}
      </Box>
    </>
  );
}
