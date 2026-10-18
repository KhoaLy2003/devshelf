import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const repoRoot = resolve(here, "..");

// Repo-relative paths of every file that carries the app version.
export const VERSION_FILES = [
  "package.json",
  "package-lock.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "src-tauri/tauri.conf.json",
  "src/lib/help-content.ts",
];

// Curated, user-facing notes. Feature PRs add bullets to unreleased.md; the release moves them
// into vX.Y.Z.md, which is the source for the CHANGELOG section, the GitHub release body and
// the in-app changelog.
export const NOTES_DIR = "release-notes";
export const UNRELEASED_NOTES = `${NOTES_DIR}/unreleased.md`;

const SEMVER_RE = /^\d+\.\d+\.\d+$/;

export function notesFileFor(version) {
  return `${NOTES_DIR}/v${version}.md`;
}

function parts(version) {
  return version.split(".").map(Number);
}

export function compareVersions(a, b) {
  const pa = parts(a);
  const pb = parts(b);
  for (let i = 0; i < 3; i++) {
    if (pa[i] !== pb[i]) return pa[i] - pb[i];
  }
  return 0;
}

export function nextVersion(current, bump) {
  const [major, minor, patch] = parts(current);
  if (bump === "patch") return `${major}.${minor}.${patch + 1}`;
  if (bump === "minor") return `${major}.${minor + 1}.0`;
  if (bump === "major") return `${major + 1}.0.0`;
  if (!SEMVER_RE.test(bump)) {
    throw new Error(`Invalid version: ${bump}`);
  }
  if (compareVersions(bump, current) <= 0) {
    throw new Error(`Version ${bump} must be greater than ${current}`);
  }
  return bump;
}

// Trims the notes and rejects anything that would break the CHANGELOG layout or the generated
// in-app file: empty notes, "## " headings (the CHANGELOG uses them for versions), and quote or
// backslash characters (the generated TypeScript keeps strings verbatim).
export function normalizeNotes(text) {
  const trimmed = text.trim();
  if (trimmed === "") {
    throw new Error("Release notes are empty: add bullets to release-notes/unreleased.md");
  }
  if (/^## /m.test(trimmed)) {
    throw new Error("Release notes must not use ## headings; use ### for subsections");
  }
  if (/["\\]/.test(trimmed)) {
    throw new Error("Release notes must not contain double quotes or backslashes");
  }
  return trimmed;
}

export function renderSection(version, date, notes) {
  return `## [${version}] - ${date}\n\n${normalizeNotes(notes)}\n`;
}

// Bullet text of one release's notes, for the in-app changelog.
export function noteItems(notes) {
  return notes
    .split(/\r?\n/)
    .filter((line) => line.startsWith("- "))
    .map((line) => line.slice(2).trim());
}

export function insertSection(changelog, section) {
  const index = changelog.search(/^## \[/m);
  if (index !== -1) {
    return changelog.slice(0, index) + section + changelog.slice(index);
  }
  const base = changelog.replace(/\s+$/, "");
  return base === "" ? section : `${base}\n\n${section}`;
}

function readText(root, relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
}

function readOrThrow(pattern, text, relativePath) {
  const match = pattern.exec(text);
  if (!match) throw new Error(`Cannot read version from ${relativePath}`);
  return match[1];
}

export function readVersions(root = repoRoot) {
  const packageJson = JSON.parse(readText(root, "package.json"));
  const lock = JSON.parse(readText(root, "package-lock.json"));
  const rootLockVersion = lock.packages?.[""]?.version;
  if (lock.version !== rootLockVersion) {
    throw new Error(`package-lock.json versions disagree: ${lock.version} vs ${rootLockVersion}`);
  }
  const tauriConf = JSON.parse(readText(root, "src-tauri/tauri.conf.json"));

  return {
    "package.json": packageJson.version,
    "package-lock.json": lock.version,
    "Cargo.toml": readOrThrow(
      /^version = "(.+)"$/m,
      readText(root, "src-tauri/Cargo.toml"),
      "src-tauri/Cargo.toml",
    ),
    "Cargo.lock": readOrThrow(
      /name = "devshelf"\nversion = "([^"]+)"/,
      readText(root, "src-tauri/Cargo.lock"),
      "src-tauri/Cargo.lock",
    ),
    "tauri.conf.json": tauriConf.version,
    "help-content.ts": readOrThrow(
      /^export const APP_VERSION = "(.+)";$/m,
      readText(root, "src/lib/help-content.ts"),
      "src/lib/help-content.ts",
    ),
  };
}

export function readChangelogTopVersion(root = repoRoot) {
  return readOrThrow(/^## \[(\d+\.\d+\.\d+)\]/m, readText(root, "CHANGELOG.md"), "CHANGELOG.md");
}

// Body text of one CHANGELOG version section, without its heading. Used as the GitHub Release notes.
export function extractChangelogSection(changelog, version) {
  const lines = changelog.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith(`## [${version}]`));
  if (start === -1) {
    throw new Error(`No CHANGELOG section for ${version}`);
  }
  let end = lines.findIndex((line, index) => index > start && line.startsWith("## ["));
  if (end === -1) {
    end = lines.length;
  }
  return lines
    .slice(start + 1, end)
    .join("\n")
    .trim();
}

export function readChangelogSection(version, root = repoRoot) {
  return extractChangelogSection(readText(root, "CHANGELOG.md"), version);
}

// Replaces the first match of `pattern`; throws if nothing matched so a
// layout change can never silently leave a stale version behind.
function replaceFirst(root, relativePath, pattern, replacement) {
  const text = readText(root, relativePath);
  const next = text.replace(pattern, replacement);
  if (next === text) {
    throw new Error(`Cannot write version to ${relativePath}`);
  }
  writeFileSync(join(root, relativePath), next, "utf8");
}

export function applyVersion(root, version) {
  replaceFirst(root, "package.json", /"version": "[^"]*"/, () => `"version": "${version}"`);
  replaceFirst(
    root,
    "src-tauri/tauri.conf.json",
    /"version": "[^"]*"/,
    () => `"version": "${version}"`,
  );
  replaceFirst(root, "src-tauri/Cargo.toml", /^version = ".*"$/m, () => `version = "${version}"`);
  replaceFirst(
    root,
    "src-tauri/Cargo.lock",
    /(name = "devshelf"\nversion = ")[^"]+(")/,
    (_m, a, b) => `${a}${version}${b}`,
  );
  replaceFirst(
    root,
    "src/lib/help-content.ts",
    /^export const APP_VERSION = ".+";$/m,
    () => `export const APP_VERSION = "${version}";`,
  );

  const lockPath = "package-lock.json";
  const lock = JSON.parse(readText(root, lockPath));
  lock.version = version;
  if (!lock.packages?.[""]) {
    throw new Error(`Cannot write version to ${lockPath}`);
  }
  lock.packages[""].version = version;
  writeFileSync(join(root, lockPath), JSON.stringify(lock, null, 2) + "\n", "utf8");
}
