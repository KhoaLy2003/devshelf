# prototype-brief.md

## Metadata

- **Status**: `draft`
- **Last updated**: 2026-10-10
- **Based on**: `idea-brief.md`, `journey-map.md`, `DESIGN.md`

> This brief is the spec for the HTML prototype (`docs/preview/`). It documents intent, coverage, and fidelity choices so nothing is left ambiguous.
>
> **Core question the prototype must answer:** *"Are we building the right product, and does the proposed workflow actually work for the people who will use it?"*
>
> **Fidelity rule:** structure and flow first. Every visual value (color, font, spacing, radius, shadow) comes from `docs/DESIGN.md`; invent none. If it is so bare the flow is hard to follow, add just enough structure to make it clear.

---

## 1. What This Prototype Covers

- Platform: Windows desktop (Tauri app), represented as HTML at desktop viewport with a maximum content width of 1280px.
- One persona (the author, a solo Windows developer) and one scenario (Monday morning after a restart; reopen the task's working set across projects, open the right environment config, copy an API key, jot a note), as defined in `journey-map.md` Sections 1 and 4.
- The eight numbered flows in `journey-map.md` Section 4: F1 Orient on Home; F2 Register a project; F3 Restore a workspace with a missing path; F4 Find and open an environment config; F5 Unlock the Vault, copy a secret, lock; F6 Write a note with autosave; F7 Set editor profile and auto-lock; F8 Quick open with Ctrl+K.
- Decision priority when something must give: Flow > Function > Usability > Data > polish.
- Traceability to the idea brief and kickoff user stories: project registry (US-001, F2); config catalog by project/environment/kind (US-002, F4); cross-project workspaces and restore with missing-path reporting (US-003, US-004, F3); editor profiles and quick open (US-005, F7, F8); private notes outside the repo (US-006, F6); masked secrets, unlock, copy, lock (US-007, US-008, US-009, F5).
- Not covered: backup/restore, import/export, config discovery, diff/validation, workspace editing, project edit/archive/delete, secret create/edit, first-run Vault creation (see `journey-map.md` Section 4).

---

## 2. Screens and Interactions

Structure: one navigation component on every screen. A left sidebar lists Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings (the active item is highlighted). A Ctrl+K quick-open palette is available on every screen. All screens live in a single page and are switched by the URL hash, so Back works and no navigation causes a full reload.

| Screen | Purpose | Key interactions | File |
|---|---|---|---|
| Home | Resume work quickly: see pinned projects and recent workspaces; start common actions. | Click a pinned project card (opens Projects detail); click a recent workspace (opens Workspace detail); quick actions "Register project", "Open quick open (Ctrl+K)"; demo state toggle for the empty state (see Section 3). | `docs/preview/index.html` (`#/home`) |
| Projects list | List registered projects with name, type, root path, tags, drive; entry to detail. | Click a row to expand a detail panel (root path, tags, registered files, linked notes and secrets count); "Register project" opens the dialog; Esc/Cancel closes it; Register validates and adds the project. | `docs/preview/index.html` (`#/projects`) |
| Register project dialog | Add a project by root folder. | Fields: root folder path (with a "Browse..." button that simulates picking `F:\sandbox\notify-worker`), display name, type (Spring Boot, React, Docker/Infra, Node, Other), tags. Register / Cancel. Inline validation messages. | `docs/preview/index.html` (modal on `#/projects`) |
| Workspaces list | List named collections with item count, projects involved, last used. | Click a row to open Workspace detail. | `docs/preview/index.html` (`#/workspaces`) |
| Workspace detail | Show ordered items and restore them. | Ordered item list (index, file name, full path, project or "No project", editor profile, status icon); "Restore" button; Restore opens the results summary panel (opened vs. missing counts; missing paths with Relink and Ignore; Close). Relink opens an inline path field with Apply / Cancel. A note states: "Restore reopens files and folders. Editor tab layout and unsaved changes are not restored." | `docs/preview/index.html` (`#/workspaces/<id>`) |
| Config Catalog | Find the right environment config. | Filter dropdowns for project, environment (local, dev, staging, production), kind (YAML, properties, .env, Docker Compose, Caddy/Nginx, JSON) and status (Available, Missing, Inaccessible); active-filter chips; "Clear filters"; per-row Open (uses the default editor profile, with a menu to pick another) or Relink for missing rows. | `docs/preview/index.html` (`#/config`) |
| Notes list | Browse project and general notes. | Click a note to open the editor; "New note"; filter by project or "General". | `docs/preview/index.html` (`#/notes`) |
| Note editor | Edit Markdown with autosave. | Title field, project selector, Markdown text area with a side-by-side rendered preview toggle; autosave indicator (Saved / Unsaved changes / Couldn't save) with Retry; a "Stored locally outside project roots" label; a demo control "Simulate save failure"; back to list. | `docs/preview/index.html` (`#/notes/<id>`) |
| Vault (locked) | Gate secrets behind the master password. | Password field and Unlock; wrong password inline error; empty-field validation; a visible demo hint line stating the demo password (see Section 5). | `docs/preview/index.html` (`#/vault`) |
| Vault (unlocked) | List secrets masked; Reveal, Copy, Lock. | Secret rows: label, type, project, environment, masked value, updated date; per-row Reveal (shows the fake value, re-masks after 10 seconds or on second click) and Copy (neutral confirmation "Copied to clipboard"); "Lock now" button; auto-lock countdown chip in the header; "Skip to timeout" demo link; filter by project/type. | `docs/preview/index.html` (`#/vault`) |
| Settings | Configure editor profiles and auto-lock. | Editor profile list (Notepad++, VS Code, Windows default) with one marked default and a "Set as default" action; fields for each profile's executable path; auto-lock timeout selector (1, 5, 15 minutes, Never while the app is open); Save settings and Reset changes; saved confirmation. | `docs/preview/index.html` (`#/settings`) |
| Quick-open palette | Jump to any project, workspace, config file or note by name. | Opens on Ctrl+K (and via a sidebar search button); results grouped by type; Up/Down/Enter to select; Esc to close; no-results message. | `docs/preview/index.html` (overlay on any screen) |

---

## 3. State Coverage

Empty states are reached through a "Demo states" panel (collapsed button at the bottom of the sidebar) that switches the dataset for Projects, Workspaces, Config Catalog, Notes and Vault between "Full data" and "Empty data" without a reload. Error and validation states are reached by user actions described in the Notes column.

| Screen | Empty state | Error / validation state | Success state | Notes |
|---|---|---|---|---|
| Home | No pinned projects and no recent workspaces: guidance text and a "Register your first project" button. | Not applicable on its own; if a recent workspace points to a missing drive the card shows a warning badge "1 item missing at last restore". | Pinned projects and recent workspaces shown with dates. | Empty state via the demo panel. |
| Projects list | "No projects registered yet" with a Register project button. | In the dialog: empty path ("Enter or pick a project folder"), path not found ("Folder `Z:\code\x` was not found"), already registered ("`D:\work\acme\orders-api` is already registered as orders-api"), empty display name. | Dialog closes; the new row appears and highlights; message "Project notify-worker registered. No project files were copied or changed." | Dialog keeps values on validation error. |
| Register project dialog | Initial blank form. | The four validations listed above, shown inline next to the field. | See Projects list success. | Triggering "path not found" requires typing any path not in the known mock set. |
| Workspaces list | "No workspaces yet. A workspace saves an ordered set of files to reopen together." | Not applicable. | List with last-used dates after a Restore updates "Last used". | Creating workspaces is not prototyped; the empty state has no create action, only the explanation. |
| Workspace detail | Workspace with no items: "This workspace has no items." (Restore disabled.) | Restore with a missing path: summary shows "N opened, 1 missing" with the missing path and Relink / Ignore. Relink with an empty or unknown path shows an inline error. Third workspace shows an all-missing outcome when its drive is flagged unreachable in the demo panel. | All items valid: summary "All 5 items opened". Relink applied: the item becomes valid and the summary updates; workspace "Last used" changes. | The saved reference is never deleted or silently changed; Ignore marks it acknowledged only. |
| Config Catalog | Filters returning nothing: "No config files match these filters" with Clear filters. With Empty data: "No config files registered" message. | Opening a Missing or Inaccessible file: inline error naming the path and the reason, with Relink offered. | After Open: confirmation "Opened application-staging.yml in VS Code". | Status uses plain-language reasons in a tooltip or sub-line. |
| Notes list | "No notes yet" with New note action. | Not applicable. | Notes sorted by last edited; edited note moves to top. | Filter by project/General. |
| Note editor | New blank note: placeholder text prompts the first line. | Autosave failure: indicator "Couldn't save" with Retry (triggered by "Simulate save failure"). | Indicator "Saved" with time; list entry updated. | Indicator sequence: Unsaved changes while typing, then Saved after about 1 second of no typing. |
| Vault | With Empty data: unlocked Vault shows "No secrets yet" (creation not prototyped). | Wrong password: "Master password is incorrect. Try again." Empty field: "Enter your master password." Expired lock: "Vault locked after 5 minutes of inactivity. Enter your master password to continue." | Unlocked list; Copy confirmation "Copied to clipboard"; Lock returns to the locked screen with "Vault locked." | Value is never in the confirmation text. Three consecutive wrong attempts show an additional line "3 failed attempts" (no lockout). |
| Settings | Not applicable (always has defaults). | Custom editor path left empty on a profile marked default: "Enter the path to the editor executable." | "Settings saved" confirmation; Open actions use the new default. | Reset changes restores last saved values. |
| Quick-open palette | Empty query: shows recent items. | No matches: "No results for this search". | Selecting a result navigates and closes the palette. | |

---

## 4. Mock Data

All data is hardcoded in the prototype page. Dates are in October 2026 (today in-story is Monday 2026-10-12). Paths are Windows paths on `C:`, `D:` and `F:`. Secret values are obviously fake and never resemble live keys.

**Projects (6)**

| Name | Type | Root path | Tags | Pinned |
|---|---|---|---|---|
| orders-api | Spring Boot API | `D:\work\acme\orders-api` | java, backend, acme | Yes |
| billing-service | Spring Boot API | `D:\work\acme\billing-service` | java, backend, acme | Yes |
| ops-dashboard | React dashboard | `C:\Users\khoa\dev\ops-dashboard` | react, frontend | Yes |
| infra-compose | Docker / reverse proxy | `F:\infra\compose-stack` | docker, caddy, nginx | Yes |
| notify-worker | Node worker | `F:\sandbox\notify-worker` | node, queue | No (not in the initial list; it is the project registered in Flow 2) |
| legacy-portal | PHP site (archived) | `F:\archive\legacy-portal` | legacy | No |

The initial list contains five projects (all except notify-worker); notify-worker is the one added in Flow 2.

**Workspaces (3)**

1. *Orders API staging deploy fix* (7 items, last used 2026-10-09 17:42, projects: orders-api, billing-service, infra-compose)

   | # | Item | Path | Project | Editor | Status |
   |---|---|---|---|---|---|
   | 1 | application-staging.yml | `D:\work\acme\orders-api\src\main\resources\application-staging.yml` | orders-api | VS Code | Valid |
   | 2 | .env.staging | `D:\work\acme\orders-api\.env.staging` | orders-api | Notepad++ | Valid |
   | 3 | staging-override.env | `D:\work\acme\orders-api\deploy\staging-override.env` | orders-api | Notepad++ | **Missing** (deliberate; file was renamed) |
   | 4 | docker-compose.staging.yml | `F:\infra\compose-stack\docker-compose.staging.yml` | infra-compose | VS Code | Valid |
   | 5 | Caddyfile | `F:\infra\compose-stack\caddy\Caddyfile` | infra-compose | Notepad++ | Valid |
   | 6 | application-staging.yml | `D:\work\acme\billing-service\src\main\resources\application-staging.yml` | billing-service | VS Code | Valid |
   | 7 | staging-rollout-2026-10.txt | `C:\Users\khoa\Documents\deploy-notes\staging-rollout-2026-10.txt` | No project (outside any registered root) | Windows default | Valid |

2. *Dashboard local setup* (5 items, last used 2026-10-07 09:15, projects: ops-dashboard, orders-api)

   | # | Item | Path | Project | Editor | Status |
   |---|---|---|---|---|---|
   | 1 | .env.local | `C:\Users\khoa\dev\ops-dashboard\.env.local` | ops-dashboard | Notepad++ | Valid |
   | 2 | vite.config.ts | `C:\Users\khoa\dev\ops-dashboard\vite.config.ts` | ops-dashboard | VS Code | Valid |
   | 3 | package.json | `C:\Users\khoa\dev\ops-dashboard\package.json` | ops-dashboard | VS Code | Valid |
   | 4 | src\api (folder) | `C:\Users\khoa\dev\ops-dashboard\src\api` | ops-dashboard | Windows default | Valid |
   | 5 | application-local.yml | `D:\work\acme\orders-api\src\main\resources\application-local.yml` | orders-api | VS Code | Valid |

3. *Notify worker incident review* (4 items, last used 2026-10-03 22:10, projects: infra-compose, billing-service; all valid, except its Drive F: items fail when "Simulate Drive F: unreachable" is toggled in the demo panel)

   | # | Item | Path | Project | Editor | Status |
   |---|---|---|---|---|---|
   | 1 | .env.production | `F:\sandbox\notify-worker\.env.production` | notify-worker (shown as "No project" until Flow 2 is done) | Notepad++ | Valid |
   | 2 | docker-compose.prod.yml | `F:\infra\compose-stack\docker-compose.prod.yml` | infra-compose | VS Code | Valid |
   | 3 | notify.conf | `F:\infra\compose-stack\nginx\conf.d\notify.conf` | infra-compose | Notepad++ | Valid |
   | 4 | incident-2026-10-03.md | `D:\work\acme\billing-service\docs\incident-2026-10-03.md` | billing-service | Windows default | Valid |

**Config Catalog (14 files)**

| File | Project | Env | Kind | Status |
|---|---|---|---|---|
| `application-local.yml` | orders-api | local | YAML | Available |
| `application-dev.yml` | orders-api | dev | YAML | Available |
| `application-staging.yml` | orders-api | staging | YAML | Available |
| `application-production.yml` | orders-api | production | YAML | Available |
| `.env.staging` | orders-api | staging | .env | Available |
| `staging-override.env` | orders-api | staging | .env | Missing |
| `application-staging.yml` | billing-service | staging | YAML | Available |
| `application.properties` | billing-service | dev | Properties | Available |
| `.env.local` | ops-dashboard | local | .env | Available |
| `docker-compose.staging.yml` | infra-compose | staging | Docker Compose | Available |
| `docker-compose.prod.yml` | infra-compose | production | Docker Compose | Available |
| `Caddyfile` | infra-compose | production | Caddy/Nginx | Available (shown as Inaccessible when "Simulate Drive F: unreachable" is on in the demo panel) |
| `notify.conf` | infra-compose | dev | Caddy/Nginx | Available |
| `.env.production` | billing-service | production | .env | Available |

Fourteen rows are listed so every environment has at least two entries; the minimum required is ten. The Missing row is the same missing file as workspace 1 item 3.

**Secrets (7)**

| Label | Type | Project | Env | Masked value shown | Fake value on Reveal / Copy | Updated |
|---|---|---|---|---|---|---|
| orders-api staging API key | API key | orders-api | staging | `••••••••••••••••` | `sk_test_FAKE_orders_staging_0001` | 2026-10-01 |
| orders-api staging DB password | Database password | orders-api | staging | `••••••••••••••••` | `demo-pass-not-real-4821` | 2026-09-28 |
| billing-service payment webhook secret | Webhook signing secret | billing-service | staging | `••••••••••••••••` | `whsec_FAKE_billing_00aa11` | 2026-10-05 |
| ops-dashboard Sentry token | Access token | ops-dashboard | local | `••••••••••••••••` | `sntrys_FAKE_dashboard_0007` | 2026-09-19 |
| infra-compose registry credential | Container registry login | infra-compose | production | `••••••••••••••••` | `registry-fake-token-9c3d` | 2026-10-02 |
| notify-worker queue connection string | Connection string | notify-worker | production | `••••••••••••••••` | `amqp://demo:FAKE@localhost:5672/demo` | 2026-10-03 |
| F: deploy key passphrase | SSH passphrase | (none) | (none) | `••••••••••••••••` | `fake-passphrase-for-demo` | 2026-08-30 |

**Notes (5)**

| Title | Project | Last edited | Content outline (Markdown) |
|---|---|---|---|
| orders-api: staging rollout log | orders-api | 2026-10-09 18:05 | Heading, checklist of rollout steps (3 ticked), a fenced block with a `docker compose` command, a "Open issues" list. |
| Checklist: before every production deploy | General | 2026-10-02 10:20 | Numbered checklist: backup config, check env file, notify channel, smoke test URL. |
| billing-service: webhook debugging | billing-service | 2026-10-05 21:40 | Observations about signature mismatches, a short log excerpt (no secrets), next steps. |
| infra-compose: Caddy TLS renewal | infra-compose | 2026-09-26 08:55 | Steps for certificate renewal, table of domains and expiry dates (fake domains such as `staging.acme-demo.test`). |
| ops-dashboard: local setup guide | ops-dashboard | 2026-09-18 14:30 | Install steps, `npm run dev`, required `.env.local` keys (names only). |

**Editor profiles (3)**: Notepad++ (`C:\Program Files\Notepad++\notepad++.exe`), VS Code (`C:\Users\khoa\AppData\Local\Programs\Microsoft VS Code\Code.exe`), Windows default (system association). Default: VS Code. Auto-lock default: 5 minutes.

**Recent items on Home**: recent workspaces are the three workspaces above, ordered by last used (the last-used dates are the ones listed with each workspace). Pinned projects are the four marked Pinned.

---

## 5. Playable Prototype Notes

- **Entry point** (which file to open first): `docs/preview/index.html`, opened directly as `file://`. It is the only HTML file; styles and scripts are inline. As required by the Step 3 build rules it loads Tailwind CSS, Phosphor Icons and the Geist font from CDNs, so icons, fonts and layout polish need an internet connection; the data and logic need none. Screens are switched by URL hash (`#/home`, `#/projects`, `#/workspaces`, `#/workspaces/<id>`, `#/config`, `#/vault`, `#/notes`, `#/notes/<id>`, `#/settings`).
- **What's clickable** (each interactive element and what it does):
  - Sidebar (every screen): Home, Projects, Workspaces, Config Catalog, Vault, Notes, Settings; active item highlighted; a search button that opens the quick-open palette; "Demo states" toggle.
  - Quick-open palette (Ctrl+K or the sidebar search button; Esc closes): type to filter projects, workspaces, config files and notes; Up/Down moves the selection; Enter or click navigates.
  - Home: pinned project cards (open project detail); recent workspace cards (open workspace detail); quick actions "Register project" and "Quick open".
  - Projects: row click expands/collapses detail; "Register project" opens the dialog; in the dialog "Browse..." fills the path with `F:\sandbox\notify-worker`, Register adds the project, Cancel/Esc closes without change; validation messages appear inline.
  - Workspaces: row click opens detail; in detail "Restore" produces the results summary; "Relink" opens a path field with Apply/Cancel; "Ignore" acknowledges the missing item; Close dismisses the summary; breadcrumb link back to the list.
  - Config Catalog: four filter dropdowns, filter chips with remove, "Clear filters", per-row "Open" (default profile), an editor menu to choose another profile, and "Relink" on Missing/Inaccessible rows.
  - Notes: list rows open the editor; "New note"; project filter; editor title field, project selector, Markdown text area, preview toggle, "Simulate save failure", Retry, back to list.
  - Vault: password field plus Unlock (Enter also works); per-row Reveal and Copy; Lock now; "Skip to timeout" demo link; filter by project/type.
  - Settings: default editor profile radio buttons, executable path fields, auto-lock timeout selector, Save settings, Reset changes.
  - Demo states panel: "Full data / Empty data" switch and "Simulate Drive F: unreachable" toggle; both take effect immediately without reload.
- **What's deliberately faked** (hardcoded data, simulated logic, no backend, no persistence):
  - No real filesystem: path validation, "Missing" and "Inaccessible" statuses and the Browse dialog are driven by hardcoded data and rules. A path is "found" only if it is in the mock set; otherwise it is "not found".
  - Editor launch is simulated: Open and Restore only show a confirmation naming the file and the editor profile (for example "Opened application-staging.yml in VS Code"); no program starts.
  - Clipboard is simulated: Copy shows "Copied to clipboard" but does not write to the real clipboard, and nothing is cleared afterward.
  - No real cryptography: the master password is compared in plain JavaScript with the hardcoded demo password `shelf-demo-2026`, which is shown on the locked screen as a visible demo hint. Secret values are plain fake strings embedded in the page; no encryption, key derivation or secure memory handling is represented. Masking is visual only.
  - Auto-lock is a real in-page countdown from the selected timeout (default 5 minutes), reset by clicks or key presses inside the app; "Skip to timeout" forces it to 0. Nothing is actually cleared or wiped on lock beyond hiding the secret list.
  - No persistence: refreshing the page resets everything, including registered projects, edited notes, settings, relinked paths, Vault lock state and the demo-state toggles. Registering a project, editing a note and relinking a path update only the in-memory data for the current page load.
  - Autosave is simulated by a timer (Saved about 1 second after typing stops); no data is stored. The save failure only occurs when "Simulate save failure" is used.
  - The Markdown preview is a minimal renderer (headings, lists, code blocks, bold, italic); it is not a full Markdown implementation.
  - Search in the quick-open palette is a simple case-insensitive substring match over the mock data.
  - Dates and "last used" values are fixed strings, except where a user action updates them (Restore sets "Last used" to "2026-10-12 09:04", Vault Copy updates nothing).
  - Workspace creation and editing, project edit/archive/delete, secret create/edit, config discovery, backup/restore, import/export and first-run Vault creation are absent.
- **Deferred platforms** (only if the idea targets several platforms: name the secondary ones; they are out of scope for this prototype): None. The idea targets Windows desktop only; there are no secondary platforms.

---

## 6. Assumptions Being Tested

Each assumption is tied to a pain point in `journey-map.md` Section 2 and a flow that exercises it.

- A1 (P3, F3): Restoring a set of files (and folders) is enough value without restoring editor tab state or unsaved buffers. Observe whether the reviewer accepts the "Restore reopens files and folders" note or asks for tab layout and cursor positions.
- A2 (P4, P5, F3): A results summary that leads with opened vs. missing counts, with per-item Relink and Ignore and no silent change to the saved workspace, is enough for a reviewer to trust Restore. Observe whether they notice the missing item, understand the actions, and whether they expect Ignore to remove the reference.
- A3 (P6, P7, F4): Environment as a first-class filter plus a visible label per row is enough to open the right environment's config without opening the wrong file. Observe whether the reviewer reaches the staging file in one filter step and understands the difference between Missing and Inaccessible.
- A4 (P8, P11, F5): Masked-by-default with explicit Reveal and Copy on the list row is not too much friction for everyday use, and a neutral "Copied to clipboard" confirmation is enough reassurance without showing the value. Observe the number of clicks the reviewer takes and whether they ask for a detail screen or a quicker path.
- A5 (P10, F5, F7): Lock-on-inactivity with a visible countdown, a configurable timeout and a clear lock message is not an annoyance. Observe the reaction when "Skip to timeout" is used mid-task, and which timeout the reviewer chooses in Settings.
- A6 (P9, F5): A calm, specific inline message after a wrong master password is enough, without any lockout. Observe whether the reviewer retries naturally or feels uncertain whether the Vault is broken.
- A7 (P2, F2): Registering a project through a short dialog (path, name, type, tags) is light enough that the author will register every project. Observe whether the dialog feels heavier than expected, and whether Browse is expected over typing the path.
- A8 (P12, P13, F6): A three-state autosave indicator and a "Stored locally outside project roots" label are enough to trust notes. Observe whether the reviewer notices the indicator while typing and understands where the note is stored.
- A9 (P1, F1, F8): Home with pinned projects and recent workspaces, plus Ctrl+K, lets the reviewer resume the Monday task without help. Observe the time and path from opening the prototype to opening the first workspace.

---

## 7. Feedback Log

<!-- Fill in as the prototype is reviewed. Classify each item:
     Requirement Change | Business Rule Change | User Flow Change | UI/UX Change | Missing Scenario | Technical Constraint | Out of Scope -->

| Date | Reviewer | Feedback | Classification | Action taken |
|---|---|---|---|---|
| 2026-10-12 | Khoa | Note editor is too small because the preview sits beside it; make preview optional and give the editing space as much room as possible. | UI/UX Change | Preview is now off by default behind a "Show preview" checkbox; editor fills the full content width and most of the window height. Preview shows a 50/50 split when ticked. |

---

## Definition of Done Checklist

- [x] Key requirements from `idea-brief.md` are traceable to at least one screen or interaction
- [x] Main user flows are complete: each has a start point, action, result, and cancel/error path
- [x] Realistic mock data used: no placeholder text like "Item 1" or "Member name"
- [x] Empty state is represented on relevant screens
- [x] At least one validation or error state is demonstrable
- [x] Success state is represented after completing a key action
- [x] All faked / hardcoded elements are listed in Section 5
- [x] Prototype opens as `file://` with no console errors
- [x] CSS serves structural clarity (layout, grouping, hierarchy, error visibility) and every visual value comes from `docs/DESIGN.md`
- [x] Known limitations and assumptions are documented
- [x] Prototype is ready for review using the review questions in the Prototype & Design step
