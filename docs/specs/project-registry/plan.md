# F02 — Project registry: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/project-registry/spec.md` (approved; D1 nested allowed with note; D2 five types; D3 tauri-plugin-dialog)
- **Branch**: `feature/project-registry`, created from `feature/foundation-and-navigation` at commit `617f7b6` (F02 depends on F01; F01 is not on `main` yet)

**Goal:** Register, validate, list, pin, and inspect projects by root folder, stored as references in SQLite, with no reads or writes inside project folders beyond an existence and type check.

**Architecture:** A `projects` Rust module owns validation, path normalisation, persistence, and pinning. It uses the connection held by `db::Storage`, through migration 2. Commands in `commands/projects.rs` expose it. The native folder picker is called from Rust with `tauri-plugin-dialog`. The UI calls typed commands only, and TanStack Query caches the list and detail.

**Tech Stack:** adds `tauri-plugin-dialog = "2"` (Rust only; no JS plugin package, since Rust calls the picker). Otherwise unchanged.

## Global Constraints

- Only Rust touches the database and file system. No SQL in the webview.
- Bound parameters for all user-supplied values. Migration 2 is a new numbered migration; migration 1 is not edited.
- Validation returns codes; the UI maps codes to the copy in spec AC-10 to AC-15 and AC-20.
- Tests use temp directories. No test writes to a real project folder or the real data folder.
- No new runtime dependency beyond `tauri-plugin-dialog`.

## Contracts

Rust `src-tauri/src/projects/mod.rs`:

```rust
pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_PATH_CHARS: usize = 260;
pub const MAX_TAG_CHARS: usize = 32;
pub const MAX_TAGS: usize = 10;
pub const PROJECT_TYPES: [&str; 5] = ["Spring Boot", "React", "Docker/Infra", "Node", "Other"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCode { PathEmpty, PathNotFound, PathIsFile, AlreadyRegistered, NameEmpty, NameTooLong, PathTooLong, TagInvalid, TooManyTags, TypeInvalid }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError { pub code: ValidationCode, pub detail: Option<String> } // detail: existing name, invalid tag, etc.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProject { pub root_path: String, pub display_name: String, pub project_type: String, pub tags: Vec<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project { pub id: i64, pub display_name: String, pub root_path: String, pub drive: String, pub project_type: String, pub tags: Vec<String>, pub pinned: bool, pub created_at: String }

pub fn normalize_path(raw: &str) -> String;                         // trim; '/' -> '\\'; strip trailing '\\' unless drive root like "F:\\"
pub fn drive_of(normalized: &str) -> String;                        // "F:" or ""
pub fn parse_tags(raw: &str) -> Result<Vec<String>, ValidationError>; // split ',', trim, lowercase, dedupe, validate
pub fn validate_new(input: &NewProject, existing: &[(i64, String, String)]) -> Result<(), ValidationError>;
pub fn register(storage: &Storage, input: NewProject) -> Result<Project, ValidationError>;
pub fn list(storage: &Storage) -> Result<Vec<Project>, ValidationError>;   // pinned first, then display_name COLLATE NOCASE
pub fn get(storage: &Storage, id: i64) -> Result<Option<Project>, ValidationError>;
pub fn set_pinned(storage: &Storage, id: i64, pinned: bool) -> Result<Option<Project>, ValidationError>;
pub fn nested_note(storage: &Storage, normalized_path: &str) -> Result<Option<String>, ValidationError>; // name of containing project
```

`Storage` exposes `pub fn connection(&self) -> &rusqlite::Connection` (replaces the `#[allow(dead_code)]` on `conn` since it is now used).

Migration 2 (`db/migrations.rs`):

```sql
CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  display_name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  path_key TEXT NOT NULL UNIQUE,
  drive TEXT NOT NULL,
  project_type TEXT NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE project_tags (
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY (project_id, tag)
);
```

`path_key` = `normalize_path(root_path).to_lowercase()`. The UNIQUE constraint is the final guard against duplicates under concurrent registration (AC-12 edge case); a violation maps to `AlreadyRegistered`.

Commands `src-tauri/src/commands/projects.rs` (each has one `…Args` struct and one return type; all state from `State<'_, StorageState>`):

| Command | Args | Returns |
|---|---|---|
| `projects_list` | `ProjectsListArgs {}` | `ProjectsListResult { projects: Vec<ProjectView>, error: Option<ProjectErrorCode> }` |
| `projects_get` | `ProjectGetArgs { id: i64 }` | `ProjectGetResult { project: Option<ProjectView>, error: Option<ProjectErrorCode> }` |
| `projects_register` | `ProjectRegisterArgs { root_path, display_name, project_type, tags_raw }` | `ProjectRegisterResult { project: Option<ProjectView>, error: Option<ProjectErrorCode>, detail: Option<String> }` |
| `projects_check_path` | `ProjectCheckPathArgs { root_path }` | `ProjectCheckPathResult { nested_in: Option<String>, error: Option<ProjectErrorCode> }` |
| `projects_set_pinned` | `ProjectSetPinnedArgs { id, pinned }` | `ProjectGetResult` |
| `projects_pick_folder` | `ProjectPickFolderArgs {}` | `ProjectPickFolderResult { path: Option<String> }` (None on cancel) |

