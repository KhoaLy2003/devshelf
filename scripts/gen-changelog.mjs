import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { NOTES_DIR, compareVersions, noteItems, repoRoot } from "./release-lib.mjs";

export const CHANGELOG_DATA_PATH = "src/lib/changelog-data.ts";

// Released versions only (unreleased.md is excluded), newest first.
export function readReleases(root = repoRoot) {
  const dir = join(root, NOTES_DIR);
  return readdirSync(dir)
    .map((name) => /^v(\d+\.\d+\.\d+)\.md$/.exec(name))
    .filter((match) => match !== null)
    .map(([file, version]) => ({
      version,
      items: noteItems(readFileSync(join(dir, file), "utf8")),
    }))
    .sort((a, b) => compareVersions(b.version, a.version));
}

// Output is kept in the exact shape Prettier produces, so `npm run lint` passes on it unchanged.
export function renderChangelogData(releases) {
  const lines = [
    "// Generated from release-notes/v*.md by scripts/gen-changelog.mjs. Do not edit by hand.",
    "",
    "export interface ReleaseNotes {",
    "  version: string;",
    "  items: readonly string[];",
    "}",
    "",
    "export const CHANGELOG_RELEASES: readonly ReleaseNotes[] = [",
  ];
  for (const release of releases) {
    lines.push("  {", `    version: "${release.version}",`, "    items: [");
    for (const item of release.items) {
      if (/["\\]/.test(item)) {
        throw new Error(`Release note item must not contain quotes or backslashes: ${item}`);
      }
      lines.push(`      "${item}",`);
    }
    lines.push("    ],", "  },");
  }
  lines.push("];", "");
  return lines.join("\n");
}

export function writeChangelogData(root = repoRoot) {
  writeFileSync(join(root, CHANGELOG_DATA_PATH), renderChangelogData(readReleases(root)), "utf8");
}

export function readCommittedChangelogData(root = repoRoot) {
  return readFileSync(join(root, CHANGELOG_DATA_PATH), "utf8");
}

if (process.argv[1] && process.argv[1].endsWith("gen-changelog.mjs")) {
  writeChangelogData();
  console.log(`Wrote ${CHANGELOG_DATA_PATH}`);
}
