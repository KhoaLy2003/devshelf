# F07 — Private project notes: Implementation Plan

- **Status**: `complete — soft gate passed`
- **Spec**: `docs/specs/project-notes/spec.md` (approved; D1 titles plaintext; D2 accept key-loss risk until F12; D3 1-second autosave; D4 preview deferred; D5 new note created immediately)
- **Branch**: `feature/project-notes`, cut from `feature/config-catalog` (commit `d9e97e4`)

**Goal:** Markdown notes with autosave and a visible save state, bodies encrypted with XChaCha20-Poly1305, the data key kept in Windows Credential Manager, notes stored only in the database.

**Architecture:** A `notes` Rust module owns crypto (`cipher` helpers), the key lifecycle (`KeyStore` trait: `CredentialKeyStore` in production, `MemoryKeyStore` in tests), persistence, and validation. Each body is encrypted with a fresh random 24-byte nonce and bound to its note id as associated data, so a ciphertext cannot be moved to another note. The React autosave logic is a pure reducer, tested in Vitest.

**Tech Stack (new runtime dependencies, versions verified on crates.io):**
- `keyring = "4"` (4.2.0; Rust 1.88 minimum; machine has 1.99.0). Windows Credential Manager backend feature as the crate documents.
- `chacha20poly1305 = "0.11"` (XChaCha20-Poly1305 type; enable the feature the crate needs for OS randomness, or use `getrandom` directly).
- `zeroize = "1"`.
- `getrandom = "0.3"` if the chosen random source needs it.
The agent MUST confirm the exact feature names with `cargo add` and report them. These are the first runtime crypto crates; record them in the session log.

## Global Constraints

- The note body, key bytes, and decrypted text never appear in logs, error text, or files (AC-34, AC-35).
- Tests never touch the real Credential Manager; they use `MemoryKeyStore`. Tests use temp dirs.
- Only `notes` calls the crypto; commands never see the key.
- Bound parameters everywhere; migration 8 numbered; migrations 1 to 7 untouched.
- One Args and one Result per command; codes snake_case.
- Cargo: no `unsafe`.

## Contracts

Rust `src-tauri/src/notes/mod.rs` (module split: `notes/crypto.rs`, `notes/keys.rs`, `notes/mod.rs`):

```rust
// keys.rs
pub trait KeyStore { fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError>; fn store(&self, key: &KeyBytes) -> Result<(), KeyStoreError>; }
pub struct KeyBytes(Zeroizing<[u8; 32]>);            // zeroizes on drop; no Debug output of bytes
pub struct CredentialKeyStore;                       // keyring::Entry::new("DevShelf", "notes-data-key")
pub struct MemoryKeyStore(std::sync::Mutex<Option<KeyBytes>>); // tests
pub enum KeyStoreError { Unavailable }

// crypto.rs
pub fn encrypt(key: &KeyBytes, note_id: i64, plaintext: &[u8]) -> Vec<u8>;       // returns nonce(24) || ciphertext+tag; AAD = note_id as 8 LE bytes
pub fn decrypt(key: &KeyBytes, note_id: i64, blob: &[u8]) -> Result<Vec<u8>, CryptoError>; // CryptoError::Invalid on any failure

// notes/mod.rs
pub const MAX_TITLE_CHARS: usize = 200;
pub const MAX_BODY_BYTES: usize = 1_000_000;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteCode { NoteNotFound, TitleTooLong, BodyTooLarge, ProjectNotFound, KeyMissing, KeyringUnavailable, StorageUnavailable }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteError { pub code: NoteCode }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSummary { pub id: i64, pub title: String, pub project_id: Option<i64>, pub project_name: Option<String>, pub updated_at: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note { pub summary: NoteSummary, pub body: String }

pub fn list(storage: &Storage) -> Result<Vec<NoteSummary>, NoteError>;           // no key needed; ORDER BY updated_at DESC, id DESC; project_name via load-once map from projects::list
pub fn get(storage: &Storage, keys: &dyn KeyStore, id: i64) -> Result<Note, NoteError>;
pub fn create(storage: &Storage, keys: &dyn KeyStore, project_id: Option<i64>) -> Result<Note, NoteError>;  // empty title, empty body; key created here if absent and no notes exist
pub fn save(storage: &Storage, keys: &dyn KeyStore, id: i64, title: &str, project_id: Option<i64>, body: &str) -> Result<NoteSummary, NoteError>;
pub fn delete(storage: &Storage, id: i64) -> Result<(), NoteError>;             // no key needed
```

