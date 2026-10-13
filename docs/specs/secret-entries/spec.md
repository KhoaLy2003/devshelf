# F09 Secret entries: spec

Status: APPROVED by user (Step 1 gate). Approved choices: types API key, Password, Token, Connection string, Other; maximum value 4096 bytes; reveal re-masks after 30 seconds of row idle; deleting or archiving a project keeps its secrets ("Project removed").
Source: docs/roadmap.md F09 (Must, M; depends on F08, F02). Constitution P1 (secrets stay in Rust), P4, P5. Spec mode: full (no secret designs exist).
Requires: F08 vault unlocked (AC-30 onward uses the unlocked state from F08).

## Scope

Store named secret entries inside the unlocked vault. Each entry has a label, a type, a project, and an environment. Values are masked by default, revealed on explicit request, and copied on request. F14 (clipboard auto-clear) is out of scope here.

## Screens

- Secrets list (locked vault redirects to the F08 lock screen).
- Secret editor (add and edit).
- Delete confirmation (inline or dialog).

## Acceptance criteria

### Add and edit

- **AC-01** The Secrets list has an "Add secret" action. It opens the editor empty.
- **AC-02** The editor fields are: Label (required), Type (required, one of the types below), Project (optional, one of registered projects or none), Environment (required, one of local, dev, staging, production), Value (required, masked input). **[DECIDE: secret types. Proposed: API key, Password, Token, Connection string, Other.]**
- **AC-03** Saving with an empty label shows "Enter a label." and saves nothing. Empty value shows "Enter a value." Empty type or environment shows "Choose a type." / "Choose an environment."
- **AC-04** Label is trimmed of leading and trailing spaces and must be unique within a project. Duplicate shows "A secret with this label already exists for this project." Labels are at most 120 characters (over-length shows "Use 120 characters or fewer.").
- **AC-05** Value is at most **[DECIDE: 4096]** bytes; longer shows "Value is too long."
- **AC-06** Editing an entry shows its label, type, project, and environment. The value is never pre-filled; the field shows "Leave blank to keep the current value." and a blank value keeps it.
- **AC-07** Saving an edit that changes only metadata does not change the stored value.
- **AC-08** The value is never shown in a toast, error text, log, or the webview outside an explicit reveal or copy.

### List and filter

- **AC-09** The list shows label, type, project name (or "No project"), environment, and the masked value "••••••••" for every entry.
- **AC-10** The list filters by project and by type, with a "Clear filters" control; filters can combine. No matches shows "No secrets match these filters."
- **AC-11** The list is ordered by label, then by environment, case-insensitively.
- **AC-12** A project deleted or archived (F15) keeps its secrets; their project shows as "Project removed" until reassigned. **[DECIDE: confirm this, or block project deletion while secrets exist.]**

### Reveal

- **AC-13** Reveal is a per-entry action. The value appears in place of the mask, with a Hide action and an automatic re-mask after **[DECIDE: 30 seconds]** of no interaction with that row.
- **AC-14** Revealing one entry does not reveal others. Only one entry may be revealed at a time; revealing another re-masks the first.
- **AC-15** Locking the vault (manual or auto) re-masks any revealed value immediately (the vault lock event from F08 applies).
- **AC-16** Reveal is served by Rust on each request. The webview never receives all values at list time.

### Copy

- **AC-17** Copy is a one-click action per entry. The value goes to the clipboard through Rust. A neutral confirmation "Copied." appears; it never shows the value or part of it.
- **AC-18** Copy does not change the list, does not reveal the value, and does not add the value to any history.
- **AC-19** Copy is available only while the vault is unlocked (AC-30).

### Delete

- **AC-20** Delete asks "Delete this secret? This cannot be undone." with Cancel and Delete. Cancel keeps the entry.
- **AC-21** Confirmed delete removes the row; the list updates without a page reload.

### Storage and safety

- **AC-22** Each secret value is encrypted with XChaCha20-Poly1305 using a fresh random nonce per write, under the vault key derived in F08. The label, type, project, and environment are stored in plaintext columns so the list and filters work without decrypting every value.
- **AC-23** The encryption key is the F08 vault key only; no separate key store is created.
- **AC-24** Secret values are bound to the entry id as associated data, so a ciphertext moved to another row fails to decrypt.
- **AC-25** Decrypt failure shows "This secret can't be read. It may be damaged." and does not reveal any partial value.
- **AC-26** Plaintext secret values exist only in Rust memory during reveal or copy and are zeroized after use. No secret value is written to logs or to SQLite.
- **AC-27** Commands are typed, with one Args and one Result per command, user-facing codes only, and TS types in `src/types/commands.ts`.
- **AC-28** Tests use temporary directories and make no network calls (P5).
- **AC-29** A wrong vault key or a tampered ciphertext fails decryption (P4 test requirement).

### Vault gating

- **AC-30** All secret commands require the vault to be unlocked. A locked vault returns a locked code and the UI shows the F08 lock screen with the reason from the lock.
- **AC-31** Opening the Secrets list while locked shows the lock screen, not an empty list.

## Edge cases

- Project is deleted while a secret references it: AC-12.
- Two entries with the same label in different projects are allowed; same label with no project and another with no project is a duplicate (AC-04).
- Editing the label to a value that already exists for the same project shows AC-04's message and keeps the stored label.
- Copy while the vault auto-locks during the copy call: the copy completes or fails with a locked code; no partial value is written anywhere.
- Reveal timer expires while the user is editing the same row: the editor is not affected; only the list row re-masks.
- Very long values (up to AC-05 limit) are handled without truncation in the editor.

## Non-goals

- Secret version history and expiry reminders (roadmap Won't).
- Injecting secrets into processes or environment variables (roadmap Won't).
- Re-authentication before reveal or copy (roadmap Won't for this round).
- Clipboard auto-clear (F14).
- Importing from or exporting to other password managers.
- Secret generation (random password generator).
- Using the vault master password for notes (F07 stays independent).

## Constitution check

- P1: plaintext values exist only in Rust memory and in reveal or copy responses (AC-16, AC-26). Reveal returns one value on request, which the constitution permits.
- P2: secrets are a domain module; no other module reads secret tables. Project name lookup goes through the projects module, not SQL joins.
- P4: decrypt failure, wrong key, and tampered ciphertext tests required (AC-24, AC-29).
- P5: no network (AC-28).
- Dark theme and English copy apply to all screens.

## Open decisions for approval

1. Secret types: API key, Password, Token, Connection string, Other (AC-02).
2. Maximum value size 4096 bytes (AC-05).
3. Reveal auto-re-mask after 30 seconds of no interaction with the row (AC-13).
4. Project deletion with secrets: keep secrets with "Project removed" (AC-12).
5. Secret generation and re-authentication stay out of scope (non-goals).
