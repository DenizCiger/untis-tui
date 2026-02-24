import { describe, expect, it } from "bun:test";
import { getShortcutSections, isShortcutPressed, type InputKey } from "./shortcuts.ts";

function key(overrides: Partial<InputKey> = {}): InputKey {
  return {
    upArrow: false,
    downArrow: false,
    leftArrow: false,
    rightArrow: false,
    pageDown: false,
    pageUp: false,
    home: false,
    end: false,
    return: false,
    escape: false,
    ctrl: false,
    shift: false,
    tab: false,
    backspace: false,
    delete: false,
    meta: false,
    ...overrides,
  };
}

describe("shortcut registry", () => {
  it("matches the settings open shortcut", () => {
    expect(isShortcutPressed("settings-open", "?", key())).toBe(true);
    expect(isShortcutPressed("settings-open", "/", key())).toBe(false);
  });

  it("requires shift for previous timetable week", () => {
    expect(
      isShortcutPressed("timetable-week-prev", "", key({ leftArrow: true, shift: true })),
    ).toBe(true);
    expect(
      isShortcutPressed("timetable-week-prev", "", key({ leftArrow: true, shift: false })),
    ).toBe(false);
  });

  it("uses lesson-jump bindings for timetable vertical nav", () => {
    expect(isShortcutPressed("timetable-up", "", key({ upArrow: true }))).toBe(true);
    expect(
      isShortcutPressed("timetable-up", "", key({ upArrow: true, shift: true })),
    ).toBe(false);

    expect(
      isShortcutPressed("timetable-up-step", "", key({ upArrow: true, shift: true })),
    ).toBe(true);
  });

  it("supports timetable paging and edge shortcuts", () => {
    expect(isShortcutPressed("timetable-page-down", "", key({ pageDown: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("timetable-page-up", "", key({ pageUp: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("timetable-home", "", key({ home: true }))).toBe(true);
    expect(isShortcutPressed("timetable-end", "", key({ end: true }))).toBe(true);
  });

  it("includes contextual sections by active tab", () => {
    const timetableSections = getShortcutSections("timetable");
    const absencesSections = getShortcutSections("absences");

    expect(timetableSections.some((section) => section.title === "Timetable")).toBe(true);
    expect(timetableSections.some((section) => section.title === "Absences")).toBe(false);
    expect(absencesSections.some((section) => section.title === "Absences")).toBe(true);
  });
});