Key lifecycle (in `get`, `create`, `save`): load the key. If none and the `notes` table has rows, return `KeyMissing` (never create a key over existing notes). If none and the table is empty, generate a random key with `getrandom` and store it; a store failure returns `KeyringUnavailable`. A `KeyStoreError::Unavailable` on load returns `KeyringUnavailable`.

`save`: validate title length (chars) and body bytes; encrypt body with the note id (for an existing row, the id is known); update `title`, `project_id`, `body_enc`, `updated_at` (UTC ISO string from `strftime`) in one UPDATE with bound parameters. `create`: insert a row with an empty encrypted body in one statement, read the id, then encrypt the empty body with that id and update it, inside one transaction.

Migration 8 (`db/migrations.rs`):

```sql
CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  project_id INTEGER,
  title TEXT NOT NULL,
  body_enc BLOB NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
-- project_id is validated by the notes module (projects::get); no foreign key enforcement.
```

Commands `src-tauri/src/commands/notes.rs` (one Args and one Result each; `#[tauri::command(async)]` for everything that uses the key store, because Credential Manager access can block):

| Command | Args | Result |
|---|---|---|
| `notes_list` | `NotesListArgs {}` | `NotesListResult { notes: Vec<NoteSummaryView>, error, detail }` |
| `notes_get` | `NoteGetArgs { id }` | `NoteResult { note: Option<NoteView>, error, detail }` |
| `notes_create` | `NoteCreateArgs { project_id: Option<i64> }` | `NoteResult` |
| `notes_save` | `NoteSaveArgs { id, title, project_id, body }` | `NoteSaveResult { summary: Option<NoteSummaryView>, error, detail }` |
| `notes_delete` | `NoteDeleteArgs { id }` | `NoteDeleteResult { error, detail }` |

`notes_save` locks storage only for the database work; the key load happens before the lock (it can block). `notes_get` and `notes_create` follow the same rule. The `KeyStore` instance is `CredentialKeyStore` in production and is held in managed state so tests can substitute it.

TypeScript `src/types/commands.ts`: `NoteCode` union (7 codes), `NoteSummaryView`, `NoteView`, and one Args and one Result per command. `NOTE_TITLE_MAX = 200`, `NOTE_BODY_MAX_BYTES = 1_000_000` mirror Rust.

## File Map

| File | Action | Task |
|---|---|---|
| `src-tauri/Cargo.toml` | Modify (keyring, chacha20poly1305, zeroize, getrandom as needed) | 1 |
| `src-tauri/src/db/migrations.rs` | Modify (migration 8) | 1 |
| `src-tauri/src/db/mod.rs` | Modify (migration count 8) | 1 |
| `src-tauri/src/notes/mod.rs`, `notes/crypto.rs`, `notes/keys.rs` | Replace placeholder / create | 1 |
| `src-tauri/src/commands/notes.rs` | Create | 2 |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Modify (module, managed `CredentialKeyStore`, handlers) | 2 |
| `src/types/commands.ts` | Modify | 3 |
| `src/lib/commands.ts` | Modify (note wrappers) | 3 |
| `src/lib/autosave.ts` | Create (pure reducer: idle, unsaved, saving, saved, error; `nextAction`) | 3 |
| `src/lib/note-form.ts` | Create (`formatNoteError`, `titleOrPlaceholder`, `saveStateLabel`, `filterNotes`) | 3 |
| `tests/autosave.test.ts`, `tests/note-form.test.ts` | Create | 3 |
| `src/screens/notes-screen.tsx` | Replace F01 placeholder | 4 |
| `src/screens/note-editor-screen.tsx` | Create | 4 |
| `src/routes.tsx` | Modify (`notes/:id`) | 4 |

## Parallel Groups

<!-- Parallel group A: tasks 1, 3 — disjoint files -->
<!-- Parallel group B: task 2 — depends on task 1 -->
<!-- Parallel group C: task 4 — depends on tasks 2 and 3 -->

---

### Task 1: Crypto, key store, migration 8, notes domain (Rust)

**Specialist:** task (Rust)

