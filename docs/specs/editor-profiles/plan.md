# F03 — Editor profiles and open in editor: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/editor-profiles/spec.md` (approved; D1 Open test in Settings; D2 tauri-plugin-opener for Windows default; D3 item path is the only argument)
- **Branch**: `feature/editor-profiles`, cut from `feature/project-registry` (commit `dd298ea`)

**Goal:** Four built-in editor profiles with one default, validated executable paths saved from Settings, and an open capability (Windows default via the opener plugin; other profiles via a fixed program and a single item argument) with the error copy the spec defines.

**Architecture:** An `editors` Rust module owns profiles, validation, command construction, and launching. Launching goes through a `Launcher` trait: the real implementation spawns with `std::process::Command`; tests record the planned program and arguments and never start an editor. Migration 3 creates `editor_profiles` with four seeded rows and a partial unique index that enforces a single default. Commands wrap the module. The Settings screen adds an editor-profiles section and an Open test section.

**Tech Stack:** adds `tauri-plugin-opener = "2"` (Rust-side `open_path`; no JS package needed). Reuses `tauri-plugin-dialog`.

## Global Constraints

- Only Rust launches processes or touches the file system beyond the picker result. No shell, no `cmd /c`, no string-concatenated command lines (spec AC-50).
- Each profile stores only kind, display name, and executable path. No arguments column (AC-51).
- Bound parameters for all SQL. Migration 3 is a new numbered migration; migrations 1 and 2 are not edited.
- Commands: one Args and one Result struct each; user-facing error codes only.
- Tests never start a real editor. Tests use temp directories.

## Contracts

Rust `src-tauri/src/editors/mod.rs`:

```rust
pub const KINDS: [&str; 4] = ["windows_default", "notepad_plus_plus", "vs_code", "custom"];
pub const MAX_NAME_CHARS: usize = 40;
pub const UNSAFE_CHARS: &[char] = &['&', '|', ';', '"', '^', '%', '<', '>'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCode {
    ProfileNotFound, ProfilePathEmpty, ExecutableNotFound, ExecutableInvalid,
    PathUnsafe, NameTooLong, ItemNotFound, LaunchFailed, StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorError { pub code: EditorCode, pub kind: Option<String>, pub detail: Option<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile { pub kind: String, pub name: String, pub exe_path: String, pub is_default: bool }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInput { pub kind: String, pub name: String, pub exe_path: String }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan { pub program: String, pub args: Vec<String> }

pub fn validate_exe_path(raw: &str, is_default: bool) -> Result<String, EditorCode>;  // "" ok unless default; trims; checks unsafe chars, existence, .exe, not a dir
pub fn validate_profiles(inputs: &[ProfileInput]) -> Result<(), EditorError>;          // name length, exe path rules per profile (windows_default ignores path)
pub fn list(storage: &Storage) -> Result<Vec<Profile>, EditorError>;                   // ordered as KINDS
pub fn save(storage: &Storage, inputs: Vec<ProfileInput>) -> Result<Vec<Profile>, EditorError>;   // validate all, then one transaction
pub fn set_default(storage: &Storage, kind: &str) -> Result<Vec<Profile>, EditorError>;
pub fn resolve_profile(storage: &Storage, kind: Option<&str>) -> Result<Profile, EditorError>;    // None = default
pub fn launch_plan(profile: &Profile, item_path: &str) -> Result<Option<LaunchPlan>, EditorError>; // None = Windows default (use opener)

pub trait Launcher {
    fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()>;
    fn open_default(&self, item_path: &str) -> Result<(), String>;
}
pub struct SystemLauncher;   // spawn: std::process::Command::new(&plan.program).args(&plan.args).spawn() (child dropped, not waited)
pub fn open_item(storage: &Storage, launcher: &dyn Launcher, kind: Option<&str>, item_path: &str) -> Result<Profile, EditorError>;
```

Migration 3 (`db/migrations.rs`):

