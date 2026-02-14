import React, { useMemo } from "react";
import { Box, Text } from "ink";
import { COLORS } from "./colors.ts";
import { getShortcutSections, type TabId } from "./shortcuts.ts";
import { fitText, truncateText } from "./timetable/text.ts";

interface SettingsModalProps {
  activeTab: TabId;
  width: number;
  height: number;
}

export default function SettingsModal({ activeTab, width, height }: SettingsModalProps) {
  const sections = useMemo(() => getShortcutSections(activeTab), [activeTab]);
  const modalWidth = Math.max(48, Math.min(96, width - 2));
  const modalHeight = Math.max(10, Math.min(30, height - 1));
  const keyColumnWidth = Math.max(12, Math.min(24, Math.floor(modalWidth * 0.32)));
  const actionWidth = Math.max(12, modalWidth - keyColumnWidth - 7);

  return (
    <Box flexGrow={1} justifyContent="center" alignItems="center" height={height}>
      <Box
        flexDirection="column"
        width={modalWidth}
        height={modalHeight}
        borderStyle="round"
        borderColor={COLORS.brand}
        backgroundColor={COLORS.neutral.black}
        paddingX={1}
      >
        <Box justifyContent="space-between">
          <Text bold color={COLORS.brand}>
            Settings
          </Text>
          <Text dimColor>{`Tab: ${activeTab === "timetable" ? "Timetable" : "Absences"}`}</Text>
        </Box>

        <Text dimColor>{truncateText("Keyboard shortcuts are grouped by context.", modalWidth - 4)}</Text>

        <Box flexDirection="column" marginTop={1} overflow="hidden" flexGrow={1}>
          {sections.map((section) => (
            <Box key={section.title} flexDirection="column" marginBottom={1}>
              <Text bold>{section.title}</Text>
              {section.items.map((item) => (
                <Box key={item.id}>
                  <Text color={COLORS.warning}>{fitText(item.keys, keyColumnWidth)}</Text>
                  <Text dimColor>{" - "}</Text>
                  <Text>{truncateText(item.action, actionWidth)}</Text>
                </Box>
              ))}
            </Box>
          ))}
        </Box>
      </Box>
    </Box>
  );
}
