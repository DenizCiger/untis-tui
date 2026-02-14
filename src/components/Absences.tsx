import React, { useEffect, useMemo, useState } from "react";
import { Box, Text, useApp, useInput, useStdout } from "ink";
import Spinner from "ink-spinner";
import TextInput from "ink-text-input";
import type { Config } from "../utils/config.ts";
import { formatDate, type ParsedAbsence } from "../utils/untis.ts";
import { COLORS } from "./colors.ts";
import { fitText, truncateText } from "./timetable/text.ts";
import { useAbsencesData } from "./absences/useAbsencesData.ts";

interface AbsencesProps {
  config: Config;
  onLogout: () => void;
  topInset?: number;
}

const STATUS_OPTIONS = [
  { value: "all", label: "All" },
  { value: "excused", label: "Excused" },
  { value: "unexcused", label: "Unexcused" },
] as const;

const WINDOW_OPTIONS = [
  { value: "all", label: "All time", days: null },
  { value: "30d", label: "30 days", days: 30 },
  { value: "90d", label: "90 days", days: 90 },
  { value: "180d", label: "180 days", days: 180 },
  { value: "365d", label: "365 days", days: 365 },
] as const;

type StatusFilter = (typeof STATUS_OPTIONS)[number]["value"];
type WindowFilter = (typeof WINDOW_OPTIONS)[number]["value"];

const SELECTED_ROW_BACKGROUND = "ansi256(24)";
const ALTERNATE_ROW_BACKGROUND = "ansi256(236)";
const CHIP_ACTIVE_BACKGROUND = "ansi256(24)";
const HISTORY_LABEL_BACKGROUND = "ansi256(238)";

function nextValue<T extends string>(values: readonly T[], current: T): T {
  const currentIndex = values.indexOf(current);
  const nextIndex = (currentIndex + 1) % values.length;
  return values[nextIndex] ?? values[0]!;
}

function getWindowDays(value: WindowFilter): number | null {
  return WINDOW_OPTIONS.find((option) => option.value === value)?.days ?? null;
}

function getWindowLabel(value: WindowFilter): string {
  return WINDOW_OPTIONS.find((option) => option.value === value)?.label ?? "All time";
}

function getStatusLabel(value: StatusFilter): string {
  return STATUS_OPTIONS.find((option) => option.value === value)?.label ?? "All";
}

function getStatusMeta(absence: ParsedAbsence): {
  shortLabel: string;
  chipLabel: string;
  longLabel: string;
  chipBackgroundColor: string;
  chipTextColor: string;
} {
  if (absence.isExcused) {
    return {
      shortLabel: "EX",
      chipLabel: "EXCUSED",
      longLabel: "Excused",
      chipBackgroundColor: "ansi256(35)",
      chipTextColor: COLORS.neutral.white,
    };
  }

  return {
    shortLabel: "UN",
    chipLabel: "UNEXCUSED",
    longLabel: "Unexcused",
    chipBackgroundColor: "ansi256(167)",
    chipTextColor: COLORS.neutral.white,
  };
}

function formatAbsenceRange(absence: ParsedAbsence): string {
  const sameDay = absence.startDate.toDateString() === absence.endDate.toDateString();
  if (sameDay) {
    return `${formatDate(absence.startDate)} ${absence.startTime}-${absence.endTime}`;
  }

  return `${formatDate(absence.startDate)} ${absence.startTime} -> ${formatDate(absence.endDate)} ${absence.endTime}`;
}

function formatAbsenceRangeCompact(absence: ParsedAbsence): string {
  const day = absence.startDate.getDate().toString().padStart(2, "0");
  const month = (absence.startDate.getMonth() + 1).toString().padStart(2, "0");
  const endDay = absence.endDate.getDate().toString().padStart(2, "0");
  const endMonth = (absence.endDate.getMonth() + 1).toString().padStart(2, "0");
  const sameDay = absence.startDate.toDateString() === absence.endDate.toDateString();

  if (sameDay) {
    return `${day}.${month}`;
  }

  return `${day}.${month}->${endDay}.${endMonth}`;
}

