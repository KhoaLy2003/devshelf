# F08 Vault access control: spec

Status: APPROVED by user (Step 1 gate). The **[DECIDE]** markers below record the approved choices: 12-character minimum; auto-lock options 1, 5, 15 minutes and Never (default 5); inactivity = no input in the DevShelf window; 60-second countdown in app chrome; no lockout after failures; notes stay independent.
Source: docs/roadmap.md F08 (Must, M; depends on F01). Constitution P1, P4, P5. Spec mode: full (no designs exist for the vault).

## Scope

The vault is a single master-password gate in front of secret storage. F08 creates the gate. It does not store secrets; F09 does that.

## Screens

- Vault setup (first use): create master password.
- Vault lock screen: unlock, shows why the vault is locked.
- Vault status and settings: lock now, auto-lock timeout, countdown.

## Acceptance criteria

### Setup

- **AC-01** On first open of the Vault screen when no master password exists, the app shows the setup form, not the lock screen.
- **AC-02** Setup asks for a master password and a confirmation. Both fields must match. A mismatch shows the inline error "Passwords don't match." and creates nothing.
- **AC-03** A password shorter than **[DECIDE: 12]** characters is rejected with the inline error "Use at least 12 characters." No other composition rules apply (no required symbols or digits).
- **AC-04** Setup states, before submit, that there is no recovery: "If you forget this password, the vault cannot be opened. There is no reset." The user must tick an acknowledgement checkbox to enable submit.
- **AC-05** On successful setup the vault is created unlocked. Setup stores only a salt, cost parameters, and a password verifier. The password itself is never stored or logged.
- **AC-06** Setup, when a vault already exists, is not offered again. The Vault screen shows the lock screen instead.

### Unlock

- **AC-07** An empty password submit shows the inline error "Enter your master password." and does not call the backend unlock.
- **AC-08** A wrong password shows the inline error "Wrong master password." The password field is cleared and focused. Nothing else changes.
- **AC-09** A correct password unlocks the vault and returns the user to the screen they came from, or to the Vault status screen if none.
- **AC-10** While unlock is computing, the submit button is disabled and shows a progress label "Unlocking…". Repeated submits are ignored.
- **AC-11** Unlock state is held only in Rust memory. The webview never receives the derived key or the password after submit.

### Manual lock

- **AC-12** The Vault status screen has a "Lock now" action. It locks immediately, clears unlocked key material from Rust memory, and shows the lock screen with the reason "You locked the vault."
- **AC-13** Locking while a secret view or copy is open closes it (F09 consumers must observe the lock event; F08 defines the event only).

### Auto-lock

- **AC-14** Auto-lock timeout is a setting with options: 1 minute, 5 minutes, 15 minutes, and Never. **[DECIDE: confirm these four options. The roadmap wording "1, 5, 15 minutes, never while open" is ambiguous. Proposed reading: "Never" is the fourth option and disables auto-lock.]** Default: 5 minutes.
- **AC-15** Inactivity means no pointer or keyboard input inside the DevShelf window. **[DECIDE: confirm. Alternative: any backend command resets the timer.]** Background or minimized windows count as inactive.
- **AC-16** Within the last 60 seconds before auto-lock, the vault status is visible in the app chrome as a countdown "Locks in 0:59". **[DECIDE: confirm 60 seconds and visibility in the app chrome.]**
- **AC-17** Any input resets the countdown and hides it again.
- **AC-18** When the timeout elapses, the vault locks, clears unlocked key material, and shows the lock screen with the reason "Locked after 5 minutes of inactivity." (the actual configured duration).
- **AC-19** The auto-lock timer never runs while the vault is already locked, and the Never setting never locks.
- **AC-20** Changing the timeout takes effect immediately. The new value persists across restarts.

### Lock screen

- **AC-21** The lock screen always states why the vault is locked: first launch with a vault, manual lock, auto-lock, or app restart. The reason is shown above the password field.

### Storage and safety

- **AC-22** The vault verifier, salt, and cost parameters are stored in SQLite through a numbered migration. Passwords, derived keys, and verifier plaintext are never written to SQLite, logs, files, or the webview.
- **AC-23** Derived key material is held in types that zeroize on drop. Locking and auto-lock drop it.
- **AC-24** Argon2id cost parameters are defined in one Rust module. Unlock and setup use that module with no per-call override. A test-only constructor with lower cost is allowed.
- **AC-25** A tampered verifier or salt causes unlock to fail with the same "Wrong master password." message. The error must not reveal whether the verifier was tampered with.
- **AC-26** Failed unlock attempts are not limited by lockout. Brute-force cost is controlled only by Argon2id. **[DECIDE: confirm, or add a short delay after repeated failures.]**
- **AC-27** The app makes no network call during setup, unlock, lock, or auto-lock (P5).

### Changing the password

- **AC-28** Changing the master password is out of scope for F08. The Vault status screen does not offer it.

## Edge cases

- Unlock or setup is interrupted by an app restart: the vault stays locked on next launch; no partial state is kept.
- Two windows are not supported; the vault state is single-instance per app process.
- Clock changes: the inactivity timer uses a monotonic clock, not wall time.
- Timeout set to 1 minute and the user is in the middle of a long copy (F09): the lock still happens; F09 handles its own cleanup.
- Passwords are never trimmed. Only a zero-length password counts as empty (AC-07). A whitespace-only password is accepted as typed at setup if it meets the 12-character rule, and it unlocks the vault when it matches. This rule was clarified in implementation (the earlier wording contradicted itself).

## Non-goals

- Master password change or recovery (AC-28, AC-04).
- Shared password with notes. Notes keep their own Credential Manager key (F07). **[DECIDE: confirm. The roadmap flags this as a Step 6 decision. Keeping notes independent avoids reopening F07.]**
- Biometric or Windows Hello unlock.
- Per-secret re-authentication before reveal or copy (idea brief open question, not MVP).
- Lockout after repeated failures, unless AC-26 is changed.
- Storing the vault on a server or syncing it.

## Constitution check

- P1: no plaintext password or derived key crosses to the webview; zeroize on lock and auto-lock (AC-11, AC-23).
- P2: vault commands are typed; the vault is a separate module; no other module reads vault tables.
- P4: tests must include wrong-password (AC-08, AC-25) and tampered-ciphertext/verifier cases; Argon2 cost lives in one module (AC-24).
- P5: no network (AC-27).
- Dark theme and English copy apply to all screens.

## Open decisions for approval

1. Minimum password length 12 (AC-03).
2. Auto-lock options and the meaning of "never while open" (AC-14).
3. Inactivity definition (AC-15).
4. Countdown window and placement (AC-16).
5. No lockout after failures (AC-26).
6. Notes remain independent of the vault (non-goals).
