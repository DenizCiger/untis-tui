import type {
  TimetableSearchItem,
  TimetableSearchTargetType,
} from "../../utils/untis.ts";

const TYPE_ORDER: Record<TimetableSearchTargetType, number> = {
  class: 0,
  room: 1,
  teacher: 2,
};

export function formatTimetableSearchTypeLabel(type: TimetableSearchTargetType): string {
  if (type === "class") return "Class";
  if (type === "room") return "Room";
  return "Teacher";
}

function normalize(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .trim()
    .toLowerCase();
}

function tokenize(value: string): string[] {
  return normalize(value).split(/\s+/).filter(Boolean);
}

function toWords(value: string): string[] {
  return normalize(value).split(/[^a-z0-9]+/).filter(Boolean);
}

function compareSearchItems(left: TimetableSearchItem, right: TimetableSearchItem): number {
  const byType = TYPE_ORDER[left.type] - TYPE_ORDER[right.type];
  if (byType !== 0) return byType;

  const byName = left.name.localeCompare(right.name);
  if (byName !== 0) return byName;

  const byLongName = left.longName.localeCompare(right.longName);
  if (byLongName !== 0) return byLongName;

  return left.id - right.id;
}

function hasAllTokens(haystack: string, tokens: string[]): boolean {
  return tokens.every((token) => haystack.includes(token));
}

function tokenContainsPenalty(haystack: string, tokens: string[]): number | null {
  let penalty = 0;
  for (const token of tokens) {
    const idx = haystack.indexOf(token);
    if (idx === -1) return null;
    penalty += idx;
  }
  return penalty;
}

function wordPrefixPenalty(words: string[], tokens: string[]): number | null {
  let penalty = 0;

  for (const token of tokens) {
    let bestWordIdx = -1;
    let bestLengthDelta = Number.POSITIVE_INFINITY;

    for (let idx = 0; idx < words.length; idx += 1) {
      const word = words[idx];
      if (!word?.startsWith(token)) continue;

      const lengthDelta = Math.max(word.length - token.length, 0);
      if (lengthDelta < bestLengthDelta) {
        bestWordIdx = idx;
        bestLengthDelta = lengthDelta;
      }
    }

    if (bestWordIdx === -1) return null;
    penalty += bestWordIdx * 8 + bestLengthDelta;
  }

  return penalty;
}

function fuzzySubsequencePenalty(haystack: string, query: string): number | null {
  if (!query) return 0;

  let haystackIndex = 0;
  let penalty = 0;
  let previousMatch = -1;

  for (const char of query) {
    const idx = haystack.indexOf(char, haystackIndex);
    if (idx === -1) return null;

    if (previousMatch !== -1) {
      penalty += Math.max(0, idx - previousMatch - 1);
    } else {
      penalty += idx;
    }

    previousMatch = idx;
    haystackIndex = idx + 1;
  }

  penalty += Math.max(0, haystack.length - haystackIndex);
  return penalty;
}

interface MatchRank {
  rank: number;
  penalty: number;
}

interface QueryContext {
  normalizedQuery: string;
  tokens: string[];
  compactQuery: string;
}

interface PreparedSearchItem {
  name: string;
  longName: string;
  searchText: string;
  compactSearch: string;
  nameWords: string[];
  longNameWords: string[];
}

const preparedItemCache = new WeakMap<TimetableSearchItem, PreparedSearchItem>();

function buildQueryContext(query: string): QueryContext {
  const normalizedQuery = normalize(query);
  return {
    normalizedQuery,
    tokens: tokenize(normalizedQuery),
    compactQuery: normalizedQuery.replace(/\s+/g, ""),
  };
}

function getPreparedSearchItem(item: TimetableSearchItem): PreparedSearchItem {
  const cached = preparedItemCache.get(item);
  if (cached) return cached;

  const name = normalize(item.name);
  const longName = normalize(item.longName);
  const searchText = normalize(item.searchText || `${item.name} ${item.longName}`);
  const prepared: PreparedSearchItem = {
    name,
    longName,
    searchText,
    compactSearch: searchText.replace(/\s+/g, ""),
    nameWords: toWords(name),
    longNameWords: toWords(longName),
  };
  preparedItemCache.set(item, prepared);
  return prepared;
}

function getMatchRank(item: PreparedSearchItem, queryContext: QueryContext): MatchRank | null {
  const { normalizedQuery, tokens, compactQuery } = queryContext;
  if (!normalizedQuery) return { rank: 0, penalty: 0 };

  const { name, longName, searchText, compactSearch, nameWords, longNameWords } =
    item;

  if (name.startsWith(normalizedQuery)) {
    return { rank: 0, penalty: name.length - normalizedQuery.length };
  }

  if (longName.startsWith(normalizedQuery)) {
    return { rank: 1, penalty: longName.length - normalizedQuery.length };
  }

  const nameWordPenalty = wordPrefixPenalty(nameWords, tokens);
  if (nameWordPenalty !== null) {
    return { rank: 2, penalty: nameWordPenalty };
  }

  const longNameWordPenalty = wordPrefixPenalty(longNameWords, tokens);
  if (longNameWordPenalty !== null) {
    return { rank: 3, penalty: longNameWordPenalty };
  }

  if (hasAllTokens(name, tokens)) {
    return { rank: 4, penalty: tokenContainsPenalty(name, tokens) ?? 0 };
  }

  if (hasAllTokens(longName, tokens)) {
    return { rank: 5, penalty: tokenContainsPenalty(longName, tokens) ?? 0 };
  }

  if (hasAllTokens(searchText, tokens)) {
    return { rank: 6, penalty: tokenContainsPenalty(searchText, tokens) ?? 0 };
  }

  const fuzzyPenalty = fuzzySubsequencePenalty(compactSearch, compactQuery);
  if (fuzzyPenalty !== null) {
    return { rank: 7, penalty: fuzzyPenalty };
  }

  return null;
}

function applyLimit(
  items: TimetableSearchItem[],
  limit?: number,
): TimetableSearchItem[] {
  if (typeof limit === "number" && Number.isFinite(limit) && limit > 0) {
    return items.slice(0, Math.floor(limit));
  }

  return items;
}

export function searchTimetableTargets(
  items: TimetableSearchItem[],
  query: string,
  limit?: number,
): TimetableSearchItem[] {
  const queryContext = buildQueryContext(query);
  if (!queryContext.normalizedQuery) {
    const sorted = [...items].sort(compareSearchItems);
    return applyLimit(sorted, limit);
  }

  const ranked = items
    .map((item) => ({
      item,
      score: getMatchRank(getPreparedSearchItem(item), queryContext),
    }))
    .filter(
      (entry): entry is { item: TimetableSearchItem; score: MatchRank } =>
        entry.score !== null,
    );

  ranked.sort((left, right) => {
    if (left.score.rank !== right.score.rank) {
      return left.score.rank - right.score.rank;
    }

    if (left.score.penalty !== right.score.penalty) {
      return left.score.penalty - right.score.penalty;
    }

    return compareSearchItems(left.item, right.item);
  });

  return applyLimit(
    ranked.map((entry) => entry.item),
    limit,
  );
}
