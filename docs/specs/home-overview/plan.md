# F10 — Home overview: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/home-overview/spec.md` (approved; D1 heading only; D2 Register navigates to Projects and opens the existing dialog; D3 missing count stored at each completed restore)
- **Branch**: `feature/home-overview`, cut from `feature/workspace-restore` (commit `fbf18da`)

**Goal:** A Home screen with pinned projects and recent workspaces side by side, a last-used line and a stored missing-item warning per workspace, a Register project action, and an empty state.

**Architecture:** One read command `home_overview` returns pinned projects and recent workspaces together (AC-51), built from the public functions of `projects` and `workspaces`. Migration 6 adds `workspaces.last_restore_missing`. The F05 restore path writes that count through `mark_restored`. Register project navigates to `/projects` with router state `{ openRegister: true }`, which the F02 projects screen reads to open its existing dialog.

**Tech Stack:** no new dependencies.

## Global Constraints

- Only Rust runs SQL and touches the file system. Bound parameters everywhere. Migration 6 is numbered; migrations 1 to 5 untouched.
- One Args and one Result per command; snake_case codes; no new persisted UI state.
- Tests use temp directories. No real data folder.

## Contracts

Rust `src-tauri/src/workspaces/mod.rs`:

```rust
pub struct HomeWorkspace { pub id: i64, pub name: String, pub item_count: i64, pub project_names: Vec<String>, pub last_restored_at: Option<String>, pub last_restore_missing: i64 }
pub fn set_restore_outcome(storage: &Storage, workspace_id: i64, missing: i64) -> Result<(), WorkspaceError>;  // replaces mark_restored: sets last_restored_at = now and last_restore_missing = missing
```

`mark_restored` is replaced by `set_restore_outcome(storage, id, missing)` in F05's restore command. The restore command computes `missing` as the count of `Missing` plus `Unreachable` outcomes in the report (ignored rows are a UI concept and do not affect it).

Ordering for recent workspaces: `ORDER BY (last_restored_at IS NULL), last_restored_at DESC, name_key`. The `workspace` list functions return that order to the command; the UI keeps it.

`src-tauri/src/projects/mod.rs`: `list` already returns pinned first. Home filters `pinned` in the command (no new function).

`src-tauri/src/commands/home.rs` (new):

```rust
#[derive(Deserialize)] pub struct HomeOverviewArgs {}
#[derive(Serialize)] pub struct HomeOverviewResult { pub pinned_projects: Vec<HomeProjectView>, pub recent_workspaces: Vec<HomeWorkspaceView>, pub error: Option<HomeErrorCode>, pub detail: Option<String> }
#[derive(Serialize)] pub struct HomeProjectView { pub id: i64, pub display_name: String, pub root_path: String, pub drive: String }
#[derive(Serialize)] pub struct HomeWorkspaceView { pub id: i64, pub name: String, pub item_count: i64, pub project_names: Vec<String>, pub last_restored_at: Option<String>, pub last_restore_missing: i64 }
#[derive(Serialize)] #[serde(rename_all = "snake_case")] pub enum HomeErrorCode { StorageUnavailable }
#[tauri::command(async)] pub fn home_overview(args: HomeOverviewArgs, state: State<'_, StorageState>) -> HomeOverviewResult
```

The command takes the lock once, reads both lists through the domain functions, releases it, and returns. `async` because it is a read but keeps the rule of no blocking on the main thread.

Migration 6 (`db/migrations.rs`):

```sql
ALTER TABLE workspaces ADD COLUMN last_restore_missing INTEGER NOT NULL DEFAULT 0;
```

TypeScript `src/types/commands.ts`: `HomeOverviewArgs`, `HomeOverviewResult`, `HomeProjectView`, `HomeWorkspaceView`, `HomeErrorCode = "storage_unavailable"`.

