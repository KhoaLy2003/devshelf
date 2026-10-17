# roadmap.md

## Metadata

- **Status**: `approved`
- **Last updated**: 2026-10-12
- **Based on**: `idea-brief.md`, `prototype/` (journey-map.md, prototype-brief.md, preview/index.html), `DESIGN.md`. `market-notes.md` does not exist: Step 2 was skipped by user decision, so there are no feature inspirations to place. See Section 6 for the priority assumptions this forces.
- **Phase 2 Scope**: `feature`
- **Admin Site**: `no`

> Product-scope decision, not a technical one. Assume no tech stack. The orchestrator reads this file to pick the next feature for Phase 2, so keep the `Status` column accurate.

---

## 1. Core User Flow

The Monday-morning journey (journey-map.md, one persona, five phases, flows F1-F8). The author opens DevShelf after a restart and needs to resume the "staging deploy fix" task.

1. **Orient** (F1, F2): open the app, see pinned projects and recent workspaces on Home, register a newly cloned project if needed.
2. **Restore the working set** (F3): open a saved workspace, press Restore, read the opened-vs-missing summary, relink or ignore the missing item. Files open in the chosen editor.
3. **Find the right environment config** (F4): filter the config catalog by project and environment, open the file in the chosen editor, relink if it is missing.
4. **Fetch a credential** (F5): unlock the vault with the master password, copy the staging key without displaying it, lock the vault (or let it auto-lock).
5. **Capture a note and close out** (F6): write a Markdown note linked to the project, with visible autosave state, stored outside any repository.
6. **Support** (F7, F8): set the default editor profile and auto-lock timeout; jump to any item with quick open (Ctrl+K).

---

## 2. Feature List

| ID | Feature | Priority | Size | Depends on | Status |
|---|---|---|---|---|---|
| F01 | **Foundation and navigation**: the app opens to a main window with the sidebar (Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings), keeps all user data locally and offline, and remembers it across restarts. Empty states for every area. Traces to: all flows. | Must | M | none | shipped |
| F02 | **Project registry**: register a project from its root folder (path, display name, type, tags) with inline validation (empty, not found, already registered); list projects with drive, tags and detail; pin projects. Nothing in the project is copied or changed. Traces to: Phase 1, F1, F2. | Must | M | F01 | shipped |
| F03 | **Editor profiles and open in editor**: define editor profiles (Windows default, Notepad++, VS Code, custom path), choose a default, and open a file or folder in a chosen profile with a clear error if it cannot be opened. Traces to: Phases 2-3, F3, F4, F7. | Must | M | F01 | shipped |
| F04 | **Workspace builder**: create, rename, and delete named workspaces; add files and folders (from any project or none) as ordered items, each with its own editor profile; reorder and remove items. Saved references are never changed unless the user changes them. Traces to: Phase 2, F3 (a workspace must exist before it can be restored). | Must | M | F02, F03 | shipped |
| F05 | **Workspace restore with missing-path report**: restore a workspace in one action; a results summary leads with opened vs. missing counts, lists missing or unreachable paths separately, and offers Relink and Ignore per item. States that tab layout and unsaved changes are not restored. Updates "last used". Traces to: Phase 2, F3 (P3-P5). | Must | M | F04 | shipped |
| F06 | **Config catalog**: register config files per project with environment (local, dev, staging, production) and kind; filter by project, environment, kind and status with visible, clearable filters; show Available, Missing or Inaccessible with a plain-language reason; open in a chosen editor or relink. Traces to: Phase 3, F4 (P6, P7). | Must | M | F02, F03 | shipped |
| F07 | **Private project notes**: create and edit Markdown notes, optionally linked to a project; autosave with a visible Saved / Unsaved changes / Couldn't save (with Retry) indicator; optional preview; browse and filter by project; notes are stored outside project roots and protected at rest, with a label saying so. Traces to: Phase 5, F6 (P12, P13). | Must | M | F01, F02 | shipped |
| F08 | **Vault access control**: create a master password on first use; unlock with specific inline errors for empty or wrong password; manual lock; auto-lock after inactivity with a visible countdown and a configurable timeout (1, 5, 15 minutes, never while open); the lock screen explains why it locked. Traces to: Phase 4, F5, F7 (P9, P10). | Must | M | F01 | shipped |
| F09 | **Secret entries**: add, edit, delete secrets with label, type, project, environment; list masked by default; explicit Reveal (re-masks automatically) and one-click Copy with a neutral confirmation that never shows the value; filter by project and type. Secrets are protected at rest. Traces to: Phase 4, F5 (P8, P11). | Must | M | F08, F02 | shipped |
| F10 | **Home overview**: pinned projects and recent workspaces (last used, item count, missing-item warning) side by side; quick actions (Register project, Quick open); first-run guidance when empty. Traces to: Phase 1, F1 (P1, P2). | Must | S | F02, F05 | shipped |
| F11 | **Quick open (Ctrl+K)**: one palette on every screen to jump to a project, workspace, config file or note by name; grouped results, keyboard selection, no-results message. Traces to: F8. | Should | S | F02, F05, F06, F07 | deprecated (not in first MVP) |
| F12 | **Encrypted backup and restore**: export all DevShelf data to a single encrypted file and restore it, with the restore verified before it replaces current data. Idea brief lists it in MVP scope; not required for the first usable release. | Could | L | F05, F06, F07, F09 | deprecated (not in first MVP) |
| F13 | **Bounded config discovery**: suggest config files for a project by scanning only the project root and user-selected folders with a fixed list of file patterns; the user approves each suggestion before it enters the catalog. Never scans the whole repository content. | Should | M | F06 | deprecated (not in first MVP) |
| F14 | **Clipboard auto-clear**: after copying a secret, clear the clipboard after a user-set delay. Default and delay are open questions in the idea brief. | Could | S | F09 | deprecated (not in first MVP) |
| F15 | **Project maintenance**: edit a project's name, type and tags; archive or unregister a project without touching its files. | Could | S | F02 | deprecated (not in first MVP) |
| F16 | **Help**: in-app guides, feature notes, privacy and security summary, troubleshooting, and a changelog for the first MVP, with deprecated features listed as not in this version. Added after the original list. | Must | S | F01 | shipped |

