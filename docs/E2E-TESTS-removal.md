# DevShelf project unregister and vault delete: E2E report

Build under test: `src-tauri/target/release/devshelf.exe`. The UI was driven through WebView2 CDP on port 9222 using real `Input.dispatchMouseEvent` and key/text events (selects were changed with real ArrowDown key events). The DB was read read-only with `bun:sqlite`. No source files were changed. No `npm install` and no build were run.

Run date: 2026-10-17, about 17:36 to 17:46 local (10:36 to 10:46 UTC). Test folders were under `%TEMP%\devshelf-removal-e2e\`. The spec is `docs/specs/project-removal-and-vault-delete/spec.md`.

## Summary

| Result | Count |
|---|---|
| PASS | 21 (U1-U10, V1-V8, P1-P3) |
| FAIL | 0 |
| BLOCKED | 0 |
| Defects found outside the scripted checks | 2 (D1 medium, D2 high) |

All 21 scripted checks passed. Two defects were found while running them (stale workspace detail, and project id reuse that re-links orphaned notes and secrets). They are listed under "Defects" and are not hidden inside a PASS. U4, U6 and U7 carry caveats that refer to them or to harness limits.

## Harness events and limits

- Start state: `%APPDATA%\app.devshelf.desktop` existed and was EMPTY (listing `[]`, 0 entries). No `notes-data-key.DevShelf` entry existed in Credential Manager, and no `devshelf.exe` was running.
- First seed attempt (PID 23780): the vault and both projects were created, then I noticed the projects had no tags, so project_tags could not be tested. I closed the app with `WM_CLOSE`, moved that DB to `C:\Users\Admin\devshelf-test-archive\removal-e2e-20261017103752-seedretry\devshelf.sqlite3` (SHA-256 `3052e73db5177e2ac4df4b0f7df0f73d59fddf5373607598187d0d813f03d3e5`), and re-seeded from empty. Everything below is from the second run (PID 23684), then after a restart PID 12480.
- Launch: `powershell -Command "& devshelf.exe 2>&1 | Tee-Object -FilePath app.log -Append"` with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. `app.log` was never created: the app wrote nothing to stdout/stderr, so `Tee-Object` never created the file (the wrapper shell was confirmed running). The P1 app.log count is therefore "no log file exists", not "scanned file with 0 matches".
- `Get-FileHash` is not available in this PowerShell ("not recognized"). Hashes came from `[System.Security.Cryptography.SHA256]` in PowerShell. I cross-checked the six values against a Bun `crypto` SHA-256 before the run and they were identical.
- The app was closed by posting `WM_CLOSE` to the DevShelf window (PIDs I started: wrapper PowerShell 16420, app 23780; wrapper 13128, app 23684; wrapper 18832, app 12480). No process was killed by image name.
- Extra folder: I created `%TEMP%\devshelf-removal-e2e\gamma` and a project "Removal Gamma" for the stale-dialog check (U7) and light-mode check (P2). It is outside the listed seed, so the final DB has no Gamma left in it (see counts).
- Out-of-band calls: for U7 and P2 I called the app's own IPC command `projects_unregister` from the page (`window.__TAURI_INTERNALS__.invoke`) to act as a "second actor". That is not a UI action and is labelled where used.

## Seed (through the UI)

- Vault set up with `E2E-Release-Pass-2026` (unlocked automatically, `vault_meta` 1 row, auto_lock 5).
- Projects: Removal Alpha (id 1, tags `alpha-tag, shared-tag`) at `...\alpha`, Removal Beta (id 2, tags `beta-tag, shared-tag`) at `...\beta`. Files: `alpha\readme.txt`, `alpha\data.bin`, `alpha\.env.example`, `beta\readme.txt`, `beta\notes.md`, `beta\.env.example`.
- Config: id 1 Alpha `alpha\.env.example` (dev, stored kind `dotenv`), id 2 Beta `beta\.env.example` (dev, `dotenv`).
- Workspaces: W1 "Fully Alpha" (item: alpha folder, project Alpha), W2 "Mixed AB" (position 0: `alpha\readme.txt` Alpha, position 1: beta folder Beta), W3 "Beta only" (beta folder, Beta).
- Notes: id 1 "Alpha note" (Alpha, body `REMOVAL-NOTE-ALPHA`), id 2 "General note" (General, body `General body GENERAL-NOTE-BODY`).
- Secrets: id 1 "Alpha key" (API key, Alpha, dev, `REMOVAL-SECRET-ALPHA`), id 2 "Beta key" (API key, Beta, dev, `REMOVAL-SECRET-BETA`).
- Alpha was pinned before U1 (to exercise AC-A10 and the Home pinned list).
- DB counts after seeding: projects 2, project_tags 4, config_files 2, workspaces 3, workspace_items 4, notes 2, secrets 2, vault_meta 1. This matches the requested 2/2/3/2/2.
- Credential Manager: `notes-data-key.DevShelf` appeared after the first note save, so it was created in this run.

SHA-256 of every file before any removal (also identical after U3 and after U8; computed by PowerShell/.NET):

| File | SHA-256 |
|---|---|
| alpha\\.env.example | bc327797b0e9d1ff6fe6428aeb9d42375301b86ce165e2656a2363560a7fc5b9 |
| alpha\\data.bin | 0f5b611bdf67c1512a9742202dff412b81a778b65ea32b4bebb22d9dd3c65fdb |
| alpha\\readme.txt | 5226605d0a8d3a3144c95fdf62fe1f6921ff0b8b790f0bcf06330606561d26cf |
| beta\\.env.example | ded5d1492bf6066b051a376b315f507511e86305f7ab6945e351619a0c16ff09 |
| beta\\notes.md | 7bb1347a5ed27cba09c239e19bc971d2ca4848c7b0b427ac540fcd80ae2fb11c |
| beta\\readme.txt | ab1a4d83c5e32810175fc8d1743ac40351fc05ee3ee7c94e258569de7e636ae9 |

## DB counts at each step

| Step | projects | project_tags | config_files | workspaces | workspace_items | notes | secrets | vault_meta |
|---|---|---|---|---|---|---|---|---|
| After seed (and after U2 Cancel) | 2 | 4 | 2 | 3 | 4 | 2 | 2 | 1 |
| Before U3 | 2 | 4 | 2 | 3 | 4 | 2 | 2 | 1 |
| After U3 (Alpha unregistered) | 1 | 2 | 1 | 2 | 2 | 2 | 2 | 1 |
| Before U8 | 1 | 2 | 1 | 2 | 2 | 2 | 2 | 1 |
| After U8 (Beta unregistered) | 0 | 0 | 0 | 0 | 0 | 2 | 2 | 1 |
| Before V5 (vault delete) | 0 | 0 | 0 | 0 | 0 | 2 | 2 | 1 |
| After V5 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| After V8 (re-setup) | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 1 |

Note for V5: projects and workspaces were already 0 before the vault delete (they were removed in U3/U8), so "projects and workspaces unaffected" is a trivial 0 = 0 here. Notes stayed at 2, which is the meaningful comparison.

## Results

### U1 Unregister preview (AC-A02, AC-A09): PASS
Opened Removal Alpha detail (`#/projects/1`, buttons Pin and Unregister) and clicked Unregister. The dialog text, exactly as read from the DOM (`innerText`):

