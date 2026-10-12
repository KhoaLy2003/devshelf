export const emptyPinnedText =
  "No pinned projects. Pin a project from its detail page to see it here.";
export const emptyRecentText =
  "No recent workspaces. A workspace saves files and folders you reopen together.";

const defaultFormatter = new Intl.DateTimeFormat("en-GB", {
  dateStyle: "medium",
  timeStyle: "short",
});

export function workspaceItemsLabel(count: number): string {
  return count === 1 ? "1 item" : `${count} items`;
}

export function missingWarning(count: number): string | null {
  if (count <= 0) return null;
  return count === 1 ? "1 item missing at last restore" : `${count} items missing at last restore`;
}

export function capList<T>(items: readonly T[], max: number): { shown: T[]; hiddenCount: number } {
  return {
    shown: items.slice(0, max),
    hiddenCount: Math.max(0, items.length - max),
  };
}

export function lastUsedLabel(
  iso: string | null,
  format: (date: Date) => string = (d) => defaultFormatter.format(d),
): string {
  if (iso === null) return "Never restored";
  return `Last restored ${format(new Date(iso))}`;
}
