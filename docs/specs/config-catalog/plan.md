# F06 — Config catalog: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/config-catalog/spec.md` (approved; D1 status computed on every load; D2 remove from catalog with confirmation; D3 kind pre-selected from file name)
- **Branch**: `feature/config-catalog`, cut from `feature/home-overview` (commit `fc2856a`)

**Goal:** Register config files per project with environment and kind; list them with a status computed from metadata on each load; filter client-side; open through the F03 editor path; relink; remove the registry entry only.

**Architecture:** A `config` Rust module owns entries, validation, status classification (`probe` trait, `FsProbe` in production), relink, and remove. It calls `projects::get` for the project check and `editors::open_with_profile` for opening. Migration 7 creates `config_files`. Commands wrap the module; the list command computes statuses outside the storage lock. The React side holds filter state only in component state.

**Tech Stack:** no new dependencies. Reuses `tauri-plugin-dialog` for pickers.

## Global Constraints

- Metadata only on config paths; no reads or writes inside files (AC-12, AC-60).
- Opening goes through `editors::open_with_profile`; no shell.
- Bound parameters everywhere; migration 7 is numbered; migrations 1 to 6 untouched.
- One Args and one Result per command; snake_case codes; locks only around database work; probes and launches outside the lock.
- Tests use temp directories, a fake probe, and a recording launcher. No real editor.

## Contracts

Rust `src-tauri/src/config/mod.rs`:

```rust
pub const ENVIRONMENTS: [&str; 4] = ["local", "dev", "staging", "production"];
pub const KINDS: [&str; 6] = ["yaml", "properties", "dotenv", "docker_compose", "caddy_nginx", "json"];
pub const MAX_PATH_CHARS: usize = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCode {
    ConfigNotFound, PathEmpty, PathNotFound, PathIsFolder, PathRelative, PathTooLong,
    DuplicatePath, ProjectNotFound, EnvironmentInvalid, KindInvalid, StorageUnavailable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError { pub code: ConfigCode, pub detail: Option<String> }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult { Found { is_dir: bool }, NotFound, Unreachable }
pub trait PathProbe { fn probe(&self, path: &str) -> ProbeResult; }
pub struct FsProbe;   // NotFound -> NotFound; other errors -> Unreachable

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Available, Missing, FolderAtPath, Inaccessible }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry { pub id: i64, pub project_id: i64, pub project_name: Option<String>, pub path: String, pub name: String, pub environment: String, pub kind: String, pub status: Status }

pub fn status_of(probe: &dyn PathProbe, path: &str) -> Status;   // Found file -> Available; Found dir -> FolderAtPath; NotFound -> Missing; Unreachable -> Inaccessible
pub fn validate_config_path(raw: &str) -> Result<String, ConfigError>;  // trim, '/'->'\\', shape (drive or UNC absolute), '..' rejected, length; metadata NOT checked here
pub fn validate_entry(probe: &dyn PathProbe, raw_path: &str, environment: &str, kind: &str) -> Result<String, ConfigError>; // path shape + probe (Found file required) + env/kind lists
pub fn list(storage: &Storage, probe: &dyn PathProbe) -> Result<Vec<ConfigEntry>, ConfigError>; // DB read only; statuses computed by caller; see note
pub fn add(storage: &Storage, project_id: i64, normalised_path: &str, environment: &str, kind: &str) -> Result<i64, ConfigError>;
pub fn relink(storage: &Storage, probe: &dyn PathProbe, id: i64, raw_path: &str) -> Result<(), ConfigError>;
pub fn remove(storage: &Storage, id: i64) -> Result<(), ConfigError>;
```

`list` is split for locking: `load_rows(storage) -> Vec<Row>` (DB only), then `with_status(rows, probe) -> Vec<ConfigEntry>` (no storage). The command takes the lock for `load_rows`, releases it, probes, and returns.

Name is the last path segment; computed in `with_status` from the stored path. `path_key` = normalised path lowercased. `UNIQUE(project_id, path_key)` enforces the per-project duplicate rule; a violation maps to `DuplicatePath`.

Migration 7 (`db/migrations.rs`):

```sql
CREATE TABLE config_files (
  id INTEGER PRIMARY KEY,
  project_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  path_key TEXT NOT NULL,
  environment TEXT NOT NULL,
  kind TEXT NOT NULL,
  UNIQUE (project_id, path_key)
);
-- foreign_keys is off; project existence is checked by the command, and entries survive project removal (F15) by design.
```