Counts: Must 10, Should 2, Could 3, Won't (deferred) 12. Total rows in this list: 15.

---

## 3. Build Order

Dependencies first, then tier. After F02 and F03 the config, notes and vault streams are independent; they are ordered by how early they support the Core User Flow.

1. F01 — Foundation and navigation: every other feature stores data and lives behind the sidebar.
2. F02 — Project registry: workspaces, config, notes, and secrets all link to a project.
3. F03 — Editor profiles and open in editor: Restore and config Open both need it.
4. F04 — Workspace builder: a workspace must exist before it can be restored.
5. F05 — Workspace restore with missing-path report: the main payoff of the product.
6. F10 — Home overview: needs registered projects and restorable workspaces to show anything useful.
7. F06 — Config catalog: needs projects and editor opening.
8. F07 — Private project notes: needs only projects; closes the Monday scenario's last step.
9. F08 — Vault access control: gate must exist before any secret is stored.
10. F09 — Secret entries: needs the unlocked vault.
11. F11 — Quick open: more useful once every searchable type exists.
12. F13 — Bounded config discovery: extends the catalog.
13. F12 — Encrypted backup and restore: must cover every data type, so it comes after all of them.
14. F14 — Clipboard auto-clear: hardens F09.
15. F15 — Project maintenance: housekeeping.

---

## 4. Explicitly Deferred (Won't this round)

| Feature | Why deferred |
|---|---|
| Embedded code editor or terminal | Idea brief no-go: DevShelf manages references; editing stays in external tools. |
| Arbitrary command execution; automatic injection of secrets into processes or environment variables | Idea brief no-go: security and trust risk, far outside the MVP problem. |
| Automatic config rewriting, synchronization or deployment | Idea brief no-go: the product principle is never to silently modify project or config files. |
| Cloud sync, team sharing, multi-user access, remote secrets service | Idea brief no-go: single local user, offline by default. |
| Repository-wide content indexing | Idea brief no-go: discovery stays bounded (F13); full indexing adds cost without helping the core flow. |
| Git operations or a source-control client | Idea brief no-go: existing tools already cover it. |
| Restoring unsaved editor buffers or editor tab layout | Idea brief no-go; Restore states this limitation explicitly (P3). |
| Config diff and validation | Not in the Monday scenario; deferred in the prototype; adds complexity, not flow value. |
| Import and export of individual items (as opposed to whole-app backup) | Not part of the core flow; F12 covers the data-safety need. |
| Re-authentication before Reveal or Copy | Open question in the idea brief; the unlock plus auto-lock is the MVP protection. Revisit after real use. |
| Secret version history and expiry reminders | Nice for later; no journey step needs it. |
| Platforms other than Windows | Idea brief: Windows desktop only. |

---

## 5. Phase 2 Scope Decision

| Criterion | Score (`all` / `feature`) |
|---|---|
| Must feature count | `all` (10, within the limit of 15) |
| Feature sizes | `all` (nine M, one S, no L among the Musts) |
| Domains | `feature` (workspace/config/notes and secrets/encryption are two domains; F08 and F09 are 2 of 10 Musts = 20%, which meets the 20% threshold) |
| Dependency structure | `feature` (a short foundation chain, then config, notes and vault become independent parallel streams) |
| Developers | `all` (solo) |

**Recommendation and rationale** (confirmed by user): `feature`, because two criteria (Domains, Dependency structure) point to it and any one overrides, even though count, sizes and team size favour `all`.

Admin Site is not counted in any criterion.

---

## 6. Assumptions & Risks

- **Step 2 skipped**: no market research, so no competitor-derived features exist, and every priority below is my own judgment drawn from the idea brief, journey map and prototype. The human decides at the gate.
- **Assumption: workspace creation is Must.** The prototype only restores existing workspaces, but restore is useless without a way to make one, so F04 is Must.
- **Assumption: editor profiles are Must.** Restore and config Open both depend on them.
- **Assumption: Home is Must.** It carries Phase 1 of the journey (P1, P2); the cost is small.
- **Assumption: notes work independently of the vault.** Notes are protected at rest (F07) but the journey does not require unlocking the vault to write one; whether they share the vault's master password is a Step 6 decision. If they must, F07 would depend on F08 and the build order would change.
- **Assumption: Config catalog entries are registered by hand in the MVP**; automatic discovery is Should (F13).
- **Assumption: encrypted backup is Could, not needed for the first usable release.** Confirmed by the user. Build order keeps it after every data type, since it must cover them all. If it becomes required later, F12 moves to Must; it is size L, so Phase 2 Scope would stay `feature`.
- **Assumption: clipboard auto-clear and re-authentication are not required for the MVP.** Open questions in the idea brief.
- **Assumption: single user, one machine, no collaboration**, Windows desktop only, offline.
- **Risk**: the vault holds real credentials. A defect in F08 or F09 has a higher cost than elsewhere and may need more verification effort than the M size suggests.
- **Risk**: whole-app backup (F12) must cover every data type, so any data type added later forces a backup change.
- **Risk**: Rust/cargo is not currently installed on the development machine (idea brief constraint); this affects schedule, not scope.

<!-- Phase 3 appends `## Release <slug>` blocks below this line. Never edit shipped rows above. -->
