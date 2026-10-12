# F05 — Workspace restore with missing-path report: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F05 (Must, size M, depends on F04 — shipped)
- **Feature slug**: `workspace-restore`
- **Spec mode**: Full. No designs in `docs/designs/workspace-restore/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` and `docs/preview/index.html` (Workspace detail: Restore button, results panel with Relink and Ignore, note on tab layout), `design_md_fallback`.

---

## 1. Goal

The user restores a workspace in one action. DevShelf checks each item, opens the ones that exist with each item's editor profile, and reports the rest. The report leads with opened and missing counts, lists missing or unreachable items separately, and offers Relink and Ignore for each. The user is told that tab layout and unsaved changes are not restored. Restore updates the workspace's "last used" time. Restore never changes any file or folder.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| R1 | Restore button and note | `/workspaces/:id` | Button enabled when the workspace has items |
| R2 | Restore results panel | `/workspaces/:id`, below the header | Shown after Restore; stays until closed or the next Restore |
| R3 | Relink field | inline in R2 for one item | Path field with Apply and Cancel |

The Workspaces list (W1) shows last-used dates; the list ordering and the Home card belong to F10.

## 3. Acceptance Criteria

### Restore action

- **AC-01** The Restore button is shown on the workspace detail screen. It is disabled when the workspace has no items, with the note "This workspace has no items." already shown by F04.
- **AC-02** The note beside Restore reads: "Restore reopens files and folders. Editor tab layout and unsaved changes are not restored."
- **AC-03** One click on Restore checks and opens every item, in position order. The button shows "Restoring…" and is disabled until the run finishes.
- **AC-04** Restore runs once per click. A second click while a run is in progress does nothing.

### Item checks

- **AC-10** Each item is checked before it is opened, by reading its metadata only:
  - Path exists and its kind matches the stored kind (file or folder): **available**.
  - Path does not exist: **missing**.
  - Path exists but is a different kind than stored (for example a folder where a file was stored): **missing** with the reason "Now a folder, not a file" or "Now a file, not a folder".
  - Metadata fails for another reason (drive not ready, network path unreachable, access denied): **unreachable**.
