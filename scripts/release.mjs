import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  VERSION_FILES,
  applyVersion,
  insertSection,
  nextVersion,
  parseCommit,
  readVersions,
  renderSection,
  repoRoot,
} from "./release-lib.mjs";

const USAGE = "Usage: npm run release -- <patch|minor|major|X.Y.Z> [--dry-run] [--since <ref>]";
const KEYWORD_BUMPS = ["patch", "minor", "major"];
const EXPLICIT_RE = /^\d+\.\d+\.\d+$/;

function git(args) {
  return execFileSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function parseArgs(argv) {
  let bump;
  let dryRun = false;
  let since = null;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--dry-run") {
      dryRun = true;
    } else if (arg === "--since") {
      since = argv[++i];
      if (!since) throw new Error(USAGE);
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
  return { bump, dryRun, since };
}

function resolveBase(since) {
  if (since) return since;
  try {
    return git(["describe", "--tags", "--abbrev=0", "--match", "v*"]).trim();
  } catch {
    throw new Error("No v* tag found; pass --since <ref>");
  }
}

function readCommits(base) {
  const raw = git(["log", `${base}..HEAD`, "--no-merges", "--format=%H%x1f%s%x1f%b%x1e"]);
  return raw
    .split("\x1e")
    .map((record) => record.trim())
    .filter((record) => record !== "")
    .map((record) => {
      const [hash, subject, body = ""] = record.split("\x1f");
      return parseCommit({ hash, subject, body });
    })
    .filter((commit) => commit !== null);
}

function main() {
  const { bump, dryRun, since } = parseArgs(process.argv.slice(2));

  const versions = readVersions();
  const [, current] = Object.entries(versions)[0];
  const disagree = Object.values(versions).some((value) => value !== current);
  if (disagree) {
    for (const [file, value] of Object.entries(versions)) {
      console.error(`  ${file}: ${value}`);
    }
    throw new Error("Version files disagree");
  }

  if (!dryRun && git(["status", "--porcelain"]).trim() !== "") {
    throw new Error("Working tree is not clean");
  }

  const base = resolveBase(since);
  const commits = readCommits(base);

  const version = nextVersion(current, bump);
  if (git(["tag", "--list", `v${version}`]).trim() !== "") {
    throw new Error(`Tag v${version} already exists`);
  }

  const date = new Date().toLocaleDateString("en-CA");
  const section = renderSection(version, date, commits);
  if (section === "") {
    throw new Error(`No releasable commits since ${base}`);
  }

  const files = [...VERSION_FILES, "CHANGELOG.md"];

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

  git(["add", ...files]);
  git(["commit", "-m", `chore(release): v${version}`]);
  git(["tag", "-a", `v${version}`, "-m", `DevShelf ${version}`]);

  console.log(`Released v${version}.`);
  console.log("Push with: git push origin HEAD --follow-tags");
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
