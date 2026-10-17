# DevShelf 0.1.0 release E2E report

Build under test: `src-tauri/target/release/devshelf.exe` (release build, 14,395,904 bytes, built 2026-10-17 15:56). The UI was driven through WebView2 CDP on port 9222 (real `Input.dispatchMouseEvent` and key/text events), and the DB was read read-only with `bun:sqlite`. No source files were changed. No `npm install` and no build were run.

Run date: 2026-10-17 (about 16:17 to 16:33 local). Test folders were under `%TEMP%\devshelf-release-e2e\`.

## Summary

| Result | Count |
|---|---|
| PASS | 19 (R01-R19, including R18 with a caveat) |
| FAIL | 0 |
| BLOCKED | 0 |
| Optional R20 | PASS (file presence and size only, not run) |

Findings and caveats are listed in the "Findings" section at the end.

## Harness events

- The first app instance (PID 17980) was started with `Bun.spawn` from the eval worker. When the worker was force-killed during the R12 reveal timing cell, that app died with it. It was restarted as PID 7060, launched with `Start-Process`. This was a harness accident, not an app crash. It gave an extra, unplanned restart: the vault came up locked with "Locked because DevShelf just started." and all data was intact.
- The formal R16/R17 restart was done on PID 7060 by posting `WM_CLOSE` to the "DevShelf" window. The app exited within about 1 s. The relaunch was PID 23680, and the final close was also by `WM_CLOSE`. `Process.CloseMainWindow()` did not close the app, so `WM_CLOSE` was used instead.
- stdout/stderr were redirected to files (`app.log`, `app2.log`, `app3.log` and the `.err.log` files) by the process launcher rather than `tee`. All six files were 0 bytes. The release build has no console subsystem.
- Real Enter key events via CDP did not submit the vault form, so the Unlock/Create/Save buttons were clicked with the mouse. I treat this as a CDP artifact, not an app defect.
- A few console errors were caused by my own probing: two `fetch()` calls from the page were blocked by the CSP (`connect-src ipc: http://ipc.localhost`), and one `GET http://ipc.localhost/storage_status` returned 405. They are not app defects. They are excluded from the console results below.

## Results

### R01 Fresh launch: PASS
- Before launch, `%APPDATA%\app.devshelf.desktop` existed and was empty (listing `[]`). It was also empty at the start of the task.
- After about 5 s, the folder contained `devshelf.sqlite3`. The UI showed Home with the empty-state "Register your first project", with no error screen.
- `PRAGMA user_version` = 10.
- Tables: `app_meta, projects, project_tags, editor_profiles, workspaces, workspace_items, config_files, notes, vault_meta, secrets`. All expected tables are present.
- CDP page target: `http://tauri.localhost/#/home`, title "DevShelf".

### R02 Vault first use: PASS
- The Vault nav showed the SETUP form ("Choose a master password to protect your vault.", with Master password and Confirm fields), not an unlock form.
- The warning "If you forget this password, the vault cannot be opened. There is no reset." is visible.
- The "Create vault" submit button was `disabled=true` with both passwords filled and the acknowledgement unticked. It became enabled after "I understand there is no recovery." was ticked.

### R03 Setup validation: PASS
- Mismatched confirmation showed "Passwords don't match."
- An 11-character password (`abcdefghijk`) in both fields showed "Use at least 12 characters." `vault_meta` stayed at 0 rows after both rejections.
- `E2E-Release-Pass-2026` created the vault, and the page showed "The vault is unlocked." and the auto-lock selector.
- `vault_meta` row: id 1, salt blob(16), m_cost 19456, t_cost 2, p_cost 1, verifier blob(72), auto_lock_minutes 5 (default), created_at 2026-10-17T09:17:58Z. It stores salt, Argon2 cost parameters and a verifier, and no password column.
- The whole `devshelf.sqlite3` file (86,016 bytes) was searched for the password as UTF-8 and UTF-16LE bytes: 0 matches for both.

### R04 Lock / wrong password / unlock: PASS
- "Lock now" showed "You locked the vault."
- A wrong password (`Wrong-Password-123`) showed "Wrong master password." and `document.activeElement` was `INPUT[type=password]`, so focus is on the password field.
- The correct password unlocked the vault ("The vault is unlocked.").

### R05 Projects (F02): PASS
- Registered "Release Project A" at `...\devshelf-release-e2e\proj-a`, type Other, tags `release, e2e`. The message was "Project Release Project A registered. No project files were copied or changed."
- Registering the same path again was rejected with "`C:\Users\Admin\AppData\Local\Temp\devshelf-release-e2e\proj-a` is already registered as Release Project A." The `projects` table kept 1 row.
- Pin: the project detail page changed to "Pinned: Yes" and the DB had `pinned=1`. The list shows `★Release Project A`.

