# F01 — Foundation and navigation: E2E smoke results

- App under test: debug build `src-tauri/target/debug/devshelf.exe` (Tauri dev; frontend served by Vite on `localhost:1420`). Identifier `app.devshelf.desktop`, data folder `C:\Users\Admin\AppData\Roaming\app.devshelf.desktop`.
- Method: the real window was launched. The WebView2 page was inspected and driven over its local remote-debugging port (`127.0.0.1:9222`, no external network). Native window title and process list came from Windows. Alt+Left/Alt+Right were sent as real OS keystrokes. Tab/Enter were sent as Chromium key events (see notes).
- No source files were modified.

| ID | Steps | Expected | Actual | Result |
|---|---|---|---|---|
| AC-01 | Launch app; read native window title (tasklist /v) and page hash | Title "DevShelf"; route `#/home` | Native title `DevShelf`; page title `DevShelf`; URL `#/home`; Home heading shown | PASS |
| AC-02 | Read sidebar `nav a` list | Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings in order | Exactly that order, 7 links | PASS |
| AC-02/03 | Click each item in turn | Heading shown for the area; active item highlighted | Each click set the hash to `#/home`, `/projects`, `/workspaces`, `/config`, `/vault`, `/notes`, `/settings`; heading matched; exactly one link had `aria-current="page"` plus `bg-surface-raised text-ink`, and it was the clicked item each time | PASS |
| AC-04a | Set `window.__marker` before navigating; click through 7 areas | Marker survives (no reload) | Marker `m1` present after every navigation; history length grew 1→7 | PASS |
| AC-04b | Real Alt+Left / Alt+Right keystrokes (OS level), then `history.back()/forward()` | Moves between visited areas | Alt+Left: `#/does-not-exist` → `#/settings`; Alt+Right: back to `#/does-not-exist`. `history.back()` from Settings → `#/notes`, forward → `#/settings`. Marker unchanged | PASS |
| AC-05 | Set `location.hash='#/does-not-exist'` in the page | "Page not found", sidebar visible, link to Home | Text "Page not found / That page does not exist in DevShelf. / Go to Home"; 7 sidebar links still present; link `Go to Home` → `#/home`. No reload (marker kept) | PASS |
| AC-06 | Focus sidebar, press Tab repeatedly (CDP key events, focus emulation on); press Enter on focused item | Each item reachable; Enter activates; focus ring visible | Tab landed on Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings in turn, each `:focus-visible` with `outline: solid 2px rgb(43,137,255)`. Enter on Projects → `#/projects`; two Tabs + Enter on Config Catalog → `#/config`. See note 1 | PASS (note 1) |
| AC-10 | Open Projects, Workspaces, Config Catalog, Vault, Notes; read text | Heading + empty-state text | Projects: "No projects registered yet / A project is a reference to a folder. DevShelf never copies or changes your files."; Workspaces: "No workspaces yet / A workspace saves an ordered set of files to reopen together."; Config Catalog: "No config files registered / Registering a file stores its path only, never its contents."; Vault: "No secrets yet"; Notes: "No notes yet". See note 2 | PASS (note 2) |
| AC-11 | Count `main button, main a` on those pages (and Home) | 0 action buttons | 0 on every page | PASS |
| AC-20 | After first launch list data folder | `devshelf.sqlite3` in `%APPDATA%\app.devshelf.desktop` | Folder existed with `devshelf.sqlite3` (12288 bytes) | PASS |
| AC-21/22 | Python sqlite3 `mode=ro`: `PRAGMA user_version`; list objects | 1 | `user_version = 1`; objects: table `app_meta` and its autoindex | PASS |
| AC-22 | Close app, relaunch, compare file | Same file, version 1, app starts normally | Relaunch started on Home, Settings shows Storage: OK. user_version 1; SHA-256 prefix `1dea1e029e893c44` and mtime identical before and after (file untouched) | PASS |
| AC-12/26/31/41 | Open Settings | "Storage: OK", data folder path, exact privacy sentence | Text: "Storage: OK", `C:\Users\Admin\AppData\Roaming\app.devshelf.desktop`, and "DevShelf stores all data on this computer. Nothing is sent over the network. Project names, labels, environment names, and file paths are stored unencrypted in the local database." (exact match) | PASS |
| AC-30a | With app running, run `devshelf.exe` a second time (3 times, including once after minimizing) | No second window; second process exits; one devshelf process left | Second process exit code 0 within ~1.1 s each time; `tasklist` showed only the original PID; CDP showed a single page target | PASS |
| AC-30b | Same; observe that the first window comes to the front | First window foregrounded | Foreground window stayed another app (Chrome / Notepad++ / terminal) after each attempt; the original window stayed visible. Minimize-and-restore could not be observed (CDP minimize did not affect the native window; Win32 FindWindow returned no handle). Code calls `unminimize/show/set_focus` (`src-tauri/src/lib.rs:14-19`), but the OS foreground-lock may block a background process stealing focus | BLOCKED (foreground change not observable in this environment) |
| AC-25 | Close app; rename data folder to `.bak`; create a regular file at the folder path; launch | Full-window "DevShelf can't start", message with folder, code `storage_unavailable`, no sidebar | Text: "DevShelf can't start / DevShelf cannot write to C:\Users\Admin\AppData\Roaming\app.devshelf.desktop. Check that the folder exists and that your Windows account can write to it. / storage_unavailable"; 0 `nav` elements. After closing: file removed, `.bak` renamed back; DB intact (user_version 1, same SHA/mtime as before) | PASS |
| AC-24 | Close app; set `user_version = 99` via Python; launch | Full-window screen, newer-version message, code `schema_too_new` | Text: "DevShelf can't start / This data was created by a newer version of DevShelf. Update DevShelf to open it. / schema_too_new"; 0 `nav` elements. After close, file still user_version 99 with the same SHA/mtime as just after my own write (app did not modify it) | PASS |
| AC-24 (recovery) | Set user_version back to 1; launch | Normal start | Home rendered, sidebar present, user_version 1 | PASS |
| Edge: re-select active | On `#/settings` click the Settings link; compare `history.length` | No new history entry | Length 7 before and 7 after, hash unchanged | PASS |
| Edge: trailing slash / query | Not requested in this run | — | Not exercised | BLOCKED (out of assigned scope) |

