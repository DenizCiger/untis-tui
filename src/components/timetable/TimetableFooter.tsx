import React from "react";
import { Box, Text } from "ink";

interface TimetableFooterProps {
  showHelp: boolean;
  footerText: string;
}

export default function TimetableFooter({
  showHelp,
  footerText,
}: TimetableFooterProps) {
  return (
    <>
      {showHelp && (
        <Box justifyContent="center">
          <Text dimColor>l logout | tab cycle overlapping lessons | h hide help</Text>
        </Box>
      )}

      <Box justifyContent="center">
        <Text dimColor>{footerText}</Text>
      </Box>
    </>
  );
}