`ProjectErrorCode` is a serde `snake_case` enum with the variants in `ValidationCode` plus `StorageUnavailable`, `ProjectNotFound`. `ProjectView` is the serde form of `Project` with `tags: Vec<String>`. The `error` field is the user-facing code; the UI never shows raw text.

TypeScript `src/types/commands.ts` mirrors every struct above; no other file defines them.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 2) | 1 |
| `src-tauri/src/db/mod.rs` | Modify (`connection()`, drop allow on conn, keep schema_version cfg(test)) | 1 |
| `src-tauri/src/projects/mod.rs` | Replace placeholder | 1 |
| `src-tauri/src/commands/projects.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` | Modify (dialog plugin, handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (project wrappers) | 3 |
| `src/lib/project-form.ts` | Create (`formatProjectError`, `tagsFromInput`, `driveLabel`) | 3 |
| `tests/project-form.test.ts` | Create | 3 |
| `src/screens/projects-screen.tsx` | Modify (list, Register button, empty state) | 4 |
| `src/components/register-project-dialog.tsx` | Create | 4 |
| `src/screens/project-detail-screen.tsx` | Create | 4 |
| `src/router.tsx`, `src/routes.tsx` | Modify (`projects/:id`) | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files; task 3 uses only the contract -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Projects domain, migration 2, Rust tests

**Specialist:** task (Rust)
**Interfaces:** Produces the `projects` module and `Storage::connection()` as in Contracts. Consumes `db::Storage`.

**Steps:**
1. Tests first in `projects/mod.rs`, each with a `tempfile::tempdir()` for both storage and project folders:
   - `normalize_path` cases: `"F:/a/b/"` → `"F:\\a\\b"`; `"F:\\"` stays `"F:\\"`; surrounding spaces trimmed.
   - `parse_tags("API, api ,  x-1,,")` → `["api", "x-1"]`; `"bad tag!"` → `TagInvalid` with detail `"bad tag!"`; 11 unique tags → `TooManyTags`; a 33-char tag → `TagInvalid`.
   - `register` on a temp folder, then `list` returns it with the drive from `drive_of`, tags, and `pinned == false`.
   - `register` the same folder with a different case or trailing slash → `AlreadyRegistered` with detail = existing display name.
   - path not existing → `PathNotFound`; path that is a file → `PathIsFile`; empty path → `PathEmpty`; whitespace name → `NameEmpty`; 101-char name → `NameTooLong`.
   - Nested: register `parent`, then `nested_note(child)` returns `Some("parent name")`; `register(child)` succeeds (AC-20). Register `child` first, then `parent`: succeeds (AC-21).
   - `set_pinned(id, true)` then reopen `Storage` via `db::open_in` on the same folder: `list` shows pinned first (AC-50, AC-31).
   - No write to a project folder: record the folder's entry list and mtimes before and after `register`; assert equal (AC-60).
2. Run `cargo test projects::`; FAIL expected.
3. Implement. Validation order in `validate_new`: path empty; path too long; path not found; path is file; already registered; name empty; name too long; type not in PROJECT_TYPES; tags. `register` runs `validate_new` and inserts project and tags in one transaction; the UNIQUE violation on `path_key` maps to `AlreadyRegistered` (look up the existing name for detail).
4. Add migration 2 to `migrations.rs` (SQL above). Update `MIGRATIONS_COUNT` consumers: the schema_version test must expect 2; update the assertion, not the constant logic.
5. Run `cargo test` for `projects::` and `db::`; all pass.

**Acceptance:** AC-10 to AC-15, AC-20, AC-21, AC-31, AC-50, AC-60, AC-61, AC-63. Constitution P2, P4.

---

### Task 2: Commands, folder picker, and wiring

**Specialist:** task (Rust)
**Interfaces:** Consumes Task 1 functions. Produces the six commands in Contracts.

**Steps:**
1. Add `tauri-plugin-dialog = "2"` to `src-tauri/Cargo.toml`. Register `tauri_plugin_dialog::init()` in `lib.rs`.
2. `commands/projects.rs`: implement each command as a thin wrapper that locks `StorageState.result` (`Mutex<Storage>`), calls the domain function, and maps `ValidationError` to `ProjectErrorCode`. A poisoned or failed storage maps to `StorageUnavailable`. Pick-folder: `app.dialog().file().blocking_pick_folder()` (Rust API; no capability needed for Rust-side calls). Verify this compiles against the installed plugin version; if the API name differs, use the one the crate exports and report it.
3. Tests for the mapping functions (pure): each `ValidationCode` maps to the matching `ProjectErrorCode`; `ProjectGetResult` for missing id returns `ProjectNotFound`; serialise `ProjectErrorCode` to snake_case strings.
4. Register the six commands in `generate_handler!`.
5. Run `cargo test` and `cargo build` from `src-tauri/`; no new warnings from touched files.

**Acceptance:** Constitution P3 (argument and return structs, codes, no raw strings); AC-02 (picker in Rust).

---

### Task 3: TypeScript contract, form helpers, Vitest

**Specialist:** frontend-developer
**Interfaces:** Produces the TS types mirroring Contracts, `projects*` wrappers, and helpers used by Task 4.

**Steps:**
1. `src/types/commands.ts`: add `ProjectView`, `ProjectErrorCode` (snake_case union), and one Args and Result type per command, exactly mirroring the Rust structs. Lint rule: use `Record<string, never>` for empty Args.
2. `src/lib/commands.ts`: `listProjects`, `getProject(id)`, `registerProject(args)`, `checkProjectPath(rootPath)`, `setProjectPinned(id, pinned)`, `pickProjectFolder()`. Each calls `invoke` with `{ args: {...} }`.
3. `src/lib/project-form.ts`:
   - `tagsFromInput(raw: string): string[]` — split on comma, trim, drop empties, lowercase, dedupe (display only; Rust is authoritative).
   - `formatProjectError(code, detail): string` — exact copy from spec AC-10 to AC-15, AC-20 (use `detail` for the path, existing name, or invalid tag as the spec shows). Path-not-found copy: "Folder `<path>` was not found." Path-is-file: "`<path>` is a file. Pick the project's root folder." Use the path from the form when detail is absent (pass path as argument).
   - `driveLabel(rootPath: string): string` — `"F:"` for a `X:` prefix, else `""`.
4. Tests first in `tests/project-form.test.ts`: tag parsing dedupe and lowercase; each error code maps to the exact spec sentence; drive label for `F:\\x`, for `\\\\server\\share` (returns ""), and for `"F:"`.
5. Run `npx vitest run tests/project-form.test.ts`; all pass.

**Acceptance:** AC-10 to AC-15, AC-20 copy; constitution P3 (types in one file), P4 (branching in Vitest).

---

### Task 4: Projects screens, Register dialog, detail

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy are fixed by the spec; layout follows the preview's Projects list and dialog.
**Frontend dispatch contract:** `design_md_fallback`: `docs/DESIGN.md` and `docs/preview/index.html` (Projects list, Register project dialog, project detail). Dark theme only; Tailwind tokens from `src/styles.css`.

**Steps:**
1. `projects-screen.tsx`: TanStack Query `["projects"]` via `listProjects`. Empty: F01 copy plus a "Register project" button (AC-32). Non-empty: rows with name, type, root path (monospace), drive, tags, pin indicator; sorted as returned by Rust; each row links to `/projects/:id`. Pinned rows show a pin mark.
2. `register-project-dialog.tsx`: fields per AC-01; "Browse..." calls `pickProjectFolder` and fills the path (AC-02). Path blur calls `checkProjectPath`; if `nested_in` is set, show "This folder is inside <name>. Both projects are kept." (AC-20). Register calls `registerProject`; on error show `formatProjectError` next to the field, keep values (AC-05, AC-16); on success close, invalidate `["projects"]`, highlight the new row, show the success message (AC-03). Cancel and Esc close and discard (AC-04). Use a native `<dialog>` element or an accessible overlay with focus trap.
3. `project-detail-screen.tsx`: `["project", id]` via `getProject`; shows AC-40 fields; Pin/Unpin calls `setProjectPinned` and invalidates both keys (AC-50, AC-51); Back link to `/projects` (AC-41); unknown id shows AC-43 copy. No count lines (AC-42).
4. `router.tsx` and `routes.tsx`: add `projects/:id` under the shell; keep the existing routes.
5. No Vitest here (logic is in Task 3). Run `npm run build` once at the end.

**Acceptance:** AC-01 to AC-06, AC-10 to AC-16 UI, AC-20, AC-30 to AC-33, AC-40 to AC-43, AC-50, AC-51.

---

## Self-review

- Spec coverage: AC-01 to 06 (T4, T2), AC-10 to 15 (T1, T3), AC-16 (T4), AC-20/21 (T1, T4), AC-30/31 (T1, T4), AC-32 (T4), AC-33 (T4: row link), AC-40 to 43 (T4), AC-50/51 (T1, T4), AC-52 (no limit, T1), AC-60 to 63 (T1), edge cases covered by T1 tests (case, slash, race via UNIQUE).
- Types: `ProjectErrorCode` names in Rust and TS match; `ValidationCode` maps 1:1 plus storage codes.
- Constitution: P2 (Rust-only FS and DB; migration 2 numbered), P3 (one arg and one return struct per command), P4 (tests listed), P5 (no network).

## Open items

- AC-33 wording in the spec ("Clicking a row in P3's list … not applicable") is noise; it is treated as "row link opens detail" in this plan. Flagged for the Step 6 reviewer.
- The nested note is shown on path blur through `projects_check_path`. This meets AC-20; the spec does not name the moment, so this is a plan choice.
