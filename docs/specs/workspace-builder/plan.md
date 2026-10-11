# F04 — Workspace builder: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/workspace-builder/spec.md` (approved; D1 reject duplicate paths; D2 default profile stored on the item at add time; D3 move up/down buttons)
- **Branch**: `feature/workspace-builder`, cut from `feature/editor-profiles` (commit `ad0bd40`)

**Goal:** Named, ordered workspaces of file and folder references, each item with an optional project and a stored editor profile, persisted in SQLite, edited through a builder screen. No file or folder inside an item is read or changed.

**Architecture:** A `workspaces` Rust module owns names, item paths, ordering, and persistence. It calls `projects::get`/`projects::list` and `editors::resolve_profile` for references, and does not touch their tables. Migration 4 creates `workspaces` and `workspace_items`. Commands wrap the module. Mutations return the full updated workspace so the UI replaces one cache entry.

**Tech Stack:** no new dependencies. Uses existing `tauri-plugin-dialog` for pickers.

## Global Constraints

- Rust only touches the database and file system. Bound parameters everywhere. Migration 4 is a new numbered migration.
- Item metadata check reads only `std::fs::metadata` (kind and existence). No reads or writes inside items (AC-23).
- Commands: one Args and one Result each; user-facing codes only.
- Tests use temp directories. Never touch the real data folder or a real project folder.

## Contracts

Rust `src-tauri/src/workspaces/mod.rs`:

```rust
pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_PATH_CHARS: usize = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCode {
    NameEmpty, NameTooLong, NameTaken, WorkspaceNotFound,
    ItemPathEmpty, ItemPathTooLong, ItemRelativePath, ItemPathNotFound, ItemDuplicate,
    ItemNotFound, ProjectNotFound, ProfileNotFound, MoveBlocked, StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceError { pub code: WorkspaceCode, pub detail: Option<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item { pub id: i64, pub position: i64, pub kind: String /* "file"|"folder" */, pub path: String, pub project_id: Option<i64>, pub editor_kind: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace { pub id: i64, pub name: String, pub items: Vec<Item> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSummary { pub id: i64, pub name: String, pub item_count: i64, pub project_names: Vec<String> }

pub fn validate_name(raw: &str) -> Result<String, WorkspaceCode>;            // trim; empty, > MAX_NAME_CHARS
pub fn validate_item_path(raw: &str) -> Result<(String, String), WorkspaceError>; // (normalised path, kind); checks empty, length, absolute (drive or UNC), metadata
pub fn list(storage: &Storage) -> Result<Vec<WorkspaceSummary>, WorkspaceError>;  // sorted by name_key
pub fn get(storage: &Storage, id: i64) -> Result<Option<Workspace>, WorkspaceError>;
pub fn create(storage: &Storage, name: &str) -> Result<Workspace, WorkspaceError>;
pub fn rename(storage: &Storage, id: i64, name: &str) -> Result<Workspace, WorkspaceError>;
pub fn delete(storage: &Storage, id: i64) -> Result<(), WorkspaceError>;     // cascades items
pub fn add_item(storage: &Storage, workspace_id: i64, normalised_path: &str, kind: &str, project_id: Option<i64>, editor_kind: &str) -> Result<Workspace, WorkspaceError>;
pub fn update_item(storage: &Storage, item_id: i64, project_id: Option<i64>, editor_kind: &str) -> Result<Workspace, WorkspaceError>;
pub fn remove_item(storage: &Storage, item_id: i64) -> Result<Workspace, WorkspaceError>;
pub fn move_item(storage: &Storage, item_id: i64, up: bool) -> Result<Workspace, WorkspaceError>;
```

Position integrity: `UNIQUE(workspace_id, position)` is required; moves swap two positions inside one transaction using a temporary sentinel position (`-1`) so the constraint is never violated. `remove_item` renumbers the following items in the same transaction. `add_item` uses `max(position)+1`.

Project and profile checks happen in the commands layer or in `add_item`/`update_item` before the write: `projects::get(project_id)` must return Some (else `ProjectNotFound`); `editors::resolve_profile(Some(kind))` must succeed (else `ProfileNotFound`). `editor_kind` values come from `editors::KINDS`.

Migration 4 (`db/migrations.rs`):

