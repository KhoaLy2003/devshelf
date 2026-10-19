import { describe, expect, it } from "vitest";
import {
  readCommittedChangelogData,
  readReleases,
  renderChangelogData,
} from "../scripts/gen-changelog.mjs";

describe("generated in-app changelog", () => {
  it("is exactly what the generator produces from release-notes/", () => {
    expect(readCommittedChangelogData()).toBe(renderChangelogData(readReleases()));
  });

  it("lists released versions newest first and excludes unreleased.md", () => {
    const releases: { version: string; items: string[] }[] = readReleases();
    const versions = releases.map((release) => release.version);
    expect(versions).not.toContain("unreleased");
    expect(versions).toEqual(
      [...versions].sort((a, b) => {
        const [aMajor, aMinor, aPatch] = a.split(".").map(Number);
        const [bMajor, bMinor, bPatch] = b.split(".").map(Number);
        return bMajor - aMajor || bMinor - aMinor || bPatch - aPatch;
      }),
    );
    expect(releases.every((release) => release.items.length > 0)).toBe(true);
  });
});
