# F01 — Foundation and navigation: Implementation Plan

- **Status**: `draft — awaiting Step 2 soft-gate summary`
- **Spec**: `docs/specs/foundation-and-navigation/spec.md` (approved; D1–D4 at recommended defaults)
- **Architecture**: `docs/architecture.md`; **Constitution**: `docs/constitution.md` v1.0.0
- **Branch (Step 3)**: `feature/foundation-and-navigation`

**Goal:** A DevShelf window with a seven-area sidebar, text-only empty states, a single SQLite database in the app data folder with numbered migrations, a single-instance guard, and a typed storage-status command that the UI uses to gate the shell.

**Architecture:** Rust owns the database: `db::open_in(folder)` creates the folder, checks `user_version` before any write, and applies migrations. Tauri `setup` opens storage once and stores the result in managed state. One command, `storage_status`, returns the result to the UI. The UI shows a full-window storage error when storage failed; otherwise it renders the shell with React Router hash routes and TanStack Query for the status call.

**Tech Stack:** Tauri 2, Rust (`rusqlite` bundled 0.37, `rusqlite_migration` 2, `tauri-plugin-single-instance` 2, `tempfile` dev-only), React 19, TypeScript, Vite 8, Tailwind CSS v4 (`@tailwindcss/vite`), React Router (`react-router`), TanStack Query (`@tanstack/react-query`), Vitest.

## Global Constraints

- No network calls, telemetry, or updater. Do not add any network plugin.
- Dark theme only. Visual tokens come from `docs/DESIGN.md` and `docs/preview/index.html` (`design_md_fallback`).
- Copy is English. Empty-state wording is taken verbatim from `docs/preview/index.html`, with the Home text trimmed as in Task 6.
- Only these new runtime dependencies: `react-router`, `@tanstack/react-query`, `tailwindcss`, `@tailwindcss/vite`, `tauri-plugin-single-instance`. No font package (see Task 3). `cmdk`, shadcn/ui components, and `tauri-plugin-dialog`/`opener`/`clipboard-manager` are not added in F01 (no F01 surface needs them).
- Every SQL statement uses bound parameters or is a fixed migration string. Schema changes only through `rusqlite_migration`.
- Every Tauri command has one argument struct and one return type, derives serde, returns a user-facing error code, and has its TypeScript types only in `src/types/commands.ts`.
- Tests use temporary directories. Never touch the real app data folder.
- No code is written into any project folder.

## Spec clarification (recorded for the Step 2 summary)

AC-31 says Settings shows the storage error code when storage failed, but AC-25 says a storage failure replaces the whole shell with S9. S9 wins: when storage fails, Settings is never rendered, so the error row is unreachable. Settings shows the storage row only in the OK state (folder path and "Storage: OK"). This removes an unreachable branch and changes no behaviour a user can see. Flagged for the reviewer, not silently changed.

## Contracts (shared by all tasks)

Rust, `src-tauri/src/db/mod.rs`:

```rust
pub const DB_FILE_NAME: &str = "devshelf.sqlite3";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError { Unavailable, SchemaTooNew }

pub struct Storage { /* owns rusqlite::Connection; schema_version: i64 */ }
impl Storage {
    pub fn schema_version(&self) -> i64;
}

/// Creates `folder` if missing, opens `folder/devshelf.sqlite3`, refuses to write
/// when user_version is newer than this build, otherwise applies pending migrations.
pub fn open_in(folder: &Path) -> Result<Storage, StorageError>;
```

Rust, `src-tauri/src/commands/storage.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct StorageStatusArgs {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageErrorCode { StorageUnavailable, SchemaTooNew }

#[derive(Debug, Serialize)]
pub struct StorageStatus {
    pub ok: bool,
    pub folder: String,
    pub error_code: Option<StorageErrorCode>,
}

/// Managed Tauri state, created once in `setup`.
pub struct StorageState { pub folder: PathBuf, pub result: Result<db::Storage, db::StorageError> }

#[tauri::command]
pub fn storage_status(args: StorageStatusArgs, state: State<'_, StorageState>) -> StorageStatus;
```

`commands::app_info` gains `pub struct AppInfoArgs {}` as its argument, to match the one-argument-struct rule. It is the only existing command.

TypeScript, `src/types/commands.ts` (the only place these types live):

