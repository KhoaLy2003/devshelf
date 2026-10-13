# F07 — Private project notes: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F07 (Must, size M, depends on F01 and F02 — both shipped locally)
- **Feature slug**: `project-notes`
- **Spec mode**: Full. No designs in `docs/designs/project-notes/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` and `docs/preview/index.html` (Notes list and Note editor), `design_md_fallback`.

---

## 1. Goal

The user writes Markdown notes, optionally linked to a project, and sees whether each change is saved. Notes live in DevShelf's local database, never in a project folder, and their text is encrypted at rest. The screen says so. Notes work whether or not the vault is locked (architecture section 4).

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| N1 | Notes list | `/notes` | Replaces the F01 empty state when notes exist; filter by project or General |
| N2 | Note editor | `/notes/:id` | Title, project selector, Markdown text area, autosave indicator, back link |
| N3 | New note | creates a note and opens N2 | Blank title; first line placeholder |

The preview's side-by-side Markdown preview is not built in F07 (D4).

## 3. Acceptance Criteria

### Notes list

- **AC-01** N1 lists notes sorted by last edited (most recent first). Each row shows title (or "Untitled note" when blank), project name or "General", and last edited date and time.
- **AC-02** A filter selector offers "All notes", "General" (no project), and each registered project. Filtering is applied to the loaded list.
- **AC-03** "New note" creates a note immediately (untitled, empty body, optional project from the filter when a project is selected) and opens N2 for it.
- **AC-04** With no notes: "No notes yet" (F01 copy) with a "New note" button.
- **AC-05** With notes but none matching the filter: "No notes match this filter." with a "Show all notes" action.
- **AC-06** Clicking a row opens N2.

### Note editor

- **AC-10** N2 has: Title field (max 200 characters; blank allowed and shown as "Untitled note" in lists), Project selector ("General" or any registered project), Markdown text area (plain textarea, monospace), and a back link to N1.
- **AC-11** Title and project changes and body changes are saved automatically (AC-20).
- **AC-12** A label on N2 reads: "Stored in DevShelf on this computer, outside project folders. Note text is encrypted." (Exact wording fixed here.)
- **AC-13** Deleting a note: a Delete control asks "Delete this note? This cannot be undone." Confirm removes it and returns to N1. Cancel keeps it.

### Autosave

- **AC-20** After the user stops typing for 1 second, the note is saved. Changes to title or project are saved within the same delay.
- **AC-21** An indicator shows one of three states:
  - "Unsaved changes" while there are edits not yet saved (including during the 1-second wait).
  - "Saved at <time>" after a successful save.
  - "Couldn't save." with a "Retry" button when a save fails. The text the user typed is kept in the editor; nothing is discarded.
- **AC-22** Retry saves the current content again. A later successful save clears the error state.
- **AC-23** Navigating away while there are unsaved changes saves them first; if that save fails, the user stays on N2 and sees "Couldn't save." with Retry.
- **AC-24** Only one save is in flight at a time. A newer edit made while a save is in flight is saved after it completes.

### Encryption and storage

- **AC-30** Note bodies are encrypted with XChaCha20-Poly1305 using a fresh random 24-byte nonce for every write (constitution P1). Titles are stored plaintext (D1).
- **AC-31** The encryption key is a random 256-bit key created on first use. It is stored in the Windows Credential Manager through the `keyring` crate, under service `DevShelf` and user `notes-data-key` (architecture section 2 and 4).
- **AC-32** If notes exist but the key is missing from the Credential Manager, the app does not create a new key. Note bodies show "Notes can't be read. The key that protects them is missing from Windows Credential Manager." with no note text shown. The list still shows titles, project, and dates. No note is deleted or rewritten.
- **AC-33** If the Credential Manager cannot be reached, notes cannot be read or saved; the message is "DevShelf can't reach Windows Credential Manager. Notes are unavailable until it can."
- **AC-34** The key bytes are held in memory only inside the notes module, wrapped in a type that zeroizes on drop. The key is never sent to the webview or written to any file or log (constitution P1).
- **AC-35** The note body is never written to the log, to the error text, or to a file.

### Boundaries