**Steps:**
1. Add the crates; `cargo build` must pass before tests. Report the exact versions and feature flags.
2. Tests first:
   - `crypto`: round trip; a different note id fails to decrypt (AAD binding); a flipped byte in the ciphertext fails; a truncated blob fails; two encryptions of the same plaintext produce different nonces (AC-30).
   - `keys`: `MemoryKeyStore` round trip; `KeyBytes` Debug does not print bytes.
   - `notes`: create with no key and no notes creates a key (store contains one); `get` returns the body after `save`; a body of `MAX_BODY_BYTES + 1` returns `BodyTooLarge`; a 201-char title returns `TitleTooLong`; project id unknown returns `ProjectNotFound`; `list` orders by updated_at desc.
   - Key missing: insert a note with a store that has a key, then use a store with no key and existing notes: `get`, `save` and `create` return `KeyMissing`, and the store remains empty (no new key created).
   - Keyring unavailable: a store whose load returns `Unavailable` gives `KeyringUnavailable` for get, save and create.
   - Tamper at the row level: copy one note's `body_enc` to another note's row and assert `get` returns `StorageUnavailable`-class decrypt failure mapped to a stable code (add `NoteCode::BodyUnreadable` for decrypt failure; add it to the Contracts and TS union).
   - Delete removes the row; `get` on it returns `NoteNotFound`.
   - Secrecy: the error types and `Debug` output for `Note`, `KeyBytes` and `NoteError` contain no body text and no key bytes (assert a known body string does not appear in `format!("{:?}", …)`).
   - Migration 8 reopen keeps rows.
3. Run `cargo test notes::` and `cargo test db::`; expect failure first (note if skipped).
4. Implement. Use `Zeroizing` for key material and drop intermediate buffers. Do not log anything.
5. Update the migration-count assertion to 8. Run both filters; all pass.

**Acceptance:** AC-30, AC-31, AC-32, AC-33, AC-34, AC-35, AC-10 (domain), AC-20 (domain save), AC-41, AC-42, AC-43. Constitution P1, P2, P4.

---

### Task 2: Commands and managed key store (Rust)

**Specialist:** task (Rust)

