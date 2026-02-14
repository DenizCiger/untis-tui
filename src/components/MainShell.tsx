import React, { useMemo, useState } from "react";
import { Box, Text, useInput, useStdout } from "ink";
import type { Config } from "../utils/config.ts";
import { COLORS } from "./colors.ts";
import Absences from "./Absences.tsx";
import Timetable from "./Timetable.tsx";

type TabId = "timetable" | "absences";

interface MainShellProps {
  config: Config;
  onLogout: () => void;
}

const TAB_ORDER: TabId[] = ["timetable", "absences"];
const INACTIVE_TAB_BACKGROUND = "ansi256(238)";

function TabButton({
  label,
  shortcut,
  active,
}: {
  label: string;
  shortcut: string;
  active: boolean;
}) {
  return (
    <Text
      color={active ? COLORS.neutral.black : COLORS.neutral.white}
      backgroundColor={active ? COLORS.brand : INACTIVE_TAB_BACKGROUND}
      bold={active}
    >
      {` ${shortcut}:${label} `}
    </Text>
  );
}

export default function MainShell({ config, onLogout }: MainShellProps) {
  const { stdout } = useStdout();
  const [activeTab, setActiveTab] = useState<TabId>("timetable");

  useInput(
    (input) => {
      if (input === "[") {
        setActiveTab((prev) => {
          const currentIndex = TAB_ORDER.indexOf(prev);
          const nextIndex = (currentIndex - 1 + TAB_ORDER.length) % TAB_ORDER.length;
          return TAB_ORDER[nextIndex] ?? TAB_ORDER[0]!;
        });
        return;
      }

      if (input === "]") {
        setActiveTab((prev) => {
          const currentIndex = TAB_ORDER.indexOf(prev);
          const nextIndex = (currentIndex + 1) % TAB_ORDER.length;
          return TAB_ORDER[nextIndex] ?? TAB_ORDER[0]!;
        });
        return;
      }

      const numericIndex = Number.parseInt(input, 10);
      if (!Number.isNaN(numericIndex) && numericIndex >= 1 && numericIndex <= TAB_ORDER.length) {
        setActiveTab(TAB_ORDER[numericIndex - 1] ?? TAB_ORDER[0]!);
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const termWidth = Math.max(50, stdout?.columns ?? 120);

  const tabs = useMemo(
    () => [
      { id: "timetable" as const, label: "Timetable", shortcut: "1" },
      { id: "absences" as const, label: "Absences", shortcut: "2" },
    ],
    [],
  );

  return (
    <Box flexDirection="column" width={termWidth}>
      <Box paddingX={1} justifyContent="space-between">
        <Text dimColor>tui-untis</Text>
        <Text dimColor>{"[ ] or 1-2 switch tabs"}</Text>
      </Box>

      <Box paddingX={1}>
        <Text dimColor>{"tabs "}</Text>

        {tabs.map((tab, index) => {
          const active = tab.id === activeTab;

          return (
            <Box key={tab.id} marginRight={index < tabs.length - 1 ? 1 : 0}>
              <TabButton
                label={tab.label}
                shortcut={tab.shortcut}
                active={active}
              />
            </Box>
          );
        })}
      </Box>

      {activeTab === "timetable" ? (
        <Timetable config={config} onLogout={onLogout} topInset={2} />
      ) : (
        <Absences config={config} onLogout={onLogout} topInset={2} />
      )}
    </Box>
  );
}