```sql
CREATE TABLE workspaces (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  name_key TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE workspace_items (
  id INTEGER PRIMARY KEY,
  workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('file','folder')),
  path TEXT NOT NULL,
  path_key TEXT NOT NULL,
  project_id INTEGER,
  editor_kind TEXT NOT NULL,
  UNIQUE (workspace_id, position),
  UNIQUE (workspace_id, path_key)
);
```

`name_key` = lowercased trimmed name. `path_key` = normalised path lowercased. `UNIQUE (workspace_id, path_key)` enforces D1 at the database level; a violation maps to `ItemDuplicate`. No foreign key on `project_id` or `editor_kind`: those are validated by the module, so the workspaces module does not depend on the projects or editors tables.

Commands `src-tauri/src/commands/workspaces.rs` (one Args and one Result each; `#[tauri::command(async)]` for anything that checks paths or opens pickers; locks only around database work, with path checks done before locking):

| Command | Args | Result |
|---|---|---|
| `workspaces_list` | `WorkspacesListArgs {}` | `WorkspacesListResult { workspaces: Vec<WorkspaceSummaryView>, error: Option<WorkspaceCode>, detail: Option<String> }` |
| `workspaces_get` | `WorkspaceGetArgs { id }` | `WorkspaceResult { workspace: Option<WorkspaceView>, error, detail }` |
| `workspaces_create` | `WorkspaceCreateArgs { name }` | `WorkspaceResult` |
| `workspaces_rename` | `WorkspaceRenameArgs { id, name }` | `WorkspaceResult` |
| `workspaces_delete` | `WorkspaceDeleteArgs { id }` | `WorkspaceDeleteResult { error, detail }` |
| `workspace_items_add` | `ItemAddArgs { workspace_id, path, project_id: Option<i64>, editor_kind: Option<String> }` (None editor = default at add time) | `WorkspaceResult` |
| `workspace_items_update` | `ItemUpdateArgs { item_id, project_id, editor_kind }` | `WorkspaceResult` |
| `workspace_items_remove` | `ItemRemoveArgs { item_id }` | `WorkspaceResult` |
| `workspace_items_move` | `ItemMoveArgs { item_id, up: bool }` | `WorkspaceResult` |
| `workspaces_pick_item` | `WorkspacePickItemArgs { folder: bool }` | `WorkspacePickResult { path: Option<String> }` |

`WorkspaceCode` serialises snake_case. `editor_kind` on add defaults to `editors::resolve_profile(None).kind` inside the command, which is the D2 behaviour.

