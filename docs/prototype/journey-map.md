# journey-map.md

## Metadata

- **Status**: `draft`
- **Last updated**: 2026-10-10
- **Based on**: `idea-brief.md`

> One map, one persona, one scenario. If the product has distinct user types with meaningfully different journeys, duplicate this template per persona rather than blending them.

---

## 1. Actor

- **Persona**: The author: a solo Windows developer working on several active projects at once (a Spring Boot API and a billing service on `D:`, a React dashboard on `C:`, a Docker/Caddy infra repo and a small Node worker on `F:`). Uses Notepad++, VS Code and command-line tools daily and handles local configuration and credentials every day. This is the Primary user from `idea-brief.md` Target Users; no other persona exists for the first release.
- **Scenario** (the situation when the journey starts): Monday morning, 2026-10-12. The machine was restarted over the weekend, so every editor tab from Friday's staging-deploy work is gone. The developer opens DevShelf, which starts locked for the Vault, and needs to get back into the "staging deploy fix" task.
- **Goal** (what they're trying to accomplish): Reopen the whole working set for the task (config files from three projects, one infra file and one loose deploy-notes file) in one action, open the right environment config (staging, not production), copy the staging API key without exposing it, and jot a note about what was found, all without leaving anything in a repository and without re-hunting for each file by hand.

---

## 2. Journey Phases

Emotion scale: 1 = frustrated, 5 = confident and relaxed.

| | Phase 1: Orient after restart | Phase 2: Restore the working set | Phase 3: Find the right environment config | Phase 4: Fetch a credential from the Vault | Phase 5: Capture a note and close out |
|---|---|---|---|---|---|
| **Actions** (what they do) | Launches DevShelf.<br>Scans pinned projects and recent workspaces on Home.<br>Presses Ctrl+K to check a project by name.<br>Optionally registers a newly cloned project (`notify-worker`) by picking its root folder. | Opens the workspace "Orders API staging deploy fix".<br>Checks the ordered item list.<br>Clicks Restore.<br>Reads the results summary (opened / missing).<br>Chooses Relink or Ignore for the missing item. | Opens Config Catalog.<br>Filters by project `orders-api` and environment `staging`.<br>Spots one file marked missing or inaccessible.<br>Opens `application-staging.yml` in the chosen editor profile (Notepad++ or VS Code). | Opens Vault (locked).<br>Types the master password; may mistype once and retry.<br>Scans the masked secret list.<br>Reveals one value briefly to confirm it is the staging key.<br>Clicks Copy and pastes into the config in the editor.<br>Locks the Vault manually, or lets the timeout lock it. | Opens Notes.<br>Selects "orders-api: staging rollout log" or creates a new note linked to the project.<br>Types Markdown findings.<br>Watches the autosave indicator.<br>Moves on to the next task. |
| **Thoughts** (what they're thinking) | "What was I doing on Friday?"<br>"Is the new repo registered yet?"<br>"I don't want to dig through `D:\work\acme` again." | "Will it really open all of them in the right editors?"<br>"Some file probably moved since Friday; I need to know which one, not have it silently dropped."<br>"I don't want it to rewrite my workspace behind my back." | "Staging, not production. Which of these is the right one?"<br>"Is this reference stale or is the file just unreadable?" | "I just want the key in my clipboard, not on screen."<br>"Did I type the password wrong or is it broken?"<br>"Is it safe to leave it unlocked while I paste?"<br>"Why did it lock while I was reading?" | "I'll forget this by Wednesday if I don't write it down."<br>"Did that save? I can't afford to lose it."<br>"This must not end up in the repo." |
| **Emotions** (rate 1-5 or describe) | 3: slightly foggy, mildly annoyed at the lost tabs; relieved once the recent workspace is visible. | 4 on click (hopeful), dips to 2 when something is missing, back to 4 if the summary is clear and offers a way forward. | 3: cautious, because choosing the wrong environment is costly; 4 once filters narrow the list. | 2-3: tension around security friction; spikes to 2 on a wrong-password error or an unexpected auto-lock; 4 after a successful copy. | 4: calm when saving is visible; 2 if save state is ambiguous or an error appears. |
| **Touchpoints** (screen/feature involved) | Home (pinned projects, recent workspaces), quick-open palette (Ctrl+K), Projects list, Register project dialog, sidebar navigation | Workspaces list, Workspace detail (ordered items), Restore action, results summary with Relink / Ignore, simulated editor launch | Config Catalog with filters (project, environment, kind, status), config row actions, editor profile choice | Vault locked screen, unlock form, secret list (masked), Reveal, Copy with neutral confirmation, Lock button, auto-lock countdown, Settings (auto-lock timeout) | Notes list, Markdown editor, autosave state indicator (saved / unsaved / error), project link on a note |
| **Pain points** | P1. Nothing shows what the "current task" is; recents without context force guessing.<br>P2. A newly cloned project must be registered before it can join any workspace; if this feels heavy it will be skipped. | P3. Restore may open only files and not the editor's tab layout; the developer may expect tab state to come back and be disappointed.<br>P4. Missing paths mixed into a long list of successes get overlooked.<br>P5. Fear that Ignore or Relink silently deletes or edits the saved workspace. | P6. Several near-identical files (`application-*.yml`, `.env.*`) across projects make the wrong environment easy to open.<br>P7. A status of "missing" vs. "inaccessible" is unclear without an explanation. | P8. Masked by default plus explicit Copy is extra clicks on every use; risk that it feels like too much friction.<br>P9. A wrong-password message that is vague or too alarming.<br>P10. Lock-on-inactivity can interrupt a paste mid-task and make the user re-enter the password.<br>P11. Fear that Copy leaves the key visible in a toast or lingering in the clipboard. | P12. Autosave state is ambiguous: no way to tell saved from pending from failed.<br>P13. Unsure whether the note is stored outside the repo and who can read it. |

---

## 3. Opportunities

- P1 (no sense of current task): Home puts recent workspaces (with last-used date and item count) directly beside pinned projects, so Monday's task is one click away.
- P2 (registering a project feels heavy): a short Register project dialog (folder path, display name, type, optional tags) with inline validation and a success confirmation, also reachable from Home and the quick-open palette.
- P3 (Restore may disappoint without tab state): state explicitly on the workspace and in the results summary that Restore reopens files and folders, not unsaved buffers or editor tab layout.
- P4 (missing paths overlooked): the Restore results summary leads with a count of opened vs. missing items and lists missing paths prominently, separated from successes.
- P5 (fear of silent change): per-item Relink and Ignore actions, stale references stay in the workspace until the user changes them, and the summary says so.
- P6 (wrong environment opened): environment is a first-class filter and a visible label on every config row; the filter state is shown above the list.
- P7 (missing vs. inaccessible): status column with plain-language reasons ("File not found at this path", "Drive F: not reachable") and a Relink action.
- P8 (masked + Copy friction): one-click Copy on the masked list row, with no detail screen required; Reveal is separate and optional. Prototype tests whether this is acceptable.
- P9 (vague wrong-password message): a calm, specific inline error ("Master password is incorrect. Try again.") with focus kept on the field and no data exposed.
- P10 (lock interrupts task): visible auto-lock countdown, configurable timeout in Settings, activity resets the timer, and the lock screen explains why it locked.
- P11 (Copy leakage fear): neutral confirmation that never shows the value ("Copied to clipboard"), and the Reveal state auto-re-masks.
- P12 (ambiguous autosave): a persistent three-state indicator (Saved, Unsaved changes, Couldn't save) with a retry action on error.
- P13 (where notes live): a note-level label stating notes are stored locally outside project roots and encrypted at rest.

---

## 4. Scope for Playable Prototype

- **Covered in prototype**: Phases 1 to 5 of the Monday-morning scenario as one clickable path on Windows desktop (HTML at desktop viewport, max width 1280px). Priority order for decisions: Flow > Function > Usability > Data > polish. The numbered flows below are the walkthrough script; each lists start point, action, what comes next, expected result and cancel/error path.

  **Navigation shared by every flow**: one sidebar (Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings) on every screen, plus a quick-open palette on Ctrl+K.

  **Flow 1: Orient on Home**
  - Start point: Home screen after launch.
  - Action: Scan pinned projects and recent workspaces; click a pinned project card.
  - What comes next: Opens that project's entry on the Projects screen (detail with root path, tags, registered files).
  - Expected result: The user can tell which projects and which workspace to resume without opening anything else.
  - Cancel/error path: Back via the sidebar. If no projects or workspaces exist (empty-state toggle), Home shows guidance with a "Register your first project" action instead of blank panels.

  **Flow 2: Register a project (US-001)**
  - Start point: Projects list, "Register project" button (also reachable from Home quick action and the Ctrl+K palette).
  - Action: Enter or pick a root folder path, display name, type, optional tags; click Register.
  - What comes next: The dialog closes and the project appears in the list with a success confirmation.
  - Expected result: New project `notify-worker` listed with its root path and tags; nothing is copied or modified.
  - Cancel/error path: Cancel closes the dialog without changes. Validation errors inline: empty path, path not found, path already registered, empty display name. The dialog stays open with the entered values kept.

  **Flow 3: Restore a workspace with a missing path (US-003, US-004)**
  - Start point: Workspaces list, open "Orders API staging deploy fix".
  - Action: Review ordered items; click Restore.
  - What comes next: A results summary appears: N opened, 1 missing. The missing item shows its path and the actions Relink and Ignore.
  - Expected result: Valid items show as opened in their editor profile (simulated). The missing item remains in the workspace. Choosing Relink with a new path marks it resolved; choosing Ignore leaves the reference in place and marks it acknowledged for this restore.
  - Cancel/error path: Close the summary at any time; the workspace is unchanged. Relink with an empty or invalid path shows an inline error. If every path is valid the summary shows an all-opened success state; if every path is missing the summary shows a clear all-failed state with a link to check the drive.

  **Flow 4: Find and open an environment config (US-002, US-005)**
  - Start point: Config Catalog, unfiltered list.
  - Action: Filter by project `orders-api`, environment `staging`; optionally by kind and status; click Open on `application-staging.yml`, choosing an editor profile.
  - What comes next: A simulated "Opened in VS Code" confirmation. Rows with status Missing show Relink instead of Open.
  - Expected result: Only matching rows appear; active filters are visible and clearable; the file opens via the chosen profile (simulated).
  - Cancel/error path: Clear filters to return to the full list. If filters match nothing, a "No config files match these filters" empty state offers Clear filters. Opening a missing file shows an inline error that names the path.

  **Flow 5: Unlock the Vault, copy a secret, lock (US-007, US-008, US-009)**
  - Start point: Vault, locked state.
  - Action: Enter the master password and click Unlock; in the masked list click Copy on "orders-api staging API key"; optionally click Reveal; click Lock.
  - What comes next: After unlock, the masked secret list appears with an auto-lock countdown. After Copy, a neutral confirmation appears ("Copied to clipboard") with no value shown. After Lock, the locked screen returns and the list is no longer visible.
  - Expected result: Secrets are masked by default; Reveal shows the value temporarily and re-masks; Copy never displays the value.
  - Cancel/error path: Wrong password shows an inline error and stays locked (field cleared, focus returned). Clicking Copy or Reveal after the countdown expired returns to the locked screen with an explanatory message. Empty password shows a validation message.

  **Flow 6: Write a note with autosave (US-006)**
  - Start point: Notes list.
  - Action: Open "orders-api: staging rollout log" or click New note; type Markdown.
  - What comes next: Indicator shows Unsaved changes while typing, then Saved a moment after the user pauses.
  - Expected result: The note is linked to a project (or marked general) and shows a "Stored locally outside project roots" label; the list entry updates.
  - Cancel/error path: A simulated save failure (triggered by a visible demo control) shows "Couldn't save" with a Retry action; retry returns to Saved. An empty list shows an empty state with a New note action.

  **Flow 7: Set editor profile and auto-lock (US-005, US-009)**
  - Start point: Settings.
  - Action: Choose the default editor profile (Notepad++, VS Code, Windows default); choose the auto-lock timeout.
  - What comes next: A "Settings saved" confirmation appears; the profile is used by Open actions in Config Catalog and Restore, and the timeout drives the Vault countdown.
  - Expected result: Subsequent simulated opens name the new profile; the new timeout is reflected in the Vault countdown.
  - Cancel/error path: Reset changes returns to the previous values. A custom editor path left empty on a custom profile shows an inline validation error.

  **Flow 8: Quick open with Ctrl+K**
  - Start point: Any screen.
  - Action: Press Ctrl+K, type part of a project, workspace, config file or note name; use arrow keys and Enter.
  - What comes next: Navigates to the matching screen/record.
  - Expected result: Results are grouped by type and the match navigates correctly.
  - Cancel/error path: Esc closes the palette. No matches shows "No results for this search".

- **Deliberately not prototyped yet** (and why):
  - Real editor launching, filesystem access, clipboard, encryption and persistence: the prototype tests flow and comprehension, not integration (see `prototype-brief.md` Section 5 for each fake).
  - Backup and restore, import/export: not part of the Monday scenario; lower flow risk than restore, copy and lock.
  - Config discovery by pattern scanning, config diff and validation: the catalog is shown with manually registered rows; discovery adds settings complexity without testing the core flow.
  - Workspace creation and item reordering editor: the scenario is restoring an existing workspace; editing is lower risk. Ordered items are shown read-only.
  - Clipboard auto-clear timer and re-authentication before reveal: open questions in `idea-brief.md`; covered by the lock/copy assumptions only.
  - Tag management, archive and delete project, secret create/edit: CRUD that does not change the journey risk.
  - Other platforms and a first-run vault creation flow: Windows desktop only; the demo vault already exists.
