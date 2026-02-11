import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
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
  const requestIdRef = useRef(0);

  const startRequest = useCallback(() => {
    requestIdRef.current += 1;
    return requestIdRef.current;
  }, []);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      const requestId = startRequest();
      setLoading(true);
      setError("");
      setData(null);

      const targetDate = addDays(new Date(), weekOffset * 7);

      try {
        const { data: cachedData, fromCache } = await getWeekTimetableWithCache(
          config,
          targetDate,
        );

        if (cancelled || requestId !== requestIdRef.current) return;

        setData(cachedData);
        setIsFromCache(fromCache);
        setLoading(false);

        if (fromCache) {
          fetchWeekTimetable(config, targetDate)
            .then((freshData) => {
              if (cancelled || requestId !== requestIdRef.current) return;
              setData(freshData);
              setIsFromCache(false);
            })
            .catch((err: any) => {
              if (cancelled || requestId !== requestIdRef.current) return;
              setError(err?.message || "Failed to refresh timetable");
            });
        }
      } catch (err: any) {
        if (cancelled || requestId !== requestIdRef.current) return;
        setError(err?.message || "Failed to fetch timetable");
        setLoading(false);
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [config, startRequest, weekOffset]);

  const refreshCurrentWeek = useCallback(() => {
    const requestId = startRequest();
    setLoading(true);
    setError("");

    fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7))
      .then((freshData) => {
        if (requestId !== requestIdRef.current) return;
        setData(freshData);
        setIsFromCache(false);
      })
      .catch((err: any) => {
        if (requestId !== requestIdRef.current) return;
        setError(err?.message || "Refresh failed");
      })
      .finally(() => {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      });
  }, [config, startRequest, weekOffset]);

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