- **AC-40** Only Rust touches the database and the Credential Manager. The React layer receives note text only through typed commands.
- **AC-41** Migration 8 adds the `notes` table. Bound parameters everywhere.
- **AC-42** Notes are never written inside a project folder (constitution P5).
- **AC-43** Deleting a project does not delete its notes; they show "No project" (same rule as F04 AC-22). Deletion of a project is F15.

## 4. Edge Cases

- A blank title: the list shows "Untitled note".
- A 200-character title: accepted; a 201-character title is rejected inline ("Use at most 200 characters.").
- A body of 1 MB or more: rejected with "This note is too large to save (limit 1 MB)." so the database stays small.
- Two windows editing the same note: not supported (single window, D5 of the app); the last save wins.
- Saving while the Credential Manager is locked by a prompt: treated as "Couldn't save." with Retry.
- A note whose project is changed to General: list filter updates on next load.
- An empty body on an untitled note: kept as an empty note, not deleted.

## 5. Non-goals (F07)

- Markdown preview (D4).
- Searching note text (F11 quick open searches titles only; full-text search is not in the architecture).
- Note history, versions, or undo beyond the text area's own undo.
- Export or backup of notes (F12).
- Key rotation or key export.
- Attachments or images.
- Tags on notes.

## 6. Decisions Needing Approval

- **D1 — Title storage.** (a) **Title plaintext; body encrypted.** (b) Title encrypted too; quick open (F11) must then decrypt titles, which needs the key for every search. *Recommended: (a). Architecture section 4 keeps labels and names plaintext, and quick open stays simple. The body is what the roadmap asks to protect.*
- **D2 — Key loss.** Losing the key makes notes unreadable permanently, and no backup exists until F12. (a) **Accept: key is kept only in Windows Credential Manager; AC-32 says what the user sees; no silent regeneration.** (b) Also write a recovery file to the data folder (adds a second copy of the key on disk, weakening protection). *Recommended: (a). A second key copy contradicts the at-rest design; F12 backup is the recovery path.* Please confirm you accept that losing the Windows user profile loses notes until F12 ships.
- **D3 — Autosave delay.** (a) **1 second after the last keystroke (AC-20).** (b) Save on blur only. *Recommended: (a), as in the preview.*
- **D4 — Markdown preview.** (a) **Defer to a later feature; F07 is text only.** (b) Add a Markdown renderer dependency now (needs a constitution amendment for the stack list and a sanitiser for untrusted HTML). *Recommended: (a). The roadmap marks the preview "optional", and a renderer plus sanitiser is a new runtime dependency with its own risk.*
- **D5 — New note placement.** (a) **Created immediately as "Untitled note" and opened (AC-03).** (b) Ask for a title first. *Recommended: (a), so typing is never blocked.*

## 7. Constitution Check

- **P1** Note fields encrypted with XChaCha20-Poly1305, fresh nonce per write; key zeroized; never logged or sent to the webview (AC-30, AC-34, AC-35). Pass.
- **P2** Notes logic in the `notes` module; project existence through `projects::get`; migration 8 numbered; bound parameters. Pass.
- **P3** One Args and one Result per command; codes: `note_not_found`, `title_too_long`, `body_too_large`, `project_not_found`, `key_missing`, `keyring_unavailable`, `storage_unavailable`. UI maps copy. Pass.
- **P4** Rust tests with an in-memory key store (never the real Credential Manager): encryption round trip; a wrong key fails to decrypt (tampering test: flip a byte of the ciphertext and assert decrypt fails); each save uses a new nonce (two saves of the same body yield different nonces); key missing with existing notes yields key_missing and does not create a key; key store unavailable yields keyring_unavailable; titles and project filters; migration 8 reopen. Vitest for autosave state transitions, the title length rule and copy. Built-app smoke check before close. Pass (planned).
- **P5** No network. The Credential Manager is local. Pass.

**Dependency note (architecture section 8):** the crypto crates were named but not verified. Current versions: `keyring` 4.2.0 (requires Rust 1.88 or later; this machine has 1.99.0) and `chacha20poly1305` 0.11.x (provides XChaCha20-Poly1305). `zeroize` is added alongside. This is the first use of these crates; the plan records the exact versions after build.

## 8. Open Questions

None beyond Section 6.