```
Unregister project

Unregister this project? Its files and folders are not changed or deleted.

This also removes 1 config entry and 1 workspace.

Cancel
Unregister
```

This matches the requested texts. W2 (mixed) was not counted.

### U2 Cancel keeps everything: PASS
After Cancel the dialog was gone, the URL stayed `#/projects/1`, the Projects list still showed `★Removal Alpha` and Removal Beta, and the DB counts were identical to the seed (2/4/2/3/4/2/2/1).

### U3 Unregister Alpha: PASS
Confirmed. The page went to `#/projects` and showed only Removal Beta.
- Home: PINNED PROJECTS showed "No pinned projects." (Alpha had been pinned). RECENT WORKSPACES showed "Beta only, 1 item, Removal Beta" and "Mixed AB, 1 item, Removal Beta". "Fully Alpha" was gone.
- Pickers: Add-config project select `Choose a project, Removal Beta`; Add-item project select `No project, Removal Beta`; Notes filter `All notes, General, Removal Beta`; Secrets project filter `All projects, No project, Removal Beta`; Config Catalog filter shows `All, Removal Beta`.
- Config Catalog listed only Beta's `.env.example`.

### U4 DB after unregister (AC-A03, AC-A05, AC-A06): PASS (DB), with defect D1 in the UI
After U3: projects 1 (id 2), project_tags 2 (`beta-tag`, `shared-tag`, both project 2; Alpha's 2 rows gone), config_files 1 (id 2, Beta), workspaces 2 (W2 and W3; W1 gone), workspace_items 2:
- id 3: workspace 2, position 0, folder, project 2. Before U3 this item was at position 1, so it was renumbered to 0 (contiguous 0..n-1). The Alpha item id 2 of W2 was removed.
- id 4: workspace 3, position 0, project 2, unchanged. W1's item id 1 was removed with W1.
- The Workspaces list screen showed "Mixed AB, Removal Beta, 1 item".
- The Workspace detail page for W2, opened afterwards, still showed two items (including the Alpha `readme.txt` row). That is D1. The DB was correct.

### U5 Disk unchanged (AC-A04): PASS
After U3, all six files still existed (alpha: `.env.example`, `data.bin`, `readme.txt`) and the PowerShell SHA-256 list was byte-for-byte identical to the "before" list above. It was identical again after U8.

### U6 Notes and secrets (AC-A07, AC-A08): PASS, with caveat (see D2)
- Notes list: "Alpha note, Project removed". Reopened the note: title `Alpha note`, textarea value `REMOVAL-NOTE-ALPHA`; its Project select showed `Project removed` (options: General, Project removed, Removal Beta). I did not edit or save it.
- Secrets list: "Alpha key, API key · Project removed · Dev". Reveal showed `REMOVAL-SECRET-ALPHA` (page text contained it); Hide returned `••••••••` and the page text no longer contained it.
- The DB kept `project_id = 1` on the note and the secret (dangling reference, no project row).
- Caveat: "Project removed" is only shown while no project has the old id. See D2.

### U7 Unknown project (AC-A11): PASS, with caveat on how the stale page was made
- Reopening Alpha's URL by history navigation (`Page.navigateToHistoryEntry` to `#/projects/1`) did NOT give a stale page. It showed "This project no longer exists." with "Back to Projects" (so the detail screen refetches). So the literal "stale page" from the brief is not reachable.
- Backend check (IPC, not UI): `projects_unregister {id:1}` returned `{"removed_configs":0,"removed_workspaces":0,"error":"project_not_found"}`; `projects_removal_preview {id:1}` returned `project_not_found`. DB counts unchanged.
- Stale dialog (UI): after U9 I registered "Removal Gamma" (id 1), opened its Unregister dialog (preview had no summary line because nothing else would be removed), then removed it out-of-band through IPC (`removed_configs 0, removed_workspaces 0, error null`), then clicked Unregister in the still-open dialog. The dialog stayed open and showed `This project is already removed.` The URL stayed `#/projects/1`, and all DB counts were unchanged by that click.

### U8 Unregister Beta, count with a mixed workspace: PASS
After U3, W2 contained only Beta's item, so it is fully owned by Beta. Actual preview text from the DOM:

```
Unregister project

Unregister this project? Its files and folders are not changed or deleted.

This also removes 1 config entry and 2 workspaces.

Cancel
Unregister
```

The summary said "2 workspaces" (W2 and W3), not "1 workspace". After confirming, `workspaces` went from 2 to 0, so the stated count (2) equals the number of workspaces the DB lost (2). Config went from 1 to 0 (stated 1). The redirect went to `#/projects`, which showed "No projects registered yet".

### U9 After U8: PASS
projects 0, project_tags 0, config_files 0, workspaces 0, workspace_items 0. Notes 2 and secrets 2 (unchanged row counts). "Alpha note" kept `project_id` 1, "Alpha key" kept 1 and "Beta key" kept 2 (dangling, no project rows); "General note" stayed NULL. The Secrets list then showed both keys as "Project removed". The whole alpha+beta hash list was re-checked: 6 of 6 files identical to the "before" values.

### U10 General note: PASS
"General note" showed "General" in the list (not "Project removed"). Opened: title `General note`, body `General body GENERAL-NOTE-BODY`, Project select `General`. DB `project_id` NULL.

### V1 "Delete vault" present when set up (AC-B01): PASS
- Unlocked: Vault page showed "The vault is unlocked.", auto-lock select, "Lock now" and "Delete vault".
- After "Lock now" ("You locked the vault."): the page showed Master password, Unlock and a still-present "Delete vault" button.

### V2 Dialog preview (AC-B03): PASS
With the vault locked and 2 secrets, the dialog text was exactly:

```
Delete vault

Delete the vault and all 2 secrets? The secrets cannot be recovered. Notes are not affected.

Master password
Type DELETE to confirm
Cancel
Delete vault
```

The sentence matches AC-B03 character for character with N = 2. The final "Delete vault" submit button (`type=submit`) `disabled` state: with an empty field, true; typed `delete`, true; typed `DELET`, true; typed `DELETE ` (trailing space), true; typed `DELETE`, false. Extra: with 0 secrets (after V8) the dialog said `Delete the vault? No secrets are stored.` (matches the spec edge case).

### V3 Wrong master password (AC-B04): PASS
`DELETE` typed, password `wrong-password-xyz`. The dialog stayed open with the inline text `Wrong master password.` DB after: vault_meta 1 row, secrets 2 rows, all other counts unchanged. Observation (not a spec item): after the error, `document.activeElement` was `BODY`, not the password field.

### V4 Empty password: PASS
`DELETE` typed, password empty: inline `Enter your master password.` DB unchanged (secrets 2, vault_meta 1).

### V5 Correct password (AC-B02, AC-B05): PASS
Done from the unlocked state: unlocked with the password, opened the dialog, password `E2E-Release-Pass-2026` and `DELETE`, submit enabled, confirmed. The dialog closed and the page stayed on `#/vault` showing the setup form ("Choose a master password to protect your vault.", Master password, Confirm master password, "I understand there is no recovery.", Create vault disabled). DB: `vault_meta` 0 rows, `secrets` 0 rows, notes 2 (unchanged), projects 0, workspaces 0. The vault row before the delete was `m_cost 19456, t_cost 2, p_cost 1, auto_lock 5, salt 16 bytes, verifier 72 bytes`.

### V6 Vault-deleted state (AC-B07): PASS
- Navigating to Secrets showed the Vault setup form, not an empty list. The page text contained no `REMOVAL-SECRET`, "Alpha key" or "Beta key".
- Restarted the app (WM_CLOSE on 23684, relaunch as 12480). Secrets again showed the setup form; DB: vault_meta 0, secrets 0, notes 2.
- Limit: AC-B07 (key cleared from memory) cannot be observed from outside the process. What I verified is the behaviour (setup form, no values) and the restart.

### V7 Notes readable without the vault: PASS
After the restart, with no vault: "General note" opened with body `General body GENERAL-NOTE-BODY`, and "Alpha note" opened with `REMOVAL-NOTE-ALPHA`. Notes use their own key; the Credential Manager entry remained.

### V8 Re-setup: PASS
Set up a new vault with `E2E-Release-Pass-2026` again. The page showed "The vault is unlocked." `vault_meta` 1 row (id 1, salt 16 bytes, verifier 72 bytes, auto_lock 5). The Secrets page showed "No secrets yet".

### P1 No secret or note plaintext leaking: PASS
Terms scanned: `REMOVAL-SECRET-ALPHA`, `REMOVAL-SECRET-BETA`, `REMOVAL-NOTE-ALPHA`, `E2E-Release-Pass-2026`, plus `GENERAL-NOTE-BODY` and the wrong password `wrong-password-xyz` as extras. Each as UTF-8 and UTF-16LE bytes.
- DB file (`devshelf.sqlite3`, 86,016 bytes, while the app was running, and again on the archived copy after close): 0 matches for all six terms, both encodings. No `-wal`/`-shm` files existed. The note `body_enc` blobs are 58 and 70 bytes and contain no plaintext. The secrets (60 and 59 bytes at seed time) were deleted in V5.
- First (retry) DB, archived: 0 matches.
- WebView console: 11 entries captured across all sessions (CDP `Runtime`/`Log`), all the verbose "Password forms should have (optionally hidden) username fields" hint. 0 matches. No exceptions.
- `app.log`: not created (no output from the app), see "Harness events".
- Not scanned: the page text and screenshots while a secret was revealed (they showed the value by design in U6).

### P2 Light mode: PASS (measured plus one screenshot looked at)
Settings > Appearance > Light gave `data-theme="light"` and `localStorage devshelf.theme = light`. Measured text contrast (computed foreground against the effective background), for the new dialogs:
- Unregister dialog: heading and body text `rgb(14,17,22)` on `rgb(244,245,247)`, 17.34:1. "This also removes 1 config entry." 17.34:1. Cancel 15.84:1. Unregister button white on `rgb(26,95,196)`, 6.04:1.
- Delete vault dialog: heading and body 17.34:1, labels `rgb(74,81,92)` 7.34:1. Submit button white on `rgb(154,91,0)` (amber), 5.43:1 (measured in its enabled state; it is shown at reduced opacity when disabled, which I did not measure).
- Warning text, `Wrong master password.` and `This project is already removed.`: amber `rgb(154,91,0)` at 16 px on `rgb(244,245,247)`, 4.97:1. This is amber/brown, not red. I looked at the `v3-wrong-light` screenshot: the error line was readable.
- Switched back to Dark (`data-theme="dark"`, `devshelf.theme = dark`).
- Not verified: I did not visually inspect every dialog state in light mode; I measured the states above and looked at one screenshot.

### P3 No uncaught error overlay: PASS
Visited Home, Projects, Workspaces, Config Catalog, Vault, Secrets, Notes, Settings and Help (real clicks). Each showed its heading, with no error overlay, no blank screen, no open dialog, and no `exception` events.

## Defects

### D2 (high): reused project id re-links "Project removed" notes and secrets to a new project
- Observed: `projects.id` is `INTEGER PRIMARY KEY` without `AUTOINCREMENT` (checked in `sqlite_master`; there is no `sqlite_sequence` table). Unregistering a project leaves `project_id` on its notes and secrets (by design, AC-A07/A08). With the projects table empty after U9, I registered "Removal Gamma"; it got id 1.
- Result in the UI: the Notes list showed "Alpha note, Removal Gamma", and after a page reload the Secrets list showed "Alpha key, API key · Removal Gamma · Dev". The Alpha note and Alpha key were attributed to an unrelated new project. After Gamma was removed (and after the app restart), they showed "Project removed" again.
- [INFERENCE] Not observed: the same can happen whenever the highest-id project is unregistered and another is registered (for example, unregistering Beta, id 2, then registering a new project would reuse id 2 and re-link "Beta key"). SQLite reuses `max(rowid)+1` when there is no AUTOINCREMENT. I only demonstrated it for id 1.
- Impact: AC-A07/AC-A08 ("show Project removed") hold only until an id is reused. It is a wrong-association bug, not data loss.
- Related smaller observation: before the reload, the Secrets list kept showing "Project removed" for Alpha key even though Gamma had taken id 1, so the Secrets list is also not refreshed after a project registration (global `staleTime: Infinity`).

### D1 (medium): Workspace detail is stale after unregister
- Observed: after U3 the DB held one item for W2, but the W2 detail page (`#/workspaces/2`), opened from the Workspaces list, showed "2 items, reopened in this order" with the removed `alpha\readme.txt` row, listed with Project "No project" and a Remove button. It did not refresh after waiting 4.5 s or after leaving and reopening the page (the list screen correctly said "1 item"). Clicking Remove on the stale row showed the normal confirm text (I cancelled, nothing changed in the DB).
- Cause, from reading the source (not from running it): `src/components/unregister-project-dialog.tsx` invalidates `projects, project, home, config-files, workspaces, notes, secrets`, but the workspace detail query key is `["workspace", id]` (`src/screens/workspace-detail-screen.tsx` line 49) and the global default is `staleTime: Infinity` (`src/app.tsx` line 9). `["workspaces"]` does not match `["workspace", id]`, so the cached detail of a workspace that was opened earlier in the same session is never refetched. It only affects workspaces whose detail page was open earlier in the session (as W2 was, since I added items to it).
- Impact: the user sees items that no longer exist; restoring from that stale page is a possible wrong action (not tested).

## Other observations (not defects)

- The dialog title is "Unregister project" / "Delete vault"; the confirm sentences match the spec. When a project has no config entries or workspaces, no "This also removes ..." line is shown (observed on the Gamma stale dialog). With only one config entry it says "1 config entry." (singular, observed in light mode).
- The note editor's Project select includes a "Project removed" option for a dangling reference.
- After a wrong vault-delete password, focus moved to `BODY`, not the password field.

## Cleanup record

- App closed via `WM_CLOSE` to the window (final PID 12480, wrapper 18832). Verified both gone with `Get-Process`. No other process touched.
- DB archive (moved, not deleted): `C:\Users\Admin\devshelf-test-archive\removal-e2e-20261017104641\devshelf.sqlite3`, 86,016 bytes, SHA-256 `b910375d97c36607012ab1fd91b9cbf5192768191437d4651b4e09af28b77a8a` (computed just before the move; no `-wal`/`-shm` files existed). Earlier aborted-seed archive: `C:\Users\Admin\devshelf-test-archive\removal-e2e-20261017103752-seedretry\devshelf.sqlite3`, SHA-256 `3052e73db5177e2ac4df4b0f7df0f73d59fddf5373607598187d0d813f03d3e5`.
- Final DB content in the archived file: projects 0, config 0, workspaces 0, notes 2, secrets 0, vault_meta 1 (the vault from V8).
- Credential Manager: `notes-data-key.DevShelf` was absent at the start and created during this run, so I ran `cmdkey /delete:notes-data-key.DevShelf` ("Credential deleted successfully."); `cmdkey /list` then no longer showed it.
- `%APPDATA%\app.devshelf.desktop` ends EMPTY (listing `[]`).
- `%TEMP%\devshelf-removal-e2e` was created by this run and was removed (it held the test folders alpha, beta, gamma, helper scripts and screenshots; the folders' hashes were recorded above).
