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
  fetchTimetableSearchIndex,
  fetchWeekTimetable,
  getDefaultTimetableTarget,
  getMonday,
  getWeekTimetableWithCache,
  type TimetableSearchItem,
  type TimetableTarget,
  type WeekTimetable,
} from "../../utils/untis.ts";

interface TimetableSessionState {
  activeTarget: TimetableTarget;
  searchIndex: TimetableSearchItem[] | null;
}

const sessionStateByProfile = new Map<string, TimetableSessionState>();
const searchIndexPromiseByProfile = new Map<string, Promise<TimetableSearchItem[]>>();

function buildProfileKey(config: Config): string {
  return `${config.server}|${config.school}|${config.username}`;
}

function getOrCreateSessionState(profileKey: string): TimetableSessionState {
  const existing = sessionStateByProfile.get(profileKey);
  if (existing) return existing;

  const nextState: TimetableSessionState = {
    activeTarget: getDefaultTimetableTarget(),
    searchIndex: null,
  };
  sessionStateByProfile.set(profileKey, nextState);
  return nextState;
}

function normalizeTarget(target: TimetableTarget): TimetableTarget {
  if (target.type === "own") {
    return getDefaultTimetableTarget();
  }

  return target;
}

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
  activeTarget: TimetableTarget;
  setActiveTarget: (target: TimetableTarget) => void;
  clearActiveTarget: () => void;
  searchIndex: TimetableSearchItem[];
  searchIndexLoading: boolean;
  searchIndexError: string;
  ensureSearchIndexLoaded: () => void;
}

export function useTimetableData(config: Config): UseTimetableDataResult {
  const profileKey = useMemo(() => buildProfileKey(config), [config]);
  const sessionState = useMemo(
    () => getOrCreateSessionState(profileKey),
    [profileKey],
  );
  const isMountedRef = useRef(true);
  const [weekOffset, setWeekOffset] = useState(0);
  const [data, setData] = useState<WeekTimetable | null>(null);
  const [loading, setLoading] = useState(true);
  const [isFromCache, setIsFromCache] = useState(false);
  const [error, setError] = useState("");
  const [activeTarget, setActiveTargetState] = useState<TimetableTarget>(
    () => sessionState.activeTarget,
  );
  const [searchIndex, setSearchIndex] = useState<TimetableSearchItem[]>(
    () => sessionState.searchIndex ?? [],
  );
  const [searchIndexLoading, setSearchIndexLoading] = useState(false);
  const [searchIndexError, setSearchIndexError] = useState("");
  const requestIdRef = useRef(0);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    const latestSessionState = getOrCreateSessionState(profileKey);
    setActiveTargetState(latestSessionState.activeTarget);
    setSearchIndex(latestSessionState.searchIndex ?? []);
    setSearchIndexLoading(false);
    setSearchIndexError("");
  }, [profileKey]);

  const startRequest = useCallback(() => {
    requestIdRef.current += 1;
    return requestIdRef.current;
  }, []);

  const setActiveTarget = useCallback(
    (target: TimetableTarget) => {
      const normalized = normalizeTarget(target);
      const latestSessionState = getOrCreateSessionState(profileKey);
      latestSessionState.activeTarget = normalized;
      setActiveTargetState(normalized);
    },
    [profileKey],
  );

  const clearActiveTarget = useCallback(() => {
    const ownTarget = getDefaultTimetableTarget();
    const latestSessionState = getOrCreateSessionState(profileKey);
    latestSessionState.activeTarget = ownTarget;
    setActiveTargetState(ownTarget);
  }, [profileKey]);

  const loadSearchIndex = useCallback(
    (forceRefresh: boolean) => {
      if (!forceRefresh && searchIndex.length > 0) {
        setSearchIndexLoading(false);
        return;
      }

      const latestSessionState = getOrCreateSessionState(profileKey);
      if (!forceRefresh && latestSessionState.searchIndex) {
        setSearchIndex(latestSessionState.searchIndex);
        setSearchIndexLoading(false);
        setSearchIndexError("");
        return;
      }

      const existingPromise = searchIndexPromiseByProfile.get(profileKey);
      if (!forceRefresh && existingPromise) {
        setSearchIndexLoading(true);
        setSearchIndexError("");
        existingPromise
          .then((items) => {
            if (!isMountedRef.current) return;
            setSearchIndex(items);
            setSearchIndexError("");
          })
          .catch((err: any) => {
            if (!isMountedRef.current) return;
            setSearchIndexError(err?.message || "Failed to load timetable targets");
          })
          .finally(() => {
            if (!isMountedRef.current) return;
            setSearchIndexLoading(false);
          });
        return;
      }

      setSearchIndexLoading(true);
      setSearchIndexError("");

      const promise = fetchTimetableSearchIndex(config);
      searchIndexPromiseByProfile.set(profileKey, promise);

      promise
        .then((items) => {
          if (!isMountedRef.current) return;
          const currentSessionState = getOrCreateSessionState(profileKey);
          currentSessionState.searchIndex = items;
          setSearchIndex(items);
          setSearchIndexError("");
        })
        .catch((err: any) => {
          if (!isMountedRef.current) return;
          setSearchIndexError(err?.message || "Failed to load timetable targets");
        })
        .finally(() => {
          if (searchIndexPromiseByProfile.get(profileKey) === promise) {
            searchIndexPromiseByProfile.delete(profileKey);
          }
          if (!isMountedRef.current) return;
          setSearchIndexLoading(false);
        });
    },
    [config, profileKey, searchIndex.length],
  );

  const ensureSearchIndexLoaded = useCallback(() => {
    loadSearchIndex(false);
  }, [loadSearchIndex]);

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
          false,
          activeTarget,
        );

        if (cancelled || requestId !== requestIdRef.current) return;

        setData(cachedData);
        setIsFromCache(fromCache);
        setLoading(false);

        if (fromCache) {
          fetchWeekTimetable(config, targetDate, activeTarget)
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

    void load();

    return () => {
      cancelled = true;
    };
  }, [activeTarget, config, startRequest, weekOffset]);

  const refreshCurrentWeek = useCallback(() => {
    const requestId = startRequest();
    setLoading(true);
    setError("");

    fetchWeekTimetable(config, addDays(new Date(), weekOffset * 7), activeTarget)
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
  }, [activeTarget, config, startRequest, weekOffset]);

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
    activeTarget,
    setActiveTarget,
    clearActiveTarget,
    searchIndex,
    searchIndexLoading,
    searchIndexError,
    ensureSearchIndexLoaded,
  };
}