```ts
export interface AppInfoArgs {}
export interface StorageStatusArgs {}
export type StorageErrorCode = "storage_unavailable" | "schema_too_new";
export interface StorageStatus { ok: boolean; folder: string; error_code: StorageErrorCode | null; }
```

`src/lib/commands.ts`: `appInfo(): Promise<AppInfo>` and `storageStatus(): Promise<StorageStatus>`, both calling `invoke` with `{ args: {} }`.

Query key: `["storage-status"]`, used by the gate and by Settings.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/src/db/mod.rs` | Modify (replace placeholder) | 1 |
| `src-tauri/src/db/migrations.rs` | Create | 1 |
| `src-tauri/Cargo.toml` | Modify (`tempfile` dev-dependency) | 1 |
| `src-tauri/src/commands/storage.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs` | Modify (`AppInfoArgs`, export storage) | 2 |
| `src-tauri/src/lib.rs` | Modify (setup, managed state, single-instance, handler) | 2 |
| `src-tauri/Cargo.toml` | Modify (`tauri-plugin-single-instance`) | 2 |
| `package.json` | Modify (deps) | 3 |
| `vite.config.ts` | Modify (Tailwind plugin, Vitest include) | 3 |
| `src/styles.css` | Modify (Tailwind import, dark tokens) | 3 |
| `src/types/commands.ts` | Modify | 4 |
| `src/lib/commands.ts` | Modify | 4 |
| `src/lib/nav.ts` | Create | 4 |
| `src/lib/storage-messages.ts` | Create | 4 |
| `tests/nav.test.ts` | Create | 4 |
| `tests/storage-messages.test.ts` | Create | 4 |
| `src/app.tsx` | Modify (QueryClient, storage gate, router) | 5 |
| `src/router.tsx` | Create | 5 |
| `src/components/app-shell.tsx` | Create | 5 |
| `src/components/sidebar.tsx` | Create | 5 |
| `src/components/nav-link.tsx` | Create | 5 |
| `src/screens/not-found-screen.tsx` | Create | 5 |
| `src/screens/storage-error-screen.tsx` | Create | 5 |
| `src/components/empty-state.tsx` | Create | 6 |
| `src/screens/home-screen.tsx` | Create | 6 |
| `src/screens/projects-screen.tsx` | Create | 6 |
| `src/screens/workspaces-screen.tsx` | Create | 6 |
| `src/screens/config-screen.tsx` | Create | 6 |
| `src/screens/vault-screen.tsx` | Create | 6 |
| `src/screens/notes-screen.tsx` | Create | 6 |
| `src/screens/settings-screen.tsx` | Create | 6 |
| `src-tauri/tauri.conf.json` | Modify (window title, min size) | 2 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3, 4 — disjoint files, no dependency between them -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 5 — depends on tasks 2, 3, 4 -->
<!-- Parallel group D: task 6 — depends on task 5 -->
<!-- Task 7 (final verification) — after all -->

Tasks 1 and 2 touch `src-tauri/Cargo.toml`; task 1 adds only the `tempfile` dev-dependency and task 2 adds only the plugin, so run them in sequence on that file, not concurrently. Task 1 must finish before task 2 starts.

---

### Task 1: Storage open, migrations, and Rust tests

**Specialist:** backend-developer
**Interfaces:**
- Produces: `db::open_in`, `db::Storage::schema_version`, `db::StorageError`, `db::DB_FILE_NAME` (contract above).
- Consumes: nothing.

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Modify: `src-tauri/Cargo.toml` (add `[dev-dependencies] tempfile = "3"`)

**Steps:**

1. Write the tests first in `src-tauri/src/db/mod.rs` under `#[cfg(test)] mod tests`, each using `tempfile::tempdir()`:
   - `open_creates_missing_folder_and_applies_migrations`: open a nested non-existent folder; assert the file exists and `schema_version() == 1`.
   - `reopen_does_not_reapply_migrations`: open, drop, open again; assert `Ok` and version 1. Migration 1 creates `app_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)`, so re-running it would fail and make this test meaningful.
   - `schema_too_new_is_refused_without_writing`: create the DB file with `PRAGMA user_version = 99` via a raw connection, record the file's bytes, call `open_in`; assert `Err(SchemaTooNew)` and the bytes are unchanged.
   - `unwritable_folder_is_storage_unavailable`: create a regular file at `tmp/blocked`, call `open_in(&tmp/blocked)`; assert `Err(Unavailable)`.
   - `corrupt_file_is_storage_unavailable_and_untouched`: write `b"not a database"` to `DB_FILE_NAME`; assert `Err(Unavailable)` and the bytes are unchanged.
