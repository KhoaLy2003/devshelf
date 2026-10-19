import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { CHANGELOG_DATA_PATH, writeChangelogData } from "./gen-changelog.mjs";
import {
  UNRELEASED_NOTES,
  VERSION_FILES,
  applyVersion,
  insertSection,
  nextVersion,
  notesFileFor,
  normalizeNotes,
  readVersions,
  renderSection,
  repoRoot,
} from "./release-lib.mjs";

const USAGE = "Usage: npm run release -- <patch|minor|major|X.Y.Z> [--dry-run]";
const KEYWORD_BUMPS = ["patch", "minor", "major"];
const EXPLICIT_RE = /^\d+\.\d+\.\d+$/;

function parseArgs(argv) {
  let bump;
  let dryRun = false;
  for (const arg of argv) {
    if (arg === "--dry-run") {
      dryRun = true;
    } else if (arg.startsWith("--")) {
      throw new Error(`Unknown option: ${arg}\n${USAGE}`);
    } else if (bump === undefined) {
      bump = arg;
    } else {
      throw new Error(`Unexpected argument: ${arg}\n${USAGE}`);
    }
  }
  if (bump === undefined) throw new Error(USAGE);
  if (!KEYWORD_BUMPS.includes(bump) && !EXPLICIT_RE.test(bump)) {
    throw new Error(`Unknown bump: ${bump}\n${USAGE}`);
  }
  return { bump, dryRun };
}

// Edits files only. Committing, branching and tagging happen in the release workflow, so the
// release commit is always the one the reviewed PR merged.
function main() {
  const { bump, dryRun } = parseArgs(process.argv.slice(2));

  const versions = readVersions();
  const [, current] = Object.entries(versions)[0];
  if (Object.values(versions).some((value) => value !== current)) {
    for (const [file, value] of Object.entries(versions)) {
      console.error(`  ${file}: ${value}`);
    }
    throw new Error("Version files disagree");
  }

  const version = nextVersion(current, bump);
  const unreleasedPath = join(repoRoot, UNRELEASED_NOTES);
  const notes = normalizeNotes(
    existsSync(unreleasedPath) ? readFileSync(unreleasedPath, "utf8") : "",
  );
  const section = renderSection(version, new Date().toLocaleDateString("en-CA"), notes);

  const files = [
    ...VERSION_FILES,
    "CHANGELOG.md",
    UNRELEASED_NOTES,
    notesFileFor(version),
    CHANGELOG_DATA_PATH,
  ];

  if (dryRun) {
    console.log(`${current} -> ${version}\n`);
    console.log(section);
    console.log("Files that would change:");
    for (const file of files) console.log(`  ${file}`);
    return;
  }

  applyVersion(repoRoot, version);

  const changelogPath = join(repoRoot, "CHANGELOG.md");
  const existing = existsSync(changelogPath) ? readFileSync(changelogPath, "utf8") : "";
  writeFileSync(changelogPath, insertSection(existing, section), "utf8");
  writeFileSync(join(repoRoot, notesFileFor(version)), notes + "\n", "utf8");
  writeFileSync(unreleasedPath, "", "utf8");
  writeChangelogData();

  console.log(`Prepared v${version}. Changed files:`);
  for (const file of files) console.log(`  ${file}`);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
