import { useCallback, useEffect, useRef, useState } from "react";
import type { Config } from "../../utils/config.ts";
import {
  addDays,
  fetchAbsencesForRange,
  type ParsedAbsence,
} from "../../utils/untis.ts";

const CHUNK_DAYS = 45;
const MAX_HISTORY_DAYS = 365 * 5;
const MAX_EMPTY_CHUNK_STREAK = 4;

interface UseAbsencesDataResult {
  absences: ParsedAbsence[];
  loadingInitial: boolean;
  loadingMore: boolean;
  error: string;
  hasMore: boolean;
  daysLoaded: number;
  chunkDays: number;
  loadMore: () => void;
  refresh: () => void;
}

function startOfDay(date: Date): Date {
  const normalized = new Date(date);
  normalized.setHours(0, 0, 0, 0);
  return normalized;
}

function getChunkRange(baseDate: Date, chunkIndex: number): {
  rangeStart: Date;
  rangeEnd: Date;
} {
  const rangeEnd = addDays(baseDate, -(chunkIndex * CHUNK_DAYS));
  const rangeStart = addDays(rangeEnd, -(CHUNK_DAYS - 1));
  return {
    rangeStart: startOfDay(rangeStart),
    rangeEnd: startOfDay(rangeEnd),
  };
}

function compareAbsenceNewestFirst(left: ParsedAbsence, right: ParsedAbsence): number {
  const byStartDate = right.startDate.getTime() - left.startDate.getTime();
  if (byStartDate !== 0) return byStartDate;

  const byStartTime = right.startTime.localeCompare(left.startTime);
  if (byStartTime !== 0) return byStartTime;

  return right.id - left.id;
}

function mergeAbsences(
  previous: ParsedAbsence[],
  incoming: ParsedAbsence[],
): ParsedAbsence[] {
  const byId = new Map<number, ParsedAbsence>();

  for (const absence of previous) {
    byId.set(absence.id, absence);
  }

  for (const absence of incoming) {
    byId.set(absence.id, absence);
  }

  return Array.from(byId.values()).sort(compareAbsenceNewestFirst);
}

export function useAbsencesData(config: Config): UseAbsencesDataResult {
  const [absences, setAbsences] = useState<ParsedAbsence[]>([]);
  const [loadingInitial, setLoadingInitial] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");
  const [hasMore, setHasMore] = useState(true);
  const [daysLoaded, setDaysLoaded] = useState(0);

  const generationRef = useRef(0);
  const chunkIndexRef = useRef(0);
  const emptyChunkStreakRef = useRef(0);
  const inFlightRef = useRef(false);
  const hasMoreRef = useRef(true);
  const baseDateRef = useRef(startOfDay(new Date()));

  const loadNextChunk = useCallback(
    async (isInitial: boolean) => {
      if (inFlightRef.current || !hasMoreRef.current) {
        return;
      }

      inFlightRef.current = true;
      const generation = generationRef.current;
      const chunkIndex = chunkIndexRef.current;
      const { rangeStart, rangeEnd } = getChunkRange(baseDateRef.current, chunkIndex);

      if (isInitial) {
        setLoadingInitial(true);
      } else {
        setLoadingMore(true);
      }

      setError("");

      try {
        const nextChunk = await fetchAbsencesForRange(config, rangeStart, rangeEnd);

        if (generation !== generationRef.current) {
          return;
        }

        chunkIndexRef.current += 1;
        setDaysLoaded(chunkIndexRef.current * CHUNK_DAYS);

        if (nextChunk.length === 0) {
          emptyChunkStreakRef.current += 1;
        } else {
          emptyChunkStreakRef.current = 0;
        }

        setAbsences((previous) => mergeAbsences(previous, nextChunk));

        const reachedMaxHistory = chunkIndexRef.current * CHUNK_DAYS >= MAX_HISTORY_DAYS;
        const reachedEmptyStreak =
          emptyChunkStreakRef.current >= MAX_EMPTY_CHUNK_STREAK;
        const nextHasMore = !reachedMaxHistory && !reachedEmptyStreak;

        hasMoreRef.current = nextHasMore;
        setHasMore(nextHasMore);
      } catch (err: any) {
        if (generation !== generationRef.current) {
          return;
        }

        setError(err?.message || "Failed to fetch absences");
      } finally {
        if (generation === generationRef.current) {
          if (isInitial) {
            setLoadingInitial(false);
          }

          setLoadingMore(false);
        }

        inFlightRef.current = false;
      }
    },
    [config],
  );

  const refresh = useCallback(() => {
    generationRef.current += 1;
    chunkIndexRef.current = 0;
    emptyChunkStreakRef.current = 0;
    hasMoreRef.current = true;
    baseDateRef.current = startOfDay(new Date());
    inFlightRef.current = false;

    setAbsences([]);
    setError("");
    setHasMore(true);
    setDaysLoaded(0);
    setLoadingMore(false);

    void loadNextChunk(true);
  }, [loadNextChunk]);

  const loadMore = useCallback(() => {
    void loadNextChunk(false);
  }, [loadNextChunk]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    absences,
    loadingInitial,
    loadingMore,
    error,
    hasMore,
    daysLoaded,
    chunkDays: CHUNK_DAYS,
    loadMore,
    refresh,
  };
}
