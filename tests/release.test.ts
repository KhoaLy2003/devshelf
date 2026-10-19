import { describe, expect, it } from "vitest";
import {
  extractChangelogSection,
  insertSection,
  nextVersion,
  noteItems,
  normalizeNotes,
  renderSection,
} from "../scripts/release-lib.mjs";

describe("nextVersion", () => {
  it("bumps each part and resets the lower ones", () => {
    expect(nextVersion("0.3.0", "patch")).toBe("0.3.1");
    expect(nextVersion("0.3.4", "minor")).toBe("0.4.0");
    expect(nextVersion("0.3.4", "major")).toBe("1.0.0");
  });

  it("accepts an explicit version only when it is higher", () => {
    expect(nextVersion("0.3.0", "0.5.0")).toBe("0.5.0");
    expect(() => nextVersion("0.3.0", "0.3.0")).toThrow(/must be greater/);
    expect(() => nextVersion("0.3.0", "0.2.9")).toThrow(/must be greater/);
  });
});

describe("normalizeNotes", () => {
  it("trims surrounding whitespace", () => {
    expect(normalizeNotes("\n\n### Fixed\n\n- A thing\n\n")).toBe("### Fixed\n\n- A thing");
  });

  it("rejects empty notes so a release never ships without them", () => {
    expect(() => normalizeNotes("  \n ")).toThrow(/Release notes are empty/);
  });

  it("rejects ## headings, which would break the CHANGELOG layout", () => {
    expect(() => normalizeNotes("## Fixed\n- x")).toThrow(/must not use ## headings/);
  });

  it("rejects quotes and backslashes, which the generated TypeScript cannot hold verbatim", () => {
    expect(() => normalizeNotes('- say "hi"')).toThrow(/quotes or backslashes/);
    expect(() => normalizeNotes("- C:\\path")).toThrow(/quotes or backslashes/);
  });
});

describe("renderSection", () => {
  it("writes a versioned heading followed by the notes", () => {
    expect(renderSection("0.3.1", "2026-10-19", "### Fixed\n\n- A thing\n")).toBe(
      "## [0.3.1] - 2026-10-19\n\n### Fixed\n\n- A thing\n",
    );
  });
});

describe("noteItems", () => {
  it("returns the bullet text, skipping headings and prose", () => {
    const notes = "### Added\n\nIntro line.\n\n- First\n- Second\n\n### Fixed\n\n- Third\n";
    expect(noteItems(notes)).toEqual(["First", "Second", "Third"]);
  });
});

describe("insertSection", () => {
  it("puts the new section above the previous release", () => {
    const existing = "# Changelog\n\n## [0.1.0] - 2026-10-17\n\n- old\n";
    const result = insertSection(existing, "## [0.2.0] - 2026-10-18\n\n- new\n");
    expect(result.indexOf("## [0.2.0]")).toBeLessThan(result.indexOf("## [0.1.0]"));
  });
});

describe("extractChangelogSection", () => {
  it("returns only the body of the requested version", () => {
    const changelog = "# Changelog\n\n## [0.2.0] - d\n\n- two\n\n## [0.1.0] - d\n\n- one\n";
    expect(extractChangelogSection(changelog, "0.2.0")).toBe("- two");
  });

  it("throws when the version has no section", () => {
    expect(() => extractChangelogSection("# Changelog\n", "9.9.9")).toThrow(/No CHANGELOG section/);
  });
});