function toSingleLine(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function FilterChip({
  hotkey,
  label,
  active,
}: {
  hotkey: string;
  label: string;
  active: boolean;
}) {
  return (
    <Text
      color={active ? COLORS.neutral.white : COLORS.neutral.gray}
      backgroundColor={active ? CHIP_ACTIVE_BACKGROUND : undefined}
    >
      {` ${hotkey} ${label} `}
    </Text>
  );
}

export default function Absences({ config, onLogout, topInset = 0 }: AbsencesProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();

  const {
    absences,
    loadingInitial,
    loadingMore,
    error,
    hasMore,
    daysLoaded,
    chunkDays,
    loadMore,
    refresh,
  } = useAbsencesData(config);

  const [selectedIdx, setSelectedIdx] = useState(0);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [windowFilter, setWindowFilter] = useState<WindowFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [searchDraft, setSearchDraft] = useState("");
  const [searchMode, setSearchMode] = useState(false);

  const termWidth = Math.max(70, stdout?.columns ?? 120);
  const termHeight = Math.max(20, (stdout?.rows ?? 24) - topInset);
  const splitPane = termWidth >= 118;
  const compactHeader = termWidth < 96;

  const cutoffDate = useMemo(() => {
    const days = getWindowDays(windowFilter);
    if (!days) return null;

    const cutoff = new Date();
    cutoff.setHours(0, 0, 0, 0);
    cutoff.setDate(cutoff.getDate() - days + 1);
    return cutoff;
  }, [windowFilter]);

  const normalizedSearch = searchQuery.trim().toLowerCase();

  const filteredAbsences = useMemo(() => {
    return absences.filter((absence) => {
      if (statusFilter === "excused" && !absence.isExcused) return false;
      if (statusFilter === "unexcused" && absence.isExcused) return false;
      if (cutoffDate && absence.endDate < cutoffDate) return false;

      if (!normalizedSearch) return true;

      const haystack = [
        absence.studentName,
        absence.reason,
        absence.text,
        absence.excuseStatus,
        formatDate(absence.startDate),
        formatDate(absence.endDate),
      ]
        .join(" ")
        .toLowerCase();

      return haystack.includes(normalizedSearch);
    });
  }, [absences, cutoffDate, normalizedSearch, statusFilter]);

  const selectedAbsence = filteredAbsences[selectedIdx] ?? null;
  const selectedStatusMeta = selectedAbsence ? getStatusMeta(selectedAbsence) : null;
  const filteredExcusedCount = filteredAbsences.filter((absence) => absence.isExcused).length;
  const filteredUnexcusedCount = filteredAbsences.length - filteredExcusedCount;
  const newestLoaded = absences[0] ? formatDate(absences[0].startDate) : "-";
  const oldestLoaded = absences[absences.length - 1]
    ? formatDate(absences[absences.length - 1]!.startDate)
    : "-";

  useEffect(() => {
    setSelectedIdx((previous) => {
      const maxIndex = Math.max(filteredAbsences.length - 1, 0);
      return Math.min(previous, maxIndex);
    });
  }, [filteredAbsences.length]);

  useEffect(() => {
    if (loadingInitial || loadingMore || !hasMore) return;
    if (filteredAbsences.length <= 1) return;

    const nearBottom = selectedIdx >= filteredAbsences.length - 2;
    if (nearBottom) {
      loadMore();
    }
  }, [
    filteredAbsences.length,
    hasMore,
    loadMore,
    loadingInitial,
    loadingMore,
    selectedIdx,
  ]);

  useInput(
    (input, key) => {
      if (searchMode) {
        if (key.escape) {
          setSearchDraft(searchQuery);
          setSearchMode(false);
        }

        return;
      }

      if (input === "q") {
        exit();
        return;
      }

      if (input === "l") {
        onLogout();
        return;
      }

      if (input === "r") {
        setSelectedIdx(0);
        refresh();
        return;
      }

      if (input === "m") {
        if (hasMore && !loadingInitial && !loadingMore) {
          loadMore();
        }
        return;
      }

      if (input === "f") {
        setStatusFilter((current) =>
          nextValue(
            STATUS_OPTIONS.map((option) => option.value),
            current,
          ),
        );
        setSelectedIdx(0);
        return;
      }

      if (input === "w") {
        setWindowFilter((current) =>
          nextValue(
            WINDOW_OPTIONS.map((option) => option.value),
            current,
          ),
        );
        setSelectedIdx(0);
        return;
      }

      if (input === "c") {
        setStatusFilter("all");
        setWindowFilter("all");
        setSearchQuery("");
        setSearchDraft("");
        setSelectedIdx(0);
        return;
      }

      if (input === "/") {
        setSearchDraft(searchQuery);
        setSearchMode(true);
        return;
      }

      if (key.upArrow || input === "k") {
        setSelectedIdx((previous) => Math.max(0, previous - 1));
        return;
      }

      if (key.downArrow || input === "j") {
        setSelectedIdx((previous) => {
          const maxIndex = Math.max(filteredAbsences.length - 1, 0);
          const next = Math.min(maxIndex, previous + 1);

          if (next >= maxIndex && hasMore && !loadingInitial && !loadingMore) {
            loadMore();
          }

          return next;
        });
        return;
      }

      const pageJump = Math.max(4, Math.floor((termHeight - 11) / 2));

      if (key.pageUp) {
        setSelectedIdx((previous) => Math.max(0, previous - pageJump));
        return;
      }

      if (key.pageDown) {
        setSelectedIdx((previous) => {
          const maxIndex = Math.max(filteredAbsences.length - 1, 0);
          const next = Math.min(maxIndex, previous + pageJump);

          if (next >= maxIndex && hasMore && !loadingInitial && !loadingMore) {
            loadMore();
          }

          return next;
        });
        return;
      }

      if (key.home) {
        setSelectedIdx(0);
        return;
      }

      if (key.end) {
        setSelectedIdx(Math.max(filteredAbsences.length - 1, 0));

        if (hasMore && !loadingInitial && !loadingMore) {
          loadMore();
        }
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const footerRows = termWidth >= 115 ? 1 : 2;
  const headerRows = searchMode ? 5 : 4;
  const syncRows = error && absences.length > 0 ? 1 : 0;
  const bodyHeight = Math.max(8, termHeight - headerRows - footerRows - syncRows);

  const listPaneWidth = splitPane ? Math.max(56, Math.floor(termWidth * 0.6)) : termWidth;
  const detailPaneWidth = splitPane
    ? Math.max(28, termWidth - listPaneWidth - 1)
    : termWidth;

  const stackedListHeight = splitPane ? bodyHeight : Math.max(7, Math.floor(bodyHeight * 0.56));
  const stackedDetailHeight = splitPane
    ? bodyHeight
    : Math.max(5, bodyHeight - stackedListHeight - 1);
  const rightPaneHeight = splitPane ? bodyHeight : stackedDetailHeight;
  const summaryPaneHeight = Math.max(4, Math.min(5, rightPaneHeight - 5));
  const detailsPaneHeight = rightPaneHeight - summaryPaneHeight;

  const listHeight = splitPane ? bodyHeight : stackedListHeight;
  const listRows = Math.max(3, listHeight - 5);
  const visibleStart = Math.min(
    Math.max(0, selectedIdx - Math.floor(listRows / 2)),
    Math.max(0, filteredAbsences.length - listRows),
  );
  const visibleAbsences = filteredAbsences.slice(visibleStart, visibleStart + listRows);

  const rowContentWidth = Math.max(30, listPaneWidth - 4);
  const statusChipCompact = rowContentWidth < 56;
  const statusChipInnerWidth = statusChipCompact ? 2 : 9;
  const statusChipWidth = statusChipInnerWidth + 2;

  let dateColWidth = splitPane ? 12 : 12;
  let noteColWidth = rowContentWidth - dateColWidth - statusChipWidth - 4;

  if (noteColWidth < 10) {
    const deficit = 10 - noteColWidth;
    dateColWidth = Math.max(12, dateColWidth - deficit);
    noteColWidth = rowContentWidth - dateColWidth - statusChipWidth - 4;
  }
  const listSummary =
    filteredAbsences.length === absences.length
      ? `Showing ${absences.length}`
      : `Showing ${filteredAbsences.length} of ${absences.length}`;

  const primaryControls =
    "[Up/Down j/k] Move [PgUp/PgDn] Jump [Home/End] [f] Status [w] Window [/] Search";
  const secondaryControls =
    "[m] Load older [c] Clear filters [r] Refresh [l] Logout [q] Quit";

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight} paddingX={1}>
      <Box justifyContent="space-between">
        <Text bold color={COLORS.brand}>
          Absence Timeline
        </Text>
        <Text dimColor>{truncateText(`${config.username}@${config.school}`, 40)}</Text>
      </Box>

      <Box justifyContent="space-between">
        <Text dimColor>{`Newest first | ${listSummary}`}</Text>
        <Text dimColor>
          {`${daysLoaded} days loaded${hasMore ? ` | +${chunkDays}d chunks` : " | fully loaded"}`}
        </Text>
      </Box>

      <Box>
        {compactHeader ? (
          <Text dimColor>
            {`[f:${getStatusLabel(statusFilter)}] [w:${getWindowLabel(windowFilter)}] [/:${truncateText(searchQuery || "none", 12)}] [c]`}
          </Text>
        ) : (
          <>
            <FilterChip
              hotkey="f"
              label={`Status: ${getStatusLabel(statusFilter)}`}
              active={statusFilter !== "all"}
            />
            <Text> </Text>
            <FilterChip
              hotkey="w"
              label={`Window: ${getWindowLabel(windowFilter)}`}
              active={windowFilter !== "all"}
            />
            <Text> </Text>
            <FilterChip
              hotkey="/"
              label={`Search: ${truncateText(searchQuery || "none", 18)}`}
              active={Boolean(searchQuery)}
            />
            <Text> </Text>
            <FilterChip hotkey="c" label="Clear" active={false} />
          </>
        )}
      </Box>

      <Box minHeight={1}>
        {searchMode ? (
          <>
            <Text color={COLORS.brand}>Search: </Text>
            <TextInput
              value={searchDraft}
              onChange={setSearchDraft}
              onSubmit={() => {
                setSearchQuery(searchDraft.trim());
                setSearchMode(false);
                setSelectedIdx(0);
              }}
              placeholder="reason, note, date, status"
              focus
            />
            <Text dimColor> (enter apply, esc cancel)</Text>
          </>
        ) : (
          <Text dimColor>
            {hasMore
              ? "Auto-load triggers near bottom. Press m to fetch older records now."
              : "Reached oldest available records in loaded history."}
          </Text>
        )}
      </Box>

      <Box flexDirection={splitPane ? "row" : "column"} height={bodyHeight} marginTop={1}>
        <Box
          flexDirection="column"
          width={listPaneWidth}
          height={splitPane ? bodyHeight : stackedListHeight}
          borderStyle="single"
          borderColor={COLORS.neutral.brightBlack}
        >
          <Box justifyContent="space-between" paddingX={1} height={1} flexShrink={0}>
            <Text bold>History</Text>
            <Text dimColor>
              {filteredAbsences.length > 0
                ? `${selectedIdx + 1}/${filteredAbsences.length}`
                : "0/0"}
            </Text>
          </Box>

          {loadingInitial ? (
            <Box justifyContent="center" flexGrow={1} alignItems="center">
              <Text color={COLORS.warning}>
                <Spinner type="dots" /> Loading absences...
              </Text>
            </Box>
          ) : error && absences.length === 0 ? (
            <Box justifyContent="center" flexGrow={1} alignItems="center" paddingX={1}>
              <Text color={COLORS.error}>{truncateText(`Error: ${error}`, Math.max(12, rowContentWidth))}</Text>
            </Box>
          ) : filteredAbsences.length === 0 ? (
            <Box justifyContent="center" flexGrow={1} alignItems="center" paddingX={1}>
              <Text color={COLORS.warning}>No absences match current filters.</Text>
            </Box>
          ) : (
            <Box flexDirection="column" paddingX={1} flexGrow={0} flexShrink={0}>
              <Box height={1}>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {"  "}
                </Text>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {fitText("When", dateColWidth)}
                </Text>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {" "}
                </Text>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {fitText("Notes", noteColWidth)}
                </Text>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {" "}
                </Text>
                <Text color={COLORS.neutral.gray} backgroundColor={HISTORY_LABEL_BACKGROUND}>
                  {fitText("State", statusChipWidth)}
                </Text>
              </Box>

              {visibleAbsences.map((absence, index) => {
                const actualIndex = visibleStart + index;
                const isSelected = actualIndex === selectedIdx;
                const status = getStatusMeta(absence);
                const reason = toSingleLine(
                  absence.text || absence.reason || "No reason",
                );
                const range = formatAbsenceRangeCompact(absence);
                const chipLabel = statusChipCompact
                  ? status.shortLabel
                  : status.chipLabel;
                const chipText = fitText(chipLabel, statusChipInnerWidth);
                const rowBackgroundColor = isSelected
                  ? SELECTED_ROW_BACKGROUND
                  : actualIndex % 2 === 1
                    ? ALTERNATE_ROW_BACKGROUND
                    : undefined;

                return (
                  <Box key={absence.id} height={1}>
                    <Text
                      color={isSelected ? COLORS.brand : COLORS.neutral.brightBlack}
                      backgroundColor={rowBackgroundColor}
                      bold={isSelected}
                    >
                      {isSelected ? "> " : "  "}
                    </Text>

                    <Text
                      color={isSelected ? COLORS.neutral.white : COLORS.neutral.gray}
                      backgroundColor={rowBackgroundColor}
                    >
                      {fitText(range, dateColWidth)}
                    </Text>

                    <Text
                      backgroundColor={rowBackgroundColor}
                    >
                      {" "}
                    </Text>

                    <Text
                      color={isSelected ? COLORS.neutral.white : COLORS.neutral.white}
                      backgroundColor={rowBackgroundColor}
                      bold={isSelected}
                    >
                      {fitText(reason, noteColWidth)}
                    </Text>

                    <Text
                      backgroundColor={rowBackgroundColor}
                    >
                      {" "}
                    </Text>

                    <Text
                      color={status.chipTextColor}
                      backgroundColor={status.chipBackgroundColor}
                      bold
                    >
                      {` ${chipText} `}
                    </Text>
                  </Box>
                );
              })}

              {loadingMore && (
                <Box height={1}><Text dimColor>{fitText("Loading older records...", rowContentWidth)}</Text></Box>
              )}

              {!loadingMore && hasMore && (
                <Box height={1}><Text dimColor>{fitText("More records available (m)", rowContentWidth)}</Text></Box>
              )}
            </Box>
          )}
        </Box>

        {splitPane ? <Box width={1} /> : <Box height={1} />}

        <Box flexDirection="column" width={detailPaneWidth} height={rightPaneHeight}>
          <Box
            flexDirection="column"
            height={summaryPaneHeight}
            borderStyle="single"
            borderColor={COLORS.neutral.brightBlack}
          >
            <Box justifyContent="space-between" paddingX={1} height={1}>
              <Text bold>Summary</Text>
              <Text dimColor>{getWindowLabel(windowFilter)}</Text>
            </Box>
            <Box flexDirection="column" paddingX={1}>
              <Text>{`${filteredExcusedCount} excused | ${filteredUnexcusedCount} unexcused`}</Text>
              <Text dimColor>{`Loaded range: ${newestLoaded} -> ${oldestLoaded}`}</Text>
            </Box>
          </Box>

          <Box
            flexDirection="column"
            height={detailsPaneHeight}
            borderStyle="single"
            borderColor={COLORS.neutral.brightBlack}
          >
            <Box justifyContent="space-between" paddingX={1}>
              <Text bold>Details</Text>
              {selectedStatusMeta ? (
                <Text
                  backgroundColor={selectedStatusMeta.chipBackgroundColor}
                  color={selectedStatusMeta.chipTextColor}
                  bold
                >
                  {` ${selectedStatusMeta.chipLabel} `}
                </Text>
              ) : (
                <Text dimColor>No selection</Text>
              )}
            </Box>

            <Box flexDirection="column" paddingX={1} flexGrow={1}>
              {selectedAbsence ? (
                <>
                  <Text dimColor>When</Text>
                  <Text>{toSingleLine(formatAbsenceRange(selectedAbsence))}</Text>

                  <Text dimColor>Reason</Text>
                  <Text>{toSingleLine(selectedAbsence.reason || "No reason")}</Text>

                  <Text dimColor>Excuse status</Text>
                  <Text>
                    {toSingleLine(
                      selectedAbsence.excuseStatus ||
                        (selectedAbsence.isExcused ? "Marked as excused" : "Not excused"),
                    )}
                  </Text>

                  <Text dimColor>Notes</Text>
                  <Text>{toSingleLine(selectedAbsence.text || "No additional notes")}</Text>
                </>
              ) : (
                <Text dimColor>Select a record from the history list.</Text>
              )}
            </Box>
          </Box>
        </Box>
      </Box>

      {error && absences.length > 0 && (
        <Text color={COLORS.error}>{truncateText(`Sync issue: ${error}`, Math.max(16, termWidth - 2))}</Text>
      )}

      <Box justifyContent="center" marginTop={1} flexDirection="column">
        {termWidth >= 115 ? (
          <Text dimColor>{truncateText(`${primaryControls} ${secondaryControls}`, Math.max(20, termWidth - 2))}</Text>
        ) : (
          <>
            <Text dimColor>{truncateText(primaryControls, Math.max(20, termWidth - 2))}</Text>
            <Text dimColor>{truncateText(secondaryControls, Math.max(20, termWidth - 2))}</Text>
          </>
        )}
      </Box>
    </Box>
  );
}
