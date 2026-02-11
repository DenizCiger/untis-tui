import {
  useEffect,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { useInput } from "ink";
import type { WeekTimetable } from "../../utils/untis.ts";
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
}

interface UseTimetableNavigationResult {
  selectedDayIdx: number;
  selectedPeriodIdx: number;
  selectedLessonIdx: number;
  scrollOffset: number;
  showHelp: boolean;
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
}: UseTimetableNavigationParams): UseTimetableNavigationResult {
  const [selectedDayIdx, setSelectedDayIdx] = useState(() => {
    const day = new Date().getDay();
    return day >= 1 && day <= 5 ? day - 1 : 0;
  });
  const [selectedPeriodIdx, setSelectedPeriodIdx] = useState(0);
  const [selectedLessonIdx, setSelectedLessonIdx] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);
  const [showHelp, setShowHelp] = useState(false);

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
      if (input === "q") {
        onQuit();
        return;
      }

      if (input === "l") {
        onLogout();
        return;
      }

      if (key.leftArrow && key.shift) {
        setWeekOffset((prev) => prev - 1);
        setSelectedPeriodIdx(0);
        return;
      }

      if (key.rightArrow && key.shift) {
        setWeekOffset((prev) => prev + 1);
        setSelectedPeriodIdx(0);
        return;
      }

      if (key.leftArrow) {
        setSelectedDayIdx((prev) => Math.max(0, prev - 1));
        return;
      }

      if (key.rightArrow) {
        setSelectedDayIdx((prev) => Math.min(4, prev + 1));
        return;
      }

      if (key.upArrow) {
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

      if (key.downArrow) {
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

      if (key.tab) {
        const day = dayLessonIndex[selectedDayIdx];
        const period = data?.timegrid[selectedPeriodIdx];
        if (!day || !period) return;

        const lessons = day.get(period.startTime) ?? EMPTY_LESSONS;
        if (lessons.length > 1) {
          setSelectedLessonIdx((prev) => (prev + 1) % lessons.length);
        }
        return;
      }

      if (input === "h") {
        setShowHelp((prev) => !prev);
        return;
      }

      if (input === "t") {
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

      if (input === "r") {
        onRefresh();
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  return {
    selectedDayIdx,
    selectedPeriodIdx,
    selectedLessonIdx,
    scrollOffset,
    showHelp,
    setSelectedPeriodIdx,
  };
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
