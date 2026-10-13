# F08 Vault access control: plan

Spec: docs/specs/vault-access-control/spec.md (approved). Branch: `feature/vault-access-control`, cut from `21ea7ff` (F07).

## Design

**Key derivation and verifier**
- Argon2id via crate `argon2` 0.6.0 (RustCrypto; MIT/Apache-2.0). Parameters live in one module (`vault/kdf.rs`): OWASP minimum m=19456 KiB, t=2, p=1. Test-only constructor takes m=8 KiB, t=1, p=1. Benchmark unlock on this machine during implementation; if the production value takes over ~2 s, record the measured time in the plan review and do not change the parameters without a decision.
- Salt: 16 random bytes from `getrandom`, generated once at setup.
- Verifier: XChaCha20-Poly1305 encryption of a fixed 32-byte constant under the derived key, with a fresh random 24-byte nonce. Unlock decrypts it; authentication failure means wrong password or tampered verifier/salt. Both return the same `vault_wrong_password` (AC-08, AC-25). No password hash is stored.
- The 32-byte derived key is the vault key. It is wrapped in `zeroize::Zeroizing` and dropped on lock.

**Session state**
- `VaultSession` (Rust, managed by Tauri): `Mutex<Inner>` with `key: Option<Zeroizing<[u8;32]>>`, `last_input: Instant` (monotonic), `auto_lock: AutoLock` (Off, or 1/5/15 min), `last_reason: LockReason`.
- Lock reasons: `Launch` (initial state after process start), `Manual`, `Inactivity { minutes }`. Stored in memory only, so a restart resets to `Launch`, matching AC-21.
- Expiry is enforced lazily: every protected accessor checks `last_input.elapsed() > timeout` and locks before returning. A background thread started in `lib.rs` also runs once per second and performs the same check, so a locked vault does not depend on UI calls.
- KDF work runs outside the mutex: unlock derives the key with no lock held, then takes the lock only to store it.

**Inactivity**
- The webview calls `vault_touch` on pointer and key events, throttled to one call every 5 seconds. Only input in the window resets `last_input` (AC-15, approved input-based). Minimized windows get no events and time out.

**Storage**
- Migration 9 creates `vault_meta` with a single-row check `id = 1`: `salt BLOB`, `m_cost INTEGER`, `t_cost INTEGER`, `p_cost INTEGER`, `verifier BLOB`, `auto_lock_minutes INTEGER NOT NULL DEFAULT 5 CHECK (auto_lock_minutes IN (0,1,5,15))` where 0 means Never, `created_at`. No plaintext password or key is stored (AC-22).
- Vault domain module reads and writes only its own table; no other module touches `vault_meta` (P2).

**Commands** (`commands/vault.rs`, each with one Args and one Result struct, user-facing codes only; TS types in `src/types/commands.ts`)
- `vault_status` → `{ state: "not_set_up" | "locked" | "unlocked", lock_reason: "launch"|"manual"|"inactivity"|null, auto_lock_minutes: 0|1|5|15, seconds_until_lock: number|null }`. `seconds_until_lock` is non-null only when unlocked, auto-lock is on, and remaining time ≤ 60 (AC-16).
- `vault_setup` `{ password, confirmation, acknowledged }` → status. Validates non-empty (`vault_password_required`), matching (`vault_password_mismatch`), length ≥ 12 chars (`vault_password_too_short`), acknowledged, and not already set up (`vault_already_set_up`). Creates the vault unlocked.
- `vault_unlock` `{ password }` → status. Empty → `vault_password_required`; no vault → `vault_not_set_up`; wrong → `vault_wrong_password`.
- `vault_lock` `{}` → status with reason `manual`.
- `vault_touch` `{}` → `{}` (no-op return struct, still one Result struct).
- `vault_set_auto_lock` `{ minutes }` → status. Accepts 0, 1, 5, 15 only; other → `vault_invalid_timeout`; persists (AC-20).
- Error codes: `vault_password_required`, `vault_password_mismatch`, `vault_password_too_short`, `vault_wrong_password`, `vault_not_set_up`, `vault_already_set_up`, `vault_invalid_timeout`, `vault_storage_unavailable`.

