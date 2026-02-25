import { describe, expect, it } from "bun:test";
import { WebUntisElementType } from "webuntis";
import {
  formatTimetableTargetLabel,
  resolveTimetableForWeekRequest,
  targetToCacheKey,
  type TimetableTarget,
} from "./untis.ts";

describe("timetable target helpers", () => {
  it("resolves own timetable request", () => {
    expect(resolveTimetableForWeekRequest({ type: "own" })).toEqual({ mode: "own" });
  });

  it("maps class/room/teacher targets to webuntis element types", () => {
    const classRequest = resolveTimetableForWeekRequest({
      type: "class",
      id: 12,
      name: "1A",
      longName: "1A Class",
    });
    const roomRequest = resolveTimetableForWeekRequest({
      type: "room",
      id: 13,
      name: "A12",
      longName: "Room A12",
    });
    const teacherRequest = resolveTimetableForWeekRequest({
      type: "teacher",
      id: 14,
      name: "MILL",
      longName: "Miller",
    });

    expect(classRequest).toEqual({
      mode: "target",
      id: 12,
      type: WebUntisElementType.CLASS,
    });
    expect(roomRequest).toEqual({
      mode: "target",
      id: 13,
      type: WebUntisElementType.ROOM,
    });
    expect(teacherRequest).toEqual({
      mode: "target",
      id: 14,
      type: WebUntisElementType.TEACHER,
    });
  });

  it("builds target-specific cache keys", () => {
    const ownTarget: TimetableTarget = { type: "own" };
    const classTarget: TimetableTarget = {
      type: "class",
      id: 42,
      name: "4AHIF",
      longName: "4AHIF Class",
    };

    expect(targetToCacheKey(ownTarget)).toBe("own");
    expect(targetToCacheKey(classTarget)).toBe("class:42");
  });

  it("formats target label for header", () => {
    expect(formatTimetableTargetLabel({ type: "own" })).toBe("My timetable");
    expect(
      formatTimetableTargetLabel({
        type: "teacher",
        id: 99,
        name: "DELL",
        longName: "Mr Dell",
      }),
    ).toBe("Teacher: DELL");
  });
});
