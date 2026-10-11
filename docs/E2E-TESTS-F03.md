# F03 Editor profiles: E2E results (Phase 2 Step 5)

- Run: `npm run tauri dev` (real app, Windows). The WebView2 UI was driven over the remote debugging port using CDP mouse and key events and `Input.insertText`. The native file dialog was driven with a PowerShell mouse click on the File name box, clipboard paste (SendKeys Ctrl+V) and Enter.
- Data: `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3`. Backup of the F02 file: `...\devshelf.sqlite3.f02-backup`, SHA-256 `b906672e76895b391c26a962daf514dc50d1e7750b78aab22048aba4d3d079f5` (user_version 2 before launch).
- Test files: `%TEMP%\devshelf-e2e-f03\Program Files\fake editor\fake editor.exe` (copy of System32 notepad.exe), `my notes\a b.txt`, `my notes\folder\`.
- Mode: dev build (`tauri dev`, Vite plus debug exe). Built-app smoke (spec P4) is NOT done and remains open; nothing here claims built-app coverage. Restart persistence was checked in dev mode only (devshelf.exe PIDs differed across launches: 15568, 22792, 6232).
- Side effects: `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (debug port 9222) was set only in the shell of the launched dev server, not system-wide; no registry, audit or ETW setting was changed. A `Register-CimIndicationEvent` attempt for process tracing failed ("Call cancelled", not admin) and was dropped.
- No source files were modified.

