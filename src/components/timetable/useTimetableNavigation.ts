import {
  useEffect,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { useInput } from "ink";
import type { WeekTimetable } from "../../utils/untis.ts";
import { isShortcutPressed } from "../shortcuts.ts";
import {
  type DayLessonIndex,
  type DayOverlayIndex,
  EMPTY_LESSONS,
} from "./model.ts";

interface UseTimetableNavigationParams {
  data: WeekTimetable | null;
  dayLessonIndex: DayLessonIndex[];
  overlayIndexByDay: DayOverlayIndex[];
  rowsPerPage: number;
  setWeekOffset: Dispatch<SetStateAction<number>>;
  onQuit: () => void;
  onLogout: () => void;
  onRefresh: () => void;
  inputEnabled: boolean;
}

interface UseTimetableNavigationResult {
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  scrollOffset: number;
  setSelectedPeriodIdx: Dispatch<SetStateAction<number>>;
}

export function useTimetableNavigation({
  data,
  dayLessonIndex,
  overlayIndexByDay,
  rowsPerPage,
  setWeekOffset,
  onQuit,
  onLogout,
  onRefresh,
  inputEnabled,
}: UseTimetableNavigationParams): UseTimetableNavigationResult {
  const [selectedDayIdx, setSelectedDayIdx] = useState(() => {
    const day = new Date().getDay();
    return day >= 1 && day <= 5 ? day - 1 : 0;
  });
  const [selectedPeriodIdx, setSelectedPeriodIdx] = useState(0);
  const [selectedLessonIdx, setSelectedLessonIdx] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);

  useEffect(() => {
    const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
    setSelectedPeriodIdx((prev) => Math.min(prev, maxPeriod));
  }, [data]);

  useEffect(() => {
    const maxScroll = Math.max((data?.timegrid.length ?? 0) - rowsPerPage, 0);
    setScrollOffset((prev) => Math.min(prev, maxScroll));
  }, [data, rowsPerPage]);

  useEffect(() => {
    if (selectedPeriodIdx < scrollOffset) {
      setScrollOffset(selectedPeriodIdx);
    } else if (selectedPeriodIdx >= scrollOffset + rowsPerPage) {
      setScrollOffset(selectedPeriodIdx - rowsPerPage + 1);
    }
  }, [selectedPeriodIdx, scrollOffset, rowsPerPage]);

  useEffect(() => {
    if (!data) {
      setSelectedLessonIdx(0);
      return;
    }

    const day = dayLessonIndex[selectedDayIdx];
    const period = data.timegrid[selectedPeriodIdx];
    const lessonCount = day && period ? (day.get(period.startTime) ?? EMPTY_LESSONS).length : 0;

    setSelectedLessonIdx((prev) => {
      if (lessonCount <= 0) return 0;
      return Math.min(prev, lessonCount - 1);
    });
  }, [data, dayLessonIndex, selectedDayIdx, selectedPeriodIdx]);

  useInput(
    (input, key) => {
      if (isShortcutPressed("quit", input, key)) {
        onQuit();
        return;
      }

      if (isShortcutPressed("logout", input, key)) {
        onLogout();
        return;
      }

      if (isShortcutPressed("timetable-week-prev", input, key)) {
        setWeekOffset((prev) => prev - 1);
        setSelectedPeriodIdx(0);
        return;
      }

      if (isShortcutPressed("timetable-week-next", input, key)) {
        setWeekOffset((prev) => prev + 1);
        setSelectedPeriodIdx(0);
        return;
      }

      if (isShortcutPressed("timetable-day-prev", input, key)) {
        setSelectedDayIdx((prev) => Math.max(0, prev - 1));
        return;
      }

      if (isShortcutPressed("timetable-day-next", input, key)) {
        setSelectedDayIdx((prev) => Math.min(4, prev + 1));
        return;
      }

      if (isShortcutPressed("timetable-up", input, key)) {
        const nextPeriodIdx =
          findNextLessonPeriodIndex(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            -1,
          ) ?? Math.max(0, selectedPeriodIdx - 1);
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-down", input, key)) {
        const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
        const nextPeriodIdx =
          findNextLessonPeriodIndex(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            1,
          ) ?? Math.min(maxPeriod, selectedPeriodIdx + 1);
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-up-step", input, key)) {
        const nextPeriodIdx = Math.max(0, selectedPeriodIdx - 1);
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-down-step", input, key)) {
        const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
        const nextPeriodIdx = Math.min(maxPeriod, selectedPeriodIdx + 1);
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-page-up", input, key)) {
        const targetPeriodIdx = Math.max(0, selectedPeriodIdx - Math.max(1, rowsPerPage - 1));
        const nextPeriodIdx =
          findNextLessonPeriodIndex(
            data,
            dayLessonIndex,
            selectedDayIdx,
            targetPeriodIdx + 1,
            -1,
          ) ?? targetPeriodIdx;
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-page-down", input, key)) {
        const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
        const targetPeriodIdx = Math.min(
          maxPeriod,
          selectedPeriodIdx + Math.max(1, rowsPerPage - 1),
        );
        const nextPeriodIdx =
          findNextLessonPeriodIndex(
            data,
            dayLessonIndex,
            selectedDayIdx,
            targetPeriodIdx - 1,
            1,
          ) ?? targetPeriodIdx;
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-home", input, key)) {
        const nextPeriodIdx = findEdgeLessonPeriodIndex(
          data,
          dayLessonIndex,
          selectedDayIdx,
          "start",
        );
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-end", input, key)) {
        const nextPeriodIdx = findEdgeLessonPeriodIndex(
          data,
          dayLessonIndex,
          selectedDayIdx,
          "end",
        );
        setSelectedPeriodIdx(nextPeriodIdx);
        setSelectedLessonIdx(
          getSelectionIndexForPeriodChange(
            data,
            dayLessonIndex,
            overlayIndexByDay,
            selectedDayIdx,
            selectedPeriodIdx,
            nextPeriodIdx,
            selectedLessonIdx,
          ),
        );
        return;
      }

      if (isShortcutPressed("timetable-cycle-overlap", input, key)) {
        const day = dayLessonIndex[selectedDayIdx];
        const period = data?.timegrid[selectedPeriodIdx];
        if (!day || !period) return;

        const lessons = day.get(period.startTime) ?? EMPTY_LESSONS;
        if (lessons.length > 1) {
          setSelectedLessonIdx((prev) => (prev + 1) % lessons.length);
        }
        return;
      }

      if (isShortcutPressed("timetable-today", input, key)) {
        setWeekOffset(0);
        setSelectedLessonIdx(0);

        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const index = data?.days.findIndex(
          (day) => new Date(day.date).setHours(0, 0, 0, 0) === today.getTime(),
        );

        if (index !== undefined && index !== -1) {
          setSelectedDayIdx(index);
        }
        return;
      }

      if (isShortcutPressed("timetable-refresh", input, key)) {
        onRefresh();
      }
    },
    { isActive: inputEnabled && Boolean(process.stdin.isTTY) },
  );

  return {
    selectedDayIdx,
    selectedPeriodIdx,
    selectedLessonIdx,
    scrollOffset,
    setSelectedPeriodIdx,
  };
}

