# F08 Vault access control: E2E results (Phase 2 Step 5)

- Spec: `docs/specs/vault-access-control/spec.md` (AC-xx). Approved decisions observed: minimum 12 characters; auto-lock Never / 1 / 5 / 15 minutes (default 5); inactivity = no pointer or keyboard input in the window; countdown in the last 60 seconds; no lockout; no password change.
- Run: `npm run tauri dev` from the project root (real app, Windows, dev build, unoptimized Argon2id). Launch 1 ran the existing `target\debug\devshelf.exe` ("Finished `dev` profile ... in 0.40s") and applied migration 9. Launch 2 (restart, check 18) printed "Finished ... in 14.75s", so cargo rebuilt something between launches (a working-tree change outside this run). The checks after it ran against that build.
- UI driving: WebView2 remote debugging port 9222 (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`, set only in the dev-server shell), Chrome DevTools Protocol from Bun. Real input: `Input.dispatchMouseEvent` (move, press, release at element centres) for every click, `Input.insertText` after a click into a field, `Input.dispatchKeyEvent` for Ctrl+A, Delete, Enter and the arrow keys on the native timeout select. DOM reads via `Runtime.evaluate` (not input, so it does not reset the inactivity timer); IPC from `Network.requestWillBeSent` to `http://ipc.localhost/<command>` (each command shows as an OPTIONS preflight plus a POST). DB reads: `bun:sqlite` with `readonly: true`. The only DB write was on a copy (check 16).
- Dev output captured with `tee` to `%TEMP%\devshelf-e2e-f08\dev.log` and `dev2.log`; both contain only build/vite lines (872 and 755 bytes); 0 matches for any password string used. WebView console: 6 entries per launch set (vite connect, React DevTools banner), 0 password matches.
- Backup of the live DB before launch (user_version 8, no `vault_meta` table, confirmed by read-only query): `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3.f07-backup`, SHA-256 `9827f23a089f6bb952cb1541bdfd492c590fecef5e155400751e2f1c3f75c28f` (identical to the live file at that moment, and unchanged at the end).
- Test password (the only one stored): `E2E-Vault-Pass-2026` (19 chars). Wrong-password attempts used throwaway strings.
- Test files: only `%TEMP%\devshelf-e2e-f08\` (scripts, logs, `tamper.sqlite3`).
- No source files were modified.
- The live data folder now contains a set-up vault (intentional, see Cleanup).

| ID | Steps | Expected | Actual | Result |
|---|---|---|---|---|
| V01 (migration, AC-22) | Launch on schema 8; read DB | `user_version` 9; `vault_meta` with CHECK on `auto_lock_minutes`; 0 rows | Before launch: no `vault_meta`, `user_version` 8. After launch `user_version` 9; tables ... `notes`, `vault_meta`. Columns: `id` INTEGER PK `CHECK (id = 1)`, `salt` BLOB, `m_cost`, `t_cost`, `p_cost` INTEGER, `verifier` BLOB, `auto_lock_minutes` INTEGER NOT NULL DEFAULT 5 `CHECK (auto_lock_minutes IN (0,1,5,15))`, `created_at` TEXT. 0 rows | PASS |
| V02 (AC-01/04) | Click Vault in the sidebar with no vault | Setup form (not lock screen); no-recovery warning; submit disabled until the checkbox is ticked | Hash `#/vault`; IPC `vault_status`. Page text: "Choose a master password to protect your vault. / Master password / Confirm master password / If you forget this password, the vault cannot be opened. There is no reset. / I understand there is no recovery. / Create vault". Submit `disabled: true`, checkbox unticked. A real click on the disabled submit sent no IPC and showed no error. After ticking the checkbox the submit was enabled | PASS |
| V03 (AC-02) | Password `E2E-Vault-Pass-2026`, confirmation `Different-Pass-9999`, tick, submit | "Passwords don't match."; nothing created | `role=alert` "Passwords don't match."; no IPC sent (client-side validation); `vault_meta` rows 0 | PASS |
| V03b (AC-03) | 11-character `Short-pass1` in both fields, submit | "Use at least 12 characters."; nothing created | `role=alert` "Use at least 12 characters."; no IPC; `vault_meta` rows 0 | PASS |
| V04 (AC-05) | Valid setup with `E2E-Vault-Pass-2026` both fields, ticked | Vault created unlocked; one row; no password in the row | Button "Creating…" (disabled) at 22 ms, screen switched to "The vault is unlocked." at 576 ms; IPC `vault_setup`. One row: salt 16 bytes, `m_cost` 19456, `t_cost` 2, `p_cost` 1, verifier 72 bytes, `auto_lock_minutes` 5. The password in UTF-8, UTF-16LE and UTF-16BE: not found in the salt, the verifier, the row's latin1 text, the base64 form, nor anywhere in the whole `devshelf.sqlite3` file (77,824 bytes; no WAL/journal file) | PASS |
| V05 (AC-06) | Home, then Vault again while unlocked | Status view, not setup | Page text: "The vault is unlocked. / Auto-lock after inactivity / Never 1 minute 5 minutes 15 minutes / Lock now"; no checkbox; IPC `vault_status` only | PASS |
| V06 (AC-12) | Click "Lock now" | Lock screen with the manual reason | IPC `vault_touch` (the click's pointerdown), `vault_lock`. Text: "Locked because you locked it. / Master password / Unlock". (Wording differs from the spec's "You locked the vault."; it is the agreed wording in this run's checks.) | PASS |
| V07 (AC-07) | Click Unlock with an empty field | "Enter your master password."; screen unchanged; no backend call | `role=alert` "Enter your master password."; no IPC; same lock screen | PASS |
| V08 (AC-08 message, clear, AC-26) | Three wrong passwords in a row (and three more), then the correct one | "Wrong master password."; field cleared; no lockout; correct password still works | Each attempt: button "Unlocking…" (disabled) for 0.5 to 0.6 s, then "Unlock" with `role=alert` "Wrong master password."; password field length 0; one `vault_unlock` IPC each. Six consecutive wrong attempts with no delay or lockout message; the correct password then unlocked (V09) | PASS |
| V08b (AC-08 focus) | Check which element has focus after the wrong password | Field focused | After every wrong attempt `document.activeElement` was `BODY` (the button disables during the call and focus is not moved to the field). The field is empty but not focused | FAIL |
| V09 (AC-09/10) | Correct password via click and via Enter; Enter pressed three times quickly | Unlocks; button "Unlocking…" and disabled; repeated submits ignored | Button "Unlocking…" disabled from 8 ms to about 570 ms (observable); then "The vault is unlocked." on the Vault screen. Three Enter presses sent exactly one `vault_unlock` POST (plus one OPTIONS preflight). A first attempt with a key event lacking `text:"\r"` did not submit (my input issue, not an app defect) | PASS |
| V10 (AC-14/20) | Set timeout to 1 minute (click select, ArrowUp, Enter); `Page.reload` | Persists after reload; DB 1 | IPC `vault_set_auto_lock`; DB `auto_lock_minutes` 1. After `Page.reload` the select still showed "1 minute" (value 1), vault still unlocked. This was a webview reload only; true persistence across a process restart was seen in V18 | PASS |
| V11 (AC-16, countdown appears) | Click once on the heading (touch), then no input; sample the page every 100 ms | Countdown in the last 60 s ("Locks in 0:59" style) | The vault screen shows "Locking in 1:00. Press any key or click to stay unlocked." 106 ms after the touch (the touch response), "0:59" at 1.05 s, then one step per second to "0:01" at 59.67 s. With a 1-minute timeout the last-60-seconds window is the whole interval, so the first value is `1:00`, not `0:59`. (The `0:33` seen at 3 ms was the stale poll from before the touch. Right after `Page.reload` with no input the line read "Locking in 0:40".) Wording is "Locking in m:ss", not the spec's "Locks in". For longer timeouts (5, 15 min) the first-appearance point was not run live (unit tests only) | PASS |
| V11b (AC-16, in the app chrome) | Navigate to Home while unlocked on a 1-minute timeout; look for the countdown | Visible in the app chrome | The countdown exists only inside the Vault screen (`vault-screen.tsx`, `UnlockedView`). On Home for 80 s there was no countdown text anywhere in the page (sidebar and header included). The spec says app chrome | FAIL |
| V12 (AC-18/21) | Same run as V11, no input until lock | Lock with reason naming the duration | Locked at 60.69 s after the touch's pointerdown (sampled every 100 ms; the vault status poll is 1 s). Text: "Locked after 1 minute of inactivity." (singular "minute"; the spec says "5 minutes"-style with the configured value) | PASS |
| V13a (AC-15/17, mouse moves) | Unlock, move the mouse (`mouseMoved`) every 10 s for 90 s on the Vault screen | Must NOT lock | 7 moves sent (at 0, 10, 20, 30, 40, 50 and 60 s; the vault locked before the 70 s move). The vault locked at 60.72 s after the unlock: "Locked after 1 minute of inactivity." Mouse moves do not reset the timer; the app listens only for `pointerdown` and `keydown` (`useVaultInputTouch`) | FAIL |
| V13b (AC-15/17, clicks, then idle) | Unlock, click the heading every 10 s for 80 s (9 clicks), then stop | Does not lock while clicking; locks about 60 s after the last input | 9 clicks at 0 to 80 s: no lock and the countdown reset to 1:00 each time. After the last click (80.0 s) the vault locked at 140.9 s, 60.9 s after the last click | PASS |
| V14 (AC-14 Never, AC-19) | Unlock, set Never (ArrowUp, Enter), no input for 90 s | Stays unlocked | DB `auto_lock_minutes` 0. Idle 95.6 s: no countdown line, no lock; still "The vault is unlocked." | PASS |
| V15 (AC-22 storage) | Read the row | No password bytes; salt 16 bytes; verifier non-empty | Salt 16 bytes, verifier 72 bytes (non-empty), `m_cost` 19456, `t_cost` 2, `p_cost` 1. Password absent in UTF-8, UTF-16LE, UTF-16BE, base64, whole file (re-run at the end: still absent) | PASS |
| V16 (AC-25 tamper) | Copy DB to `%TEMP%\devshelf-e2e-f08\tamper.sqlite3`, flip one verifier byte there, point the app at it | Wrong master password; no hint of tampering | Copy made with the live file untouched (SHA before and after copy `eb592d75...e4a31`). One verifier byte flipped in the copy (72-byte verifier, 1 hex char differs). The app cannot be pointed at another file: the data folder comes from `app.path().app_data_dir()` (`lib.rs:24`) with no override, and changing the live DB is not allowed. The Rust unit tests cover the tampered-verifier case; not exercised against the real app | BLOCKED |
| V17 (AC-27) | Watch network use through setup, unlock, lock, auto-lock | No outbound call | CDP `Network` log (both launches): only `http://localhost:1420` (Vite dev assets) and `ipc.localhost` requests; no other host. A single end-of-run TCP snapshot for `devshelf.exe` and its WebView2 processes showed only `::1:1420` (Vite) and the loopback CDP port 9222. No continuous packet capture, so this is a sampled observation, not proof | PASS |
| V18 (AC-21 launch, AC-20 restart) | Stop the app (job controller), start it again, open Vault; unlock with the correct password | Locked at start with "Locked because DevShelf just started."; timeout still 1 | After restart (new `devshelf.exe`) Vault showed "Locked because DevShelf just started. / Master password / Unlock". The vault was unlocked in the previous run, so the restart lock was due to a new process. DB `auto_lock_minutes` still 1, `user_version` 9, 1 row. Correct password unlocked (V09 step with three Enter presses: one POST) and the status screen showed "Locking in 1:00" | PASS |
| V19 (cleanup) | Set the timeout to 5 minutes through the UI | DB 5 | ArrowDown, Enter on the select; select value 5; IPC `vault_set_auto_lock`; DB `auto_lock_minutes` 5, `user_version` 9, 1 row | PASS |
| VX1 (extra, AC-15) | Set 1 minute, click Home, click the page heading every 10 s for 80 s (7 clicks on Home), return to Vault | Activity anywhere in the window keeps the vault unlocked | The vault was locked when I came back: "Locked after 1 minute of inactivity." The input hook (`useVaultInputTouch`) is mounted only while the Vault screen is shown, so clicks and keys on other screens do not reset the timer, and nothing on those screens shows a countdown or the lock. The exact lock time was not sampled (about 60 s after the Home click) | FAIL |

Counts: PASS 19, FAIL 4, BLOCKED 1 (24 rows).

- PASS: V01, V02, V03, V03b, V04, V05, V06, V07, V08, V09, V10, V11, V12, V13b, V14, V15, V17, V18, V19.
- FAIL: V08b, V11b, V13a, VX1.
- BLOCKED: V16.

## Failures

1. V08b (AC-08, "field cleared and focused"): cleared, but focus ends on `BODY`. `LockView` disables the button during the call and has no `ref`/`focus()` on the password input after an error.
2. V11b (AC-16, countdown "visible in the app chrome"): the countdown is rendered only by the Vault status view. It is not in the sidebar or header, and it reads "Locking in 1:00. Press any key or click to stay unlocked." instead of "Locks in 0:59".
3. V13a (AC-15, "no pointer ... input"): mouse movement is not counted. Only `pointerdown` and `keydown` reset the timer. If "pointer input" means movement as well, the vault locks while the user is moving the mouse. Clicks and keys do reset it (V13b), and the reset is throttled to once per 5 s (`TOUCH_THROTTLE_MS`), so a reset can be up to 5 s late.
4. VX1 (AC-15, whole-window inactivity): input outside the Vault screen is not counted, so an active user on Home, Notes etc. is locked after the timeout with no warning (and the countdown is not visible there, V11b).

## BLOCKED and observations

- V16 (AC-25): no safe way to point the app at a different DB file (no override; `app_data_dir()` only). Not run. A tampered copy was made only in `%TEMP%` (now removed).
- The 1-minute timeout makes the countdown start at 1:00 (whole interval); for 5 and 15 minutes the first-appearance point (at 1:00 left) was not measured live.
- Lock reasons: manual "Locked because you locked it.", inactivity "Locked after 1 minute of inactivity.", launch "Locked because DevShelf just started." The spec's AC-12 wording "You locked the vault." differs from the shipped manual text.
- Setup in the dev build took 576 ms with "Creating…"; unlock and wrong-password attempts took 0.5 to 0.62 s (unoptimized Argon2id, m=19456 KiB, t=2, p=1). A release build will be faster, so the "Unlocking…" window is shorter there.
- The status view polls `vault_status` once per second while unlocked; polling does not reset the timer.

## Timings (checks 11 to 14)

| Check | Measurement |
|---|---|
| 11 first countdown value / when | `1:00` at 106 ms after the touch (timeout 1 minute, so the window is the whole interval); `0:59` at 1.05 s; `0:01` at 59.67 s |
| 12 lock after no input | 60.69 s after the touch (sample step 100 ms, status poll 1 s); "Locked after 1 minute of inactivity." |
| 13a mouse moves every 10 s, 7 moves | locked at 60.72 s after unlock |
| 13b clicks every 10 s, 9 clicks to 80.0 s | no lock while clicking; locked at 140.9 s, 60.9 s after the last click |
| 14 Never, no input | 95.6 s, no countdown, no lock |

## Cleanup and final state

- Timeout set back to 5 minutes through the UI (V19). The app was left unlocked until I stopped it. I did not lock anything else and did not delete the vault.
- The vault remains set up in the real data folder, with the test password `E2E-Vault-Pass-2026`. Rolling back to `devshelf.sqlite3.f07-backup` (schema 8, SHA-256 `9827f23a089f6bb952cb1541bdfd492c590fecef5e155400751e2f1c3f75c28f`) would drop `vault_meta`.
- Final `vault_meta`: 1 row, `id` 1, salt 16 bytes, `m_cost` 19456, `t_cost` 2, `p_cost` 1, verifier 72 bytes, `auto_lock_minutes` 5, `created_at` 2026-10-13T13:03:52Z. Final `user_version` 9.
- Final live DB SHA-256 `b8fc24d7807ff1c1b6030c38d4389115314a8ee177cf99ba98e42050498f8613`.
- Processes: the two dev jobs were stopped through the job controller (wrapper PIDs 10756 and 12780). Afterwards no `devshelf.exe` or `cargo.exe` was running and ports 1420 and 9222 no longer listen.
- `%TEMP%\devshelf-e2e-f08` (logs, scripts, `tamper.sqlite3`) removed after the report was written; the dev-log findings above are the record.

## Re-check after UI fixes

Targeted re-run of the four FAIL rows (V08b, V11b, V13a, VX1) plus the manual lock copy and a persistence regression, in the built dev app (`npm run tauri dev`, real `target\debug\devshelf.exe`, WebView2 CDP on 9222). Real CDP input events only (`Input.dispatchMouseEvent` move/press/release, `Input.dispatchKeyEvent`, `Input.insertText` for typing); DB read-only through `bun:sqlite`. No source changes.

- Backup before launch: `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3.f08-recheck-backup`, SHA-256 `b8fc24d7807ff1c1b6030c38d4389115314a8ee177cf99ba98e42050498f8613` (identical to the live file at that time).
- The vault was already set up (timeout 5 at the start); test password `E2E-Vault-Pass-2026`.

| Row | Check | Evidence | Result |
|---|---|---|---|
| R1 (V08b, AC-08) | Wrong password on the lock screen, 3 times; check focus and field | Each attempt: "Unlocking…" then "Unlock", `role=alert` "Wrong master password." after 0.71, 0.81 and 0.81 s. After each: `document.activeElement` is the password `INPUT` (identical to `input[type=password]`), field length 0 | PASS |
| R2 (V11b, AC-16) | Timeout 1 minute (click select, ArrowUp, Enter; select value 1), click Home, no further input | `[data-testid="vault-countdown"]` in the app chrome on Home, text "Locks in 0:59" first seen 0.3 s after the Home click (1.6 s after unlock; the Home click is the last input, so the first poll missed the 1:00 value), then one step per second down to "Locks in 0:01" at 59.0 s; it then disappeared as the vault locked (the next lock screen text: "Locked after 1 minute of inactivity."). 291 samples at 200 ms. (In the R4 run the first value was "Locks in 1:00" at 0.3 s.) | PASS |
| R3 (V13a, AC-15) | Unlock, `mouseMoved` every 10 s, 10 moves (0 to 90 s), no clicks | No lock during the 90 s of moves. After the last move (90.1 s) the vault locked at 151.1 s after unlock, 60.9 s after the last move: "Locked after 1 minute of inactivity." | PASS |
| R4 (VX1, AC-15 scope) | Unlock, go Home, 9 inputs on Home every 10 s (alternating click on the heading and a key press `a`, 0 to 80.1 s) | No lock during the inputs (the countdown reset each time; "Locks in 1:00" at 0.3 s, "0:59" at 0.9 s, "0:58" at 1.9 s). After the last input (80.1 s) the countdown disappeared (lock) at 141.1 s, 60.9 s after the last input. On Vault: "Locked after 1 minute of inactivity." | PASS |
| R5 (copy, AC-12) | Vault screen, click Lock now | Lock screen text exactly "You locked the vault." | PASS |
| R6 (regression) | Unlock at timeout 1, Home, `Page.reload`; then Vault; set 5 minutes | Home before reload "Locks in 0:59"; after reload "Locks in 0:57" (vault still unlocked, countdown present). Vault: select value 1, countdown "Locks in 1:00" (touch reset). Then ArrowDown, Enter set 5 minutes: select 5; on Home the countdown is absent (timeout 5 min, more than 60 s left). DB `auto_lock_minutes` 5, `user_version` 9, salt 16 B, verifier 72 B | PASS |

Counts: PASS 6, FAIL 0, BLOCKED 0 (R1 to R6). Previous FAIL rows V08b, V11b, V13a and VX1 now pass.

Notes:
- The countdown wording is now "Locks in m:ss" (as in the spec) and renders in the app shell, not only on the Vault screen.
- Lock detection on Home (R2, R4) was via the countdown disappearing, since Home shows no lock text; the lock text was then read on the Vault screen. Resolution about 0.2 s plus the 1 s status poll.
- Final auto-lock timeout: 5 minutes. The vault remains set up in the live data folder (intentional).
- Final live DB SHA-256 `25d8f5d0a470a90c5f0cf4e0d754e8fcfe42c18153225289a930c696ff955ca5` (changed from the backup by the timeout toggles and unlock/lock state writes).
- Processes: I stopped only the dev job I started (wrapper PID 19864) and its leftover children (devshelf.exe 12320 with its WebView2 tree, vite node 372); cargo 12568 had already exited. No kill by image name. Ports 1420 and 9222 are no longer listening. Temporary scripts removed.