| ID | Steps | Expected | Actual | Result |
|---|---|---|---|---|
| C01 | Launch app; read DB | user_version 3; 4 rows; only windows_default is_default=1 | user_version 3; rows windows_default, notepad_plus_plus, vs_code, custom; is_default 1/0/0/0 | PASS |
| C02 (AC-01/04) | Open Settings | 4 profiles in order, Windows default marked Default, its path disabled, note shown | Section "Editors": Windows default (Default badge), Notepad++, VS Code, Custom. Windows default path input disabled; note "Opens with the Windows file association" | PASS |
| C03 (AC-05) | Save a valid Custom path, click Set as default on Custom, restart app | Custom becomes default, Windows default marker clears, persists | Marker moved to Custom, DB custom=1 and others 0; after restart still Custom default | PASS |
| C03b (AC-05/20) | Click Set as default on Custom while its path is empty | (check expects default to switch) | Refused: "Enter the path to the editor executable." shown in the Custom row; DB default unchanged. Consistent with AC-20, differs from literal AC-05 | PASS (observation) |
| C04a (AC-20) | Custom is default; clear its path and edit VS Code; Save | "Enter the path to the editor executable." beside Custom; nothing saved | Message shown in Custom row; DB unchanged (vs_code stayed empty) | PASS |
| C04b (AC-22) | Path `fake editor.exe`; Save | report exactly | "Pick an editor program (.exe)." (also when Custom is default); nothing saved | PASS |
| C04c (AC-21) | Path `C:\Nope\missing\ed.exe`; Save | "Can't find `<path>`. Check the path or pick the editor again." | "Can't find `C:\Nope\missing\ed.exe`. Check the path or pick the editor again." | PASS |
| C04d (AC-22) | Path to a b.txt; then path to `my notes\folder`; Save | "Pick an editor program (.exe)." | Both showed "Pick an editor program (.exe)."; nothing saved | PASS |
| C05 (AC-23) | Paths `C:\x&y\a.exe` and `C:\x%y\a.exe`; Save | "This path has characters that are not allowed in an editor path." | Exact message shown for both | PASS |
| C06a (Browse) | Notepad++ Browse..., native Open dialog, paste fake editor path, Enter | Path fills the field | Dialog opened (class #32770 "Open"); the field's DOM value became the fake editor path, and C06b shows it persisted. The first dialog attempt failed (paste went to the wrong window); the retry worked | PASS |
| C06b (AC-10/11) | Save | "Settings saved"; persisted | "Settings saved" shown; DB notepad_plus_plus.exe_path = fake editor path | PASS |
| C06c (AC-10) | Type `C:\junk.exe` in VS Code; Reset | Reverts to saved values | VS Code field back to empty; Notepad++ kept saved value | PASS |
| C06d (AC-12) | VS Code = valid path, Custom = `C:\x&y\a.exe`; Save; restart | Error on Custom; nothing else saved | Error on Custom; DB vs_code stayed empty, also after restart | PASS |
| C07 (AC-40/41/36) | Open test, Windows default, a b.txt | "Opened a b.txt in Windows default." and an editor window | Message shown; window "a b.txt - Notepad" (WindowsApps Notepad.exe, 2 processes). Closed both by PID | PASS |
| C08a (AC-30-32) | Open test, Notepad++ (path = copy), a b.txt | "Opened a b.txt in Notepad++." | Message shown; Open returned in about 0.7 s. The launch itself (argument, process image) was not verified | PARTIAL (message only) |
| C08b (AC-30-32) | Check window title containing "a b.txt" and the process image path | Notepad window with title; image path = the copy | Not observed. Started by hand with the same arguments, the copy exits with code 0 and shows no window, so no process or title could be inspected. Argv tracing was not done (stopped by watchdog instruction); the Rust fake-exe tests cover argv. A running "Notepad++ ... *new 10" process (C:\Program Files\Notepad++) was already open and is not ours | BLOCKED (copy of System32 notepad.exe exits immediately, no window; argv not verified in E2E) |
| C09 (AC-33) | Open `my notes\missing.txt` with Notepad++ | "`<path>` was not found. It may have moved or been renamed." | Exact message. No editor window appeared; an empty Notepad.exe tasklist afterwards is weak evidence only (process start not traced) | PARTIAL (message only; "nothing launched" not verified) |
| C10 (AC-34) | Windows default is default; Custom path cleared via UI and saved; select Custom; Open a b.txt | "Set the path to Custom in Settings to open files with it." | Exact message. Also with VS Code (empty): "Set the path to VS Code in Settings to open files with it." | PASS |
| C11 (AC-35) | Notepad++ path saved; delete the copy; Open with Notepad++; restore copy | "Notepad++ could not start. Check the path in Settings." | Exact message (not a "Can't find" message). Copy restored (exists) | PASS |
| C12 (AC-36) | Open with Windows default | Open returns; editor stays open; UI responsive | Open returned within about 0.7 s, Notepad window stayed open, UI accepted the next actions. Not checkable with the fake exe (it exits) | PASS |
| C13 (AC-41) | Type path in Open test, restart | Path not persisted | Field held the path before restart; empty after restart | PASS |
| C14 (AC-50/51) | Read source | Program plus args, no shell | `src-tauri/src/editors/mod.rs:264-267` builds `LaunchPlan { program, args: vec![item_path] }`; `:279-284` `Command::new(&plan.program).args(&plan.args).spawn()`; `:3-4` doc states no shell. No `cmd`/`powershell` use found in mod.rs or commands/editors.rs. Profile stores only name and exe_path | PASS |
| C15 (AC-53) | grep SQL in mod.rs | Bound parameters or fixed strings | SQL at `:144` (fixed SELECT), `:207-208`, `:212-213`, `:235-236` (`params![]`), `:231` (fixed). `format!(` appears only in tests (`:395`, `:433`, `:475`, `:535`) on test paths, not SQL | PASS |
| C16 | Read Settings bottom | Privacy text and storage row unchanged | "Storage: OK", data folder path and "DevShelf stores all data on this computer. Nothing is sent over the network. Project names, labels, environment names, and file paths are stored unencrypted in the local database." shown | PASS |
| C17 (AC-11/20) | Set as default on Custom with empty path (error shown), then Save | Stale error gone after a successful Save | "Enter the path to the editor executable." stayed beside Custom together with "Settings saved" | FAIL (stale default error not cleared on successful Save) |

PASS 21, PARTIAL 2 (C08a, C09: message verified, launch not), FAIL 1 (C17), BLOCKED 1 (C08b). Restart-persistence rows (C03, C06d, C13) are dev-mode only. Built-app smoke (spec P4) was not run and is pending.

## Cleanup and final state

- Default: Windows default. All paths empty (Notepad++, VS Code, Custom); Custom name "Custom". user_version 3.
- Fake exe copy restored. Test Notepad windows ("a b.txt - Notepad", PID 14416, and PID 6288, which showed no window title; the pre-run Notepad.exe list was empty) were closed by PID; none remained afterwards. The pre-existing Notepad++ (PID 6128, C:\Program Files\Notepad++) was not touched and is not the profile under test.
- Dev server and devshelf.exe stopped.
