# F10 Home overview: E2E results (Phase 2 Step 5)

- Spec: `docs/specs/home-overview/spec.md` (AC-xx).
- Run: `npm run tauri dev` from the project root (real app, Windows, dev build). Cargo printed "Finished `dev` profile ... in 0.49s", so the existing `target\debug\devshelf.exe` was run without a rebuild; the F10 migration was applied by it on launch (H01).
- UI driving: WebView2 remote debugging port 9222 (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`, set only in the dev-server shell). Chrome DevTools Protocol `Input.dispatchMouseEvent` (real mouse move/press/release at element centers; every navigation was a click on a sidebar link or a list row), `Input.insertText`, `Input.dispatchKeyEvent` (ArrowDown on the native Project select, Ctrl+A). Reload used CDP `Page.reload`. DOM reads used `Runtime.evaluate`. IPC calls were counted from `Network.requestWillBeSent` POSTs to `http://ipc.localhost/<command>`. No native file dialogs were used: paths were typed into the path fields.
- DB reads: read-only (`bun:sqlite` with `readonly: true`).
- Backup of the pre-F10 file (user_version 5): `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3.f05-backup`, SHA-256 `f978a7e81fcdc800080cd2dd7add385f9999d2183ab3c1deac0805805c43edeb` (identical to the live file's hash at that moment).
- Pre-existing DB data (from earlier features, not created by this run): projects 1 `App A`, 2 `App A Sub`, 3 `app b` (pinned), 4 `App C`; 0 workspaces. Pins were not part of the deliverable data; the run unpinned `app b` (H08) and re-pinned it at cleanup.
- Test files (all removed afterwards): `%TEMP%\devshelf-e2e-f10\ops\runbook.md`, `ops\logs\a.log`, `ops\other.md` (relink target), `gone\x.txt` (added to Ops, then `gone\` deleted before the first restore), and a helper `close.ps1`.
- Workspaces created in the UI: `Ops` (item 1 `ops\runbook.md`, project App C, Windows default; item 2 `ops\logs`; item 3 `gone\x.txt` added in H13), `Scratch` (`ops\logs`, no project), a 100-character name (`Z` + `0123456789` x 9 + `abcdefghi`) with no items, and `Alpha` with no items (extra, created only to check the never-restored sort by name).
- Windows opened by Restore: the Windows default handler for `.md` on this machine is Visual Studio Code, so Restore of Ops launched VS Code (process start 19:37:48, after my first restore; no VS Code process existed before the run) and an Explorer window `logs`. Closed with `WM_CLOSE`: VS Code windows (hwnd 133484 "other.md - Visual Studio Code", 67974, 198862, 329992 "runbook.md - Visual Studio Code") and Explorer window `logs` (hwnd 1575212). Afterwards no VS Code process was running. Pre-existing Explorer windows ("feedback", "observability-smoke") were not touched. No editor other than the Windows default was started.
- "Reload" means a WebView reload plus a DB read, not an app restart.
- No source files were modified. Dev-mode only.

| ID | Steps | Expected | Actual | Result |
|---|---|---|---|---|
| H01 (AC-40) | Launch app on DB at user_version 5; read DB read-only | user_version 6; `workspaces.last_restore_missing` exists, default 0 | `pragma user_version` = 6. `workspaces` columns: id, name, name_key, created_at, last_restored_at (TEXT, nullable), `last_restore_missing INTEGER NOT NULL DEFAULT 0` | PASS |
| H02 (AC-03/04) | Open Home | Heading "Home"; lead "Pick up where you left off."; no greeting; no date line | `<h1>` "Home"; paragraph "Pick up where you left off."; no match for "good morning/afternoon/evening"; no weekday, month or year text in the page | PASS |
| H03 (AC-10/11/12) | Read header actions | "Register project" only; no Quick open; no other buttons | `document.querySelectorAll('button')` (whole page) = ["Register project"]; no "quick open" text | PASS |
| H04 (AC-10) | Click Register project | Navigates to /projects; Register dialog opens once | URL `#/projects`; exactly one open `<dialog>` titled "Register project" (fields: Root folder path, Display name, Type, Tags) | PASS |
| H05 (AC-10, D2) | Click Cancel; `Page.reload` on Projects | Dialog does not reopen | After Cancel 0 open dialogs. After reload on `#/projects`: 0 open dialogs; `history.state.usr` = null | PASS |
| H06 (AC-42) | Check whether DB has projects/workspaces to empty-state test | Empty state only if both empty; else BLOCKED | DB had 4 projects, 0 workspaces. The "no projects and no workspaces" state cannot be reached without deleting projects, which this E2E must not do | BLOCKED: DB has 4 registered projects; not deleted |
| H06b (AC-42, AC-22) | With 4 registered projects, 0 pinned, 0 workspaces (after unpinning `app b`), open Home | Spec: empty state only when no projects and no workspaces; with projects present, Home should show the two sections with their own empty texts (AC-02/AC-22/AC-34) | Home showed "Nothing here yet" / "Register a project and save a workspace to see pinned projects and recent workspaces on this page." / "Register your first project" (that button navigated to `#/projects` and opened one dialog). Neither section was shown. Same screen seen again at the end of cleanup (4 projects, 0 pinned, 0 workspaces). Source read for cause (not edited): `home-screen.tsx` `isEmpty` is `pinned.length === 0 && recent.length === 0`, not "no projects and no workspaces" | FAIL |
| H07 (AC-20/21) | Home with `app b` pinned; read the row | Display name, monospace root path, drive; links to detail | Row: "app b", `C:\Users\Admin\AppData\Local\Temp\devshelf-e2e\app-b` (computed font-family `ui-monospace, SFMono-Regular, Menlo...`), "C:" (font Inter). Link `href="#/projects/3"`; click opened `#/projects/3` | PASS |
| H08a (AC-22) | Unpin `app b` from its detail page (Unpin button; DB pinned 0 for all four); open Home with 0 workspaces | "No pinned projects. Pin a project from its detail page to see it here." | Showed the whole-page empty state of H06b instead; the AC-22 text did not appear | FAIL |
| H08b (AC-22) | Same, after Ops and Scratch existed (H12) | Text shown in the Pinned projects section | "PINNED PROJECTS" then "No pinned projects. Pin a project from its detail page to see it here." with the Recent workspaces section next to it | PASS |
| H09 (AC-20/23) | From Projects open `App A` (id 1), click Pin, click Home in the sidebar (no restart); later click Unpin on it and Home | Pinned list updates on the next visit | After Pin: Home listed "App A", root path, "C:" (DB pinned 1). After Unpin: Home showed "No pinned projects..." (DB pinned 0). Each visit issued one `home_overview` | PASS |
| H10 (AC-23) | More than 6 pinned | "All projects" link; otherwise BLOCKED | Only 4 projects exist, so at most 4 can be pinned. Link not seen | BLOCKED: fewer than 7 projects registered; none created (not asked) |
| H10b (AC-35) | More than 6 recent workspaces | "All workspaces" link; otherwise BLOCKED | At most 4 workspaces existed at once. Link not seen | BLOCKED: only 4 workspaces created; AC-35 cap not exercised |
| H11 (AC-34) | Home with `app b` pinned and 0 workspaces | "No recent workspaces. A workspace saves files and folders you reopen together." | Text shown under "RECENT WORKSPACES" (seen at first load and at the end of cleanup) | PASS |
| H12 (AC-30/31/35) | Create `Ops` (runbook.md with project App C, logs), create `Scratch` (logs, no project); open Home | "2 items" / "1 item"; project names or "No registered projects"; "Never restored" both | Ops: "2 items", "App C", "Never restored". Scratch: "1 item", "No registered projects", "Never restored". Order Ops, Scratch (never restored, by name). One `home_overview` call for the visit | PASS |
| H13 (AC-32/40) | Add `gone\x.txt` as Ops item 3, delete `gone\`, Restore Ops, open Home | "Last restored <date time>"; badge "1 item missing at last restore" | Restore panel "2 opened, 1 missing, 0 unreachable, 0 could not open." (x.txt "Path not found."). DB Ops `last_restored_at` 2026-10-12T12:37:48Z, `last_restore_missing` 1. Home: Ops "3 items", "App C", "Last restored 12 Oct 2026, 19:37" (local UTC+7), badge "1 item missing at last restore". The date format is day-month-year 24-hour (`en-GB` style) | PASS |
| H14 (AC-32/40) | Restore Scratch (all available); open Home | Last restored shown; no badge | Panel "All 1 items opened." DB Scratch `last_restored_at` 12:38:00Z, `last_restore_missing` 0. Home: Scratch "Last restored 12 Oct 2026, 19:38", no badge; order Scratch (most recent), Ops | PASS |
| H15 (AC-41) | Relink the missing item to `ops\other.md`; do not restore again; open Home | Badge still "1 item missing at last restore"; DB count unchanged | The Restore panel (the only place Relink exists) is gone after leaving the workspace, so I restored Ops once more first (still 1 missing; DB `last_restored_at` 12:38:18Z, count 1), then Relink (typed path, Apply): "Relinked. It will open on the next restore." No further restore. DB still count 1, `last_restored_at` 12:38:18Z. Home: Ops still shows "1 item missing at last restore" | PASS |
| H16 (AC-40/32) | Restore Ops again; open Home | Badge disappears | Panel header no longer listed a missing group. DB Ops `last_restored_at` 12:38:38Z, `last_restore_missing` 0. Home: Ops "Last restored 12 Oct 2026, 19:38", no badge | PASS |
| H17 (AC-30) | Record Home order after H15/H16, then with never-restored workspaces | Restored first (most recent restore first), then never-restored by name | After H15: Ops (12:38:18Z), Scratch (12:38:00Z). After H16 and creating Alpha and the 100-char one (both never restored): Ops (12:38:38Z), Scratch (12:38:00Z), Alpha, `Z0123...abcdefghi` | PASS |
| H18 (AC-21/33) | Click a workspace row and a pinned project row on Home | `/workspaces/:id`; `/projects/:id` | Rows `Ops` -> `#/workspaces/1`, `Scratch` -> `#/workspaces/2`, long-name row -> `#/workspaces/3`; pinned `app b` -> `#/projects/3`, `App A` -> `#/projects/1` | PASS |
| H19 (AC-50/51) | Visit Home from Projects, Workspaces, Settings, Projects (sidebar clicks) and by reload; count IPC POSTs | One `home_overview` per visit | Each of the 4 sidebar visits: exactly one `home_overview` and nothing else. Reload: `storage_status` and one `home_overview`. No SQL or file-system calls observable from the page | PASS |
| H20 (AC-52) | Reload Home; inspect browser storage | Same data from DB; no new persisted UI state | `innerText` of Home identical before and after `Page.reload`. `localStorage` and `sessionStorage` have 0 keys | PASS |
| H21 (edge) | Create a 100-character workspace name; open Home | Single-line truncation; full name in title attribute | Name length 100 (DB `length(name)` 100). Name span: `white-space: nowrap`, `overflow: hidden`, `text-overflow: ellipsis`, scrollWidth 860 > clientWidth 283, one line, `title` equals the full 100-character name | PASS |
| H22 | Storage failure on Home | Not testable without changing the DB | Not run | BLOCKED: needs a broken or locked DB, which this E2E must not create |

PASS 19, FAIL 2 (H06b, H08a), BLOCKED 4 (H06, H10, H10b, H22).

## Failures

- H06b / H08a (AC-42, AC-22): Home treats "no pinned projects and no recent workspaces" as the empty state. With projects registered but none pinned and no workspaces, the user gets "Nothing here yet" and never sees the AC-22 text, and the AC-42 trigger ("no projects and no workspaces") is wider in practice than the spec says. To fix, either the home command needs to return whether any project exists, or the spec's AC-42 condition needs to change to "nothing pinned and no workspaces". Reported only; no source touched.

## Observations (not failures)

- Restore results (and the Relink control) live only in the workspace detail component state. Leaving the page loses them, so H15 needed an extra restore before Relink (stored count stayed 1 as it should).
- The date on Home reads "12 Oct 2026, 19:37"; the workspace list from F05 reads "10/12/2026, 7:06:37 PM". Different formats on the two screens.
- The Windows default handler for `.md` here is VS Code; each Restore of Ops launched or reused a VS Code window.
- Real app restart across a relaunch: NOT RUN (H20 used a WebView reload).
- Run used the dirty working tree (F10 files uncommitted).

## Cleanup and final state

- Ops, Scratch, the 100-character workspace and Alpha were deleted through the UI (Delete workspace). Final DB: user_version 6, `workspaces` 0 rows, `workspace_items` 0 rows.
- `App A` was pinned by me and unpinned. `app b` was pinned before the run, unpinned in H08 and re-pinned through the UI at cleanup; final pins: `app b` only (same as before).
- `%TEMP%\devshelf-e2e-f10` removed. Dev server wrapper stopped; the leftover Tauri/vite/cargo/devshelf tree (all started 19:35:16-19:35:22, at my launch) was killed with `taskkill /T`. Ports 1420 and 9222 no longer listen; no devshelf process remains.
- Backup `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3.f05-backup` kept, SHA-256 `f978a7e81fcdc800080cd2dd7add385f9999d2183ab3c1deac0805805c43edeb`. The live file's hash after the run is `2adc698577294372ba2258fc3f3a0aa5288f3ffc40ae6f29f2b4daae9b1e2cb3` (migrated to version 6).

## Re-check after empty-state fix

- Method: `npm run tauri dev` from the project root with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. The existing `target\debug\devshelf.exe` ran (Cargo "Finished" in 0.50s). UI driven over CDP: `Input.dispatchMouseEvent` real clicks on sidebar links, project rows and Pin/Unpin buttons; DOM text read with `Runtime.evaluate`. DB read-only via `bun:sqlite`. No source changed.
- Start state: user_version 6; projects App A, App A Sub, app b (pinned), App C; 0 workspaces.

| ID | Check | Evidence | Result |
|---|---|---|---|
| R1a | 4 projects, 0 workspaces, `app b` pinned: Home shows two sections | "PINNED PROJECTS" lists `app b`, `C:\Users\Admin\AppData\Local\Temp\devshelf-e2e\app-b`, `C:`; "RECENT WORKSPACES" shows "No recent workspaces. A workspace saves files and folders you reopen together." | PASS |
| R1b | Same with nothing pinned (Unpin `app b` on its detail page) | "PINNED PROJECTS" then "No pinned projects. Pin a project from its detail page to see it here."; "RECENT WORKSPACES" then the recent text above; no "Nothing here yet" | PASS |
| R2 | Header "Register project" present | In both states `document.querySelectorAll('button')` = ["Register project"] | PASS |
| R3 | Pin `App A`, Home lists it; Unpin, empty pinned text returns | After Pin: Home pinned section listed `App A`, `...\app-a`, `C:`. After Unpin: "No pinned projects. Pin a project from its detail page to see it here." | PASS |
| R4 | No "Register your first project" with projects present | Page text includes neither "Register your first project" nor "Nothing here yet" (checked with the unpinned, 0-workspace Home) | PASS |
| Cleanup | Stop processes; final DB | See below (cleanup, not a check) | n/a |

Result: R1-R4 PASS (4 of 4), FAIL 0. H06b / H08a are resolved. Liveness: R1-R4 ran against the live `devshelf.exe` I launched (CDP on 9222 answered; Pin/Unpin through the app's IPC changed the DB `pinned` values, confirmed by read-only DB reads). The supervisor's "exit code 1" for that launch was reported after I deliberately killed the tree at cleanup (`taskkill /IM devshelf.exe` and `/IM node.exe`, an image-name kill that could also have ended unrelated node processes). An earlier launch with wrong env-var syntax (no 9222) was killed and discarded.

Final state: `app b` re-pinned through the UI (as at start); App A, App A Sub, App C unpinned; user_version 6; 0 workspaces. Dev server, vite, devshelf.exe killed (`taskkill /T`); no devshelf process and no LISTEN on 1420 or 9222 (only TIME_WAIT entries).