## Notes

1. Tab/Enter were sent as Chromium key events via the debug port with focus emulation enabled, because OS-level SendKeys could not keep DevShelf in the foreground (the foreground was a different app). The Tab walk started mid-sidebar from a prior focus, so order from Home was observed as Home→Projects→Workspaces→Config Catalog continuing to Vault→Notes→Settings→(body)→Home. Enter activation was observed on two items (Projects, Config Catalog); Enter was not repeated on every item.
2. Compared with `docs/preview/index.html`: copy is split into heading + paragraph in the app, so strings differ in structure. Vault in the preview is "No secrets yet." and Notes is "No notes yet." (with periods); the app shows "No secrets yet" / "No notes yet" as headings with no body. Projects/Workspaces/Config/Home body sentences match the preview text. Judged cosmetic, recorded for the reviewer.
3. The DB bytes after check 11 differ from the original file (SHA prefix `a5d1d9d1abf503c9` vs `1dea1e029e893c44`) because of my `user_version` 99 → 1 write (SQLite header change counter). Content/schema is the same (`app_meta`, user_version 1).
4. Real keystrokes sent by `SendKeys` to test Alt+Left/Right and the first Tab attempts: the foreground was not always DevShelf. Alt+Left/Right clearly took effect in DevShelf. Other early Tab/Enter presses may have gone to whatever window was in front (the Orca host window at the time); no effect on it was observed or checked.

## AppData changes

- `%APPDATA%\app.devshelf.desktop` did not exist before the run; the app created it with `devshelf.sqlite3` (this is the expected smoke-run artifact and it remains).
- Check AC-25: renamed that folder to `app.devshelf.desktop.bak`, created a plain file at the folder path, then deleted the file and renamed the folder back. Verified restored.
- Check AC-24: set `user_version` to 99, then back to 1 on `devshelf.sqlite3`.
- Temporary helper scripts were created in `%TEMP%` and removed.
- All devshelf processes, the dev server, and the Vite server were stopped at the end.
