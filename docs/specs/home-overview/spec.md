# F10 — Home overview: Spec

- **Status**: `approved (D1 heading only; D2 Register navigates to Projects; D3 stored missing count at completed restore)`
- **Roadmap entry**: F10 (Must, size S, depends on F02 and F05 — both shipped locally)
- **Feature slug**: `home-overview`
- **Spec mode**: Full. No designs in `docs/designs/home-overview/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` and `docs/preview/index.html` (Home: pinned projects and recent workspaces side by side; empty state), `design_md_fallback`.

---

## 1. Goal

Home is the first screen. It shows what the user is likely to resume: pinned projects and recent workspaces, side by side. Recent workspaces show when each was last used and warn when items were missing at the last restore. Two quick actions (Register project and, once F11 exists, Quick open) start common tasks. When nothing exists yet, Home explains what to do first.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| H1 | Home | `/home` | Replaces the F01 empty placeholder |
| H2 | Home empty state | `/home` when no projects and no workspaces | Guidance and Register button |

Register project opens the F02 dialog. The dialog is reached from Home by navigating to `/projects` with a request to open it; no new dialog is built here (see D2).

## 3. Acceptance Criteria

### Layout

- **AC-01** Home shows two sections side by side on wide windows and stacked on narrow ones: "Pinned projects" and "Recent workspaces". Each has an eyebrow heading as in the preview.
- **AC-02** Each section has its own empty text when that section alone is empty (see AC-20 to AC-22).
- **AC-03** The page heading is "Home". The F01 lead text "Pick up where you left off." is kept.
- **AC-04** The preview's greeting ("Good morning, Khoa") and date line are not built. DevShelf has no user name or profile; the heading is "Home" only (D1).

### Quick actions

- **AC-10** A "Register project" button is always shown. It navigates to `/projects` and opens the F02 Register dialog.
- **AC-11** Quick open is not shown on Home in F10 (its feature is F11). When F11 ships, the button is added then.
- **AC-12** The actions row contains no other buttons.

### Pinned projects

- **AC-20** Pinned projects are listed with display name, root path (monospace), and drive. Sorted as F02 lists them (name, case-insensitive).
- **AC-21** Each pinned project links to its detail page `/projects/:id`.
- **AC-22** With no pinned projects, the section shows "No pinned projects. Pin a project from its detail page to see it here."
- **AC-23** At most 6 pinned projects are shown. If there are more, the section ends with a link "All projects" to `/projects`.

### Recent workspaces

- **AC-30** Recent workspaces are sorted by last used (most recent first). Workspaces never restored come after those restored, sorted by name.
- **AC-31** Each row shows: name, item count ("N items"; "1 item" for one), projects used by its items (or "No registered projects"), and last used as "Last restored <local date and time>" or "Never restored".
- **AC-32** When the last completed restore had missing or unreachable items, the row shows a warning badge: "N items missing at last restore" (or "1 item missing at last restore"). The count is stored at restore time (AC-40).
- **AC-33** Each row links to `/workspaces/:id`.
- **AC-34** With no workspaces, the section shows "No recent workspaces. A workspace saves files and folders you reopen together."
- **AC-35** At most 6 recent workspaces are shown, with an "All workspaces" link to `/workspaces` when there are more.

### Empty state

- **AC-40** Storage stores the number of missing-or-unreachable items at the last completed restore (migration 6, column `last_restore_missing`). A restore that does not complete does not change it. Ignored rows (F05 D2) are not subtracted: the count is the real number of missing and unreachable items at restore time.
- **AC-41** Relinking or later restoring does not change the stored count until the next completed restore.
- **AC-42** When the database has no projects and no workspaces, Home shows H2 instead of the two sections: heading "Nothing here yet", message "Register a project and save a workspace to see pinned projects and recent workspaces on this page.", and the "Register your first project" button (AC-10 action).

### Data and boundaries

- **AC-50** Home reads only through typed commands. The React layer does not run SQL and does not touch the file system.
- **AC-51** Home loads pinned projects and recent workspaces in one command so the page renders from one consistent read.
- **AC-52** No count, date, or name on Home is cached beyond TanStack Query's normal cache; no new persisted UI state is added.

## 4. Edge Cases

- A pinned project whose folder has been deleted: still listed (F02 does not check folders after registration); no status is shown on Home.
- A workspace whose items were all removed: item count "0 items"; restore is disabled there (F04). It still appears in recent workspaces.
- A workspace restored with missing items and then all items deleted: the stored missing count stays as it was at that restore until the next completed restore. This is intended (AC-41).
- Exactly 6 pinned or 6 recent: no "All" link.
- Very long names: truncated with CSS (single line, full name in the title attribute).

## 5. Non-goals (F10)

- Quick open button and palette (F11).
- Greeting, user name, or date line (D1).
- Config catalog items, notes, or secrets on Home (F06, F07, F09).
- Missing-folder checks for pinned projects.
- Editing from Home (no inline pin, rename, or restore on Home; those stay on their detail screens).
- Restore from Home.

## 6. Decisions Needing Approval

- **D1 — Greeting and date line.** (a) **Omit; heading "Home".** (b) Show the date only, no name. *Recommended: (a). DevShelf has no user profile, and a hard-coded name would be made-up data.*
- **D2 — Register project from Home.** (a) **Navigate to `/projects` and open the existing dialog.** (b) Open a second dialog on Home. *Recommended: (a). One dialog implementation, no duplicate validation.*
- **D3 — Stored missing count (AC-40).** (a) **Store the count at each completed restore (migration 6).** (b) Compute from current item paths on each Home load. *Recommended: (a). Computing would probe every item on every Home visit, which is slow on network paths and contradicts the restore-time report the roadmap describes.*

## 7. Constitution Check

- **P1** No secrets. Pass.
- **P2** Home query lives in the `workspaces` and `projects` modules as public functions; no table reach-through. Migration 6 is numbered. Bound parameters. The restore command writes the new count through the existing `mark_restored` path. Pass.
- **P3** One Args and one Result for the home command. Codes reuse existing ones (`storage_unavailable`). Pass.
- **P4** Rust tests: count stored only on completed restores (completed run sets it; a failed run does not); sort order (last used desc, never-restored by name); pinned sort and cap; migration 6 reopen. Vitest for the warning label pluralisation and the list-cap logic. Built-app smoke check before close. Pass (planned).
- **P5** No network. Pass.

## 8. Open Questions

None beyond Section 6.
