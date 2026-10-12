import { describe, expect, it } from "vitest";
import {
  formatRelinkError,
  groupOutcomes,
  outcomeReason,
  relinkedNotice,
  restoreSummaryLine,
} from "../src/lib/restore-form";
import type { RestoreOutcomeView } from "../src/types/commands";

const opened = (id: number): RestoreOutcomeView => ({
  kind: "opened",
  item_id: id,
  reason: null,
  code: null,
});
const missing = (
  id: number,
  reason: "not_found" | "now_folder" | "now_file" = "not_found",
): RestoreOutcomeView => ({ kind: "missing", item_id: id, reason, code: null });
const unreachable = (id: number): RestoreOutcomeView => ({
  kind: "unreachable",
  item_id: id,
  reason: null,
  code: null,
});
const failed = (id: number): RestoreOutcomeView => ({
  kind: "could_not_open",
  item_id: id,
  reason: null,
  code: "executable_not_found",
});

describe("restoreSummaryLine", () => {
  it("reports all items opened when nothing else happened", () => {
    expect(restoreSummaryLine(3, 0, 0, 0, 0)).toBe("All 3 items opened.");
  });

  it("uses the singular-free fixed wording for one item", () => {
    expect(restoreSummaryLine(1, 0, 0, 0, 0)).toBe("All 1 items opened.");
  });

  it("shows every count in a mixed run, zeros included", () => {
    expect(restoreSummaryLine(2, 1, 0, 0, 0)).toBe(
      "2 opened, 1 missing, 0 unreachable, 0 could not open.",
    );
    expect(restoreSummaryLine(0, 2, 1, 3, 0)).toBe(
      "0 opened, 2 missing, 1 unreachable, 3 could not open.",
    );
  });

  it("appends the ignored count to the missing count when above zero", () => {
    expect(restoreSummaryLine(2, 1, 1, 0, 2)).toBe(
      "2 opened, 1 missing (ignored 2), 1 unreachable, 0 could not open.",
    );
  });
});

describe("groupOutcomes", () => {
  it("groups in position order and drops opened outcomes", () => {
    const groups = groupOutcomes([
      missing(1),
      opened(2),
      unreachable(3),
      failed(4),
      missing(5, "now_file"),
      unreachable(6),
    ]);
    expect(groups.missing.map((o) => o.item_id)).toEqual([1, 5]);
    expect(groups.unreachable.map((o) => o.item_id)).toEqual([3, 6]);
    expect(groups.couldNotOpen.map((o) => o.item_id)).toEqual([4]);
  });

  it("returns empty arrays for empty groups", () => {
    expect(groupOutcomes([opened(1), opened(2)])).toEqual({
      missing: [],
      unreachable: [],
      couldNotOpen: [],
    });
    expect(groupOutcomes([])).toEqual({ missing: [], unreachable: [], couldNotOpen: [] });
  });
});

describe("outcomeReason", () => {
  it("explains each missing reason", () => {
    expect(outcomeReason(missing(1, "not_found"))).toBe("Path not found.");
    expect(outcomeReason(missing(1, "now_folder"))).toBe("Now a folder, not a file.");
    expect(outcomeReason(missing(1, "now_file"))).toBe("Now a file, not a folder.");
  });

  it("explains unreachable paths", () => {
    expect(outcomeReason(unreachable(1))).toBe(
      "Can't be reached. The drive may not be ready or the network path is unavailable.",
    );
  });

  it("uses the open copy for could_not_open codes", () => {
    expect(outcomeReason(failed(1))).toBe(
      "The editor could not start. Check the path in Settings.",
    );
    expect(
      outcomeReason({
        kind: "could_not_open",
        item_id: 1,
        reason: null,
        code: "profile_path_empty",
      }),
    ).toBe("Set the path to The editor in Settings to open files with it.");
  });

  it("returns an empty string for opened outcomes", () => {
    expect(outcomeReason(opened(1))).toBe("");
  });
});

describe("formatRelinkError", () => {
  it("tells the user which kind to pick on a kind mismatch", () => {
    expect(formatRelinkError("item_kind_mismatch", "file")).toBe("This is a folder. Pick a file.");
    expect(formatRelinkError("item_kind_mismatch", "folder")).toBe(
      "This is a file. Pick a folder.",
    );
  });

  it("formats path errors", () => {
    expect(formatRelinkError("item_path_not_found", "F:\\a.txt")).toBe(
      "`F:\\a.txt` was not found.",
    );
    expect(formatRelinkError("item_relative_path", null)).toBe(
      "Enter an absolute path, for example `F:\\work\\notes.txt`.",
    );
    expect(formatRelinkError("item_duplicate", "F:\\a.txt")).toBe(
      "`F:\\a.txt` is already in this workspace.",
    );
    expect(formatRelinkError("item_path_empty", null)).toBe("Enter a file or folder path.");
  });

  it("uses the unreachable copy for item_path_unreachable", () => {
    expect(formatRelinkError("item_path_unreachable", "Z:\\a.txt")).toBe(
      "`Z:\\a.txt` can't be reached. The drive may not be ready or the network path is unavailable.",
    );
  });

  it("falls back to the workspace copy for other codes", () => {
    expect(formatRelinkError("storage_unavailable", null)).toBe(
      "DevShelf cannot reach its data folder. Restart DevShelf and try again.",
    );
  });
});

describe("relinkedNotice", () => {
  it("returns the relink confirmation", () => {
    expect(relinkedNotice()).toBe("Relinked. It will open on the next restore.");
  });
});
