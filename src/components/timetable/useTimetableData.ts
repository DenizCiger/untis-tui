import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import type { Config } from "../../utils/config.ts";
import {
  addDays,
  fetchWeekTimetable,
  getMonday,
  getWeekTimetableWithCache,
  type WeekTimetable,
} from "../../utils/untis.ts";

interface UseTimetableDataResult {
  weekOffset: number;
  setWeekOffset: Dispatch<SetStateAction<number>>;
  data: WeekTimetable | null;
  loading: boolean;
  isFromCache: boolean;
  error: string;
  refreshCurrentWeek: () => void;
  currentMonday: Date;
  currentFriday: Date;
}

export function useTimetableData(config: Config): UseTimetableDataResult {
  const [weekOffset, setWeekOffset] = useState(0);
  const [data, setData] = useState<WeekTimetable | null>(null);
  const [loading, setLoading] = useState(true);
  const [isFromCache, setIsFromCache] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setError("");
      setData(null);

      const targetDate = addDays(new Date(), weekOffset * 7);

      try {
        const { data: cachedData, fromCache } = await getWeekTimetableWithCache(
          config,
          targetDate,
        );

        if (cancelled) return;

        setData(cachedData);
        setIsFromCache(fromCache);
        setLoading(false);

        if (fromCache) {
          fetchWeekTimetable(config, targetDate)
            .then((freshData) => {
              if (cancelled) return;
              setData(freshData);
              setIsFromCache(false);
            })
            .catch(() => {});
        }
      } catch (err: any) {
        if (cancelled) return;
        setError(err?.message || "Failed to fetch timetable");
        setLoading(false);
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [config, weekOffset]);

  const refreshCurrentWeek = useCallback(() => {
    setLoading(true);
    setError("");

    fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7))
      .then((freshData) => {
        setData(freshData);
        setIsFromCache(false);
      })
      .catch((err: any) => {
        setError(err?.message || "Refresh failed");
      })
      .finally(() => setLoading(false));
  }, [config, weekOffset]);

  const currentMonday = useMemo(
    () => getMonday(addDays(new Date(), weekOffset * 7)),
    [weekOffset],
  );
  const currentFriday = useMemo(() => addDays(currentMonday, 4), [currentMonday]);

  return {
    weekOffset,
    setWeekOffset,
    data,
    loading,
    isFromCache,
    error,
    refreshCurrentWeek,
    currentMonday,
    currentFriday,
  };
}