2. Run `cargo test db::` from `src-tauri/`; expect compile failures or FAIL.
3. Implement `migrations.rs`:

```rust
use rusqlite_migration::{Migrations, M};

pub const MIGRATIONS_COUNT: usize = 1;

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        "CREATE TABLE app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )])
}
```

4. Implement `db/mod.rs` `open_in` in this order, with no write before step (c):
   a. `std::fs::create_dir_all(folder)` → on error `Unavailable`.
   b. `rusqlite::Connection::open(folder.join(DB_FILE_NAME))` → on error `Unavailable`.
   c. `conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))` → on error `Unavailable` (this is where a corrupt file fails).
   d. If `version > MIGRATIONS_COUNT as i64` → `Err(SchemaTooNew)`.
   e. `migrations::migrations().to_latest(&mut conn)` → on error `Unavailable`.
   f. Read `user_version` again and return `Storage { conn, schema_version }`.
   Do not delete, rename, or overwrite any file on failure.
5. Run `cargo test db::`; expect all five PASS.

**Acceptance:** AC-20, AC-21, AC-22, AC-24, AC-25 at the Rust level; constitution P2 (migrations only through `rusqlite_migration`), P4 (migration, reopen, wrong-version, and corrupt-file tests).

---

### Task 2: Storage command, single-instance guard, and app wiring

**Specialist:** backend-developer
**Interfaces:**
- Consumes: `db::open_in`, `db::StorageError`, `db::Storage` (task 1).
- Produces: `commands::storage::storage_status`, `StorageStatus`, `StorageErrorCode`, `StorageState`, `AppInfoArgs`.

**Files:**
- Create: `src-tauri/src/commands/storage.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` (add `tauri-plugin-single-instance = "2"`), `src-tauri/tauri.conf.json`

**Steps:**

1. Tests first in `commands/storage.rs`:
   - `storage_status_maps_ready_state`: build `StorageState` with a `Ok(Storage)` opened in a tempdir; call the pure helper `status_from_state(&state)`; assert `ok == true`, `error_code == None`, `folder` equals the tempdir path string.
   - `storage_status_maps_unavailable`: `Err(Unavailable)` → `ok == false`, `error_code == Some(StorageUnavailable)`.
   - `storage_status_maps_schema_too_new`: `Err(SchemaTooNew)` → `Some(SchemaTooNew)`.
   - `storage_error_code_serialises_snake_case`: `serde_json::to_string` yields `"\"storage_unavailable\""` and `"\"schema_too_new\""`. Add `serde_json` as a dev-dependency if not already present.
   Keep these tests to the mapping and the serialisation contract. Do not test Tauri wiring.
2. Run `cargo test commands::`; expect FAIL.
3. Implement:
   - `status_from_state(state: &StorageState) -> StorageStatus` (pure; the command calls it).
   - `#[tauri::command] pub fn storage_status(args: StorageStatusArgs, state: State<'_, StorageState>) -> StorageStatus { let _ = args; status_from_state(&state) }`. Use `let _ = args;` only if clippy requires it; otherwise name the parameter `_args`, which keeps the one-argument-struct contract.
   - In `commands/mod.rs`: `pub mod storage;` and `#[derive(Debug, Deserialize)] pub struct AppInfoArgs {}`; `app_info(args: AppInfoArgs) -> AppInfo` (same body). Update the existing test in `commands/mod.rs` if its call changes.
4. `lib.rs` `run()`:
   - Register `tauri_plugin_single_instance::init(|app, _args, _cwd| { if let Some(w) = app.get_webview_window("main") { let _ = w.unminimize(); let _ = w.show(); let _ = w.set_focus(); } })` **first**, before any setup, so a second process exits before it opens the database.
   - `.setup(|app| { let folder = app.path().app_data_dir().unwrap_or_default(); let result = db::open_in(&folder); app.manage(StorageState { folder, result }); Ok(()) })`. Opening happens once, here.
   - `invoke_handler` lists `commands::app_info` and `commands::storage::storage_status`.
5. `tauri.conf.json`: window title `DevShelf`; `minWidth` 960, `minHeight` 640. Confirm the main window label is `main`; if it is not set, add `"label": "main"`.
6. Run `cargo test` and `cargo build` from `src-tauri/`; expect PASS.