TypeScript `src/types/commands.ts`: mirror all structs; `WorkspaceCode` union (14 codes); `WORKSPACE_ITEM_KINDS = ["file", "folder"]` with comment that Rust is authoritative.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 4) | 1 |
| `src-tauri/src/workspaces/mod.rs` | Replace placeholder | 1 |
| `src-tauri/src/db/mod.rs` | Modify (migration-count test) | 1 |
| `src-tauri/src/commands/workspaces.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Modify (handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (workspace wrappers) | 3 |
| `src/lib/workspace-form.ts` | Create (`formatWorkspaceError`, `kindLabel`) | 3 |
| `tests/workspace-form.test.ts` | Create | 3 |
| `src/screens/workspaces-screen.tsx` | Modify (list, New workspace, empty state) | 4 |
| `src/components/new-workspace-dialog.tsx` | Create | 4 |
| `src/screens/workspace-detail-screen.tsx` | Create (builder: header with rename and delete, item table) | 4 |
| `src/components/add-item-dialog.tsx` | Create | 4 |
| `src/routes.tsx` | Modify (`workspaces/:id`) | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files; task 3 uses only the contract -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Migration 4, workspaces domain, Rust tests

**Specialist:** task (Rust)

**Steps:**
1. Tests first in `workspaces/mod.rs`, using temp directories for storage and for item folders and files (empty files; never executed):
   - `validate_name`: "  Ops  " → "Ops"; "" → NameEmpty; 101 chars → NameTooLong; 100 chars ok.
   - `validate_item_path`: empty → ItemPathEmpty; `.\\src` and `src` → ItemRelativePath; `F:\\missing` → ItemPathNotFound; existing file → kind "file"; existing folder → kind "folder"; `F:` normalises to `F:\\`; 261-char path → ItemPathTooLong. A UNC path to a non-existent share → ItemPathNotFound (no hang: the test must not wait on network; use a local non-existent UNC name and accept ItemPathNotFound or a metadata error mapped to ItemPathNotFound).
   - `create` then `list`: summary with item_count 0 and "No registered projects" represented as empty project_names; names sorted case-insensitively.
   - `create` duplicate name in different case → NameTaken; `rename` to the same workspace's own name in different case succeeds; `rename` to another workspace's name → NameTaken.
   - `add_item` appends positions 0,1,2 in order; duplicate normalised path (different case, trailing separator) → ItemDuplicate (D1); `project_id` pointing to a missing project is rejected by the command layer, not here: test `add_item` accepts an opaque project id and the command-layer test covers the lookup (note in test name).
   - `move_item`: moving the first item up → MoveBlocked; moving the last down → MoveBlocked; middle item up swaps with previous; positions remain 0..n-1 and unique after every move.
   - `remove_item` renumbers following items to 0..n-1; removing an unknown id → ItemNotFound.
   - `update_item`: changes project_id and editor_kind; path and kind unchanged (assert the stored path string equals the original after update).
   - `delete` cascades items: count of workspace_items rows for the id is 0 after delete; the item folders still exist on disk afterwards (AC-06).
   - Persistence: create and add, drop the Storage, `db::open_in` the same folder, `get` returns the same items in order (AC-20).
   - No write to item paths: record entries and mtimes of an item folder before and after add/move/remove; equal (AC-23).
2. Run `cargo test workspaces::`; FAIL expected first (note any deviation if the red step is skipped).
3. Implement migration 4 and the module per the contract. Each mutating function runs in one transaction. Use bound parameters only. `validate_item_path` runs before any database work.
4. Update the migration-count assertion in `db/mod.rs` to 4.
5. Run `cargo test workspaces::` and `cargo test db::`; all pass.

**Acceptance:** AC-04, AC-05 (domain), AC-12, AC-13, AC-14, AC-15 (domain), AC-17 via D1, AC-20, AC-21, AC-23, AC-25. Constitution P2, P4.

---

### Task 2: Commands and picker wiring

**Specialist:** task (Rust)

**Steps:**
1. `commands/workspaces.rs`: implement the ten commands per the table. For `workspace_items_add`: validate the path with `validate_item_path` first (no lock held); resolve the project with `projects::get` if `project_id` is Some (ProjectNotFound otherwise); resolve the editor kind (`editors::resolve_profile(None)` when `editor_kind` is None; `ProfileNotFound` for unknown kinds); then lock once for the insert. Map `WorkspaceError` to `WorkspaceResult` fields. Storage failure → `StorageUnavailable`.
2. `workspaces_pick_item` uses the dialog plugin's blocking file or folder picker, as in `editor_pick_item`.
3. Pure mapping tests: each `WorkspaceCode` maps to its snake_case string; `MoveBlocked` and `ItemDuplicate` results carry their detail.
4. Register the ten commands in `generate_handler!`.
5. Run `cargo test` and `cargo build` from `src-tauri/`; no warnings in touched files.

**Acceptance:** Constitution P3, P2; AC-11 (picker, default profile via D2), AC-24.

---

### Task 3: TypeScript contract, form helpers, Vitest

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: `WorkspaceCode` union (14 snake_case codes), `WorkspaceItemView`, `WorkspaceView`, `WorkspaceSummaryView`, and one Args and one Result per command. Use `Record<string, never>` for empty Args.
2. `src/lib/commands.ts`: `listWorkspaces`, `getWorkspace(id)`, `createWorkspace(name)`, `renameWorkspace(id, name)`, `deleteWorkspace(id)`, `addWorkspaceItem(args)`, `updateWorkspaceItem(args)`, `removeWorkspaceItem(itemId)`, `moveWorkspaceItem(itemId, up)`, `pickWorkspaceItem(folder)`.
3. `src/lib/workspace-form.ts`:
   - `formatWorkspaceError(code, detail, name)`: exact copy:
     - name_empty: "Enter a workspace name."
     - name_too_long: "Use at most 100 characters."
     - name_taken: "A workspace named `<name>` already exists."
     - workspace_not_found: "This workspace no longer exists."
     - item_path_empty: "Enter a file or folder path."
     - item_path_too_long: "This path is too long for Windows."
     - item_relative_path: "Enter an absolute path, for example `F:\\work\\notes.txt`."
     - item_path_not_found: "`<path>` was not found."
     - item_duplicate: "`<path>` is already in this workspace."
     - item_not_found: "This item no longer exists."
     - project_not_found: "That project no longer exists. Choose another project or No project."
     - profile_not_found: "That editor profile no longer exists."
     - move_blocked: "This item is already at that end of the list."
     - storage_unavailable: "DevShelf cannot reach its data folder. Restart DevShelf and try again."
   - `kindLabel(kind)`: "File" or "Folder".
4. Tests first in `tests/workspace-form.test.ts`: each code yields the exact copy (with the detail inserted); kindLabel; formatWorkspaceError uses detail for path codes.
5. Run `npx vitest run tests/workspace-form.test.ts`; all pass.

**Acceptance:** AC-04, AC-05, AC-06 copy, AC-12, AC-14, AC-15 messages; constitution P3, P4.

---

### Task 4: Workspace screens and dialogs

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy are fixed by the spec; layout follows the preview's Workspaces list and detail (without Restore).
**Frontend dispatch contract:** `design_md_fallback` (`docs/DESIGN.md`, `docs/preview/index.html` Workspaces screens). Dark theme; existing Tailwind tokens.

**Steps:**
1. `workspaces-screen.tsx`: query `["workspaces"]` via `listWorkspaces`. Empty state (F01 copy) with "New workspace" button (AC-02). Otherwise rows with name, item count, project names (or "No registered projects"); each row links to `/workspaces/:id`.
2. `new-workspace-dialog.tsx`: name field; create calls `createWorkspace`; on error show `formatWorkspaceError` inline and keep the value (AC-04); on success navigate to `/workspaces/:id` (AC-03). Cancel and Esc discard. Native `<dialog>` or accessible overlay with focus management.
3. `workspace-detail-screen.tsx`: query `["workspace", id]`. Header with name, Rename (inline field with the same rules; AC-05), Delete (confirmation text from AC-06; on confirm, invalidate `["workspaces"]` and navigate to `/workspaces`). Item table (AC-10): position number, kind label, name (last path segment), full path monospace, project select ("No project" plus registered projects, from `listProjects`), editor profile select (from `listEditorProfiles`), Move up and Move down buttons (disabled at ends; AC-15), Remove with confirmation text (AC-14). Project and profile selects save immediately (AC-16) via `updateWorkspaceItem`. Empty state "This workspace has no items." (no Restore control). "Add item" opens the add dialog. Keep all mutation results in the `["workspace", id]` cache.
4. `add-item-dialog.tsx`: "Browse file...", "Browse folder..." (both via `pickWorkspaceItem`), typed path field, project select (default No project), editor profile select (default: the profile flagged as default in `["editor-profiles"]`, which the server also defaults to when omitted). Submit calls `addWorkspaceItem`; errors shown next to the path field via `formatWorkspaceError`; values kept; success closes and refreshes the workspace.
5. `routes.tsx`: add `workspaces/:id` under the shell.
6. Run `npm run build` once and report.

**Acceptance:** AC-01 to AC-06 UI, AC-10 to AC-16 UI.

---

### Task 5: Verification

Run after all tasks: `npx prettier --write` on touched files, `npm run build`, `npm run lint`, `npx vitest run`, `cargo test`, `cargo build`.

## Self-review

- Spec coverage: AC-01/02/03 (T4), AC-04/05 (T1, T3, T4), AC-06 (T1 cascade, T4), AC-10 (T4), AC-11 (T2, T4), AC-12 (T1, T3), AC-13 (T1), AC-14 (T1, T4), AC-15 (T1, T4), AC-16 (T1 update_item, T4), AC-17/D1 (T1 unique constraint and test), AC-20 (T1), AC-21 (T1 path normalisation), AC-22 (no FK; project_id opaque), AC-23 (T1 mtime test), AC-24 (T2 pickers), AC-25 (T1 transactions).
- Names: `WorkspaceCode` strings match the TS union; command table matches the TS Args.
- Constitution: P2 (no table reach-through; FKs only within this module's tables), P3 (one Args/Result per command), P4 (tests listed), P5 (no network).

## Notes

- The `project_id` is not a foreign key so the module does not depend on the projects table. F15 (project maintenance) may later delete projects; the item then reports `project_not_found` on read and the UI shows "No project" (AC-22).