```sql
CREATE TABLE editor_profiles (
  kind TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  exe_path TEXT NOT NULL DEFAULT '',
  is_default INTEGER NOT NULL DEFAULT 0
);
INSERT INTO editor_profiles (kind, name, exe_path, is_default) VALUES
  ('windows_default', 'Windows default', '', 1),
  ('notepad_plus_plus', 'Notepad++', '', 0),
  ('vs_code', 'VS Code', '', 0),
  ('custom', 'Custom', '', 0);
CREATE UNIQUE INDEX editor_one_default ON editor_profiles(is_default) WHERE is_default = 1;
```

Item existence (AC-33) is checked in `open_item` with `std::fs::metadata` before launching. Executable existence is checked in `validate_exe_path` and again in `launch_plan`, so a deleted executable yields `ExecutableNotFound` at open time, not a silent failure. A `LaunchFailed` code is for `spawn` errors only (AC-35).

Commands `src-tauri/src/commands/editors.rs` (one Args and one Result each; `#[tauri::command(async)]` for anything that touches the file system or spawns; locks only around database work):

| Command | Args | Result |
|---|---|---|
| `editor_profiles_list` | `EditorProfilesListArgs {}` | `EditorProfilesResult { profiles: Vec<EditorProfileView>, error: Option<EditorCode>, kind: Option<String>, detail: Option<String> }` |
| `editor_profiles_save` | `EditorProfilesSaveArgs { profiles: Vec<EditorProfileInputArgs> }` | `EditorProfilesResult` (same shape) |
| `editor_profiles_set_default` | `EditorSetDefaultArgs { kind }` | `EditorProfilesResult` |
| `editor_open` | `EditorOpenArgs { item_path, kind: Option<String> }` | `EditorOpenResult { opened_name: Option<String>, error: Option<EditorCode>, kind: Option<String> }` |
| `editor_pick_executable` | `EditorPickExecutableArgs {}` | `EditorPickResult { path: Option<String> }` |
| `editor_pick_item` | `EditorPickItemArgs { folder: bool }` | `EditorPickResult { path: Option<String> }` |

`EditorCode` serialises snake_case: `profile_not_found`, `profile_path_empty`, `executable_not_found`, `executable_invalid`, `path_unsafe`, `name_too_long`, `item_not_found`, `launch_failed`, `storage_unavailable`. `kind` in results names the offending profile for field-level copy.

Opener: `tauri_plugin_opener::OpenerExt` on the app handle, `app.opener().open_path(item_path, None::<&str>)`. `SystemLauncher::open_default` is not on the `Launcher` trait's test path; the command passes an `AppOpener` wrapper that implements `Launcher` with the real app handle, and tests use a recording launcher.