### R06 Editor profiles (F03): PASS
- Entering `cmd /c echo x` as the Notepad++ executable and clicking Save was rejected with "Pick an editor program (.exe)." The DB `exe_path` stayed empty.
- A valid profile was saved: Custom, renamed "Release Editor", executable `C:\Windows\System32\notepad.exe`. The UI showed "Settings saved" and the DB row `custom | Release Editor | C:\Windows\System32\notepad.exe`.
- The test editor (`Open test` and the profile) was never launched. Only saving was exercised.

### R07 Workspaces (F04): PASS
- Created "Release Workspace A" and added the folder `proj-a\src` and the file `proj-a\src\main.txt`, both linked to Release Project A with the Windows default editor. A third item, the folder `ws\gone-dir`, was added for R08.
- DB `workspace_items` rows: 3 (positions 0, 1, 2; kinds folder, file, folder; `project_id` 1; `editor_kind` windows_default).
- The workspace page showed "3 items, reopened in this order."

### R08 Restore (F05): PASS
- `gone-dir` was deleted from `%TEMP%` and Restore was clicked. The result was "2 opened, 1 missing, 0 unreachable, 0 could not open." The "Missing (1)" section listed `gone-dir` with its path and the reason "Path not found.", and offered Relink and Ignore.
- The folder was re-created and Restore clicked again. The result was "All 3 items opened." The DB showed `last_restore_missing = 0` and `last_restored_at` set.
- The restore opens items through the Windows default handler. The `.txt` file opened in a Notepad++ instance that was already running (PID 18164, started 15:27, before the run, so not mine). I did not touch it. Folder windows opened by the shell may still be open on the desktop.

### R09 Config catalog (F06): PASS
- Added `proj-a\.env.example` with environment Dev and kind `.env` (stored `dotenv`). Status: Available. A second entry, `proj-a\app.yml` (Production, YAML), was added for the filter checks.
- Deleted `.env.example`, left the Config Catalog page, and returned to it. Status: Missing with "The file was not found at this path." and Relink / Remove from catalog actions.
- Timing: Missing was visible 168 ms after the nav click that reloaded the page. This includes the 100 ms sleep in the script and the 50 ms polling interval. The status is refreshed when the page loads, not by a file watcher.
- Filters: Environment Dev showed only `.env.example`, Production only `app.yml`. Kind YAML showed only `app.yml`, kind `.env` only `.env.example`. Status Missing showed only `.env.example`, Available only `app.yml`. Project A + Available showed `app.yml`. Status Inaccessible showed "No config files match these filters." "Clear filters" reset all four selects to `all` and both entries returned.

### R10 Home (F10): PASS
- Home showed "PINNED PROJECTS: Release Project A" (path and drive) and "RECENT WORKSPACES: Release Workspace A, 3 items, Release Project A, Last restored 17 Oct 2026, 16:20".
- The item count is shown on the workspace card. The pinned project card shows path and drive and has no item count.
- The "Register project" quick action opened `#/projects` with the register dialog. The pinned project link went to `#/projects/1` and the workspace link to `#/workspaces/1`.

### R11 Notes (F07): PASS
- Created "Release Note A" linked to Release Project A, with the body `REL-NOTE-SECRET-7781`. "Saved at 16:21" appeared 1.33 s after the text was typed (100 ms polling; typed with one `insertText` call). This is consistent with a debounce of about 1 s.
- DB: `title = 'Release Note A'` in plaintext. `body_enc` is 60 bytes. It does not contain the body as UTF-8 or UTF-16LE bytes, and the same holds for the whole DB file.
- Credential Manager: before the first save there was no `notes-data-key.DevShelf` entry. After it: `Target: LegacyGeneric:target=notes-data-key.DevShelf`, `User: notes-data-key`. Only presence was checked.
- Quick navigation: typed ` EDIT2`, then clicked Home 12 ms later. The DB `body_enc` length changed 60 to 66, and on reopen the textarea held `REL-NOTE-SECRET-7781 EDIT2`.
- Filters: with "Release Note General" (General) and "Release Note A" (project), "General" showed only the General note, "Release Project A" showed only Release Note A, and "All notes" showed both.