**Acceptance:** AC-01 (title), AC-20 (folder via `app_data_dir`), AC-30 (single instance), AC-50, AC-51. Constitution P3 (argument struct, error code enum, no raw strings).

---

### Task 3: Tailwind v4 setup and dark theme tokens

**Specialist:** task (general-purpose; configuration and visual tokens)
**Interfaces:**
- Produces: Tailwind utilities and CSS custom properties used by Tasks 5 and 6.
- Consumes: nothing.

**Files:**
- Modify: `package.json` (add `tailwindcss`, `@tailwindcss/vite` as dev-dependencies or dependencies, per Tailwind v4 convention)
- Modify: `vite.config.ts` (add `tailwindcss()` plugin; add `test: { include: ["tests/**/*.test.ts"] }` if absent)
- Modify: `src/styles.css`

**Steps:**

1. Install with `npm install -D tailwindcss @tailwindcss/vite`.
2. Read `docs/DESIGN.md` (colour, typography, spacing, radius, and elevation tokens) and the CSS variables at the top of `docs/preview/index.html`. Copy only the tokens the F01 screens use into `src/styles.css` as `@theme` variables: canvas background, surface, hairline border, foreground, muted text, accent, and the warning colour for the storage error. Do not copy the product-card colours or any other token F01 does not use.
3. `src/styles.css`: `@import "tailwindcss";`, then the `@theme` block, then `html, body, #root { height: 100%; }` and a body background set to the canvas token. Font family: the system stack with `Inter` first (`Inter, "Segoe UI", system-ui, sans-serif`). Do not add a font package; the architecture names Inter as a stand-in and a font dependency would be an unapproved runtime dependency.
4. Ensure `npm run build` still type-checks and bundles. Do not run lint or the full test suite mid-flight.

**Acceptance:** Dark theme only (AC-visual). No runtime dependency added beyond the Tailwind packages.

---

### Task 4: Typed contracts, storage messages, and nav logic (Vitest)

**Specialist:** frontend-developer
**Interfaces:**
- Produces: TS types (contract), `appInfo`, `storageStatus`, `NAV_ITEMS`, `isNavActive`, `normalizePath`, `storageErrorMessage`.
- Consumes: the Rust contract above (types only; no runtime dependency on Tasks 1–2).

**Files:**
- Modify: `src/types/commands.ts`, `src/lib/commands.ts`
- Create: `src/lib/nav.ts`, `src/lib/storage-messages.ts`, `tests/nav.test.ts`, `tests/storage-messages.test.ts`

**Steps:**

1. `src/types/commands.ts`: add the types exactly as in the Contracts section. Keep the existing `AppInfo`, `ErrorCode`, `CommandError`.
2. `src/lib/commands.ts`: `appInfo()` calls `invoke<AppInfo>("app_info", { args: {} })`; add `storageStatus()` calling `invoke<StorageStatus>("storage_status", { args: {} })`.
3. Tests first:
   - `tests/nav.test.ts`:
     - `NAV_ITEMS` labels and paths in order: Home `/home`, Projects `/projects`, Workspaces `/workspaces`, Config Catalog `/config`, Vault `/vault`, Notes `/notes`, Settings `/settings` (AC-02).
     - `normalizePath("/home/") === "/home"`; `normalizePath("") === "/"`.
     - `isNavActive("/projects", "/projects")` true; `isNavActive("/projects/", "/projects")` true; `isNavActive("/projects", "/home")` false; `isNavActive("/projectsX", "/projects")` false (no prefix matching).
   - `tests/storage-messages.test.ts`:
     - `storageErrorMessage("schema_too_new", folder)` equals exactly the AC-24 sentence.
     - `storageErrorMessage("storage_unavailable", "C:\\Users\\x\\AppData\\Roaming\\devshelf")` equals the AC-25 sentence with the folder inserted. The folder appears in the message; no other text varies.
4. Run `npx vitest run tests/nav.test.ts tests/storage-messages.test.ts`; expect FAIL.
5. Implement `nav.ts` (`NAV_ITEMS` as `as const` array of `{ path, label }`, `normalizePath` strips one trailing slash and maps `""` to `/`, `isNavActive` compares normalised paths for equality) and `storage-messages.ts` (`storageErrorMessage(code, folder)` with a switch on `StorageErrorCode`, returning the exact spec copy).
6. Run the two test files; expect PASS.

