# F06 — Config catalog: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F06 (Must, size M, depends on F02 and F03 — both shipped locally)
- **Feature slug**: `config-catalog`
- **Spec mode**: Full. No designs in `docs/designs/config-catalog/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` and `docs/preview/index.html` (Config Catalog: filters, active chips, Clear filters, rows with status, Open with profile menu, Relink), `design_md_fallback`.

---

## 1. Goal

The user registers the config files each project uses (for example `application-staging.yml`), tagged with environment and kind. The catalog lists them, filters by project, environment, kind and status, and shows whether each file is Available, Missing, or Inaccessible, with the reason in plain language. The user opens a file in a chosen editor, or relinks it when it has moved. DevShelf stores paths and labels only; it never reads or changes file contents.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| C1 | Config Catalog | `/config` | Replaces the F01 empty state when entries exist |
| C2 | Add config file dialog | modal on C1 | Path, project, environment, kind |
| C3 | Relink field | inline on a row | Path field with Apply and Cancel |

Config discovery (F13), config diff, and config content are not in this feature.

## 3. Acceptance Criteria

### Registration

- **AC-01** "Add config file" opens C2 with: "Browse file..." or a typed absolute path; a project (required, from registered projects); an environment (local, dev, staging, production; required); a kind (YAML, Properties, .env, Docker Compose, Caddy/Nginx, JSON; required).
- **AC-02** When a file is picked, the kind is pre-selected from its name: `.yml` and `.yaml` → YAML (or Docker Compose if the name starts with `docker-compose`); `.properties` → Properties; `.env` or names starting with `.env` → .env; `.json` → JSON; `Caddyfile` or names containing `nginx` → Caddy/Nginx. The user can change it.
- **AC-03** Save validates the path with the F04 rules (absolute, existing, a file, not a folder). Errors: "Enter a file path." / "`<path>` was not found." / "This is a folder. Pick a file." / "Enter an absolute path, for example `F:\\work\\application.yml`."
- **AC-04** The same file path can be registered once per project. A duplicate in the same project: "`<path>` is already in this project's config list." The same path under a different project is allowed.
- **AC-05** Save adds the entry and closes the dialog; the new row is highlighted. Errors keep the typed values. Cancel and Esc discard.
- **AC-06** The entry's label is the file name (last path segment). Nothing else about the file is stored.

### Status

- **AC-10** Each entry's status is computed from the file's metadata on every catalog load (D1):
  - **Available**: the path exists and is a file.
  - **Missing**: the path does not exist, or exists as a folder.
  - **Inaccessible**: the metadata check fails for another reason (drive not ready, network path unreachable, access denied).
- **AC-11** A status shows a plain-language reason on hover and in a sub-line:
  - Available: "The file is in place."
  - Missing: "The file was not found at this path." / "A folder is at this path, not a file."
  - Inaccessible: "The file can't be reached. The drive may not be ready or access is denied."
- **AC-12** Only metadata is read. The catalog never opens, reads, or changes file contents (AC-60 of F04 applies).

### Filters

- **AC-20** Four filters: project, environment, kind, status. Each is a select with an "All" option.
- **AC-21** Active filters appear as removable chips (for example "Environment: staging"). "Clear filters" removes all chips.
- **AC-22** Filtering combines with AND. Filters are applied to the loaded list on the client; no new command is needed per filter change.
- **AC-23** With no matches: "No config files match these filters." with a "Clear filters" action.
- **AC-24** With no entries at all: "No config files registered" (the F01 empty copy), with an "Add config file" action.
- **AC-25** Filter state is not persisted; leaving the screen resets it.

### Open

- **AC-30** Each Available row has "Open", which opens the file with the default editor profile (F03). A menu next to Open lists the other profiles to choose one for that open.
- **AC-31** Opening goes through the F03 open capability. The success message: "Opened <file name> in <profile name>." Errors use the F03 open copy.
- **AC-32** Open on a Missing or Inaccessible row is not offered; the row shows Relink instead.
- **AC-33** Opening never changes the registry entry.

### Relink

- **AC-40** Each Missing or Inaccessible row has Relink. Relink opens an inline path field with Browse file..., Apply, and Cancel.
- **AC-41** Apply checks the new path (AC-03 rules) and the duplicate rule (AC-04) for the same project. Errors shown inline.
- **AC-42** A successful Apply saves the new path and re-computes the status. The entry keeps its environment, kind, and project.
- **AC-43** Cancel closes the field and changes nothing.

### Remove

- **AC-50** Each row has "Remove from catalog". It asks: "Remove `<file name>` from the catalog? The file is not changed." Confirm deletes the registry entry only. Cancel keeps it.
- **AC-51** Removing an entry never touches the file.

### Boundaries

- **AC-60** The React layer never touches the file system or the database; all checks and launches happen in Rust.
- **AC-61** Migration 7 adds the `config_files` table. Bound parameters everywhere.
- **AC-62** Statuses are not stored; they are computed on each load (D1).

## 4. Edge Cases

- The same file appears under two projects: allowed; each entry has its own status.
- A file moves into a different folder and a second entry for it is added: both show their own status.
- A project is later removed (F15): its entries remain and show "No project" until removed. This feature does not delete entries when a project changes.
- Environment or kind values outside the lists: rejected by the command with `environment_invalid` or `kind_invalid`.
- Very long path: "This path is too long for Windows." (same copy as F04).
- Opening a file whose editor profile executable was removed: F03 error copy.

## 5. Non-goals (F06)

- Config discovery and bulk suggestions (F13).
- Reading or validating file contents, config diff, or config validation (deferred in the roadmap).
- Editing file contents or rewriting config (roadmap no-go).
- Sorting by column (filters only).
- Persisting filter state.
- Renaming an entry's label (the label is always the file name).

## 6. Decisions Needing Approval

- **D1 — Status freshness.** (a) **Compute on every catalog load (metadata only).** (b) Store the last-known status and refresh on demand. *Recommended: (a). Config entries are few, metadata checks are cheap for local paths, and a stored status would show stale Available rows after a file moves.* Network paths can be slow; the status is computed per entry and each probe is metadata only.
- **D2 — Remove from catalog.** (a) **Include, with confirmation (AC-50).** (b) Leave out; entries can only be relinked. *Recommended: (a). A wrong or obsolete entry must be removable, and the action touches only DB rows, not files.*
- **D3 — Kind selection.** (a) **Pre-select from the file name, user confirms (AC-02).** (b) Always require the user to pick. *Recommended: (a). Saves a step for the common cases and the user still decides.*

## 7. Constitution Check

- **P1** No secrets stored. Pass.
- **P2** Config logic in the `config` module; it calls `projects::get` and the F03 `editors` open function; no table reach-through. Migration 7 is numbered. Bound parameters. Pass.
- **P3** One Args and one Result per command. Codes: `config_not_found`, `path_empty`, `path_not_found`, `path_is_folder`, `path_relative`, `path_too_long`, `duplicate_path`, `project_not_found`, `environment_invalid`, `kind_invalid`, `storage_unavailable`, plus the F03 open codes. Pass.
- **P4** Rust tests: kind inference from names (in the TS helper and mirrored in Rust validation if the command infers); status classification (available, missing, folder-at-path, inaccessible via a fake probe); duplicate rule per project; relink re-computes status; remove touches only the row; no writes to config files (mtimes unchanged); migration 7 reopen. Vitest for filter logic, chip labels, and the reason copy. Built-app smoke check before close. Pass (planned).
- **P5** No network. Pass.

## 8. Open Questions

None beyond Section 6.
