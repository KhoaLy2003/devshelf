# F04 — Workspace builder: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F04 (Must, size M, depends on F02 and F03 — both shipped locally)
- **Feature slug**: `workspace-builder`
- **Spec mode**: Full. No designs in `docs/designs/workspace-builder/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` (dark theme only) and `docs/preview/index.html` (Workspaces list, Workspace detail, item table), `design_md_fallback`. Restore is not built here (F05): the preview's Restore button and results panel are not rendered.

---

## 1. Goal

The user builds named workspaces: an ordered list of files and folders that they reopen together later. Each item can come from any registered project or from no project, and each item has its own editor profile. Creating, renaming, deleting, reordering, and removing items never changes the files or folders they point to.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| W1 | Workspaces list | `/workspaces` | Replaces the F01 empty state when workspaces exist |
| W2 | New workspace dialog | modal on W1 | Name only |
| W3 | Workspace detail (builder) | `/workspaces/:id` | Ordered item table; add, edit, reorder, remove; rename; delete |
| W4 | Add item dialog | modal on W3 | Path via picker or typed, project, editor profile |

Last-used, item-status checks, restore, relink, and missing-path reporting belong to F05. Home (F10) shows nothing from this feature.

## 3. Acceptance Criteria

### Workspaces

- **AC-01** W1 lists every workspace with name, item count, and the project names used by its items (distinct, sorted, "No registered projects" when none).
- **AC-02** Empty state (F01 copy) with a "New workspace" button when there are none.
- **AC-03** "New workspace" opens W2 with a name field. Create saves the workspace and opens W3 for it.
- **AC-04** Workspace names are 1–100 characters after trimming, must not be blank, and must be unique case-insensitively. Errors appear inline: "Enter a workspace name.", "Use at most 100 characters.", "A workspace named `<name>` already exists."
- **AC-05** Rename on W3 uses the same rules; the error keeps the typed value.
- **AC-06** Delete on W3 asks for confirmation: "Delete workspace `<name>`? Its files and folders are not changed." Confirm deletes the workspace and its items, then returns to W1. Cancel keeps it.

### Items

- **AC-10** W3 shows items in order with: position number, kind (file or folder), file or folder name, full path (monospace), project name or "No project", and editor profile name.
- **AC-11** "Add item" opens W4 with: "Browse file...", "Browse folder...", or a typed absolute path; a project selector ("No project" or any registered project); an editor profile selector (the four profiles from F03, defaulting as decided in D2).
- **AC-12** The path must be an existing absolute path to a file (kind `file`) or folder (kind `folder`), checked when the item is added. Errors: "Enter a file or folder path." / "`<path>` was not found." / "Enter an absolute path, for example `F:\\work\\notes.txt`." The check reads metadata only; nothing in the file or folder is read or changed.
- **AC-13** Adding an item appends it at the end. Adding saves immediately.
- **AC-14** Remove deletes the item from the workspace after confirmation: "Remove `<name>` from this workspace? The file or folder is not changed." Other items keep their order, and positions renumber.
- **AC-15** Move up and move down change the order of an item and save immediately. The first item cannot move up; the last cannot move down (controls disabled).
- **AC-16** Changing an item's project or editor profile on W3 saves immediately. The item's path and kind cannot be edited; to change the path, remove the item and add it again.
- **AC-17** Items with the same path in one workspace are handled by D1.

### Persistence and boundaries

- **AC-20** Workspaces and items persist across restarts through migration 4.
- **AC-21** Item paths are stored as typed after normalisation (same rules as F02 paths: absolute, separators normalised, trailing separator removed except on drive roots). Saved references are never changed by the app; only the user's edits change them.
- **AC-22** Deleting or renaming a project (F15 later; currently unreachable) must not break items: the item keeps its path and shows "No project" if its project no longer exists. Until F15, project_id values always resolve.
- **AC-23** Nothing reads, creates, or modifies any file or folder inside a workspace item beyond the metadata check in AC-12. The database stores references only.
- **AC-24** The React layer never touches the file system. Pickers run through Rust, as in F02 and F03.
- **AC-25** Every SQL statement uses bound parameters; item order is kept in a position column updated in one transaction for each move, remove, or add.

## 4. Edge Cases

- Workspace with no items: W3 shows "This workspace has no items." and the Restore control is not present (F05).
- Two workspaces with the same name in different case: rejected by AC-04.
- Adding a path that is already an item in the same workspace: handled by D1.
- A project's root folder added as an item of that project: allowed; the item is a folder kind.
- Path to a drive root (`F:\`): allowed as a folder.
- Deleting the editor profile's executable later: not checked here; the item keeps the profile kind. F05 and F03's open path report the error when opening.
- Reordering while another save is in flight: the last successful transaction wins; the UI shows the list returned by the save.
- Very long path (over 260 characters): rejected with "This path is too long for Windows." (same copy as F02).

## 5. Non-goals (F04)

- Restore, opening items, missing or unreachable status, relink, ignore, last-used (F05).
- Drag-and-drop reordering (D3 decides the reorder control).
- Editing an item's path in place.
- Quick open for workspaces (F11).
- Import or export of workspaces (F12 covers backup).
- Automatic project detection for items.
- Workspace tab layout or unsaved editor buffers (never restored).

## 6. Decisions Needing Approval

- **D1 — Same path added twice to one workspace.** (a) **Reject with "`<path>` is already in this workspace."** (b) Allow duplicates. *Recommended: (a). Duplicates would open the same file twice in F05 and make the missing-path report confusing.*
- **D2 — Default editor profile for a new item.** (a) **The profile that is the default when the item is added; stored on the item, so changing the default later does not change existing items.** (b) Always follow the current default at restore time (store nothing). *Recommended: (a). "Saved references are never changed unless the user changes them" (roadmap F04) requires the stored value to stay fixed.*
- **D3 — Reorder control.** (a) **Move up and move down buttons.** (b) Drag and drop. *Recommended: (a). It works with keyboard and screen readers and needs no extra dependency.*

## 7. Constitution Check

- **P1** No secrets stored. Pass.
- **P2** Workspace logic in `workspaces` module; items reference projects by id and editors by kind (no table reach-through into `projects` or `editor_profiles`; the module calls the projects and editors public functions). Migration 4 adds `workspaces` and `workspace_items`. Bound parameters. Pass.
- **P3** One Args and one Result per command; error codes such as `name_empty`, `name_too_long`, `name_taken`, `workspace_not_found`, `item_path_empty`, `item_not_found`, `item_relative_path`, `item_duplicate`, `path_too_long`, `project_not_found`, `profile_not_found`, `storage_unavailable`, mapped to copy by the UI. Pass.
- **P4** Rust tests: name rules, case-insensitive uniqueness, path rules (absolute, existing, kind detection), duplicate rule per D1, position integrity after add, remove, move (including first and last), persistence across reopen, cascade delete of items on workspace delete, and no writes into item paths (entries and mtimes unchanged). Item ordering is server-side and covered by the Rust tests. Vitest covers error copy. Built-app smoke check before close (see the Step 5 note in the session log). Pass (planned).
- **P5** No network. Pass.

## 8. Open Questions

None beyond Section 6.
