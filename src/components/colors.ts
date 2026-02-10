const SUBJECT_STRIPE_COLORS = [
  "ansi256(45)",
  "ansi256(41)",
  "ansi256(220)",
  "ansi256(201)",
  "ansi256(39)",
  "ansi256(196)",
  "ansi256(15)",
] as const;

export const COLORS = {
  brand: "ansi256(45)",
  warning: "ansi256(220)",
  error: "ansi256(196)",
  info: "ansi256(201)",
  neutral: {
    white: "ansi256(15)",
    black: "ansi256(16)",
    gray: "ansi256(244)",
    brightBlack: "ansi256(240)",
  },
  selection: {
    emptyCellBackground: "ansi256(236)",
  },
  lesson: {
    byType: {
      cancelled: {
        background: {
          base: "ansi256(167)",
          focused: "ansi256(124)",
        },
        text: {
          title: "ansi256(255)",
          subtext: "ansi256(252)",
          focusedTitle: "ansi256(255)",
          focusedSubtext: "ansi256(252)",
        },
      },
      exam: {
        background: {
          base: "ansi256(179)",
          focused: "ansi256(172)",
        },
        text: {
          title: "ansi256(16)",
          subtext: "ansi256(236)",
          focusedTitle: "ansi256(16)",
          focusedSubtext: "ansi256(236)",
        },
      },
      substitution: {
        background: {
          base: "ansi256(35)",
          focused: "ansi256(28)",
        },
        text: {
          title: "ansi256(255)",
          subtext: "ansi256(251)",
          focusedTitle: "ansi256(255)",
          focusedSubtext: "ansi256(251)",
        },
      },
      default: {
        background: {
          base: "ansi256(238)",
          focused: "ansi256(236)",
        },
        text: {
          title: "ansi256(255)",
          subtext: "ansi256(250)",
          focusedTitle: "ansi256(255)",
          focusedSubtext: "ansi256(250)",
        },
      },
    },
  },
  cellStateChip: {
    exam: { backgroundColor: "ansi256(179)", color: "ansi256(16)" },
    confirmed: { backgroundColor: "ansi256(35)", color: "ansi256(255)" },
    substitution: { backgroundColor: "ansi256(35)", color: "ansi256(255)" },
    cancelled: { backgroundColor: "ansi256(167)", color: "ansi256(255)" },
    default: { backgroundColor: "ansi256(238)", color: "ansi256(255)" },
  },
  subjectStripeCycle: SUBJECT_STRIPE_COLORS,
} as const;

export function getCellStateChipColors(cellState: string) {
  const normalized = cellState.trim().toUpperCase();

  switch (normalized) {
    case "EXAM":
      return COLORS.cellStateChip.exam;
    case "CONFIRMED":
      return COLORS.cellStateChip.confirmed;
    case "SUBSTITUTION":
    case "ADDITIONAL":
      return COLORS.cellStateChip.substitution;
    case "CANCELLED":
      return COLORS.cellStateChip.cancelled;
    default:
      return COLORS.cellStateChip.default;
  }
}