- **AC-11** Available items are opened with their stored editor profile, using the same open capability as F03 (AC-30 to AC-36 of the F03 spec). Windows default uses the OS association; the other profiles launch with the item path as the single argument.
- **AC-12** If opening an available item fails (for example the stored editor's executable was removed), the item is reported under "Could not open" with the F03 error copy for that failure. It is not counted as opened.
- **AC-13** Items are checked and opened in position order. A failed item does not stop the run.

### Results panel

- **AC-20** The panel's first line is the counts: "N opened, M missing, U unreachable, F could not open." Zero counts are shown. When everything opened: "All N items opened."
- **AC-21** Below the counts, the panel lists in this order: Missing items, Unreachable items, Could not open items. Each row shows the item name, full path (monospace), and the reason in plain language.
- **AC-22** Empty groups are not shown. If nothing is missing, unreachable, or failed, no list appears.
- **AC-23** The panel has a Close control. Close hides the panel; the next Restore replaces it.

### Relink

- **AC-30** Each missing or unreachable row has a Relink control. Relink opens an inline path field with Browse file..., Browse folder... (matching the item's stored kind), Apply and Cancel.
- **AC-31** Apply checks the new path with the same rules as F04 AC-12 and the kind must match the stored kind. Errors shown inline: "Enter a file or folder path.", "`<path>` was not found.", "This is a folder. Pick a file." / "This is a file. Pick a folder.", "Enter an absolute path, for example `F:\\work\\notes.txt`."
- **AC-32** Apply saves the new path to the item immediately (the reference changes only because the user changed it). The row then reads as opened on the next Restore; Apply does not open it by itself. The panel shows "Relinked. It will open on the next restore." until the next Restore.
- **AC-33** Relinking to a path that is already another item of the same workspace is rejected with F04's duplicate copy.
- **AC-34** Cancel closes the field and changes nothing.

### Ignore

- **AC-40** Each missing or unreachable row has an Ignore control. Ignore removes the row from the current panel only. It does not delete or change the item, and the item is still checked and reported on the next Restore.
- **AC-41** Ignored rows do not count in the missing or unreachable counts shown after Ignore. The counts line is recalculated to show "N opened, M missing (ignored K)" so the total still matches the workspace.

### Last used

- **AC-50** A Restore run that completes (even with missing items) sets the workspace's last-used time to the current time. A run that fails before any item is checked does not change it.
- **AC-51** The Workspaces list (W1) shows the last-used date and time for each workspace. Workspaces never restored show "Never restored".

### Safety and boundaries

- **AC-60** Restore and Relink read only metadata of the item paths. Nothing in a file or folder is read, written, or changed. The check for "was not found" uses metadata only.
- **AC-61** Opening goes through the F03 open path. The React layer never launches processes or touches the file system; Rust does all checks and opens.
- **AC-62** Restore results are not persisted between app sessions. Only last-used is stored.
- **AC-63** Migration 5 adds `last_restored_at` to `workspaces`. No other schema change.

## 4. Edge Cases

- A workspace whose items all point to missing paths: the panel reads "0 opened, N missing" and no item is opened.
- A folder item that exists but is empty: available; opened as a folder.
- A file item whose extension changed after storage: still checked by kind and path; reported as available.
- Two Restore clicks in quick succession: AC-04.
- Restoring while the app's storage is unavailable: the Restore button shows an error with the storage copy from F04 and no items are opened.
- Relink to a path on a drive that is not ready: the metadata check reports "unreachable" and the Apply shows the same message as Restore's unreachable reason.
- An item whose project was removed: no effect on restore (AC-22 of F04).

## 5. Non-goals (F05)

- Restoring editor tab layout or unsaved buffers (never).
- Reopening a window with a particular layout or position.
- Background or automatic restore on app start.
- Watching paths for changes.
- Relink or Ignore persisted across sessions (Ignore is for the current panel only, D2).
- The Home card (F10) and the missing-item badge there.
- Restoring a workspace on another machine.

## 6. Decisions Needing Approval

- **D1 — Restore runs items one by one or all at once.** (a) **One by one, in position order.** (b) Parallel. *Recommended: (a). Editor launches are cheap but ordered output is easier to read, and one slow UNC path must not reorder the results.*
- **D2 — Ignore scope.** (a) **Current panel only; the item stays and is reported on the next Restore.** (b) Store an "ignored" flag per item. *Recommended: (a). A stored flag would silently hide a broken reference, which the roadmap's missing-path reporting exists to prevent.*
- **D3 — Unreachable versus missing.** (a) **Separate groups, as the roadmap asks ("lists missing or unreachable paths separately").** (b) One group with a reason. *Recommended: (a).*

## 7. Constitution Check

- **P1** No secrets. Pass.
- **P2** Restore logic in the `workspaces` module, which calls the `editors` open function and the path metadata helper. It does not reach into the editors table directly. Migration 5 is a new numbered migration. Bound parameters. Pass.
- **P3** One Args and one Result per command. Codes: `workspace_not_found`, `storage_unavailable`, and the item-level codes already defined (`item_path_not_found`, `item_relative_path`, `item_duplicate`, `item_path_empty`, `item_path_too_long`). The per-item outcomes use a result enum: `opened`, `missing`, `unreachable`, `could_not_open` with the F03 error code. Pass.
- **P4** Rust tests: classification of each item (available, missing, kind mismatch, unreachable simulated through a path that errors other than not-found, where possible), position order, a failed open not stopping the run (with a recording launcher; no real editor), last-used set only on a completed run, Relink kind rules and duplicate rule, Ignore not changing the stored item, and no writes inside item paths (mtimes unchanged). Vitest for count-line and group-ordering helpers. Built-app smoke check before close. Pass (planned).
- **P5** No network. Pass.

## 8. Open Questions

None beyond Section 6.