**Password handling**: the password arrives as a `String` argument (user input). The Rust side zeroizes its own copies after derivation; serde-owned buffers inside the command framework cannot be scrubbed. Documented as a limitation.

**UI** (`src/screens/vault-screen.tsx`, route `/vault`, sidebar entry "Vault")
- Three views from `vault_status.state`: setup form (AC-02 to AC-05), lock screen with reason line (AC-21) and password form (AC-07 to AC-10), status view with "Lock now", timeout select (Never/1/5/15), countdown line from `seconds_until_lock` (AC-16).
- Status is polled every 1 second while the screen is mounted and unlocked, plus `staleTime: 0` (per the F10 fix). Input listeners (pointerdown, keydown on window) call `vault_touch` throttled to 5 s; attached in one hook while the vault is unlocked.
- Pure helpers in `src/lib/vault-form.ts`: `validateSetup(password, confirmation, acknowledged)` returns the inline error code or null; `lockReasonText(reason, minutes)`; `formatCountdown(seconds)` → `0:59`. Vitest covers each branch.
- The webview never holds the derived key; passwords are cleared from React state after each submit.

## Tests

Rust (`cargo test`, temp dir per test, low-cost KDF constructor):
- Setup then unlock with correct password succeeds; wrong password fails with `vault_wrong_password` (AC-08).
- Tampered verifier fails with the same code; tampered salt fails with the same code (AC-25).
- Setup rejects short, mismatched, unacknowledged, and repeated setup (AC-02, AC-03, AC-04, AC-06).
- Empty password on unlock is `vault_password_required` and does not reach the KDF (AC-07).
- Lock drops the key: `key()` returns not-unlocked afterward (AC-12, AC-23).
- Auto-lock uses an injected clock: idle beyond the timeout locks with reason `inactivity`; touch resets; Never never locks; a locked vault stays locked (AC-14, AC-18, AC-19).
- Countdown: `seconds_until_lock` is None above 60 s and Some(≤60) inside the window (AC-16).
- Timeout persists across a reopened DB and rejects values outside {0,1,5,15} (AC-20).
- Migration 9 applies on a fresh DB; the DB row contains no password bytes (AC-22, AC-27).

Vitest:
- `validateSetup` branches (AC-02, AC-03, AC-04).
- `lockReasonText` for each reason (AC-21).
- `formatCountdown` boundaries (`0:59`, `1:00`, `0:00`).

## Work slices

- **Rust (one agent):** `vault/` (kdf, verifier, session, mod), migration 9, `commands/vault.rs`, `lib.rs` registration and background thread, cargo tests, Argon2 benchmark note. Owns all Rust files.
- **UI (one agent, starts after the TS contract is written by the orchestrator):** `src/types/commands.ts` vault types, `src/lib/commands.ts` wrappers, `src/lib/vault-form.ts`, `vault-screen.tsx`, route and nav entry, input hook, Vitest tests. Owns all TS/TSX files.
- Contract for both slices: the command names, argument and result shapes, and error codes above are fixed. The orchestrator writes `src/types/commands.ts` first so both agents share it.

## Verification (Step 4 and 5)

- `npx tsc --noEmit`, `npm run build`, `npm run lint` (0 errors), `npx vitest run`, `cargo test --lib`, `cargo build` (zero warnings in touched files).
- E2E (Step 5, dev mode): setup, wrong password, unlock, manual lock, reason lines, timeout countdown (use the 1-minute option and watch the countdown), inactivity with no input, Never setting, restart shows Launch reason, DB row has no password bytes.
- Built-app smoke is still outstanding for all features (P4); record it as a gap unless the phase close run covers F08.

## Risks

- Argon2 cost on this machine may push unlock past comfortable latency; measured in implementation.
- Lazy plus thread expiry both lock on the same rule; a race could lock twice. Lock is idempotent, so this is acceptable.
- Password arguments pass through the IPC layer as plain strings; limitation recorded, not fixed.