function hasLessonsAtPeriod(
  data: WeekTimetable | null,
  dayLessonIndex: DayLessonIndex[],
  dayIdx: number,
  periodIdx: number,
): boolean {
  if (!data) return false;
  const day = dayLessonIndex[dayIdx];
  const period = data.timegrid[periodIdx];
  if (!day || !period) return false;
  return (day.get(period.startTime) ?? EMPTY_LESSONS).length > 0;
}

function findNextLessonPeriodIndex(
  data: WeekTimetable | null,
  dayLessonIndex: DayLessonIndex[],
  dayIdx: number,
  fromPeriodIdx: number,
  direction: -1 | 1,
): number | null {
  if (!data) return null;

  const maxPeriod = data.timegrid.length - 1;
  let periodIdx = fromPeriodIdx + direction;
  while (periodIdx >= 0 && periodIdx <= maxPeriod) {
    if (hasLessonsAtPeriod(data, dayLessonIndex, dayIdx, periodIdx)) {
      return periodIdx;
    }

    periodIdx += direction;
  }

  return null;
}

function findEdgeLessonPeriodIndex(
  data: WeekTimetable | null,
  dayLessonIndex: DayLessonIndex[],
  dayIdx: number,
  edge: "start" | "end",
): number {
  if (!data || data.timegrid.length === 0) return 0;

  if (edge === "start") {
    for (let periodIdx = 0; periodIdx < data.timegrid.length; periodIdx += 1) {
      if (hasLessonsAtPeriod(data, dayLessonIndex, dayIdx, periodIdx)) {
        return periodIdx;
      }
    }

    return 0;
  }

  for (let periodIdx = data.timegrid.length - 1; periodIdx >= 0; periodIdx -= 1) {
    if (hasLessonsAtPeriod(data, dayLessonIndex, dayIdx, periodIdx)) {
      return periodIdx;
    }
  }

  return data.timegrid.length - 1;
}

function getSelectionIndexForPeriodChange(
  data: WeekTimetable | null,
  dayLessonIndex: DayLessonIndex[],
  overlayIndexByDay: DayOverlayIndex[],
  dayIdx: number,
  fromPeriodIdx: number,
  toPeriodIdx: number,
  selectedLessonIdx: number,
): number {
  if (!data) return 0;

  const day = dayLessonIndex[dayIdx];
  const fromPeriod = data.timegrid[fromPeriodIdx];
  const toPeriod = data.timegrid[toPeriodIdx];
  if (!day || !fromPeriod || !toPeriod) return 0;

  const fromLessons = day.get(fromPeriod.startTime) ?? EMPTY_LESSONS;
  const toLessons = day.get(toPeriod.startTime) ?? EMPTY_LESSONS;

  if (toLessons.length === 0) {
    return 0;
  }

  const selectedEntry = fromLessons[selectedLessonIdx] ?? null;
  if (!selectedEntry) {
    return Math.min(selectedLessonIdx, toLessons.length - 1);
  }

  const continuityMatchIdx = toLessons.findIndex(
    (entry) => entry.continuityKey === selectedEntry.continuityKey,
  );
  if (continuityMatchIdx !== -1) {
    return continuityMatchIdx;
  }

  const instanceMatchIdx = toLessons.findIndex(
    (entry) => entry.lessonInstanceId === selectedEntry.lessonInstanceId,
  );
  if (instanceMatchIdx !== -1) {
    return instanceMatchIdx;
  }

  const dayOverlay = overlayIndexByDay[dayIdx];
  const fromOverlay = dayOverlay?.get(fromPeriod.startTime);
  const toOverlay = dayOverlay?.get(toPeriod.startTime);

  if (fromOverlay?.split && toOverlay?.split) {
    const fromLaneIdx = fromOverlay.lanes.findIndex(
      (entry) => entry?.lessonInstanceId === selectedEntry.lessonInstanceId,
    );

    if (fromLaneIdx !== -1) {
      const targetLaneEntry = toOverlay.lanes[fromLaneIdx] ?? null;
      if (targetLaneEntry) {
        const laneMappedIdx = toLessons.findIndex(
          (entry) => entry.lessonInstanceId === targetLaneEntry.lessonInstanceId,
        );

        if (laneMappedIdx !== -1) {
          return laneMappedIdx;
        }
      }
    }
  }

  return 0;
}
