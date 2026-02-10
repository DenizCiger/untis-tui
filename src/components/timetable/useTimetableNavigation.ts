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
  EMPTY_LESSONS,
  getSelectedLessonRange,
} from "./model.ts";

interface UseTimetableNavigationParams {
  data: WeekTimetable | null;
  dayLessonIndex: DayLessonIndex[];
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
    if (selectedPeriodIdx < scrollOffset) {
      setScrollOffset(selectedPeriodIdx);
    } else if (selectedPeriodIdx >= scrollOffset + rowsPerPage) {
      setScrollOffset(selectedPeriodIdx - rowsPerPage + 1);
    }
  }, [selectedPeriodIdx, scrollOffset, rowsPerPage]);

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
        setSelectedLessonIdx(0);
        return;
      }

      if (key.rightArrow && key.shift) {
        setWeekOffset((prev) => prev + 1);
        setSelectedPeriodIdx(0);
        setSelectedLessonIdx(0);
        return;
      }

      if (key.leftArrow) {
        setSelectedDayIdx((prev) => Math.max(0, prev - 1));
        setSelectedLessonIdx(0);
        return;
      }

      if (key.rightArrow) {
        setSelectedDayIdx((prev) => Math.min(4, prev + 1));
        setSelectedLessonIdx(0);
        return;
      }

      if (key.upArrow) {
        if (data) {
          const range = getSelectedLessonRange(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            selectedLessonIdx,
          );
          if (range) {
            setSelectedPeriodIdx(Math.max(0, range.startPeriodIdx - 1));
          } else {
            setSelectedPeriodIdx((prev) => Math.max(0, prev - 1));
          }
        } else {
          setSelectedPeriodIdx((prev) => Math.max(0, prev - 1));
        }
        setSelectedLessonIdx(0);
        return;
      }

      if (key.downArrow) {
        const maxPeriod = Math.max((data?.timegrid.length ?? 1) - 1, 0);
        if (data) {
          const range = getSelectedLessonRange(
            data,
            dayLessonIndex,
            selectedDayIdx,
            selectedPeriodIdx,
            selectedLessonIdx,
          );
          if (range) {
            setSelectedPeriodIdx(Math.min(maxPeriod, range.endPeriodIdx + 1));
          } else {
            setSelectedPeriodIdx((prev) => Math.min(maxPeriod, prev + 1));
          }
        } else {
          setSelectedPeriodIdx((prev) => Math.min(maxPeriod, prev + 1));
        }
        setSelectedLessonIdx(0);
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
