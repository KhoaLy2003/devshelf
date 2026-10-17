import { describe, expect, it } from "vitest";
import { HELP_SECTIONS, getSection } from "../src/lib/help-content";

describe("HELP_SECTIONS", () => {
  it("has non-empty id, title and body for every section", () => {
    expect(HELP_SECTIONS.length).toBeGreaterThan(0);
    for (const s of HELP_SECTIONS) {
      expect(s.id.trim()).not.toBe("");
      expect(s.title.trim()).not.toBe("");
      expect(s.body.length).toBeGreaterThan(0);
      for (const p of s.body) expect(p.trim()).not.toBe("");
    }
  });

  it("has unique ids", () => {
    const ids = HELP_SECTIONS.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe("getSection", () => {
  it("returns the matching section", () => {
    expect(getSection("vault").title).toBe("Vault");
  });
  it("falls back to the first section", () => {
    expect(getSection("nope")).toBe(HELP_SECTIONS[0]);
    expect(getSection(undefined)).toBe(HELP_SECTIONS[0]);
  });
});
