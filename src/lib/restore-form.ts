import type { RestoreOutcomeView, WorkspaceCode } from "../types/commands";
import { formatOpenError } from "./editor-form";
import { formatWorkspaceError } from "./workspace-form";

type Missing = Extract<RestoreOutcomeView, { kind: "missing" }>;
type Unreachable = Extract<RestoreOutcomeView, { kind: "unreachable" }>;
type CouldNotOpen = Extract<RestoreOutcomeView, { kind: "could_not_open" }>;

export interface OutcomeGroups {
  missing: Missing[];
  unreachable: Unreachable[];
  couldNotOpen: CouldNotOpen[];
}

export function restoreSummaryLine(
  opened: number,
  missing: number,
  unreachable: number,
  failed: number,
  ignored: number,
): string {
  if (missing === 0 && unreachable === 0 && failed === 0 && ignored === 0) {
    return `All ${opened} items opened.`;
  }
  const missingText =
    ignored > 0 ? `${missing} missing (ignored ${ignored})` : `${missing} missing`;
  return `${opened} opened, ${missingText}, ${unreachable} unreachable, ${failed} could not open.`;
}

// Preserves the incoming (position) order within each group.
export function groupOutcomes(outcomes: RestoreOutcomeView[]): OutcomeGroups {
  const groups: OutcomeGroups = { missing: [], unreachable: [], couldNotOpen: [] };
  for (const outcome of outcomes) {
    switch (outcome.kind) {
      case "missing":
        groups.missing.push(outcome);
        break;
      case "unreachable":
        groups.unreachable.push(outcome);
        break;
      case "could_not_open":
        groups.couldNotOpen.push(outcome);
        break;
      case "opened":
        break;
    }
  }
  return groups;
}

export function outcomeReason(outcome: RestoreOutcomeView): string {
  switch (outcome.kind) {
    case "opened":
      return "";
    case "missing":
      switch (outcome.reason) {
        case "not_found":
          return "Path not found.";
        case "now_folder":
          return "Now a folder, not a file.";
        case "now_file":
          return "Now a file, not a folder.";
      }
      break;
    case "unreachable":
      return "Can't be reached. The drive may not be ready or the network path is unavailable.";
    case "could_not_open":
      return formatOpenError(outcome.code, null, "The editor", null);
  }
}

// `detail` is the path for path codes and the stored item kind for item_kind_mismatch.
export function formatRelinkError(code: WorkspaceCode, detail: string | null): string {
  switch (code) {
    case "item_kind_mismatch":
      return detail === "folder"
        ? "This is a file. Pick a folder."
        : "This is a folder. Pick a file.";
    default:
      return formatWorkspaceError(code, detail, "");
  }
}

export function relinkedNotice(): string {
  return "Relinked. It will open on the next restore.";
}