Router: `src/lib/home-nav.ts` exports `REGISTER_PROJECT_STATE = { openRegister: true } as const`. The F02 projects screen reads `useLocation().state?.openRegister` once on mount, opens its dialog, and clears the state with `navigate(".", { replace: true, state: null })`.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/migrations.rs` | Modify (migration 6) | 1 |
| `src-tauri/src/db/mod.rs` | Modify (migration count 6) | 1 |
| `src-tauri/src/workspaces/mod.rs` | Modify (`set_restore_outcome`, `HomeWorkspace`, `list_for_home`) | 1 |
| `src-tauri/src/projects/mod.rs` | Modify only if a test helper is needed (no behaviour change) | 1 |
| `src-tauri/src/commands/workspaces.rs` | Modify (restore uses `set_restore_outcome` with the missing count) | 2 |
| `src-tauri/src/commands/home.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Modify (module, handler) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (`getHomeOverview`) | 3 |
| `src/lib/home-form.ts` | Create (`workspaceItemsLabel`, `missingWarning`, `capList`, `lastUsedLabel`) | 3 |
| `src/lib/home-nav.ts` | Create (`REGISTER_PROJECT_STATE`) | 3 |
| `tests/home-form.test.ts` | Create | 3 |
| `src/screens/projects-screen.tsx` | Modify (read router state, open Register dialog once) | 4 |
| `src/screens/home-screen.tsx` | Replace F01 placeholder | 4 |
| `src/components/home-section.tsx` | Create | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Migration 6 and domain queries (Rust)

**Specialist:** task (Rust)

**Steps:**
1. Tests first in `workspaces/mod.rs` (temp storage):
   - `set_restore_outcome` sets `last_restored_at` and `last_restore_missing`; a second call overwrites both.
   - `list_for_home` order: a restored workspace (older) and a never-restored one: restored first; two never-restored sorted by name_key; two restored sorted by `last_restored_at` desc.
   - `list_for_home` item_count and project_names match the summary function.
   - `last_restore_missing` defaults to 0 for a new workspace.
   - Migration 6 reopen keeps the stored count.
2. Run `cargo test workspaces::`; expect failure first (note if skipped).
3. Implement migration 6, `set_restore_outcome` (one UPDATE with bound parameters), and `list_for_home` (one query for workspaces ordered as the Contracts state, then the same per-workspace project-name lookup the summary uses).
4. Update the migration-count assertion to 6.
5. Run `cargo test workspaces::` and `cargo test db::`; all pass.

**Acceptance:** AC-30 (ordering), AC-31 (domain fields), AC-40, AC-41, AC-51 (domain part). Constitution P2, P4.

---

### Task 2: Restore wiring and home command (Rust)

**Specialist:** task (Rust)

**Steps:**
1. In `commands/workspaces.rs`, replace the `mark_restored` call in `run_restore` with `set_restore_outcome(s, id, missing)` where `missing` counts `Missing` and `Unreachable` outcomes. Keep the completed-only rule (only on `report.completed`). Keep the F05 M2 behaviour (report kept when the save fails).
2. Remove `mark_restored` if nothing else uses it; if the F05 tests use it, switch them to `set_restore_outcome`.
3. Add a test: a completed run with two missing items stores 2; a run that does not complete stores nothing.
4. Create `commands/home.rs` per the Contracts; `home_overview` takes the lock once, calls `projects::list` (filter pinned, keep the order) and `workspaces::list_for_home`, maps to views, and returns `storage_unavailable` on failure.
5. Pure mapping tests: `HomeErrorCode` snake_case string; pinned filter keeps only pinned projects in order.
6. Register `home_overview` in `generate_handler!`; add `pub mod home;` in `commands/mod.rs`.
7. Run `cargo test --lib` and `cargo build` from `src-tauri/`; no warnings from touched files.

**Acceptance:** AC-40, AC-50, AC-51, P3.

---