Commands `src-tauri/src/commands/config.rs` (one Args and one Result each; `#[tauri::command(async)]` for list, add, relink, and open, which probe or launch):

| Command | Args | Result |
|---|---|---|
| `config_files_list` | `ConfigListArgs {}` | `ConfigListResult { entries: Vec<ConfigEntryView>, error, detail }` |
| `config_files_add` | `ConfigAddArgs { project_id, path, environment, kind }` | `ConfigEntryResult { entry: Option<ConfigEntryView>, error, detail }` |
| `config_files_relink` | `ConfigRelinkArgs { id, path }` | `ConfigEntryResult` |
| `config_files_remove` | `ConfigRemoveArgs { id }` | `ConfigRemoveResult { error, detail }` |
| `config_files_open` | `ConfigOpenArgs { id, editor_kind: Option<String> }` | `ConfigOpenResult { opened_name: Option<String>, error: Option<EditorCode>, detail: Option<String> }` |
| `config_files_pick` | `ConfigPickArgs {}` | `ConfigPickResult { path: Option<String> }` |

`config_files_open` reads the entry under the lock, releases it, runs `probe` (must be Available, else `ConfigCode::PathNotFound`-style error mapped to the Missing or Inaccessible copy on the UI side: return `error: Some(EditorCode::ItemNotFound)` for Missing and `StorageUnavailable`-free code for Inaccessible via `ItemNotFound` detail "inaccessible"), then calls `editors::resolve_profile(editor_kind)` and `editors::open_with_profile` with `AppOpener` (made available from `commands/editors.rs`, as F05 did).

`config_files_add` checks the project with `projects::get` before taking the lock for the insert; `config_files_relink` probes before the lock and locks for the update.

Entry view `ConfigEntryView`: `id, project_id, project_name: Option<String>, path, name, environment, kind, status: "available"|"missing"|"folder_at_path"|"inaccessible"`. `project_name` is None when the project no longer exists (AC edge case).