TypeScript `src/types/commands.ts`: mirror every struct; `EditorCode` union; `EDITOR_KINDS` as a const array with comment that Rust `editors::KINDS` is authoritative (same pattern as F02 `PROJECT_TYPES`).

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 3) | 1 |
| `src-tauri/src/editors/mod.rs` | Replace placeholder | 1 |
| `src-tauri/src/commands/editors.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` | Modify (opener, handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (editor wrappers) | 3 |
| `src/lib/editor-form.ts` | Create (`formatEditorError`) | 3 |
| `tests/editor-form.test.ts` | Create | 3 |
| `src/components/editor-profiles-section.tsx` | Create | 4 |
| `src/components/editor-open-test.tsx` | Create | 4 |
| `src/screens/settings-screen.tsx` | Modify (add both sections above the privacy text) | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files; task 3 uses only the contract -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Migration 3, editors domain, Rust tests

**Specialist:** task (Rust)

**Steps:**
1. Tests first in `editors/mod.rs`, with temp directories for storage and fake executables (empty files named `fake-editor.exe`; they are never run):
   - `validate_exe_path`: `""` ok when not default, `ProfilePathEmpty` when default; `"C:\\x\\a&b.exe"` → `PathUnsafe` (each char in UNSAFE_CHARS); missing file → `ExecutableNotFound`; existing `notes.txt` → `ExecutableInvalid`; a directory named `x.exe` → `ExecutableInvalid`; existing `fake-editor.exe` with spaces in its folder (`tmp\\Program Files\\x\\fake editor.exe`) → `Ok` and trimmed.
   - `validate_profiles`: name of 41 chars → `NameTooLong`; `windows_default` with a bad path is ignored.
   - `list` after migration 3: four profiles in KINDS order; only windows_default is_default.
   - `save`: valid paths persist and reopen via `db::open_in` same folder; an invalid path in one profile saves nothing (check the other profile's path unchanged).
   - `set_default("vs_code")`: exactly one default; previous default cleared; a second call with unknown kind → `ProfileNotFound`.
   - `resolve_profile(None)` returns the default.
   - `launch_plan`: for `notepad_plus_plus` with exe `C:\\Program Files\\Notepad++\\notepad++.exe` and item `D:\\my notes\\a b.txt` → program is the exe, args `["D:\\my notes\\a b.txt"]` (one element; no shell); empty exe → `ProfilePathEmpty` with kind; windows_default → `Ok(None)`.
   - `open_item` with a recording launcher: item missing → `ItemNotFound` and launcher not called; item present, default windows_default → `open_default` called once; item present, profile with exe deleted after save → `ExecutableNotFound`; launcher returns Err from spawn → `LaunchFailed`.
   - Migration 3 reopen: the partial unique index blocks a second default at the SQL level (attempt an INSERT/UPDATE setting two defaults; expect an error).
2. Run `cargo test editors::` and `cargo test db::`; FAIL expected first.
3. Implement migration 3, then the module per the contract. `save` validates everything first, then runs one transaction: clear defaults and set the new default in the same transaction, using bound parameters.
4. Update the db test that asserts `MIGRATIONS_COUNT` to 3 (the constant follows the migration vec).
5. Run the two filters; all pass.

**Acceptance:** AC-01 to AC-06, AC-10 to AC-12 (domain), AC-20 to AC-23, AC-30 to AC-35, AC-50 to AC-53 (domain part). Constitution P2, P4.

---

### Task 2: Commands, opener, pickers, wiring

**Specialist:** task (Rust)

**Steps:**
1. Add `tauri-plugin-opener = "2"` to `src-tauri/Cargo.toml`; register `tauri_plugin_opener::init()` in `lib.rs`. Confirm the crate exports `OpenerExt` and `open_path`; report the exact call that compiled.
2. `commands/editors.rs`: implement the six commands per the table. `AppOpener<'a>(&'a AppHandle)` implements `Launcher`: `spawn` uses `SystemLauncher`; `open_default` calls `open_path`. `editor_open` builds it from the app handle. `editor_pick_executable` uses the dialog plugin with an `exe` filter; `editor_pick_item` picks a file or a folder by the `folder` flag.
3. Map `EditorError` to `EditorResult` fields; storage failure → `StorageUnavailable`.
4. Tests for the pure mapping: each `EditorCode` maps to its snake_case string; `EditorCode::ProfilePathEmpty` result carries the kind.
5. Register all six in `generate_handler!`.
6. Run `cargo test` and `cargo build` from `src-tauri/`; no warnings in touched files.

**Acceptance:** Constitution P3; AC-02 (opener), AC-40 (picker for Open test).

---

### Task 3: TypeScript contract, form helpers, Vitest

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: `EDITOR_KINDS` const array (four kinds; Rust authoritative), `EditorKind`, `EditorCode` union (nine codes), `EditorProfileView`, `EditorProfileInputArgs`, and one Args and one Result type per command, mirroring Rust. Empty Args use `Record<string, never>`.
2. `src/lib/commands.ts`: `listEditorProfiles`, `saveEditorProfiles(profiles)`, `setDefaultEditor(kind)`, `openInEditor(itemPath, kind | null)`, `pickExecutable()`, `pickItem(folder)`.
3. `src/lib/editor-form.ts`: `formatEditorError(code, kind, name, detail)` with exact copy:
   - profile_path_empty: "Enter the path to the editor executable." (on save); at open time: "Set the path to <name> in Settings to open files with it."
   - executable_not_found: "Can't find `<path>`. Check the path or pick the editor again."
   - executable_invalid: "Pick an editor program (.exe)."
   - path_unsafe: "This path has characters that are not allowed in an editor path."
   - name_too_long: "Use at most 40 characters."
   - item_not_found: "`<path>` was not found. It may have moved or been renamed."
   - launch_failed: "<name> could not start. Check the path in Settings."
   - profile_not_found: "That editor profile no longer exists."
   - storage_unavailable: "DevShelf cannot reach its data folder. Restart DevShelf and try again."
   Detail carries the path where the copy names it.
   - `openSuccessMessage(fileName, profileName)`: "Opened <file name> in <profile name>."
4. Tests first in `tests/editor-form.test.ts`: each code yields the exact copy; the open-time vs save-time variant of profile_path_empty; the success message; file name is the last path segment for both `\\` and `/` separators.
5. Run `npx vitest run tests/editor-form.test.ts`; all pass.

**Acceptance:** AC-20 to AC-23, AC-33 to AC-36 copy; constitution P3, P4.

---

### Task 4: Settings sections (profiles and Open test)

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy are fixed; layout follows the preview's Settings.
**Frontend dispatch contract:** `design_md_fallback` (`docs/DESIGN.md`, `docs/preview/index.html` Settings). Dark theme; existing Tailwind tokens.

**Steps:**
1. `editor-profiles-section.tsx`: query `["editor-profiles"]`. Lists the four profiles in order with name (Custom has an editable name, max 40), executable path field (disabled for windows_default with the note "Opens with the Windows file association"), a "Default" marker, and "Set as default" (calls `setDefaultEditor`, invalidates the query; AC-05). Local draft state for paths and name; Save (AC-10) calls `saveEditorProfiles` with all four; success shows "Settings saved" until the next edit (AC-11); a rejected save shows `formatEditorError` next to the offending profile and keeps drafts (AC-12). Reset restores the last loaded values (AC-10). Browse... next to each executable path calls `pickExecutable` and fills the field.
2. `editor-open-test.tsx`: path field with "Browse..." (file or folder: two buttons, "Browse file..." and "Browse folder...", both calling `pickItem`), profile select defaulting to the default profile, and "Open" calling `openInEditor`. Result line: the success message (AC-36) or the matching error (AC-33 to AC-35). The path is not persisted (AC-41).
3. `settings-screen.tsx`: insert the two sections above the existing storage row and privacy text; keep the privacy text exact.
4. Run `npm run build` once from PROJECT_ROOT and report.

**Acceptance:** AC-01 to AC-06 UI, AC-10 to AC-12, AC-40, AC-41, AC-52.

---

### Task 5: Verification

Run the full checks after all tasks: `npm run build`, `npm run lint` (run `npx prettier --write` on touched files first), `npx vitest run`, `cargo test`, `cargo build`.

## Self-review

- Spec coverage: AC-01/04 (T1 seed, partial index), AC-02 (T2 opener), AC-03 (T4), AC-05 (T1 set_default, T4), AC-06 (T1), AC-10/11/12 (T1 save, T4), AC-20 to 23 (T1, T3), AC-30 to 36 (T1 open_item, T2, T3), AC-40/41 (T4), AC-50 to 53 (T1 construction and no-argument model, T2 no shell path, T4 no launches in React).
- Names: `EditorCode` snake_case strings match `EditorCode` union in TS; `KINDS` matches `EDITOR_KINDS`.
- Constitution: P2 (Rust-only launch, bound params, migration 3), P3 (one Args/Result per command), P4 (tests listed, no real editor launches), P5 (opener is local).

## Deviations from the constitution's test wording

- Spec P4 asked for launch tests with a fake executable. The plan uses a recording `Launcher` and never spawns, so no test starts a process. The real `SystemLauncher::spawn` is covered by reading the constructed plan and by a missing-program test (`LaunchFailed`) that spawns nothing. Stated here so the reviewer can judge it.
