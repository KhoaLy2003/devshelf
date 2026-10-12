# F05 — Workspace restore with missing-path report: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/workspace-restore/spec.md` (approved; D1 one by one in position order; D2 Ignore for the current panel only; D3 missing and unreachable as separate groups)
- **Branch**: `feature/workspace-restore`, cut from `feature/workspace-builder` (commit `eedf4c5`)

**Goal:** One-click restore of a workspace: each item is classified by metadata (available, missing, kind mismatch, unreachable), available items open through their stored editor profile, and the results panel reports counts and groups with Relink and Ignore. Completed runs set last-used.

**Architecture:** The `workspaces` module gets a `restore` function that takes a `PathProbe` trait (real: `std::fs::metadata`; tests: a map of canned results) and a `Launcher`-backed open via `editors::open_with_profile`. Restore returns a per-item outcome list in position order. `last_restored_at` is written only when the run completes. Relink and Apply reuse the F04 path validation and duplicate rule, plus the kind check. Ignore is UI-only state.

**Tech Stack:** no new dependencies.

## Global Constraints

- Metadata only on item paths. No reads or writes inside items (AC-60).
- Opening goes through `editors::open_with_profile` (F03). No shell, no new process paths.
- Bound parameters; migration 5 is numbered; migrations 1 to 4 untouched.
- One Args and one Result per command; snake_case codes; locks only around database work; path probes and launches run outside the lock.
- Tests use temp directories and a recording launcher. No test starts a real editor.

## Contracts

