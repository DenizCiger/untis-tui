import React, { useDeferredValue, useEffect, useMemo, useState } from "react";
import { Box, Text, useApp, useStdout } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import TextInput from "./TextInput.tsx";
import type { Config } from "../utils/config.ts";
import GridRow from "./timetable/GridRow.tsx";
import TimetableDetails from "./timetable/TimetableDetails.tsx";
import TimetableHeader from "./timetable/TimetableHeader.tsx";
import { useInputCapture } from "./inputCapture.tsx";
import { isShortcutPressed } from "./shortcuts.ts";
import {
  buildOverlayIndex,
  EMPTY_LESSONS,
  findCurrentPeriodIndex,
  indexLessonsByPeriod,
} from "./timetable/model.ts";
import {
  formatTimetableTargetLabel,
  type TimetableTarget,
} from "../utils/untis.ts";
import {
  formatTimetableSearchTypeLabel,
  searchTimetableTargets,
} from "./timetable/search.ts";
import { buildGridDivider, centerText } from "./timetable/text.ts";
import { useTimetableData } from "./timetable/useTimetableData.ts";
import { useTimetableNavigation } from "./timetable/useTimetableNavigation.ts";
import { useStableInput } from "./useStableInput.ts";

interface TimetableProps {
  config: Config;
  onLogout: () => void;
  topInset?: number;
  inputEnabled?: boolean;
  onTargetLabelChange?: (label: string) => void;
}

const GRID_ROW_HEIGHT = 3;
const TIMETABLE_HEADER_ROWS = 2;
const DAY_HEADER_ROWS = 2;
const MAX_SCROLL_HINT_ROWS = 2;
const MIN_DETAILS_ROWS = 4;

