import { describe, expect, it } from "vitest";
import { readChangelogTopVersion, readVersions } from "../scripts/release-lib.mjs";

describe("version sync", () => {
  const versions = readVersions();

  it("keeps every version location identical", () => {
    const unique = new Set(Object.values(versions));
    expect(unique.size).toBe(1);
  });

  it("matches the newest CHANGELOG.md release heading", () => {
    expect(readChangelogTopVersion()).toBe(Object.values(versions)[0]);
  });
});