### Task 3: TypeScript contract and home helpers

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: the types in the Contracts section; `HomeErrorCode` union.
2. `src/lib/commands.ts`: `getHomeOverview()` calls `invoke("home_overview", { args: {} })`.
3. `src/lib/home-nav.ts`: `REGISTER_PROJECT_STATE`.
4. `src/lib/home-form.ts`:
   - `workspaceItemsLabel(count)`: "0 items", "1 item", "N items".
   - `missingWarning(count)`: returns null for 0; "1 item missing at last restore" for 1; "N items missing at last restore" otherwise.
   - `capList(items, max)`: returns `{ shown, hiddenCount }`; `max` 6 by default in callers.
   - `lastUsedLabel(iso)`: "Never restored" for null; otherwise "Last restored <local date and time>" formatted with `Intl.DateTimeFormat` (en-GB style, local time zone; the function takes an injectable formatter argument for testing).
   - `emptyPinnedText` and `emptyRecentText` exact copy (AC-22, AC-34).
5. Tests first in `tests/home-form.test.ts`: label pluralisation for 0, 1, 2; missing warning for 0, 1, 2; capList with 0, 6, 7 items; lastUsedLabel for null and for a fixed ISO string with an injected formatter; exact copy strings.
6. Run `npx vitest run tests/home-form.test.ts`; all pass. Run `npx tsc --noEmit`; report.

**Acceptance:** AC-22, AC-23, AC-31, AC-32, AC-34, AC-35 copy and logic.

---

### Task 4: Home screen and Register navigation

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy are fixed by the spec; layout follows the preview's Home.
**Frontend dispatch contract:** `design_md_fallback` (`docs/DESIGN.md`, `docs/preview/index.html` Home). Dark theme; Tailwind tokens.

**Steps:**
1. `home-screen.tsx`: query `["home"]` via `getHomeOverview`. Heading "Home" and lead "Pick up where you left off." (AC-03). "Register project" button (AC-10): navigates to `/projects` with `state: REGISTER_PROJECT_STATE`. No Quick open button (AC-11, AC-12).
2. If both lists are empty (AC-42): show the empty state with "Nothing here yet", the message from AC-42, and the "Register your first project" button (same navigation).
3. Otherwise two sections side by side on wide windows and stacked on narrow (Tailwind grid), via `home-section.tsx`:
   - Pinned projects: rows with display name, root path (monospace), drive; link to `/projects/:id`; cap with `capList(…, 6)`; "All projects" link when more; empty text from `home-form`.
   - Recent workspaces: rows with name, `workspaceItemsLabel`, project names (or "No registered projects"), `lastUsedLabel`, and the `missingWarning` badge when non-null; link to `/workspaces/:id`; cap and "All workspaces" link likewise; empty text.
   - Names truncate with CSS and carry the full name in `title`.
4. `projects-screen.tsx`: on mount read `useLocation().state?.openRegister`; if true, open the Register dialog once and clear the state with `navigate(".", { replace: true, state: null })`. No other change.
5. Run `npm run build` once and report.

**Acceptance:** AC-01 to AC-03, AC-10 to AC-12, AC-20 to AC-23, AC-30 to AC-35, AC-42, D2 wiring.

---

### Task 5: Verification

Run after all tasks: `npx prettier --write` on touched files, `npx tsc --noEmit`, `npm run build`, `npm run lint`, `npx vitest run`, `cargo test --lib`, `cargo build`.

## Self-review

- Spec coverage: AC-01/02/03 (T4), AC-04 (no greeting, T4), AC-10 to 12 (T4), AC-20 to 23 (T3 helpers, T4), AC-30 to 35 (T1 order, T3 labels, T4), AC-40/41 (T1, T2), AC-42 (T4), AC-50 to 52 (T2 one command, T4 no extra persistence).
- Names: `set_restore_outcome` replaces `mark_restored` everywhere; `HomeErrorCode` matches the TS union.
- Constitution: P2 (no SQL outside workspaces module; one command), P3 (one Args/Result), P4 (tests listed), P5 (no network).

## Notes

- F10 changes F05 behaviour only by what it stores: the missing count. Existing workspaces restored before migration 6 show 0 until their next completed restore; this is acceptable because the migration cannot know the old count.
