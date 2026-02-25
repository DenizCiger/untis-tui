import { describe, expect, it } from "bun:test";
import { buildWeekCacheKey, getWeekLookupKeys } from "./cache.ts";

describe("week cache keys", () => {
  it("creates target-scoped keys", () => {
    const monday = "2026-02-23";
    expect(buildWeekCacheKey(monday, "own")).toBe("own:2026-02-23");
    expect(buildWeekCacheKey(monday, "class:42")).toBe("class:42:2026-02-23");
    expect(buildWeekCacheKey(monday, "room:7")).toBe("room:7:2026-02-23");
  });

  it("includes legacy own key fallback", () => {
    const keys = getWeekLookupKeys("2026-02-23", "own");
    expect(keys).toEqual(["own:2026-02-23", "2026-02-23"]);
  });

  it("does not include legacy fallback for non-own keys", () => {
    const keys = getWeekLookupKeys("2026-02-23", "teacher:11");
    expect(keys).toEqual(["teacher:11:2026-02-23"]);
  });
});
