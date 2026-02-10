export function truncateText(value: string, maxWidth: number): string {
  if (maxWidth <= 0) return "";
  if (value.length <= maxWidth) return value;
  if (maxWidth <= 3) return value.slice(0, maxWidth);
  return `${value.slice(0, maxWidth - 3)}...`;
}

export function fitText(value: string, width: number): string {
  return truncateText(value, width).padEnd(Math.max(0, width), " ");
}

export function centerText(value: string, width: number): string {
  const clipped = truncateText(value, width);
  const pad = Math.max(0, width - clipped.length);
  const left = Math.floor(pad / 2);
  const right = pad - left;
  return `${" ".repeat(left)}${clipped}${" ".repeat(right)}`;
}

export function buildGridDivider(
  timeWidth: number,
  dayWidth: number,
  dayCount: number,
  junction: string,
): string {
  return (
    "─".repeat(timeWidth) +
    Array.from({ length: dayCount }, () => `${junction}${"─".repeat(Math.max(1, dayWidth - 1))}`).join(
      "",
    )
  );
}
