# F09 Secret entries: plan

Spec: docs/specs/secret-entries/spec.md (approved). Branch: `feature/secret-entries`, cut from `84b66e3` (F08).

## Design

**Storage** (migration 10, module `secrets/`)
- Table `secrets`: `id INTEGER PRIMARY KEY`, `label TEXT NOT NULL`, `secret_type TEXT NOT NULL CHECK (secret_type IN ('api_key','password','token','connection_string','other'))`, `project_id INTEGER NULL` (no foreign key: deleting a project must keep the row so the list can show "Project removed", AC-12), `environment TEXT NOT NULL CHECK (environment IN ('local','dev','staging','production'))`, `body_enc BLOB NOT NULL`, `created_at`, `updated_at`.
- Uniqueness (AC-04): unique index on `(COALESCE(project_id, -1), label COLLATE NOCASE)`. Case-insensitive, matching the note title policy. Trimmed before save.
- Only the `secrets` module touches this table (P2). Project names come from the projects module.

**Encryption** (AC-22 to AC-24, AC-29)
- Value ciphertext = 24-byte random nonce || XChaCha20-Poly1305 output; key = the F08 vault key, read through a new `vault::with_key(|key| ...)` accessor that fails with a locked error when locked. Associated data = the row id as 8 little-endian bytes, so a moved ciphertext fails.
- Value is encrypted before the UPDATE/INSERT statement is built; a failed encrypt returns `secrets_storage_unavailable` and writes nothing.
- Decrypt failure maps to `secrets_unreadable` (AC-25).

**Reveal and copy** (AC-13 to AC-19, AC-26)
- `secrets_reveal` returns one value for one id. The webview keeps at most one revealed row (AC-14); the UI timer re-masks after 30 s of row idle and on any lock (AC-15). The re-mask is UI-side; Rust returns a value only on request.
- `secrets_copy` decrypts in Rust, writes to the clipboard in Rust, and returns only a confirmation. The clipboard crate is chosen and verified by the Rust slice (the Tauri clipboard plugin is alpha; prefer a stable crate such as `arboard` if its version and maintenance check out, and report the choice). Copy never returns the value to the webview.
- Plaintext buffers are `Zeroizing` in Rust. The IPC response for reveal carries the value as a String; that copy cannot be scrubbed (same limitation as the F08 password). Recorded as a limitation.

**Commands** (`commands/secrets.rs`; one Args and one Result per command; codes only)
- `secrets_list` `{ project_id: number|null|undefined, secret_type: string|null }` → `{ items: SecretSummary[] }` with masked metadata only. The list is ordered by label then environment, case-insensitive (AC-11). Project filter uses `project_id`; "No project" is an explicit filter value (use `filter_project: "any" | "none" | number` as separate fields if simpler; the Rust slice decides and the contract below is authoritative).
- `secrets_create` `{ label, secret_type, project_id, environment, value }`.
- `secrets_update` `{ id, label, secret_type, project_id, environment, value: string|null }`; `value: null` keeps the stored value (AC-06, AC-07).
- `secrets_reveal` `{ id }` → `{ value }`.
- `secrets_copy` `{ id }` → `{}` plus status; confirmation only.
- `secrets_delete` `{ id }`.
- Each Result carries `error: SecretCode | null` and `detail: string | null`, the same pattern as notes.
- Error codes: `secrets_label_required`, `secrets_value_required`, `secrets_type_required`, `secrets_environment_required`, `secrets_label_too_long`, `secrets_value_too_long`, `secrets_duplicate_label`, `secrets_not_found`, `secrets_unreadable`, `secrets_locked`, `secrets_storage_unavailable`, `secrets_clipboard_unavailable`.
- Locked vault: every command first checks the vault session and returns `secrets_locked` (AC-30).

**Project name display**: the list resolves project names via the projects module (one call that returns the registered projects), not a SQL join. Missing project → "Project removed" (AC-12).

**UI** (`src/screens/secrets-screen.tsx`, `src/screens/secret-editor-screen.tsx`, routes `/secrets`, `/secrets/new`, `/secrets/:id`; nav entry "Secrets")
- Locked state: the shell's vault status gates the screen; when locked the lock screen shows (AC-31).
- Pure helpers in `src/lib/secret-form.ts`: `validateSecretForm(input, isEdit)` returns the first inline error (AC-03, AC-04 length, AC-05 size is checked by Rust but the UI can pre-check the byte count); `maskedValue()` returns "••••••••"; `revealTimerExpired(lastInteractionMs, nowMs)`; `sortSecrets(items)` for AC-11 when the list is re-sorted client-side (optional; Rust orders).
- Reveal state is one `{ id, expiresAt }` in React state, cleared on lock and on timer.
- Copy: call `secretsCopy`, show "Copied." for 2 s; never display the value.
- Delete confirmation uses the exact copy in AC-20.

## Tests

Rust (`cargo test`, temp dirs, low-cost KDF constructor for vault setup):
- Create then reveal returns the value; list never includes the value (AC-09, AC-16).
- Wrong vault key fails decrypt with `secrets_unreadable`; tampered ciphertext fails (AC-25, AC-29).
- Ciphertext moved to another row fails (AC-24).
- Duplicate label per project, case-insensitive; same label allowed across projects (AC-04, edge cases).
- Label, value, and length limits (AC-03, AC-04, AC-05).
- Update with `value: null` keeps the ciphertext byte-identical (AC-06, AC-07).
- Every command returns `secrets_locked` when locked (AC-30).
- Copy writes to the clipboard path through an injectable clipboard trait so the test does not touch the real clipboard; the result contains no value (AC-17, AC-18).
- Migration 10 applies; the DB contains no plaintext value bytes (AC-08, AC-22).

Vitest:
- `validateSecretForm` branches (AC-02, AC-03, AC-04, AC-06 blank value on edit).
- `maskedValue` constant output; `revealTimerExpired` boundary at 30 s.

## Work slices

- **Rust (one agent):** `secrets/` module, `vault::with_key` accessor, migration 10, `commands/secrets.rs`, clipboard abstraction, `lib.rs` registration, cargo tests. Owns all Rust files.
- **UI (one agent):** TS contract (written by the orchestrator now), commands wrappers, `secret-form.ts`, screens and routes, nav entry, Vitest tests. Owns all TS/TSX files.

## Verification

- `npx tsc --noEmit`, `npm run build`, `npm run lint` (0 errors), `npx vitest run`, `cargo test --lib`, `cargo build` (zero warnings in touched files).
- E2E (Step 5, dev mode): add, edit with blank value, duplicate label, reveal and re-mask at 30 s, one-at-a-time reveal, lock re-masks, copy confirmation without value, delete confirmation, project removed display, filters, restart and locked state, DB holds no plaintext.
- Built-app smoke remains a phase-close item (P4).

## Risks

- Clipboard crate choice: alpha plugin avoided in favour of a stable crate; the Rust slice must verify the crate version and report.
- Project deletion does not exist until F15, so "Project removed" can be tested only through a direct DB change in the E2E (report as such).
- Reveal values traverse IPC as strings (limitation, same as F08).
