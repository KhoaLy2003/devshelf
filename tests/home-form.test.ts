import { describe, expect, it } from "vitest";
import {
  capList,
  emptyPinnedText,
  emptyRecentText,
  lastUsedLabel,
  missingWarning,
  workspaceItemsLabel,
} from "../src/lib/home-form";
import { REGISTER_PROJECT_STATE } from "../src/lib/home-nav";

describe("workspaceItemsLabel", () => {
  it("pluralises", () => {
    expect(workspaceItemsLabel(0)).toBe("0 items");
    expect(workspaceItemsLabel(1)).toBe("1 item");
    expect(workspaceItemsLabel(2)).toBe("2 items");
  });
});

describe("missingWarning", () => {
  it("is null for zero and pluralises otherwise", () => {
    expect(missingWarning(0)).toBeNull();
    expect(missingWarning(1)).toBe("1 item missing at last restore");
    expect(missingWarning(2)).toBe("2 items missing at last restore");
  });
});

describe("capList", () => {
  const make = (n: number) => Array.from({ length: n }, (_, i) => i);

  it("shows everything for an empty list", () => {
    expect(capList([], 6)).toEqual({ shown: [], hiddenCount: 0 });
  });

  it("shows exactly max items without hiding any", () => {
    expect(capList(make(6), 6)).toEqual({ shown: make(6), hiddenCount: 0 });
  });

  it("hides the items beyond max", () => {
    expect(capList(make(7), 6)).toEqual({ shown: make(6), hiddenCount: 1 });
  });
});

describe("lastUsedLabel", () => {
  it("says never restored for null", () => {
    expect(lastUsedLabel(null)).toBe("Never restored");
  });

  it("uses the injected formatter on the parsed date", () => {
    const iso = "2026-03-04T10:20:30Z";
    const seen: Date[] = [];
    const label = lastUsedLabel(iso, (d) => {
      seen.push(d);
      return "04/03/2026, 10:20";
    });
    expect(label).toBe("Last restored 04/03/2026, 10:20");
    expect(seen).toHaveLength(1);
    expect(seen[0]?.toISOString()).toBe("2026-03-04T10:20:30.000Z");
  });
});

describe("copy", () => {
  it("matches the spec exactly", () => {
    expect(emptyPinnedText).toBe(
      "No pinned projects. Pin a project from its detail page to see it here.",
    );
    expect(emptyRecentText).toBe(
      "No recent workspaces. A workspace saves files and folders you reopen together.",
    );
  });
});

describe("REGISTER_PROJECT_STATE", () => {
  it("asks the projects screen to open Register", () => {
    expect(REGISTER_PROJECT_STATE).toEqual({ openRegister: true });
  });
});