### R12 Secrets (F09): PASS
- With the vault unlocked, added "Release API key", API key, Release Project A, Dev, value `REL-SECRET-VALUE-4412`. The list showed `API key · Release Project A · Dev` with `••••••••`. The value was not in the page text or HTML after saving.
- Reveal showed the value with Hide/Copy/Edit/Delete buttons.
- Re-mask timing measured in the page with a `MutationObserver` and `performance.now()`: shown at 95478.2 ms, hidden at 125490.1 ms, so 30.01 s. A first measurement with polling from outside the page gave 35.6 to 36.6 s. The difference is polling overhead, not app behaviour.
- Copy showed "Copied." and the value was not in the page text. `Get-Clipboard` returned `REL-SECRET-VALUE-4412`. The clipboard still held the value after 40 s (no auto-clear, consistent with F14 being "not in this version"). It was cleared with `Set-Clipboard -Value $null` and `Get-Clipboard` then returned an empty string.
- Delete showed "Delete this secret? This cannot be undone." with Cancel and Delete. Cancel kept the secret. The real delete was done at cleanup and `secrets` went to 0 rows.
- DB: `body_enc` is 61 bytes and does not contain the value as UTF-8 or UTF-16LE (row and whole file). The password was also not found in the file.

### R13 Vault auto-lock (F08): PASS
- Timeout set to 1 minute (`auto_lock_minutes = 1` in DB), then no input. A countdown "Locks in 0:54" appeared 12 ms after the setting change (6 s of the minute had already elapsed since my last click), then decreased every second to "Locks in 0:01". At about 53.8 s after the change (about 60 s after the last input), the page showed "Locked after 1 minute of inactivity."
- Unlocked again with the password.
- Mouse keep-alive: on Home with the 1-minute setting, `mouseMoved` events were sent at about 15, 35, 55 and 75 s. The countdown reset to "Locks in 1:00" at each move (the lowest value reached was 0:41). After 85 s the vault was still unlocked and the page was still `#/home`.
- Timeout set back to 5 minutes (`auto_lock_minutes = 5` in DB).

### R14 Secrets while locked: PASS
- With the vault locked, navigating to Secrets (`#/secrets`) showed the Vault lock screen ("Master password", "Unlock"). The text "Release API key" was not in the page. Notes opened normally while locked.

### R15 Help (F16): PASS
- `#/help` opened. Sections listed: Start here, Projects, Editor profiles, Workspaces, Config catalog, Notes, Vault, Secrets, Home, Privacy and security, Troubleshooting, Changelog, About. Each of the 13 rendered text without error.
- The Changelog shows "VERSION 0.1.0" with F01 to F10 and F16 as shipped, and "NOT IN THIS VERSION" with F11, F12, F13, F14 and F15, each "not in this version."
- The Appearance paragraph is present in Start here: "Appearance: choose Dark or Light in Settings. Dark is the default."

### R16 Appearance: PASS (visual judgement partly measured, not fully human-verified)
- Settings > Appearance > Light: `data-theme="light"`, `localStorage devshelf.theme = light`, body background `rgb(255,255,255)`, text `rgb(14,17,22)`. The screenshot shows a light sidebar and cards with dark text, and everything is readable.
- Measured text contrast on 8 pages in light mode (computed foreground vs. nearest opaque background): the lowest ratios were 4.37 ("Available" status text on Config Catalog), 4.55 (Secrets), and 4.76 (accent buttons on Home, Projects, Workspaces, Notes). On the Settings page the lowest was 3.99 for the accent "Default" badge text on a grey card. All the other text on all pages was above 4.76.
- Warning text on forms: the duplicate-registration message in the Register dialog is amber `rgb(154,91,0)` on the light dialog surface. Screenshot checked, readable (measured 4.97:1).
- Dark: returned to `data-theme="dark"`, black background, white text, as in the screenshot.
- Light, then restart (WM_CLOSE and relaunch): after relaunch the theme was still light (`data-theme="light"`, `localStorage devshelf.theme = light`). Then it was set back to Dark (`localStorage devshelf.theme = dark`) before the end.
- Not verified: I cannot say a human reviewer would find every part of every screen readable; I only measured text contrast on 8 pages and looked at 2 screenshots.

### R17 Restart persistence: PASS
- After closing the window and relaunching: the Vault page showed "Locked because DevShelf just started."
- DB counts after restart: projects 1, notes 2, secrets 1, workspaces 1, workspace_items 3, config_files 2, `user_version` 10. These match the counts before the restart. After unlocking, the Secrets list showed "Release API key" and the Notes list showed "Release Note General" and "Release Note A".
- The same locked-at-start behaviour was seen after the unplanned restart described under "Harness events".

