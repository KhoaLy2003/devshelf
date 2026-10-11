# F01 — Foundation and navigation: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F01 (Must, size M, depends on none)
- **Feature slug**: `foundation-and-navigation`
- **Spec mode**: Full. No designs exist in `docs/designs/foundation-and-navigation/` and Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` (dark theme only) and `docs/preview/index.html` (`design_md_fallback`).

---

## 1. Goal

A user opens DevShelf and gets a main window with a sidebar to seven areas. Every area renders, even with no data. Local data lives in one SQLite database in the user's application data folder and survives restarts. The app makes no network calls.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| S1 | Main window shell | all routes | Sidebar + content area |
| S2 | Home | `/home` | Empty state only in F01 |
| S3 | Projects | `/projects` | Empty state only |
| S4 | Workspaces | `/workspaces` | Empty state only |
| S5 | Config Catalog | `/config` | Empty state only |
| S6 | Vault | `/vault` | Empty state only; no lock, unlock, or password flow (F08) |
| S7 | Notes | `/notes` | Empty state only |
| S8 | Settings | `/settings` | Privacy text and app storage status only; no editor profiles or auto-lock (F03, F08) |
| S9 | Storage error | shown in place of S1 | Full-window message when the database cannot be opened |
| S10 | Page not found | any unknown route | Message with link to Home |

## 3. Acceptance Criteria

### Shell and navigation

- **AC-01** The main window title is "DevShelf". On launch the app opens to `/home`.
- **AC-02** The sidebar lists, in this order: Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings. Each item navigates to its route.
- **AC-03** The active sidebar item is visibly highlighted and matches the current route on every screen.
- **AC-04** Navigating between areas does not reload the window. Browser-style Back and Forward move between previously visited areas.
- **AC-05** Opening an unknown route shows S10 with a link to Home. The sidebar remains visible.
- **AC-06** Keyboard: every sidebar item is reachable with Tab and activatable with Enter. The focus ring is visible.

### Empty states

- **AC-10** Each of S2–S7 shows a heading for the area and an empty-state message. Copy matches `docs/preview/index.html` for that area's empty state.
- **AC-11** Empty-state copy is text only in F01. No create, register, or unlock action appears until the feature that provides it is built (see Decision D1).
- **AC-12** S8 shows the privacy text in AC-41 and the storage status in AC-31.

### Local storage

- **AC-20** On first launch the app creates one SQLite database file in the user's application data folder (the folder Tauri reports as the app data directory for the `devshelf` identifier). The folder is created if missing.
- **AC-21** The schema is created only through numbered `rusqlite_migration` migrations. F01 ships migration 1, which creates a schema-version record and nothing else user-visible.
- **AC-22** After a restart, the app opens the same database and does not re-run applied migrations. The stored schema version is unchanged.
- **AC-23** No file is written into any project folder or into the application's source/install folder.
- **AC-24** If the database schema is newer than this build understands, the app shows S9 with the code `schema_too_new` and the message "This data was created by a newer version of DevShelf. Update DevShelf to open it." It does not modify the file.
- **AC-25** If the data folder cannot be created or the database cannot be opened, the app shows S9 with the code `storage_unavailable` and the message "DevShelf cannot write to <folder>. Check that the folder exists and that your Windows account can write to it." The folder path is shown; no raw error text is shown.
- **AC-26** The data folder path is shown in S8 (AC-31) so the user can find the file.

### Single instance

- **AC-30** Starting DevShelf while it is already running does not open a second window or a second database connection. The second launch focuses the existing window and exits.

### Storage status (Settings)

- **AC-31** S8 shows a storage status row: "Storage: OK" with the data folder path, or the same error code and message as AC-25/AC-24 when storage failed.

### Privacy text

- **AC-40** The app makes no network calls. It works with the network disabled.
- **AC-41** S8 shows this text: "DevShelf stores all data on this computer. Nothing is sent over the network. Project names, labels, environment names, and file paths are stored unencrypted in the local database." (Final wording may be adjusted at approval; the statement that metadata is unencrypted must stay.)

### Typed boundary

- **AC-50** The UI reads storage status only through a typed Tauri command. The webview does not touch the database or the file system.
- **AC-51** The command returns a user-facing error code and never a raw error string.

## 4. Edge Cases

- Data folder exists but is read-only: AC-25.
- Database file is present but corrupted or not a SQLite file: treated as a failed open; AC-25 with the same code. The file is not deleted or overwritten.
- Two launches start at the same moment: exactly one instance runs (AC-30).
- Window resized to a narrow width: the sidebar stays visible; content scrolls. Minimum supported width is set in the plan.
- Unknown route with a trailing slash or query string: resolves to the matching area if one exists, otherwise S10.
- Navigating to the current route again does not add a history entry.

## 5. Non-goals (F01)

- Quick open, the sidebar search button, and Ctrl+K (F11).
- The "Demo states" panel from the prototype. It is a prototype tool and is not part of the app.
- Any create, edit, or delete action in any area (F02, F04, F06, F07, F09).
- Editor profiles and default editor (F03). Auto-lock settings and the master password (F08).
- Home content: pinned projects, recent workspaces, quick actions (F10).
- Restoring the last-visited screen on launch (see D2).
- Window size and position memory.
- Light theme or theme switching. The dark theme is the only theme.
- Backup, export, or update checks of any kind.
- Encryption of any data. F01 stores no secrets and no notes.

## 6. Decisions Needing Approval

Recommended default in bold. The spec above uses the recommended default in each case.

- **D1 — Empty-state actions.** (a) **Text only; no action buttons until the feature exists.** (b) Disabled buttons labelled "Coming soon". (c) Buttons that open an informational message. *Recommended: (a). Disabled or placeholder buttons read as broken and invite testing of unbuilt flows.*
- **D2 — Launch screen.** (a) **Always Home.** (b) Last visited area. *Recommended: (a). "Remembers across restarts" applies to data, and Home is the Monday entry point in the journey map. Last-visited adds state the roadmap does not ask for.*
- **D3 — Single-instance guard in F01.** (a) **Include now.** (b) Defer to F02. *Recommended: (a). Architecture names it as the guard against two processes opening the same SQLite file; F02 is the first feature that writes data.*
- **D4 — Privacy text in F01.** (a) **Include in Settings now (AC-41).** (b) Defer to F12. *Recommended: (a). Architecture section 7 requires the plaintext-metadata trade-off to appear in the app's privacy text, and no later feature is guaranteed to be built before real data is stored.*

## 7. Constitution Check

- **P1 Secrets stay in the Rust core**: no secrets in F01. No logging of paths beyond the storage-status message. Pass.
- **P2 Only Rust touches the DB and OS**: all DB open and migration code lives in `src-tauri/src/db/`; the UI calls one command. Schema change only via `rusqlite_migration`. Pass.
- **P3 Typed boundary**: one Rust argument struct and one return type per command; error codes `schema_too_new` and `storage_unavailable` are added to `src/types/commands.ts`. Pass.
- **P4 Testing**: cargo tests for migration apply, reopen without re-migrating, schema-too-new, and storage-unavailable, all in temp directories. Vitest for route and sidebar behaviour where branching exists. Manual end-to-end smoke on the built app before close. Pass (planned).
- **P5 Local-first and offline**: no network, no telemetry, no updater. Pass.
- **Additional constraints**: stack unchanged. Dark theme only. English copy. Pass.

## 8. Open Questions

None beyond Section 6.
