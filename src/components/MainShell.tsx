import React, { useMemo, useState } from "react";
import { Box, Text, useInput, useStdout } from "ink";
import type { Config } from "../utils/config.ts";
import { COLORS } from "./colors.ts";
import Absences from "./Absences.tsx";
import { InputCaptureProvider } from "./inputCapture.tsx";
import SettingsModal from "./SettingsModal.tsx";
import Timetable from "./Timetable.tsx";
import { isShortcutPressed, type TabId } from "./shortcuts.ts";
import { truncateText } from "./timetable/text.ts";

interface MainShellProps {
  config: Config;
  onLogout: () => void;
}

const TAB_ORDER: TabId[] = ["timetable", "absences"];
const INACTIVE_TAB_BACKGROUND = "ansi256(238)";

function TabButton({ label, active }: { label: string; active: boolean }) {
  return (
    <Text
      color={active ? COLORS.neutral.black : COLORS.neutral.white}
      backgroundColor={active ? COLORS.brand : INACTIVE_TAB_BACKGROUND}
      bold={active}
    >
      {` ${label} `}
    </Text>
  );
}

export default function MainShell({ config, onLogout }: MainShellProps) {
  const { stdout } = useStdout();
  const [activeTab, setActiveTab] = useState<TabId>("timetable");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [globalShortcutsBlocked, setGlobalShortcutsBlocked] = useState(false);
  const [timetableTargetLabel, setTimetableTargetLabel] = useState("My timetable");

  useInput(
    (input, key) => {
      if (settingsOpen) {
        if (isShortcutPressed("settings-close", input, key)) {
          setSettingsOpen(false);
        }
        return;
      }

      if (globalShortcutsBlocked) {
        return;
      }

      if (isShortcutPressed("settings-open", input, key)) {
        setSettingsOpen(true);
        return;
      }

      if (isShortcutPressed("tab-prev", input, key)) {
        setActiveTab((prev) => {
          const currentIndex = TAB_ORDER.indexOf(prev);
          const nextIndex =
            (currentIndex - 1 + TAB_ORDER.length) % TAB_ORDER.length;
          return TAB_ORDER[nextIndex] ?? TAB_ORDER[0]!;
        });
        return;
      }

      if (isShortcutPressed("tab-next", input, key)) {
        setActiveTab((prev) => {
          const currentIndex = TAB_ORDER.indexOf(prev);
          const nextIndex = (currentIndex + 1) % TAB_ORDER.length;
          return TAB_ORDER[nextIndex] ?? TAB_ORDER[0]!;
        });
        return;
      }

      if (isShortcutPressed("tab-timetable", input, key)) {
        setActiveTab("timetable");
        return;
      }

      if (isShortcutPressed("tab-absences", input, key)) {
        setActiveTab("absences");
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const termWidth = Math.max(50, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - 2);

  const tabs = useMemo(
    () => [
      { id: "timetable" as const, label: "Timetable" },
      { id: "absences" as const, label: "Absences" },
    ],
    [],
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight + 2}>
      <Box justifyContent="space-between">
        <Box flexDirection="row" minWidth={20}>
          {tabs.map((tab, index) => {
            const active = tab.id === activeTab;

            return (
              <Box key={tab.id} marginRight={0}>
                <TabButton label={tab.label} active={active} />
              </Box>
            );
          })}
        </Box>
        <Box flexGrow={1} justifyContent="center">
          {activeTab === "timetable" ? (
            <Text dimColor>
              {truncateText(timetableTargetLabel, Math.max(12, termWidth - 34))}
            </Text>
          ) : null}
        </Box>
        <Box minWidth={8} justifyContent="flex-end">
          <Text color={COLORS.neutral.white} bold={settingsOpen}>
            {settingsOpen ? "Settings" : ""}
          </Text>
        </Box>
      </Box>

      <InputCaptureProvider onBlockedChange={setGlobalShortcutsBlocked}>
        {activeTab === "timetable" ? (
          <Timetable
            config={config}
            onLogout={onLogout}
            topInset={2}
            inputEnabled={!settingsOpen}
            onTargetLabelChange={setTimetableTargetLabel}
          />
        ) : (
          <Absences
            config={config}
            onLogout={onLogout}
            topInset={2}
            inputEnabled={!settingsOpen}
          />
        )}
      </InputCaptureProvider>

      {settingsOpen && (
        <Box position="absolute" width={termWidth} height={termHeight + 2}>
          <SettingsModal
            activeTab={activeTab}
            width={termWidth}
            height={termHeight + 2}
          />
        </Box>
      )}
    </Box>
  );
}