**Acceptance:** AC-02, AC-03 (matching logic), AC-24, AC-25 copy. Constitution P3 (types in one file), P4 (branching logic under Vitest).

---

### Task 5: App gate, router, shell, sidebar, not-found and storage-error screens

**Specialist:** frontend-developer
**Solution-space note:** the route table and gate are given; the visual treatment is open to the DESIGN contract.
**Interfaces:**
- Consumes: `storageStatus` and `StorageStatus` (task 4), `NAV_ITEMS`, `isNavActive`, `normalizePath` (task 4), `storageErrorMessage` (task 4), Tailwind tokens (task 3), `["storage-status"]` query key.
- Produces: `App` (default export not required; named export `App` as today), `router` (`createHashRouter`), `AppShell`, `Sidebar`, `NavLink`, `NotFoundScreen`, `StorageErrorScreen`.

**Frontend dispatch contract:** the screen has no per-screen design file, so `design_md_fallback`: pass `docs/DESIGN.md` and `docs/preview/index.html` as the visual reference. Implement the sidebar and shell to match the preview's grid (sidebar 248px, content fluid, dark canvas).

**Files:**
- Modify: `src/app.tsx`
- Create: `src/router.tsx`, `src/components/app-shell.tsx`, `src/components/sidebar.tsx`, `src/components/nav-link.tsx`, `src/screens/not-found-screen.tsx`, `src/screens/storage-error-screen.tsx`

**Steps:**

1. `src/app.tsx`:
   - Create a `QueryClient` once (module scope) and wrap in `QueryClientProvider`.
   - `const status = useQuery({ queryKey: ["storage-status"], queryFn: storageStatus })`.
   - While loading: render a centred "Starting DevShelf…" text (no sidebar).
   - If `status.data.ok === false`: render `<StorageErrorScreen code={status.data.error_code} folder={status.data.folder} />` (AC-24, AC-25). The shell is not rendered.
   - If `status.isError` (command itself failed): render the storage error screen with `storage_unavailable` and the folder from the query's fallback text "unknown location". Do not show a raw error string.
   - Otherwise: `<RouterProvider router={router} />`.
2. `src/router.tsx`: `createHashRouter` with a root `AppShell` route:
   - index → `<Navigate to="/home" replace />`
   - children `home`, `projects`, `workspaces`, `config`, `vault`, `notes`, `settings` (screen components are imported from `src/screens/*`; Task 6 creates them, so in this task use a placeholder import list is NOT allowed. Instead, import the screen modules by the file names listed in Task 6 and create the files in Task 6; run Task 5 and Task 6 in the same worktree and build together in Task 7.)
   - `*` → `NotFoundScreen`.
   - Hash routing (AC-04: no reload; Back/Forward work).
3. `AppShell`: grid of `Sidebar` plus `<main>` with `<Outlet />`. Keep the sidebar visible on every route, including NotFound.
4. `Sidebar`: renders `NAV_ITEMS` through `NavLink`. Header brand text "DevShelf". No search button, no demo panel (non-goals).
5. `NavLink`: wraps React Router `Link`. Sets `aria-current="page"` and a visible highlight when `isNavActive(location.pathname, path)`. Pass `replace` when already active so re-selecting the current item adds no history entry (edge case in spec).
6. `NotFoundScreen`: heading "Page not found", message, and a link to `/home`.
7. `StorageErrorScreen({ code, folder })`: full window, no sidebar; heading "DevShelf can't start"; body = `storageErrorMessage(code, folder)`. No retry action (not in spec).
8. No new tests in this task. Behaviour is covered by Task 7's smoke run. The branching logic it uses is in Task 4 tests.

**Acceptance:** AC-01, AC-02, AC-03, AC-04, AC-05, AC-06 (sidebar keyboard reachable via real `<a>` elements), AC-25 UI, AC-24 UI, AC-50 (UI reads status only through the command).

---

### Task 6: Empty-state screens and Settings

**Specialist:** frontend-developer
**Solution-space note:** copy is fixed by the preview; layout follows the DESIGN contract.
**Interfaces:**
- Consumes: `storageStatus` via the `["storage-status"]` query key (task 4 / 5), `EmptyState` (this task).
- Produces: one default-exported screen component per route file.

**Frontend dispatch contract:** `design_md_fallback` as in Task 5.