### R18 Privacy: PASS with caveat
- Page level: the CDP network log recorded only `http://tauri.localhost/...` and `http://ipc.localhost/...` requests (plus `null` for non-network entries). All responses were 200.
- `netstat -ano` for the `devshelf.exe` PID and its process tree:
  - `devshelf.exe` itself had no connection, in two samples (one on PID 7060, one on PID 23680).
  - The only localhost connections were the WebView2 browser process on the debug port 9222 (the CDP session I opened).
  - In the second sample (session 3, PID 23680), WebView2 processes had outbound TCP 443 connections over IPv6: `msedgewebview2.exe` browser process (PID 24220) to `[2603:1046:c01:2460::2]:443` (two connections) and the `NetworkService` utility process (PID 23928) to `[2620:1ec:33:2::11]:443`. They were still present in three consecutive samples 2 s apart. The first sample (session 2) showed no such connection. I did not identify the remote owner (the address prefixes look like Microsoft ranges, but I did not verify it) or the purpose, and I did not look at what was sent. These connections belong to the WebView2 runtime, not to the app's own process or to page requests, and the page-level log shows no request to any external host.
- Limits: `netstat` is a sample at a point in time, not a capture; short connections would be missed.
- Secret scan: counts of `REL-NOTE-SECRET-7781`, `REL-SECRET-VALUE-4412` and `E2E-Release-Pass-2026` were 0 in all six app log files (all 0 bytes) and 0 in the captured WebView console (CDP `Runtime`/`Log` events from session 2 and 3, 14 entries). The first session's console was only watched live (nothing but CSP lines from my own probes). The console entries were: "Password forms should have (optionally hidden) username fields for accessibility" (verbose DOM hints, on the Vault and Secrets pages) and my own CSP/405 probe errors.

### R19 Error display and the startup 404: PASS
- Visited Home, Projects, Workspaces, Config Catalog, Vault, Secrets, Notes, Settings and Help. Each showed its heading, with no error overlay and no blank screen.
- No uncaught exception events and no non-200 responses were seen in the page. There was no 404 at startup in this build: `http://tauri.localhost/favicon.ico` returned 200 every time it was requested (17 requests logged in the second session), so there is no 404 URL to report.
- Console messages seen: only the verbose "Password forms should have (optionally hidden) username fields" hint on the Vault and Secrets pages, plus the probe errors listed under "Harness events".

### R20 Installer (optional): PASS
- `src-tauri/target/release/bundle/nsis/DevShelf_0.1.0_x64-setup.exe`, 3,537,524 bytes (about 3.37 MiB). An MSI also exists: `bundle/msi/DevShelf_0.1.0_x64_en-US.msi`. The installer was not run, so its behaviour is not verified.

## Findings and notes

1. **WebView2 outbound 443 traffic (R18).** `devshelf.exe` made no network connection, but the WebView2 runtime processes had established outbound HTTPS connections during session 3. If "no network connection" must hold for the whole process tree, this needs a decision. I did not determine what the traffic was.
2. **Contrast (R16).** The lowest measured contrast in light mode is 3.99:1 (accent "Default" badge on Settings) and 4.37:1 ("Available" on Config Catalog). If the target is WCAG AA 4.5:1 for small text, these two are slightly below it. This is my measurement against the usual threshold, not a failing check of the spec.
3. **`<html class="dark">` stays set** when `data-theme="light"` (observed with `document.documentElement.className`). The visible UI is light, so this looks harmless.
4. **Reset in Settings does not clear saved editor profiles.** After "Reset", the DB still had the Custom profile "Release Editor". It only reverts unsaved edits. The profile left in the DB was archived with the DB file.
5. **No UI to delete a project or the vault.** The project and the `vault_meta` row stayed in the DB until it was archived.
6. **The clipboard is not auto-cleared** after Copy (still set after 40 s). The Changelog lists F14 "Clipboard auto-clear" as not in this version.
7. **Restore uses the Windows default handler** for `.txt` files and folders. It reused a Notepad++ that was already running and may have opened Explorer windows.

## Cleanup record

- Test secret, both notes, the workspace and both config entries were deleted in the UI (confirmations shown; DB counts 0). The project and the vault could not be removed in the UI.
- App closed with `WM_CLOSE` to the window (PIDs I started: 17980 died with the worker, 7060 and 23680 were closed). No process was killed by image name. The pre-existing Notepad++ (PID 18164) was left alone.
- Credential Manager: `notes-data-key.DevShelf` was absent before the first note save and present after, so it was created in this run. `cmdkey /delete:"notes-data-key.DevShelf"` reported "Credential deleted successfully." and `cmdkey /list` no longer showed it.
- DB SHA-256 before the move: `3b27d77c4f3aa05a88f214cbb1585750456e2615ce10483c058ae1d67f7f186c` (86,016 bytes).
- Archive: `C:\Users\Admin\devshelf-test-archive\release-e2e-20261017093342\devshelf.sqlite3` (the timestamp is UTC; no `-wal`/`-shm` files existed).
- `%APPDATA%\app.devshelf.desktop` ends empty (listing `[]`).
- `%TEMP%\devshelf-release-e2e` was created by this run and was removed (it held the test projects, helper scripts, screenshots and log files).