export default function Timetable({
  config,
  onLogout,
  topInset = 0,
  inputEnabled = true,
  onTargetLabelChange,
}: TimetableProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [colorMap] = useState(() => new Map<string, string>());

  const {
    weekOffset,
    setWeekOffset,
    data,
    loading,
    isFromCache,
    error,
    refreshCurrentWeek,
    currentMonday,
    currentFriday,
    activeTarget,
    setActiveTarget,
    clearActiveTarget,
    searchIndex,
    searchIndexLoading,
    searchIndexError,
    ensureSearchIndexLoaded,
  } = useTimetableData(config);

  const [now, setNow] = useState(new Date());
  const [searchMode, setSearchMode] = useState(false);
  const [searchDraft, setSearchDraft] = useState("");
  const [searchSelectedIdx, setSearchSelectedIdx] = useState(0);
  const [searchScrollOffset, setSearchScrollOffset] = useState(0);
  const deferredSearchDraft = useDeferredValue(searchDraft);

  useInputCapture(searchMode);

  const termWidth = Math.max(50, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - topInset);
  const compact = termWidth < 90 || termHeight < 24;
  const timeColumnWidth = compact ? 12 : 16;
  const dayColumnWidth = Math.max(
    compact ? 10 : 14,
    Math.floor((termWidth - timeColumnWidth - 2) / 5),
  );

  const searchResults = useMemo(
    () => searchTimetableTargets(searchIndex, deferredSearchDraft),
    [deferredSearchDraft, searchIndex],
  );
  const searchResultsAreDeferred = searchDraft !== deferredSearchDraft;
  const targetLabel = useMemo(
    () => formatTimetableTargetLabel(activeTarget),
    [activeTarget],
  );
  const searchModalWidth = Math.max(56, Math.min(112, termWidth - 8));
  const searchModalHeight = Math.max(12, Math.min(30, termHeight - 4));
  const searchResultRows = Math.max(3, searchModalHeight - 7);
  const visibleSearchResults = useMemo(
    () => searchResults.slice(searchScrollOffset, searchScrollOffset + searchResultRows),
    [searchResults, searchScrollOffset, searchResultRows],
  );

  const reservedRows =
    TIMETABLE_HEADER_ROWS +
    DAY_HEADER_ROWS +
    MAX_SCROLL_HINT_ROWS +
    MIN_DETAILS_ROWS;
  // Keep a stable grid budget so dynamic details content cannot clip row bottoms.
  const gridHeight = Math.max(GRID_ROW_HEIGHT, termHeight - reservedRows);
  const rowsPerPage = Math.max(1, Math.floor(gridHeight / GRID_ROW_HEIGHT));

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    onTargetLabelChange?.(targetLabel);
  }, [onTargetLabelChange, targetLabel]);

  useEffect(() => {
    if (!searchMode) return;
    ensureSearchIndexLoaded();
  }, [ensureSearchIndexLoaded, searchMode]);

  useEffect(() => {
    setSearchSelectedIdx((prev) =>
      Math.min(prev, Math.max(searchResults.length - 1, 0)),
    );
  }, [searchResults.length]);

  useEffect(() => {
    const maxScroll = Math.max(searchResults.length - searchResultRows, 0);
    setSearchScrollOffset((prev) => Math.min(prev, maxScroll));
  }, [searchResultRows, searchResults.length]);

  useEffect(() => {
    if (!searchMode) return;

    if (searchSelectedIdx < searchScrollOffset) {
      setSearchScrollOffset(searchSelectedIdx);
      return;
    }

    if (searchSelectedIdx >= searchScrollOffset + searchResultRows) {
      setSearchScrollOffset(searchSelectedIdx - searchResultRows + 1);
    }
  }, [searchMode, searchResultRows, searchScrollOffset, searchSelectedIdx]);

  const applySearchSelection = () => {
    const instantResults = searchResultsAreDeferred
      ? searchTimetableTargets(searchIndex, searchDraft)
      : searchResults;
    const boundedIndex = Math.max(
      0,
      Math.min(searchSelectedIdx, Math.max(instantResults.length - 1, 0)),
    );
    const selected = instantResults[boundedIndex];
    if (!selected) {
      setSearchMode(false);
      return;
    }

    const nextTarget: TimetableTarget = {
      type: selected.type,
      id: selected.id,
      name: selected.name,
      longName: selected.longName,
    };
    setActiveTarget(nextTarget);
    setSearchMode(false);
  };

  const moveSearchSelection = (delta: number) => {
    setSearchSelectedIdx((prev) =>
      Math.max(0, Math.min(prev + delta, Math.max(searchResults.length - 1, 0))),
    );
  };

  useStableInput(
    (input, key) => {
      if (isShortcutPressed("timetable-search", input, key)) {
        setSearchDraft("");
        setSearchSelectedIdx(0);
        setSearchScrollOffset(0);
        setSearchMode(true);
        ensureSearchIndexLoaded();
        return;
      }

      if (isShortcutPressed("timetable-target-clear", input, key)) {
        clearActiveTarget();
      }
    },
    { isActive: inputEnabled && !searchMode && Boolean(process.stdin.isTTY) },
  );

  const dayLessonIndex = useMemo(
    () => (data ? indexLessonsByPeriod(data.days, data.timegrid) : []),
    [data],
  );

  const overlayIndexByDay = useMemo(() => {
    if (!data) return [];
    return dayLessonIndex.map((dayIndex) => buildOverlayIndex(dayIndex, data.timegrid, 2));
  }, [data, dayLessonIndex]);

  const {
    selectedDayIdx,
    selectedPeriodIdx,
    selectedLessonIdx,
    scrollOffset,
    setSelectedPeriodIdx,
  } = useTimetableNavigation({
    data,
    dayLessonIndex,
    overlayIndexByDay,
    rowsPerPage,
    setWeekOffset,
    onQuit: exit,
    onLogout,
    onRefresh: refreshCurrentWeek,
    inputEnabled: inputEnabled && !searchMode,
  });

  const visiblePeriods = useMemo(() => {
    if (!data) return [];
    return data.timegrid.slice(scrollOffset, scrollOffset + rowsPerPage);
  }, [data, scrollOffset, rowsPerPage]);

  useEffect(() => {
    if (!data || weekOffset !== 0) return;
    const currentIdx = findCurrentPeriodIndex(data.timegrid);
    if (currentIdx !== -1) {
      setSelectedPeriodIdx(currentIdx);
    }
  }, [data, weekOffset, setSelectedPeriodIdx]);

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const todayIdx =
    data?.days.findIndex(
      (day) => new Date(day.date).setHours(0, 0, 0, 0) === today.getTime(),
    ) ?? -1;

  const currentTime = `${now.getHours().toString().padStart(2, "0")}:${now
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;

  const currentPeriodIdx =
    data?.timegrid.findIndex(
      (period) =>
        currentTime >= period.startTime && currentTime <= period.endTime,
    ) ?? -1;

  const topHintRows = scrollOffset > 0 ? 1 : 0;
  const bottomHintRows =
    data && scrollOffset + rowsPerPage < data.timegrid.length ? 1 : 0;

  const detailsMaxRows = Math.max(
    2,
    termHeight -
      TIMETABLE_HEADER_ROWS -
      (data ? DAY_HEADER_ROWS : 0) -
      topHintRows -
      bottomHintRows -
      visiblePeriods.length * GRID_ROW_HEIGHT,
  );

  const selectedLesson = useMemo(() => {
    if (!data) return null;

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return null;

    return (day.get(period.startTime) ?? EMPTY_LESSONS)[selectedLessonIdx]?.lesson ?? null;
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx, selectedLessonIdx]);

  const selectedLessonCount = useMemo(() => {
    if (!data) return 0;

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return 0;

    return (day.get(period.startTime) ?? EMPTY_LESSONS).length;
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx]);

  const selectedEntry = useMemo(() => {
    if (!data) return null;

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return null;

    return (day.get(period.startTime) ?? EMPTY_LESSONS)[selectedLessonIdx] ?? null;
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx, selectedLessonIdx]);

  const selectedLessonPosition = useMemo(() => {
    if (!data || selectedLessonCount <= 0) return 0;

    const period = data.timegrid[selectedPeriodIdx];
    const overlay = overlayIndexByDay[selectedDayIdx]?.get(period?.startTime ?? "");
    if (overlay?.split && selectedEntry) {
      const laneIdx = overlay.lanes.findIndex(
        (entry) =>
          entry?.continuityKey === selectedEntry.continuityKey ||
          entry?.lessonInstanceId === selectedEntry.lessonInstanceId,
      );

      if (laneIdx !== -1) {
        const position =
          overlay.lanes.slice(0, laneIdx).filter((entry) => !!entry).length + 1;
        return Math.min(position, selectedLessonCount);
      }
    }

    return Math.min(selectedLessonIdx + 1, selectedLessonCount);
  }, [
    data,
    overlayIndexByDay,
    selectedDayIdx,
    selectedEntry,
    selectedLessonCount,
    selectedLessonIdx,
    selectedPeriodIdx,
  ]);

  const overlappingLessons = useMemo(() => {
    if (!data || !selectedLesson) return [];

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    if (!day || !period) return [];

    const lessons = day.get(period.startTime) ?? EMPTY_LESSONS;
    return lessons
      .filter((entry) => entry.lesson.instanceId !== selectedLesson.instanceId)
      .map((entry) => entry.lesson);
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx, selectedLesson]);

  const headerDividerLine = buildGridDivider(timeColumnWidth, dayColumnWidth, 5, "┼");
  const bottomDividerLine = buildGridDivider(timeColumnWidth, dayColumnWidth, 5, "┴");
  const searchVisibleStart = searchResults.length > 0 ? searchScrollOffset + 1 : 0;
  const searchVisibleEnd = Math.min(
    searchResults.length,
    searchScrollOffset + visibleSearchResults.length,
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight} paddingX={0}>
      <TimetableHeader
        compact={compact}
        config={config}
        termWidth={termWidth}
        isFromCache={isFromCache}
        loading={loading}
        currentMonday={currentMonday}
        currentFriday={currentFriday}
        weekOffset={weekOffset}
      />

      {data && (
        <Box flexDirection="column">
          <Box flexDirection="row">
            <Box width={timeColumnWidth} paddingLeft={1} paddingRight={1}>
              <Text bold dimColor>
                Time
              </Text>
            </Box>

            {data.days.map((day, idx) => (
              <Box key={`header-day-${idx}`} width={dayColumnWidth} flexDirection="row">
                <Box width={1}>
                  <Text dimColor>│</Text>
                </Box>
                <Box width={Math.max(1, dayColumnWidth - 1)}>
                  <Text
                    bold
                    color={idx === todayIdx ? COLORS.brand : COLORS.neutral.white}
                  >
                    {centerText(
                      compact ? day.dayName.slice(0, 2) : day.dayName.slice(0, 3),
                      Math.max(1, dayColumnWidth - 1),
                    )}
                  </Text>
                </Box>
              </Box>
            ))}
          </Box>
          <Box>
            <Text dimColor>{headerDividerLine}</Text>
          </Box>
        </Box>
      )}

      {loading ? (
        <Box justifyContent="center" marginTop={1} alignItems="center">
          <Text color={COLORS.warning}>
            <Spinner type="dots" /> Loading timetable...
          </Text>
        </Box>
      ) : error ? (
        <Box justifyContent="center">
          <Text color={COLORS.error}>Error: {error}</Text>
        </Box>
      ) : data ? (
        <Box flexDirection="column">
          {scrollOffset > 0 && (
            <Box flexDirection="row" height={1}>
              <Box width={timeColumnWidth} />
              {data.days.map((_, idx) => (
                <Box key={`more-top-${idx}`} width={dayColumnWidth} flexDirection="row">
                  <Box width={1}>
                    <Text dimColor>│</Text>
                  </Box>
                  <Box width={Math.max(1, dayColumnWidth - 1)}>
                    <Text dimColor>
                      {idx === 2
                        ? centerText(`▲ ${scrollOffset} more ▲`, Math.max(1, dayColumnWidth - 1))
                        : " ".repeat(Math.max(1, dayColumnWidth - 1))}
                    </Text>
                  </Box>
                </Box>
              ))}
            </Box>
          )}

          {visiblePeriods.map((period, idx) => {
            const actualIndex = idx + scrollOffset;
            return (
              <GridRow
                key={`period-${actualIndex}-${period.startTime}`}
                period={period}
                periodIdx={actualIndex}
                dayLessonIndex={dayLessonIndex}
                overlayIndexByDay={overlayIndexByDay}
                colorMap={colorMap}
                selectedDayIdx={selectedDayIdx}
                selectedPeriodIdx={selectedPeriodIdx}
                selectedLessonIdx={selectedLessonIdx}
                currentPeriodIdx={currentPeriodIdx}
                compact={compact}
                timeColumnWidth={timeColumnWidth}
                dayColumnWidth={dayColumnWidth}
              />
            );
          })}

          {scrollOffset + rowsPerPage < data.timegrid.length && (
            <Box flexDirection="row" height={1}>
              <Box width={timeColumnWidth} />
              {data.days.map((_, idx) => (
                <Box key={`more-bottom-${idx}`} width={dayColumnWidth} flexDirection="row">
                  <Box width={1}>
                    <Text dimColor>│</Text>
                  </Box>
                  <Box width={Math.max(1, dayColumnWidth - 1)}>
                    <Text dimColor>
                      {idx === 2
                        ? centerText(
                            `▼ ${data.timegrid.length - (scrollOffset + rowsPerPage)} more ▼`,
                            Math.max(1, dayColumnWidth - 1),
                          )
                        : " ".repeat(Math.max(1, dayColumnWidth - 1))}
                    </Text>
                  </Box>
                </Box>
              ))}
            </Box>
          )}

        </Box>
      ) : null}

      <TimetableDetails
        bottomDividerLine={bottomDividerLine}
        selectedLesson={selectedLesson}
        selectedLessonPosition={selectedLessonPosition}
        selectedLessonCount={selectedLessonCount}
        overlappingLessons={overlappingLessons}
        termWidth={termWidth}
        maxRows={detailsMaxRows}
      />

      {searchMode && (
        <Box
          position="absolute"
          width={termWidth}
          height={termHeight}
          justifyContent="center"
          alignItems="center"
        >
          <Box
            flexDirection="column"
            width={searchModalWidth}
            height={searchModalHeight}
            borderStyle="round"
            borderColor={COLORS.brand}
            backgroundColor={COLORS.neutral.black}
            paddingX={1}
          >
            <Box justifyContent="space-between">
              <Text bold color={COLORS.brand}>
                Timetable Target Search
              </Text>
              <Text dimColor>
                {searchResults.length > 0
                  ? `${Math.min(searchSelectedIdx + 1, searchResults.length)}/${searchResults.length}`
                  : "0/0"}
              </Text>
            </Box>

            <Box>
              <Text color={COLORS.brand}>{"> "}</Text>
              <TextInput
                value={searchDraft}
                onChange={(value) => {
                  setSearchDraft(value);
                  setSearchSelectedIdx(0);
                  setSearchScrollOffset(0);
                }}
                onSubmit={() => {
                  applySearchSelection();
                }}
                onKey={(input, key) => {
                  if (isShortcutPressed("timetable-search-cancel", input, key)) {
                    setSearchMode(false);
                    return true;
                  }

                  if (isShortcutPressed("timetable-search-up", input, key)) {
                    moveSearchSelection(-1);
                    return true;
                  }

                  if (isShortcutPressed("timetable-search-down", input, key)) {
                    moveSearchSelection(1);
                    return true;
                  }

                  if (key.pageUp) {
                    moveSearchSelection(-searchResultRows);
                    return true;
                  }

                  if (key.pageDown) {
                    moveSearchSelection(searchResultRows);
                    return true;
                  }

                  if (key.home) {
                    setSearchSelectedIdx(0);
                    return true;
                  }

                  if (key.end) {
                    setSearchSelectedIdx(Math.max(searchResults.length - 1, 0));
                    return true;
                  }

                  return false;
                }}
                placeholder="class, room, teacher"
                focus
              />
            </Box>

            <Box minHeight={1}>
              {searchIndexLoading ? (
                <Text color={COLORS.warning}>
                  <Spinner type="dots" /> Loading timetable targets...
                </Text>
              ) : searchIndexError ? (
                <Text color={COLORS.error}>{`Target load failed: ${searchIndexError}`}</Text>
              ) : searchResultsAreDeferred ? (
                <Text dimColor>Updating results...</Text>
              ) : (
                <Text dimColor>
                  Use ↑/↓, PgUp/PgDn, Home/End, Enter apply, Esc cancel.
                </Text>
              )}
            </Box>

            <Box flexDirection="column" flexGrow={1} overflow="hidden">
              {!searchIndexLoading && !searchIndexError && searchResults.length === 0 && (
                <Text dimColor>No targets found for this query.</Text>
              )}

              {!searchIndexLoading &&
                !searchIndexError &&
                visibleSearchResults.map((result, idx) => {
                  const absoluteIdx = searchScrollOffset + idx;
                  const selected = absoluteIdx === searchSelectedIdx;
                  return (
                    <Box key={`${result.type}:${result.id}`}>
                      <Text
                        color={selected ? COLORS.brand : COLORS.neutral.gray}
                        bold={selected}
                      >
                        {selected ? "> " : "  "}
                      </Text>
                      <Text dimColor>{`[${formatTimetableSearchTypeLabel(result.type)}] `}</Text>
                      <Text>
                        {`${result.name}${result.longName !== result.name ? ` (${result.longName})` : ""}`}
                      </Text>
                    </Box>
                  );
                })}
            </Box>

            <Box justifyContent="space-between">
              <Text dimColor>
                {searchResults.length > 0
                  ? `Showing ${searchVisibleStart}-${searchVisibleEnd}`
                  : "Showing 0-0"}
              </Text>
              <Text dimColor>
                {searchResults.length > searchResultRows
                  ? `Scroll ${searchScrollOffset}/${Math.max(searchResults.length - searchResultRows, 0)}`
                  : " "}
              </Text>
            </Box>
          </Box>
        </Box>
      )}
    </Box>
  );
}
