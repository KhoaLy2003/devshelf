import { describe, expect, it } from "vitest";
import {
  extractChangelogSection,
  insertSection,
  nextVersion,
  parseCommit,
  renderSection,
} from "../scripts/release-lib.mjs";

const hash = "0123456789abcdef0123456789abcdef01234567";

describe("parseCommit", () => {
  it("reads type and scope from a scoped subject", () => {
    expect(parseCommit({ hash, subject: "feat(projects): add x" })).toMatchObject({
      type: "feat",
      scope: "projects",
      summary: "add x",
      breaking: false,
    });
  });

  it("treats a bang as breaking", () => {
    expect(parseCommit({ hash, subject: "fix!: y" })).toMatchObject({
      type: "fix",
      breaking: true,
    });
  });

  it("treats a BREAKING CHANGE footer as breaking", () => {
    expect(
      parseCommit({
        hash,
        subject: "feat: z",
        body: "details\n\nBREAKING CHANGE: z is gone",
      }),
    ).toMatchObject({ breaking: true });
  });

  it("leaves scope null when absent and shortens the hash", () => {
    expect(parseCommit({ hash, subject: "feat: a" })).toMatchObject({
      scope: null,
      short: "0123456",
    });
  });

  it("ignores subjects that are not Conventional Commits", () => {
    expect(parseCommit({ hash, subject: "Merge pull request #1 from x/y" })).toBeNull();
  });
});

describe("renderSection", () => {
  const commits = [
    parseCommit({ hash, subject: "chore: bump deps" }),
    parseCommit({ hash, subject: "refactor(vault): split module" }),
    parseCommit({ hash, subject: "fix(projects): keep ids" }),
    parseCommit({ hash, subject: "docs: readme" }),
    parseCommit({ hash, subject: "feat(home): add overview" }),
  ].filter((commit) => commit !== null);

  it("orders Added, Fixed, Changed and omits docs and chore", () => {
    const section = renderSection("0.2.0", "2026-01-02", commits);
    const added = section.indexOf("### Added");
    const fixed = section.indexOf("### Fixed");
    const changed = section.indexOf("### Changed");
    expect(added).toBeGreaterThan(-1);
    expect(added).toBeLessThan(fixed);
    expect(fixed).toBeLessThan(changed);
    expect(section).not.toContain("bump deps");
    expect(section).not.toContain("readme");
  });

  it("lists breaking commits only under Breaking changes", () => {
    const breaking = parseCommit({ hash, subject: "feat(config)!: drop old key" });
    const section = renderSection("0.2.0", "2026-01-02", [breaking, ...commits]);
    const breakingIndex = section.indexOf("### Breaking changes");
    const addedIndex = section.indexOf("### Added");
    expect(breakingIndex).toBeGreaterThan(-1);
    expect(section.slice(breakingIndex, addedIndex)).toContain("**config:** drop old key");
    expect(section.slice(addedIndex)).not.toContain("drop old key");
  });

  it("returns an empty string when nothing is releasable", () => {
    const docsOnly = [parseCommit({ hash, subject: "docs: readme" })];
    expect(renderSection("0.2.0", "2026-01-02", docsOnly)).toBe("");
  });
});

describe("nextVersion", () => {
  it("bumps by keyword", () => {
    expect(nextVersion("0.1.0", "minor")).toBe("0.2.0");
    expect(nextVersion("0.1.0", "patch")).toBe("0.1.1");
    expect(nextVersion("0.1.0", "major")).toBe("1.0.0");
  });

  it("accepts an explicit version compared numerically", () => {
    expect(nextVersion("0.9.0", "0.10.0")).toBe("0.10.0");
  });

  it("rejects versions that are not greater than current", () => {
    expect(() => nextVersion("0.1.0", "0.1.0")).toThrow("must be greater than");
    expect(() => nextVersion("0.1.0", "0.0.9")).toThrow("must be greater than");
  });

  it("rejects malformed versions", () => {
    expect(() => nextVersion("0.1.0", "abc")).toThrow("Invalid version");
  });
});

describe("insertSection", () => {
  it("places the new section above the existing one and keeps the intro first", () => {
    const existing = [
      "# Changelog",
      "",
      "Intro line.",
      "",
      "## [0.1.0] - 2026-01-01",
      "",
      "### Added",
      "",
      "- old",
      "",
    ].join("\n");
    const section = "## [0.2.0] - 2026-02-01\n\n### Added\n\n- new\n\n";
    const result = insertSection(existing, section);
    expect(result.startsWith("# Changelog\n\nIntro line.\n\n## [0.2.0]")).toBe(true);
    expect(result.indexOf("## [0.2.0]")).toBeLessThan(result.indexOf("## [0.1.0]"));
  });
});

describe("extractChangelogSection", () => {
  const changelog = [
    "# Changelog",
    "",
    "## [0.2.0] - 2026-10-18",
    "",
    "### Added",
    "- New thing",
    "",
    "## [0.1.0] - 2026-10-01",
    "",
    "### Fixed",
    "- Old fix",
    "",
  ].join("\n");

  it("returns the body of a non-last section without its heading", () => {
    expect(extractChangelogSection(changelog, "0.2.0")).toBe("### Added\n- New thing");
  });

  it("returns the last section through end of file", () => {
    expect(extractChangelogSection(changelog, "0.1.0")).toBe("### Fixed\n- Old fix");
  });

  it("throws when the version has no section", () => {
    expect(() => extractChangelogSection(changelog, "9.9.9")).toThrow(
      "No CHANGELOG section for 9.9.9",
    );
  });
});
