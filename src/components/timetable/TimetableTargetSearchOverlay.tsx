import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import type { TimetableSearchItem, TimetableTarget } from "../../utils/untis.ts";
import { COLORS } from "../colors.ts";
import TextInput from "../TextInput.tsx";
import {
  formatTimetableSearchTypeLabel,
  searchTimetableTargets,
} from "./search.ts";

interface TimetableTargetSearchOverlayProps {
  termWidth: number;
  termHeight: number;
  searchIndex: TimetableSearchItem[];
  searchIndexLoading: boolean;
  searchIndexError: string;
  onClose: () => void;
  onApplyTarget: (target: TimetableTarget) => void;
}

const SEARCH_DEBOUNCE_MS = 32;
const SEARCH_CACHE_LIMIT = 32;

interface SearchSnapshot {
  queryKey: string;
  results: TimetableSearchItem[];
}

function toQueryKey(query: string): string {
  return query.trim().toLowerCase().replace(/\s+/g, " ");
}

export default function TimetableTargetSearchOverlay({
  termWidth,
  termHeight,
  searchIndex,
  searchIndexLoading,
  searchIndexError,
  onClose,
  onApplyTarget,
}: TimetableTargetSearchOverlayProps) {
  const [draft, setDraft] = useState("");
  const [debouncedDraft, setDebouncedDraft] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);
  const searchCacheRef = useRef(new Map<string, TimetableSearchItem[]>());
  const lastSearchRef = useRef<SearchSnapshot>({
    queryKey: "",
    results: [],
  });

  useEffect(() => {
    searchCacheRef.current.clear();
    lastSearchRef.current = {
      queryKey: "",
      results: [],
    };
  }, [searchIndex]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedDraft(draft);
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [draft]);

  const runSearch = useCallback(
    (query: string) => {
      const queryKey = toQueryKey(query);
      const cached = searchCacheRef.current.get(queryKey);
      if (cached) {
        lastSearchRef.current = { queryKey, results: cached };
        return cached;
      }

      const previous = lastSearchRef.current;
      const canNarrowFromPrevious =
        queryKey.length > 0 &&
        queryKey.startsWith(previous.queryKey) &&
        previous.results.length > 0 &&
        previous.results.length < searchIndex.length;

      const sourceItems = canNarrowFromPrevious ? previous.results : searchIndex;
      const results = searchTimetableTargets(sourceItems, query);

      const cache = searchCacheRef.current;
      cache.set(queryKey, results);
      if (cache.size > SEARCH_CACHE_LIMIT) {
        const oldestKey = cache.keys().next().value;
        if (typeof oldestKey === "string") {
          cache.delete(oldestKey);
        }
      }

      lastSearchRef.current = { queryKey, results };
      return results;
    },
    [searchIndex],
  );

  const searchResults = useMemo(
    () => runSearch(debouncedDraft),
    [debouncedDraft, runSearch],
  );
  const searchResultsAreDeferred = draft !== debouncedDraft;

  const searchModalWidth = Math.max(56, Math.min(112, termWidth - 8));
  const searchModalHeight = Math.max(12, Math.min(30, termHeight - 4));
  const searchResultRows = Math.max(3, searchModalHeight - 7);
  const visibleSearchResults = useMemo(
    () => searchResults.slice(scrollOffset, scrollOffset + searchResultRows),
    [searchResults, scrollOffset, searchResultRows],
  );

  useEffect(() => {
    setSelectedIdx((prev) => Math.min(prev, Math.max(searchResults.length - 1, 0)));
  }, [searchResults.length]);

  useEffect(() => {
    const maxScroll = Math.max(searchResults.length - searchResultRows, 0);
    setScrollOffset((prev) => Math.min(prev, maxScroll));
  }, [searchResultRows, searchResults.length]);

  useEffect(() => {
    if (selectedIdx < scrollOffset) {
      setScrollOffset(selectedIdx);
      return;
    }

    if (selectedIdx >= scrollOffset + searchResultRows) {
      setScrollOffset(selectedIdx - searchResultRows + 1);
    }
  }, [searchResultRows, scrollOffset, selectedIdx]);

  const moveSelection = useCallback(
    (delta: number) => {
      setSelectedIdx((prev) =>
        Math.max(0, Math.min(prev + delta, Math.max(searchResults.length - 1, 0))),
      );
    },
    [searchResults.length],
  );

  const applySelection = useCallback(
    (query: string) => {
      const instantResults =
        searchResultsAreDeferred || query !== debouncedDraft
          ? runSearch(query)
          : searchResults;

      const boundedIndex = Math.max(
        0,
        Math.min(selectedIdx, Math.max(instantResults.length - 1, 0)),
      );
      const selected = instantResults[boundedIndex];

      if (!selected) {
        onClose();
        return;
      }

      onApplyTarget({
        type: selected.type,
        id: selected.id,
        name: selected.name,
        longName: selected.longName,
      });
    },
    [
      debouncedDraft,
      onApplyTarget,
      onClose,
      runSearch,
      searchResults,
      searchResultsAreDeferred,
      selectedIdx,
    ],
  );

  const visibleStart = searchResults.length > 0 ? scrollOffset + 1 : 0;
  const visibleEnd = Math.min(
    searchResults.length,
    scrollOffset + visibleSearchResults.length,
  );

  return (
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
              ? `${Math.min(selectedIdx + 1, searchResults.length)}/${searchResults.length}`
              : "0/0"}
          </Text>
        </Box>

        <Box>
          <Text color={COLORS.brand}>{"> "}</Text>
          <TextInput
            value={draft}
            onChange={(value) => {
              setDraft(value);
              setSelectedIdx(0);
              setScrollOffset(0);
            }}
            onSubmit={(value) => {
              applySelection(value);
            }}
            onKey={(_input, key) => {
              if (key.escape) {
                onClose();
                return true;
              }

              if (key.upArrow) {
                moveSelection(-1);
                return true;
              }

              if (key.downArrow) {
                moveSelection(1);
                return true;
              }

              if (key.pageUp) {
                moveSelection(-searchResultRows);
                return true;
              }

              if (key.pageDown) {
                moveSelection(searchResultRows);
                return true;
              }

              if (key.home) {
                setSelectedIdx(0);
                return true;
              }

              if (key.end) {
                setSelectedIdx(Math.max(searchResults.length - 1, 0));
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
            <Text dimColor>Use ↑/↓, PgUp/PgDn, Home/End, Enter apply, Esc cancel.</Text>
          )}
        </Box>

        <Box flexDirection="column" flexGrow={1} overflow="hidden">
          {!searchIndexLoading && !searchIndexError && searchResults.length === 0 && (
            <Text dimColor>No targets found for this query.</Text>
          )}

          {!searchIndexLoading &&
            !searchIndexError &&
            visibleSearchResults.map((result, idx) => {
              const absoluteIdx = scrollOffset + idx;
              const selected = absoluteIdx === selectedIdx;
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
              ? `Showing ${visibleStart}-${visibleEnd}`
              : "Showing 0-0"}
          </Text>
          <Text dimColor>
            {searchResults.length > searchResultRows
              ? `Scroll ${scrollOffset}/${Math.max(searchResults.length - searchResultRows, 0)}`
              : " "}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