**Steps:**
1. `commands/notes.rs`: the five commands per the table. Each keyed command loads the key before locking (via the managed `CredentialKeyStore`), then locks for the database work. `notes_list` and `notes_delete` lock only.
2. Register `CredentialKeyStore` as managed state in `lib.rs` `setup`, next to `StorageState`.
3. Map `NoteError` to result fields; `NoteCode::BodyUnreadable` included.
4. Pure mapping tests: each `NoteCode` maps to its snake_case string; `KeyMissing` result carries no note body; the summary view contains no body field (assert via the struct's fields).
5. Register the five commands in `generate_handler!`; add `pub mod notes;` in `commands/mod.rs`.
6. Run `cargo test --lib` and `cargo build` from `src-tauri/`; no warnings in touched files.

**Acceptance:** Constitution P1 (no key or body to the webview beyond `notes_get`'s explicit result), P3; AC-40.

---

### Task 3: TypeScript contract, autosave reducer, note helpers

**Specialist:** frontend-developer

**Steps:**
1. `src/types/commands.ts`: `NoteCode` union (add `body_unreadable` to the seven codes), `NOTE_TITLE_MAX = 200`, `NOTE_BODY_MAX_BYTES = 1_000_000`, `NoteSummaryView`, `NoteView`, Args and Result types.
2. `src/lib/commands.ts`: `listNotes`, `getNote(id)`, `createNote(projectId)`, `saveNote(args)`, `deleteNote(id)`.
3. `src/lib/autosave.ts` (pure, no React):
   - State: `{ phase: "idle" | "unsaved" | "saving" | "saved" | "error", savedAt: string | null, pendingVersion: number, savedVersion: number }`.
   - Events: `edit` (bumps pendingVersion, phase unsaved), `saveStart`, `saveOk(savedAt, version)`, `saveFail`, `retry`.
   - `nextAction(state, now)`: returns `{ kind: "wait" } | { kind: "save" }`; save when phase is unsaved and at least 1000 ms passed since the last edit (AC-20), and never while phase is saving (AC-24). A newer edit during saving keeps phase unsaved and the reducer resolves so that a save follows the in-flight one.
   - `saveStateLabel(state)`: "Unsaved changes", "Saved at <time>" (time formatted by an injected formatter), "Couldn't save." for error (AC-21), "" for idle.
4. `src/lib/note-form.ts`: `formatNoteError(code)` exact copy: note_not_found "This note no longer exists."; title_too_long "Use at most 200 characters."; body_too_large "This note is too large to save (limit 1 MB)."; project_not_found "That project no longer exists. Choose General or another project."; key_missing "Notes can't be read. The key that protects them is missing from Windows Credential Manager."; keyring_unavailable "DevShelf can't reach Windows Credential Manager. Notes are unavailable until it can."; body_unreadable "This note can't be read. Its text may be damaged."; storage_unavailable "DevShelf cannot reach its data folder. Restart DevShelf and try again."; `titleOrPlaceholder(title)` returns "Untitled note" for blank; `filterNotes(notes, filter)` with filter "all", "general", or a project id; `noteProjectLabel`.
5. Tests first in `tests/autosave.test.ts` and `tests/note-form.test.ts`: edit then wait until 1000 ms then save; no save before 1000 ms; no save while saving; a newer edit during saving schedules a second save; retry from error returns to unsaved; saveOk clears error; label strings; each code's exact copy; title placeholder; filter behaviour.
6. Run both test files; `npx tsc --noEmit`; report.

**Acceptance:** AC-01, AC-02, AC-20 to AC-24 logic and copy, AC-30 copy for key errors, AC-10 title placeholder.

---

### Task 4: Notes screens

**Specialist:** frontend-developer
**Solution-space note:** behaviour and copy fixed by spec; layout follows the preview's Notes list and editor (without the preview toggle).
**Frontend dispatch contract:** `design_md_fallback` (`docs/DESIGN.md`, `docs/preview/index.html` Notes). Dark theme; Tailwind tokens.

**Steps:**
1. `notes-screen.tsx`: query `["notes"]` (`staleTime: 0`, `refetchOnMount: "always"`, the same pattern as Home and Config). Filter select (All notes, General, each project from `listProjects`). "New note" button calls `createNote(filterProjectId or null)` then navigates to `/notes/:id` (AC-03). Rows: title via `titleOrPlaceholder`, project name or "General", last edited (`lastUsedLabel`-style formatter from `home-form` or a local helper using the same en-GB format). Empty states AC-04 and AC-05.
2. `note-editor-screen.tsx`: query `["note", id]` (`getNote`, `staleTime: 0`, `refetchOnMount: "always"`). Title input (max 200 chars; over-limit is blocked inline with "Use at most 200 characters."), project select, monospace textarea for the body, back link to `/notes`, the label from AC-12 verbatim, and Delete with confirmation (AC-13).
   - Autosave: a `useReducer` over `autosave.ts`; a timer ticks every 250 ms while unsaved to call `nextAction`; on save call `saveNote`; on success update the `["note", id]` cache and `["notes"]`; on failure enter error. Retry calls save again (AC-22).
   - Navigating away (back link or route change) flushes pending changes first; if the flush fails, stay and show the error (AC-23). Use the router's `useBlocker` only if it is available in the installed react-router; otherwise flush in the back-link handler and mention the limitation in the report.
   - Indicator from `saveStateLabel`; Couldn't save shows Retry.
   - `key_missing`, `keyring_unavailable`, `body_unreadable` on load show the copy from `formatNoteError` in place of the editor; no note text is shown (AC-32).
3. `routes.tsx`: add `notes/:id` under the shell.
4. `npm run build` once; report.

**Acceptance:** AC-01 to AC-06, AC-10 to AC-13, AC-20 to AC-24, AC-30 (label), AC-32, AC-33 UI copy.

---

### Task 5: Verification

Run after all tasks: `npx prettier --write` on touched files, `npx tsc --noEmit`, `npm run build`, `npm run lint`, `npx vitest run`, `cargo test --lib`, `cargo build`.

## Self-review

- Spec coverage: AC-01 (T4, list order T1), AC-02 (T4), AC-03 (T4), AC-04/05/06 (T4), AC-10 to 13 (T4), AC-20 to 24 (T3, T4), AC-30 to 35 (T1, T2), AC-40 to 43 (T1, T2).
- Names: codes match across Rust and TS; `body_unreadable` added to both sides in this plan.
- Constitution: P1 (encryption, zeroize, no logs), P2 (only notes module touches key and DB), P3 (one Args/Result per command), P4 (tests listed, no real Credential Manager), P5 (no network, no project-folder writes).

## Notes

- AAD binding to the note id means a body cannot be copied to another note; tests in Task 1 cover this.
- `keyring` on Windows uses Credential Manager generic credentials; the entry is per user account, which is why D2 says a new Windows profile loses access.
