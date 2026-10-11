# F02 — Project registry: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F02 (Must, size M, depends on F01 — shipped)
- **Feature slug**: `project-registry`
- **Spec mode**: Full. No designs exist in `docs/designs/project-registry/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` (dark theme only) and `docs/preview/index.html` (Projects list, Register project dialog, project detail), `design_md_fallback`.

---

## 1. Goal

The user registers a project by pointing DevShelf at its root folder. DevShelf stores a reference (path, display name, type, tags) in the local database and lists projects with their drive, tags, and detail. The user can pin projects. Registering a project never copies, reads the contents of, or changes anything in the project folder.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| P1 | Projects list | `/projects` | Replaces the F01 empty state when projects exist |
| P2 | Register project dialog | modal on `/projects` | Opened from the list; Cancel/Esc closes it |
| P3 | Project detail | `/projects/:id` | Expands from a row on P1 (preview shows an expanded panel); route used so Back works |

Home (F10), config files (F06), notes (F07), and secrets (F09) are out of scope. P3 shows no counts for those features.

## 3. Acceptance Criteria

### Registration

- **AC-01** The dialog has four fields: root folder path, display name, type, tags. Type is one of: Spring Boot, React, Docker/Infra, Node, Other. Tags are optional.
- **AC-02** "Browse..." opens a native folder picker. A picked folder fills the path field. The path can also be typed.
- **AC-03** Register saves the project, closes the dialog, adds the row to P1, highlights it, and shows the message: "Project <display name> registered. No project files were copied or changed."
- **AC-04** Cancel and Esc close the dialog without saving. Values are discarded.
- **AC-05** A validation error keeps the values the user entered in every field.

### Validation (inline, next to the field)

- **AC-10** Path empty: "Enter or pick a project folder."
- **AC-11** Path does not exist, or is a file instead of a folder: "Folder `<path>` was not found." (Use "was not found" for both cases; the reason is shown as a separate line only when it is a file: "`<path>` is a file. Pick the project's root folder.")
- **AC-12** Path already registered (compared after normalisation, case-insensitive on Windows): "`<path>` is already registered as <existing display name>."
- **AC-13** Display name empty or only whitespace: "Enter a display name."
- **AC-14** Display name longer than 100 characters, or path longer than 260 characters: "Use at most 100 characters." / "This path is too long for Windows."
- **AC-15** Tags: comma-separated; each tag is trimmed, lowercased, and must be 1–32 characters of letters, digits, `-`, `_`, or `.`. An invalid tag shows: "Tag `<tag>` can use letters, digits, - _ and . only (max 32)." Duplicate tags are removed silently. At most 10 tags per project; more shows: "Use at most 10 tags."
- **AC-16** All validation runs on Register. The dialog does not block typing.

### Nested paths

- **AC-20** A path inside an already registered project's folder is allowed. The dialog shows an inline note: "This folder is inside <existing name>. Both projects are kept." (Decision D1.)
- **AC-21** A path that contains an already registered project's folder is allowed without a note.

### Listing

- **AC-30** P1 lists every project with: display name, type, root path, drive letter (for example `F:`), tags, and a pin indicator.
- **AC-31** The list is sorted with pinned projects first, then by display name (case-insensitive).
- **AC-32** Empty state (F01 text) when no projects exist: "No projects registered yet" with a "Register project" button (the F01 empty state is replaced by this; the button is added in F02 because the action now exists).
- **AC-33** Clicking a row opens P3. Clicking a row in P3's list (none in F02) is not applicable.

### Detail

- **AC-40** P3 shows: display name, type, root path, drive, tags, pinned state, and date registered.
- **AC-41** P3 has Pin / Unpin and a Back link to P1.
- **AC-42** P3 does not show counts for files, config, notes, or secrets in F02. The preview's linked-counts line is not built.
- **AC-43** Opening P3 for an unknown id shows "This project no longer exists." with a link back to P1.

### Pinning

- **AC-50** Pin / Unpin changes the pinned state immediately and persists across restarts.
- **AC-51** Pinned state appears in P1 (pin indicator, sort order) and P3.
- **AC-52** Pinning has no limit in F02.

### Safety and boundaries

- **AC-60** Registering or listing a project does not read, copy, create, or modify any file in the project folder. Only the path is checked for existence and type (folder or file).
- **AC-61** The database stores paths; no project file content is stored.
- **AC-62** The React layer does not touch the file system. The path existence check and the native folder picker run in Rust (the picker is a Tauri dialog invoked from Rust or via the dialog plugin's Rust API; see plan).
- **AC-63** Only the data folder is written (SQLite, via migration 2).

## 4. Edge Cases

- Path with trailing backslash or mixed slashes: normalised before the duplicate check and before storage.
- Path is a network or drive-letter root such as `F:\`: allowed (it is a folder). The drive column shows `F:`.
- Path points to a folder that is later deleted: the registry keeps the entry; a missing folder is not checked again in F02 (no status column; F06 and F05 handle missing paths).
- Two registrations at the same moment with the same path: one succeeds, the other shows AC-12.
- Display name contains characters that are unusual but valid in Windows names: stored as typed (trimmed).
- A tag appears twice in different case ("API", "api"): stored once as "api".
- Pinned flag on a project later removed: not applicable in F02 (no removal feature).

## 5. Non-goals (F02)

- Editing name, type, or tags after registration (F15).
- Archiving or unregistering a project (F15).
- Scanning project contents, detecting project type automatically, or config discovery (F13).
- Linked counts for files, config, notes, secrets (later features).
- Home pinned-projects card (F10).
- Quick open for projects (F11).
- Any git information, drive free-space, or file watching.
- Importing projects from a file.

## 6. Decisions Needing Approval

- **D1 — Nested project paths (AC-20, AC-21).** (a) **Allow both, with an inline note.** (b) Reject a path inside another project. *Recommended: (a). Monorepos and sub-apps are common; rejecting them blocks real use, while the note keeps the relationship visible without enforcing a rule the roadmap does not state.*
- **D2 — Type list.** (a) **Use the preview's five types (Spring Boot, React, Docker/Infra, Node, Other).** (b) Free text. *Recommended: (a), matching the prototype. Free text makes the list and filters inconsistent.*
- **D3 — Dialog plugin.** The architecture names `tauri-plugin-dialog` for F02 folder pickers; this spec uses it. *Recommended: approve.* No other new runtime dependency is needed.

## 7. Constitution Check

- **P1** No secrets. Pass.
- **P2** Path checks and the registry live in Rust (`projects` module). SQL uses bound parameters; migration 2 adds the `projects` table through `rusqlite_migration`. The `projects` module does not reach into other modules' tables. Pass.
- **P3** Each command has one argument struct and one return type; validation returns user-facing codes (for example `path_empty`, `path_not_found`, `path_is_file`, `already_registered`, `name_empty`, `name_too_long`, `path_too_long`, `tag_invalid`, `too_many_tags`), and the UI maps them to the copy in AC-10 to AC-15. Pass.
- **P4** Rust tests for normalisation, duplicate detection (including case and trailing slash), validation rules, nested allowance, pin persistence across reopen, and no writes to the project folder (the folder's file list and modified times are unchanged after register). Vitest for tag parsing and validation mapping where branching exists. E2E on the built app. Pass (planned).
- **P5** No network. Pass.

## 8. Open Questions

None beyond Section 6.
