import type { TimetableSearchItem, TimetableSearchTargetType } from "../../utils/untis.ts";

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
  return value.trim().toLowerCase();
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

function getMatchRank(item: TimetableSearchItem, normalizedQuery: string): number | null {
  if (!normalizedQuery) return 0;

  const name = normalize(item.name);
  const longName = normalize(item.longName);
  const searchText = normalize(item.searchText || `${item.name} ${item.longName}`);

  if (name.startsWith(normalizedQuery)) return 0;
  if (longName.startsWith(normalizedQuery)) return 1;
  if (name.includes(normalizedQuery)) return 2;
  if (longName.includes(normalizedQuery)) return 3;
  if (searchText.includes(normalizedQuery)) return 4;

  return null;
}

export function searchTimetableTargets(
  items: TimetableSearchItem[],
  query: string,
  limit: number = 12,
): TimetableSearchItem[] {
  const normalizedQuery = normalize(query);
  const ranked = items
    .map((item) => ({ item, rank: getMatchRank(item, normalizedQuery) }))
    .filter((entry): entry is { item: TimetableSearchItem; rank: number } => entry.rank !== null);

  ranked.sort((left, right) => {
    if (left.rank !== right.rank) return left.rank - right.rank;
    return compareSearchItems(left.item, right.item);
  });

  return ranked.slice(0, Math.max(1, limit)).map((entry) => entry.item);
}