**Files:**
- Create: `src/components/empty-state.tsx`, `src/screens/home-screen.tsx`, `src/screens/projects-screen.tsx`, `src/screens/workspaces-screen.tsx`, `src/screens/config-screen.tsx`, `src/screens/vault-screen.tsx`, `src/screens/notes-screen.tsx`, `src/screens/settings-screen.tsx`

**Steps:**

1. `EmptyState({ heading, message })`: heading and paragraph; no button (D1 = text only).
2. Screens, with copy taken from `docs/preview/index.html` empty states (read the file to confirm wording; the strings below are the expected text):
   - `home-screen.tsx`: `<h1>Home</h1>`, lead "Pick up where you left off.", `EmptyState` heading "Nothing here yet", message "Register a project and save a workspace to see pinned projects and recent workspaces on this page." Button removed (D1); the "Register your first project" button from the preview is NOT rendered.
   - `projects-screen.tsx`: `<h1>Projects</h1>`; `EmptyState` "No projects registered yet".
   - `workspaces-screen.tsx`: `<h1>Workspaces</h1>`; `EmptyState` "No workspaces yet", message "A workspace saves an ordered set of files to reopen together."
   - `config-screen.tsx`: `<h1>Config Catalog</h1>`; `EmptyState` "No config files registered".
   - `vault-screen.tsx`: `<h1>Vault</h1>`; `EmptyState` "No secrets yet". No lock, unlock, or password UI (F08).
   - `notes-screen.tsx`: `<h1>Notes</h1>`; `EmptyState` "No notes yet".
   - `settings-screen.tsx`: `<h1>Settings</h1>`. Uses `useQuery(["storage-status"])`. Shows:
     - Storage row: "Storage: OK" and the folder path in monospace, when `data.ok` (the error state never renders here; see the spec clarification).
     - Privacy paragraph with the approved AC-41 text verbatim: "DevShelf stores all data on this computer. Nothing is sent over the network. Project names, labels, environment names, and file paths are stored unencrypted in the local database."
     - Nothing else. No editor profiles, no auto-lock.
3. Each screen has a single `<h1>` and no interactive controls. Keyboard focus lands on `<h1>` or `<main>` (no change needed beyond semantic markup).

**Acceptance:** AC-10, AC-11, AC-12, AC-26, AC-31 (OK state), AC-41.

---

### Task 7: Final verification and cleanup

**Specialist:** task (general-purpose)
**Interfaces:** all above.

**Steps:**

1. From the project root: `npm run build`, `npm run test`, `npm run lint`. Fix any failure in the files this feature touched. Run `npm run format` on touched files only if lint reports Prettier drift.
2. From `src-tauri/`: `cargo test` and `cargo build`.
3. Confirm `tests/placeholder.test.ts` still passes or is removed if it now duplicates real tests (it is a scaffold placeholder; deleting it is in scope only if it fails or is empty).
4. Smoke run for Step 5 is out of scope for this task; record that it is pending.

**Acceptance:** All checks above exit 0.

---

## Self-review (run before the gate)

- **Spec coverage:** AC-01 (T2, T5), AC-02/03/04/05/06 (T4, T5), AC-10/11/12 (T6), AC-20/21/22 (T1), AC-23 (no project writes; T1 uses only app data folder), AC-24/25 (T1, T4, T5), AC-26/31/41 (T6), AC-30 (T2), AC-40 (no network dep; T3 adds none), AC-50/51 (T2, T4, T5). Covered.
- **Placeholders:** none intended. Task 5 step 2 references screen files created in Task 6, so Tasks 5 and 6 are built together in Task 7; this is a sequencing note, not a placeholder.
- **Type consistency:** `StorageErrorCode` snake_case strings match the TS union; `StorageStatus` fields `ok`, `folder`, `error_code` match in Rust and TS; `StorageError` variants `Unavailable`/`SchemaTooNew` map to `StorageUnavailable`/`SchemaTooNew`.
- **Constitution:** P1 no secrets; P2 DB and OS only in Rust, bound parameters, migrations numbered; P3 argument/return structs, error codes, one TS types file; P4 Rust tests cover migration/reopen/wrong-version/corrupt; Vitest covers branching; P5 no network, no updater.

## Open items for the reviewer

- Spec clarification on AC-31 vs S9 (above).
- `commands::app_info` gains an argument struct to meet the convention; this touches an existing command.
- Font: system stack with Inter first; no font package (stack-list amendment avoided).
- shadcn/ui is not initialised in F01; the first feature that needs a shadcn component adds it. The stack list is unchanged.
