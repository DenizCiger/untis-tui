export type TabId = "timetable" | "absences";

export interface InputKey {
  upArrow: boolean;
  downArrow: boolean;
  leftArrow: boolean;
  rightArrow: boolean;
  pageDown: boolean;
  pageUp: boolean;
  home: boolean;
  end: boolean;
  return: boolean;
  escape: boolean;
  ctrl: boolean;
  shift: boolean;
  tab: boolean;
  backspace: boolean;
  delete: boolean;
  meta: boolean;
}

interface ShortcutDefinition {
  id: string;
  keys: string;
  action: string;
  match: (input: string, key: InputKey) => boolean;
}

export interface ShortcutSection {
  title: string;
  items: Array<Pick<ShortcutDefinition, "id" | "keys" | "action">>;
}

const SHORTCUTS: ShortcutDefinition[] = [
  {
    id: "settings-open",
    keys: "?",
    action: "Open shortcuts/settings",
    match: (input) => input === "?",
  },
  {
    id: "settings-close",
    keys: "Esc or ?",
    action: "Close settings modal",
    match: (input, key) => key.escape || input === "?",
  },
  {
    id: "tab-prev",
    keys: "[",
    action: "Previous tab",
    match: (input) => input === "[",
  },
  {
    id: "tab-next",
    keys: "]",
    action: "Next tab",
    match: (input) => input === "]",
  },
  {
    id: "tab-timetable",
    keys: "1",
    action: "Jump to Timetable tab",
    match: (input) => input === "1",
  },
  {
    id: "tab-absences",
    keys: "2",
    action: "Jump to Absences tab",
    match: (input) => input === "2",
  },
  {
    id: "quit",
    keys: "q",
    action: "Quit app",
    match: (input) => input === "q",
  },
  {
    id: "logout",
    keys: "l",
    action: "Logout",
    match: (input) => input === "l",
  },
  {
    id: "timetable-week-prev",
    keys: "Shift+Left",
    action: "Previous week",
    match: (_input, key) => key.leftArrow && key.shift,
  },
  {
    id: "timetable-week-next",
    keys: "Shift+Right",
    action: "Next week",
    match: (_input, key) => key.rightArrow && key.shift,
  },
  {
    id: "timetable-day-prev",
    keys: "Left",
    action: "Move focus to previous day",
    match: (_input, key) => key.leftArrow && !key.shift,
  },
  {
    id: "timetable-day-next",
    keys: "Right",
    action: "Move focus to next day",
    match: (_input, key) => key.rightArrow && !key.shift,
  },
  {
    id: "timetable-up",
    keys: "Up",
    action: "Move focus up",
    match: (_input, key) => key.upArrow,
  },
  {
    id: "timetable-down",
    keys: "Down",
    action: "Move focus down",
    match: (_input, key) => key.downArrow,
  },
  {
    id: "timetable-cycle-overlap",
    keys: "Tab",
    action: "Cycle overlapping lessons",
    match: (_input, key) => key.tab,
  },
  {
    id: "timetable-today",
    keys: "t",
    action: "Jump to current week/day",
    match: (input) => input === "t",
  },
  {
    id: "timetable-refresh",
    keys: "r",
    action: "Refresh timetable",
    match: (input) => input === "r",
  },
  {
    id: "absences-up",
    keys: "Up or k",
    action: "Move selection up",
    match: (input, key) => key.upArrow || input === "k",
  },
  {
    id: "absences-down",
    keys: "Down or j",
    action: "Move selection down",
    match: (input, key) => key.downArrow || input === "j",
  },
  {
    id: "absences-page-up",
    keys: "PageUp",
    action: "Jump one page up",
    match: (_input, key) => key.pageUp,
  },
  {
    id: "absences-page-down",
    keys: "PageDown",
    action: "Jump one page down",
    match: (_input, key) => key.pageDown,
  },
  {
    id: "absences-home",
    keys: "Home",
    action: "Jump to first record",
    match: (_input, key) => key.home,
  },
  {
    id: "absences-end",
    keys: "End",
    action: "Jump to last loaded record",
    match: (_input, key) => key.end,
  },
  {
    id: "absences-status",
    keys: "f",
    action: "Cycle status filter",
    match: (input) => input === "f",
  },
  {
    id: "absences-window",
    keys: "w",
    action: "Cycle time window",
    match: (input) => input === "w",
  },
  {
    id: "absences-search",
    keys: "/",
    action: "Open search",
    match: (input) => input === "/",
  },
  {
    id: "absences-clear",
    keys: "c",
    action: "Clear all filters",
    match: (input) => input === "c",
  },
  {
    id: "absences-load-more",
    keys: "m",
    action: "Load older records",
    match: (input) => input === "m",
  },
  {
    id: "absences-refresh",
    keys: "r",
    action: "Refresh absences",
    match: (input) => input === "r",
  },
  {
    id: "absences-search-submit",
    keys: "Enter",
    action: "Apply search query",
    match: (_input, key) => key.return,
  },
  {
    id: "absences-search-cancel",
    keys: "Esc",
    action: "Cancel search edit",
    match: (_input, key) => key.escape,
  },
];

const SHORTCUT_BY_ID = new Map(SHORTCUTS.map((shortcut) => [shortcut.id, shortcut]));

export function isShortcutPressed(id: string, input: string, key: InputKey): boolean {
  const shortcut = SHORTCUT_BY_ID.get(id);
  if (!shortcut) return false;
  return shortcut.match(input, key);
}

function pick(ids: string[]): Array<Pick<ShortcutDefinition, "id" | "keys" | "action">> {
  return ids
    .map((id) => SHORTCUT_BY_ID.get(id))
    .filter((shortcut): shortcut is ShortcutDefinition => Boolean(shortcut))
    .map(({ id, keys, action }) => ({ id, keys, action }));
}

export function getShortcutSections(activeTab: TabId): ShortcutSection[] {
  const globalSections: ShortcutSection[] = [
    {
      title: "Global",
      items: pick([
        "settings-open",
        "tab-prev",
        "tab-next",
        "tab-timetable",
        "tab-absences",
        "logout",
        "quit",
      ]),
    },
    {
      title: "Settings Modal",
      items: pick(["settings-close"]),
    },
  ];

  if (activeTab === "timetable") {
    return [
      ...globalSections,
      {
        title: "Timetable",
        items: pick([
          "timetable-week-prev",
          "timetable-week-next",
          "timetable-day-prev",
          "timetable-day-next",
          "timetable-up",
          "timetable-down",
          "timetable-cycle-overlap",
          "timetable-today",
          "timetable-refresh",
        ]),
      },
    ];
  }

  return [
    ...globalSections,
    {
      title: "Absences",
      items: pick([
        "absences-up",
        "absences-down",
        "absences-page-up",
        "absences-page-down",
        "absences-home",
        "absences-end",
        "absences-status",
        "absences-window",
        "absences-search",
        "absences-clear",
        "absences-load-more",
        "absences-refresh",
      ]),
    },
    {
      title: "Absences Search Input",
      items: pick(["absences-search-submit", "absences-search-cancel"]),
    },
  ];
}