TypeScript `src/types/commands.ts`: `CONFIG_ENVIRONMENTS`, `CONFIG_KINDS` (Rust authoritative), `ConfigEnvironment`, `ConfigKind`, `ConfigStatus`, `ConfigCode`, `ConfigEntryView`, and one Args and one Result per command.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 7) | 1 |
| `src-tauri/src/db/mod.rs` | Modify (migration count 7) | 1 |
| `src-tauri/src/config/mod.rs` | Replace placeholder | 1 |
| `src-tauri/src/commands/config.rs` | Create | 2 |
| `src-tauri/src/commands/editors.rs` | Modify (`AppOpener` already pub(crate); confirm) | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Modify (module, handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (config wrappers) | 3 |
| `src/lib/config-form.ts` | Create (`kindFromFileName`, `applyFilters`, `activeChips`, `statusReason`, `formatConfigError`, `environmentLabel`, `kindLabel`) | 3 |
| `tests/config-form.test.ts` | Create | 3 |
| `src/screens/config-screen.tsx` | Replace F01 placeholder (filters, list, empty states, Add, Clear) | 4 |
| `src/components/add-config-dialog.tsx` | Create | 4 |
| `src/components/config-row.tsx` | Create (status, Open with profile menu, Relink, Remove) | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Migration 7 and config domain (Rust)

**Specialist:** task (Rust)

**Steps:**
1. Tests first in `config/mod.rs` with a `FakeProbe` (HashMap keyed by lowercased path) and temp storage:
   - `status_of`: file → Available; directory → FolderAtPath; NotFound → Missing; Unreachable → Inaccessible.
   - `validate_config_path`: empty → PathEmpty; `src\\app.yml` → PathRelative; `..` segment → PathNotFound-compatible code `PathRelative`; 261 chars → PathTooLong; `F:` → rejected as PathIsFolder-compatible? No: a drive root without a file name is PathIsFolder (validated against probe in validate_entry, not here). Test `F:\\` normalises and is accepted by shape.
   - `validate_entry`: unknown environment → EnvironmentInvalid; unknown kind → KindInvalid; probe Found dir → PathIsFolder; NotFound → PathNotFound; Unreachable → PathNotFound with detail "inaccessible" is NOT acceptable: return the same as NotFound but detail "unreachable" (test asserts code PathNotFound and detail "unreachable").
   - `add`: duplicate path in the same project → DuplicatePath; same path in a different project → allowed; the same path with different case → DuplicatePath.
   - `list` + `with_status`: a registered entry whose file is later removed reports Missing; a folder reports FolderAtPath; name is the last segment; project_name None when the project id is unknown (use a row whose project id has no project).
   - `relink`: changes path and keeps environment, kind, project; duplicate within same project rejected; probe must find a file.
   - `remove`: deletes only the row; the file's mtime and the directory listing unchanged; removing an unknown id → ConfigNotFound.
   - No writes to config files: record mtimes and entries before and after add, relink, and remove.
   - Migration 7 reopen keeps rows.
2. Run `cargo test config::` and `cargo test db::`; expect failure first (note if skipped).
3. Implement migration 7, the module per the Contracts, and keep every mutation a single statement or transaction with bound parameters.
4. Update the migration-count assertion to 7.
5. Run both filters; all pass.

**Acceptance:** AC-03, AC-04, AC-05 (domain), AC-06, AC-10, AC-11 (status classification and reason keys), AC-41 (domain), AC-42, AC-50, AC-51, AC-61, AC-62. Constitution P2, P4.

---

### Task 2: Commands and wiring (Rust)

**Specialist:** task (Rust)

**Steps:**
1. Confirm `AppOpener` in `commands/editors.rs` is `pub(crate)` (it is, from F05).
2. `commands/config.rs`: implement the six commands per the table.
   - `config_files_list`: `load_rows` under the lock; release; `with_status` with `FsProbe`; return views.
   - `config_files_add`: `validate_config_path` and the probe-based check via `validate_entry` run before the lock (the probe needs no storage); `projects::get` inside one short lock; then `add` under the lock. Map errors to the result.
   - `config_files_relink`: `validate_entry` before the lock; lock for `relink`.
   - `config_files_remove`: lock for `remove`.
   - `config_files_open`: read the entry under the lock; release; probe; map Missing and Inaccessible to `EditorCode::ItemNotFound` with a detail string "missing" or "inaccessible" for the UI to choose copy; for Available, call `editors::resolve_profile` (under the lock) and `editors::open_with_profile(&AppOpener(&app), profile, path)` outside the lock.
   - `config_files_pick`: dialog file picker (like `workspaces_pick_item` with folder false).
3. Pure mapping tests: `ConfigCode` snake_case; `Status` to view string; `project_name` None when the project is missing.
4. Register all six in `generate_handler!`; add `pub mod config;` in `commands/mod.rs`.
5. Run `cargo test --lib` and `cargo build` from `src-tauri/`; no warnings in touched files.

**Acceptance:** Constitution P3; AC-01, AC-02 (picker), AC-30, AC-31 (open wiring), AC-40, AC-50, AC-60.

---

### Task 3: TypeScript contract and config helpers

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: the constants and types from the Contracts. `ConfigCode` union with the 11 snake_case codes. Add `ConfigEntryView`, `ConfigStatus` (`"available" | "missing" | "folder_at_path" | "inaccessible"`), and one Args and one Result per command using `Record<string, never>` for empty Args.
2. `src/lib/commands.ts`: `listConfigFiles`, `addConfigFile(args)`, `relinkConfigFile(id, path)`, `removeConfigFile(id)`, `openConfigFile(id, editorKind | null)`, `pickConfigFile()`.
3. `src/lib/config-form.ts`:
   - `kindFromFileName(name)`: per AC-02: `docker-compose*.y(a)ml` → "docker_compose"; `.yml`/`.yaml` → "yaml"; `.properties` → "properties"; `.env` or names starting with `.env` → "dotenv"; `.json` → "json"; `Caddyfile` or names containing `nginx` → "caddy_nginx"; otherwise null (no pre-selection).
   - `applyFilters(entries, { project, environment, kind, status })`: AND; "all" or empty means no filter; `project` matches project_name.
   - `activeChips(filters)`: list of `{ key, label }` such as "Environment: staging"; empty when no filter is active.
   - `statusReason(status)`: exact copy from AC-11 (folder case uses "A folder is at this path, not a file." for `folder_at_path`).
   - `formatConfigError(code, detail, name)`: exact copy: path_empty "Enter a file path."; path_not_found "`<path>` was not found." (detail path; "unreachable" detail gives the inaccessible sentence); path_is_folder "This is a folder. Pick a file."; path_relative "Enter an absolute path, for example `F:\\work\\application.yml`."; path_too_long "This path is too long for Windows."; duplicate_path "`<path>` is already in this project's config list."; project_not_found "That project no longer exists. Choose another project."; environment_invalid "Pick an environment from the list."; kind_invalid "Pick a kind from the list."; config_not_found "This config file is no longer in the catalog."; storage_unavailable as F04.
   - `environmentLabel(env)` and `kindLabel(kind)`: display labels "Local", "Dev", "Staging", "Production"; "YAML", "Properties", ".env", "Docker Compose", "Caddy/Nginx", "JSON".
4. Tests first in `tests/config-form.test.ts`: kind inference for each example name; applyFilters AND behaviour and "all"; chips order and labels; statusReason for each status; formatConfigError exact copy for each code including the unreachable path variant; environment and kind labels.
5. Run `npx vitest run tests/config-form.test.ts`; all pass. Run `npx tsc --noEmit`; report.

**Acceptance:** AC-02, AC-03, AC-04, AC-11, AC-20, AC-21, AC-22 logic and copy; constitution P3, P4.

---

### Task 4: Config screen, add dialog, row actions

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy fixed by spec; layout follows the preview's Config Catalog.
**Frontend dispatch contract:** `design_md_fallback` (`docs/DESIGN.md`, `docs/preview/index.html` Config Catalog). Dark theme; Tailwind tokens.

**Steps:**
1. `config-screen.tsx`: query `["config-files"]` via `listConfigFiles`. Filter state in component state only (AC-25). Filter selects: Project (from `listProjects`), Environment, Kind, Status, each with "All" (AC-20). Active chips with remove buttons and "Clear filters" (AC-21). Filtered list via `applyFilters`.
   - No entries at all: "No config files registered" with an "Add config file" button (AC-24).
   - Entries exist but none match: "No config files match these filters." with "Clear filters" (AC-23).
   - Otherwise rows via `config-row.tsx`.
2. `add-config-dialog.tsx`: "Browse file..." (`pickConfigFile`), typed path, project select (required), environment select (required), kind select (required). Picking a file sets the kind from `kindFromFileName` when one matches (AC-02). Submit calls `addConfigFile`; errors shown next to the path field (or the project/environment/kind field for those codes) via `formatConfigError`; values kept (AC-05). Success closes, highlights the new row, and invalidates `["config-files"]`. Native `<dialog>` or accessible overlay with focus management; Cancel and Esc discard.
3. `config-row.tsx`: columns name, project (or "No project" when project_name is null), environment label, kind label, path (monospace), status pill with `statusReason` as title and sub-line.
   - Available: "Open" button calling `openConfigFile(id, null)`; a select beside it lists the profiles from `listEditorProfiles` to open with a different profile (AC-30). Success message "Opened <name> in <profile>." (AC-31). Errors use the F03 open copy via `formatOpenError`.
   - Missing or Inaccessible: no Open; a Relink control (AC-32, AC-40). Relink opens an inline field with Browse file... (`pickConfigFile`), Apply (`relinkConfigFile`; errors via `formatConfigError`), Cancel (AC-43). Success invalidates `["config-files"]`.
   - All rows: "Remove from catalog" opens the confirmation "Remove `<name>` from the catalog? The file is not changed." with Cancel and a "Remove" button (AC-50). Confirm calls `removeConfigFile` and invalidates the query.
4. `npm run build` once and report.

**Acceptance:** AC-01 to AC-06 UI, AC-10, AC-11, AC-20 to AC-25 UI, AC-30 to AC-33, AC-40 to AC-43, AC-50, AC-51.

---

### Task 5: Verification

Run after all tasks: `npx prettier --write` on touched files, `npx tsc --noEmit`, `npm run build`, `npm run lint`, `npx vitest run`, `cargo test --lib`, `cargo build`.

## Self-review

- Spec coverage: AC-01/02 (T2 picker, T3 inference, T4 dialog), AC-03/04/05/06 (T1 validate, add, duplicate; T4 errors), AC-10/11/12 (T1 status, T3 reason, T4 display), AC-20 to 25 (T3 applyFilters, chips, copy; T4 UI), AC-30 to 33 (T2 open, T4 row), AC-40 to 43 (T1 relink, T4 row), AC-50/51 (T1 remove, T4 confirm), AC-60 to 62 (T1 domain, T2 wiring).
- Names: `ConfigCode` and `ConfigStatus` strings match across Rust and TS; `ENVIRONMENTS` and `KINDS` match.
- Constitution: P2 (Rust only probes, locks, launches; editors via public function), P3 (one Args/Result per command), P4 (tests listed), P5 (no network).

## Notes

- Opening a Missing or Inaccessible entry returns an `EditorCode::ItemNotFound` with a detail distinguishing "missing" and "inaccessible"; the UI never offers Open on those rows, so this path is defensive.
- `project_name` is None when a project id no longer resolves (F15 future); the row shows "No project".