Rust `src-tauri/src/workspaces/restore.rs` (new module file under `workspaces/`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult { Found { is_dir: bool }, NotFound, Unreachable }

pub trait PathProbe { fn probe(&self, path: &str) -> ProbeResult; }
pub struct FsProbe;   // std::fs::metadata; NotFound -> NotFound; other errors -> Unreachable

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus { Available, Missing, KindMismatch, Unreachable }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOutcome {
    Opened { item_id: i64 },
    Missing { item_id: i64, reason: MissingReason },
    Unreachable { item_id: i64 },
    CouldNotOpen { item_id: i64, code: EditorCode },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingReason { NotFound, NowFolder, NowFile }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport { pub workspace_id: i64, pub outcomes: Vec<ItemOutcome>, pub completed: bool }

pub fn classify(probe: &dyn PathProbe, path: &str, stored_kind: &str) -> ItemStatus;   // and MissingReason derived
pub fn restore(storage: &Storage, probe: &dyn PathProbe, launcher: &dyn editors::Launcher, workspace_id: i64) -> Result<RestoreReport, WorkspaceError>;
pub fn mark_restored(storage: &Storage, workspace_id: i64) -> Result<(), WorkspaceError>;  // sets last_restored_at = now (UTC ISO)
pub fn relink(storage: &Storage, probe: &dyn PathProbe, item_id: i64, raw_path: &str) -> Result<Workspace, WorkspaceError>;
```

`restore` order: load workspace (WorkspaceNotFound if missing); if no items, return `completed = true` with no outcomes (the UI disables the button, so this is a guard only). For each item in position order: classify; if Available, resolve the item's `editor_kind` via `editors::resolve_profile(Some(kind))` and call `editors::open_with_profile(launcher, profile, path)`; map `Ok` to `Opened`, `Err(e)` to `CouldNotOpen { code: e.code }`. Missing and mismatch map to `Missing` with the reason. Unreachable maps to `Unreachable`. Profile lookups and probes happen with the storage lock released between items; the lock is taken only to read items at the start and to call `mark_restored` at the end when `completed` is true. `completed` is set true after the loop. The caller (command) calls `mark_restored` only when `completed` is true.

`relink`: `validate_item_path` (F04 rules) for shape; `probe` for existence and kind; kind must match the stored kind (`item_kind_mismatch` with the stored kind as detail); duplicate check through the existing `UNIQUE(workspace_id, path_key)` mapping (`ItemDuplicate`); update path and path_key in one statement. Relink does not open anything.

Migration 5 (`db/migrations.rs`):

```sql
ALTER TABLE workspaces ADD COLUMN last_restored_at TEXT;
```

`WorkspaceView` gains `last_restored_at: Option<String>`; `WorkspaceSummaryView` gains the same field so W1 can show it.

Commands `src-tauri/src/commands/restore.rs` (or added to `commands/workspaces.rs`; use `commands/workspaces.rs` to keep one module per domain):

| Command | Args | Result |
|---|---|---|
| `workspace_restore` | `WorkspaceRestoreArgs { id }` | `WorkspaceRestoreResult { outcomes: Vec<RestoreOutcomeView>, completed: bool, error: Option<WorkspaceCode>, detail: Option<String> }` |
| `workspace_items_relink` | `ItemRelinkArgs { item_id, path }` | `WorkspaceResult` (existing shape) |

`RestoreOutcomeView` is serde-tagged: `{ kind: "opened"|"missing"|"unreachable"|"could_not_open", item_id, reason: Option<"not_found"|"now_folder"|"now_file">, code: Option<EditorCode snake_case> }`.

`workspace_restore` is `#[tauri::command(async)]`. It builds `AppOpener` (already in `commands/editors.rs`; make it `pub(crate)` and reuse it) and an `FsProbe`.

Picker: the Relink picker reuses `workspaces_pick_item` with the item kind from the UI (`folder` flag). No new picker command.

TypeScript `src/types/commands.ts`: `RestoreOutcomeView`, `WorkspaceRestoreArgs/Result`, `ItemRelinkArgs`; add `last_restored_at` to the view types. `WorkspaceCode` gains `no_items`, `item_kind_mismatch`, `storage_unavailable` already present, so only `no_items` and `item_kind_mismatch` are added (update the union and the Rust enum together).

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 5) | 1 |
| `src-tauri/src/db/mod.rs` | Modify (migration count 5) | 1 |
| `src-tauri/src/workspaces/restore.rs` | Create | 1 |
| `src-tauri/src/workspaces/mod.rs` | Modify (`last_restored_at` in Workspace and summary; `WorkspaceCode` gains `NoItems`, `ItemKindMismatch`; `mod restore;`) | 1 |
| `src-tauri/src/commands/workspaces.rs` | Modify (restore, relink, view fields, command registration) | 2 |
| `src-tauri/src/commands/editors.rs` | Modify (`AppOpener` to `pub(crate)`) | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Modify (handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify | 3 |
| `src/lib/restore-form.ts` | Create (`restoreSummaryLine`, `groupOutcomes`, `outcomeReason`, `formatRelinkError`) | 3 |
| `tests/restore-form.test.ts` | Create | 3 |
| `src/components/restore-panel.tsx` | Create (counts, groups, Close, Ignore, Relink inline) | 4 |
| `src/screens/workspace-detail-screen.tsx` | Modify (Restore button and note; panel) | 4 |
| `src/screens/workspaces-screen.tsx` | Modify (last used column: date or "Never restored") | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files; task 3 uses only the contract -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Migration 5, restore domain, Rust tests

**Specialist:** task (Rust)

**Steps:**
1. Tests first in `workspaces/restore.rs` with a `FakeProbe` (HashMap path to ProbeResult) and a `RecordingLauncher` from `editors` tests (or a local copy implementing `editors::Launcher`, recording calls, never spawning):
   - `classify`: Found file with stored file → Available; Found dir with stored file → Missing(NowFolder); NotFound → Missing(NotFound); Unreachable → Unreachable; Found file with stored folder → Missing(NowFile).
   - `restore` over three items (file ok, missing, folder ok): outcomes in position order; launcher called twice (windows_default → `open_default` once per available item); completed is true.
   - An item whose stored profile's executable was removed → `CouldNotOpen { code: ExecutableNotFound }` and the run continues to the next item (AC-13).
   - An unreachable item is reported as Unreachable and not opened.
   - `restore` on an empty workspace returns completed with no outcomes and launches nothing.
   - `mark_restored` sets `last_restored_at`; `restore` alone does not (test: after `restore`, `get` shows None; after `mark_restored`, Some).
   - `relink`: kind mismatch → `ItemKindMismatch` with detail = stored kind; duplicate path → ItemDuplicate; valid relink updates path and path_key, item id unchanged, position unchanged; relink does not call the launcher.
   - No writes inside item paths: record mtimes and entries of an item folder before and after `restore` and `relink`; equal.
   - Migration 5 reopen: `last_restored_at` survives reopen.
2. Run `cargo test workspaces::restore` and `cargo test db::`; expect failure first (note if the red run is skipped).
3. Implement migration 5, `FsProbe`, `classify`, `restore`, `mark_restored`, `relink`, and the view field additions. Keep `restore` lock-free between items: read items under one short lock, then release.
4. Update the migration-count assertion to 5.
5. Run both filters; all pass.

**Acceptance:** AC-03, AC-10 to AC-13, AC-20 to AC-22 (domain), AC-30 to AC-34 (domain), AC-50, AC-60, AC-62, AC-63. Constitution P2, P4.

---

### Task 2: Commands and wiring

**Specialist:** task (Rust)

**Steps:**
1. Make `AppOpener` in `commands/editors.rs` `pub(crate)`.
2. In `commands/workspaces.rs`, add `workspace_restore` (`async`): read the workspace under the lock only for the item list, run `restore` with `FsProbe` and `AppOpener` outside the lock, then take the lock once to call `mark_restored` when `completed`. Map `WorkspaceError` to `error`/`detail`.
3. Add `workspace_items_relink` (`async`): `relink` with `FsProbe`. Probe and path shape checks run before the lock; the lock covers the update only. Map to `WorkspaceResult`.
4. Add `last_restored_at` to the view conversions.
5. Pure mapping tests: `NoItems` and `ItemKindMismatch` snake_case; each `RestoreOutcome` variant maps to its view `kind` string and reason/code fields.
6. Register both commands in `generate_handler!`.
7. Run `cargo test` and `cargo build` from `src-tauri/`; no warnings from touched files.

**Acceptance:** Constitution P3; AC-03, AC-11, AC-30 to AC-32 wiring, AC-61.

---

### Task 3: TypeScript contract and restore helpers

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: add `no_items` and `item_kind_mismatch` to `WorkspaceCode`; `RestoreOutcomeView` (discriminated by `kind`); `WorkspaceRestoreArgs`, `WorkspaceRestoreResult`, `ItemRelinkArgs`; `last_restored_at: string | null` on the view types.
2. `src/lib/commands.ts`: `restoreWorkspace(id)`, `relinkWorkspaceItem(itemId, path)`.
3. `src/lib/restore-form.ts`:
   - `restoreSummaryLine(opened, missing, unreachable, failed, ignored)`: "All N items opened." when only opened; otherwise "N opened, M missing, U unreachable, F could not open." with "(ignored K)" appended to the missing count when K > 0 (AC-41). Zero counts stay shown.
   - `groupOutcomes(outcomes)`: returns `{ missing, unreachable, couldNotOpen }` in position order; empty groups are empty arrays (the panel hides them).
   - `outcomeReason(outcome)`: "Path not found." for NotFound; "Now a folder, not a file." / "Now a file, not a folder."; "Can't be reached. The drive may not be ready or the network path is unavailable."; for could_not_open, the F03 open copy for the code (import from `editor-form.ts` `formatOpenError`).
   - `formatRelinkError(code, detail)`: item_kind_mismatch → "This is a folder. Pick a file." / "This is a file. Pick a folder." (chosen by detail: stored kind); item_path_not_found → "`<path>` was not found."; item_relative_path → "Enter an absolute path, for example `F:\\work\\notes.txt`."; item_duplicate → "`<path>` is already in this workspace."; item_path_empty → "Enter a file or folder path."; other codes fall back to `formatWorkspaceError`.
   - `relinkedNotice()`: "Relinked. It will open on the next restore."
4. Tests first in `tests/restore-form.test.ts`: summary line variants (all opened, mixed, ignored count); grouping order and empty groups; each reason string; relink error copy for the kind mismatch both ways.
5. Run `npx vitest run tests/restore-form.test.ts`; all pass.

**Acceptance:** AC-02, AC-20, AC-21, AC-22, AC-31, AC-32, AC-41 copy; constitution P3, P4.

---

### Task 4: Restore UI

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy fixed by the spec; layout within the preview.
**Frontend dispatch contract:** `design_md_fallback`.

**Steps:**
1. `workspace-detail-screen.tsx`: add the Restore button (disabled with no items; text "Restoring…" while the mutation runs; AC-03, AC-04) and the note from AC-02 beside it. On completion, store the outcomes in component state (results are not persisted, AC-62). If `completed`, invalidate `["workspaces"]` and `["workspace", id]` so last-used updates (AC-50). If the error is `storage_unavailable`, show the F04 storage copy.
2. `restore-panel.tsx`: first line from `restoreSummaryLine`; groups from `groupOutcomes` in the order Missing, Unreachable, Could not open (AC-21), hiding empty groups (AC-22). Each row: item name (last path segment), full path monospace, reason. Relink control on missing and unreachable rows: opens an inline field with Browse (file or folder by stored kind via `pickWorkspaceItem`), Apply (`relinkWorkspaceItem`; on success show `relinkedNotice`, update the cache, and keep the row labelled relinked), Cancel (AC-34). Ignore control on missing and unreachable rows: hides the row for this panel only, adds to the ignored count (AC-40, AC-41). Close button hides the panel (AC-23).
3. `workspaces-screen.tsx`: add a last-used column: formatted date and time from `last_restored_at`, or "Never restored" when null (AC-51).
4. Run `npm run build` once and report.

**Acceptance:** AC-01 to AC-04, AC-20 to AC-23, AC-30 to AC-34, AC-40, AC-41, AC-50, AC-51, AC-62.

---

### Task 5: Verification

Run after all tasks: `npx prettier --write` on touched files, `npm run build`, `npm run lint`, `npx vitest run`, `cargo test`, `cargo build`.

## Self-review

- Spec coverage: AC-01 to 04 (T4), AC-10 to 13 (T1, T4), AC-20 to 23 (T3, T4), AC-30 to 34 (T1, T2, T3, T4), AC-40/41 (T3, T4), AC-50/51 (T1, T4), AC-60 to 63 (T1 mtime test, T2 wiring, T1 migration).
- Names: `ItemStatus`, `ItemOutcome`, `RestoreOutcomeView` kinds match the TS union; codes match across Rust and TS.
- Constitution: P2 (Rust only probes and launches; editors through its public function), P3 (one Args/Result per command), P4 (tests listed; recording launcher), P5 (no network).

## Notes

- `classify` for a missing item does not distinguish a parent folder that is missing from the file being missing; both report NotFound, matching the spec's "was not found".
- Ignore state lives in the React component, so it resets on a new Restore, which is the intended D2 behaviour.
