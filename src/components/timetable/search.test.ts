import { describe, expect, it } from "bun:test";
import type { TimetableSearchItem } from "../../utils/untis.ts";
import { searchTimetableTargets } from "./search.ts";

function item(overrides: Partial<TimetableSearchItem>): TimetableSearchItem {
  return {
    type: overrides.type ?? "teacher",
    id: overrides.id ?? 1,
    name: overrides.name ?? "ABC",
    longName: overrides.longName ?? "Alphabet",
    searchText:
      overrides.searchText ??
      `${overrides.name ?? "ABC"} ${overrides.longName ?? "Alphabet"}`.toLowerCase(),
  };
}

describe("timetable search ranking", () => {
  it("matches contains case-insensitively", () => {
    const results = searchTimetableTargets(
      [
        item({ id: 1, name: "MrMiller", longName: "Miller" }),
        item({ id: 2, name: "Room A12", longName: "Science Room" }),
      ],
      "MILL",
      10,
    );

    expect(results.map((entry) => entry.id)).toEqual([1]);
  });

  it("prioritizes starts-with over contains matches", () => {
    const results = searchTimetableTargets(
      [
        item({ id: 1, name: "Tina", longName: "Teacher Tina" }),
        item({ id: 2, name: "Math", longName: "Advanced Tina Group" }),
        item({ id: 3, name: "Bio", longName: "Tina Biology" }),
      ],
      "ti",
      10,
    );

    expect(results.map((entry) => entry.id)).toEqual([1, 3, 2]);
  });

  it("keeps mixed-type ordering stable for equal rank", () => {
    const results = searchTimetableTargets(
      [
        item({ id: 2, type: "teacher", name: "A-Name" }),
        item({ id: 1, type: "class", name: "A-Name" }),
        item({ id: 3, type: "room", name: "A-Name" }),
      ],
      "a-",
      10,
    );

    expect(results.map((entry) => `${entry.type}:${entry.id}`)).toEqual([
      "class:1",
      "room:3",
      "teacher:2",
    ]);
  });

  it("matches multi-token queries across name fields", () => {
    const results = searchTimetableTargets(
      [
        item({
          id: 1,
          type: "teacher",
          name: "Max Mustermann",
          longName: "MMAX",
          searchText: "max mustermann mmax",
        }),
        item({
          id: 2,
          type: "teacher",
          name: "Max Muster",
          longName: "MMUS",
          searchText: "max muster mmus",
        }),
      ],
      "max mmax",
    );

    expect(results.map((entry) => entry.id)).toEqual([1]);
  });

  it("returns all matches when no limit is provided", () => {
    const results = searchTimetableTargets(
      [
        item({ id: 1, name: "AA" }),
        item({ id: 2, name: "AB" }),
        item({ id: 3, name: "AC" }),
      ],
      "a",
    );

    expect(results).toHaveLength(3);
  });
});
