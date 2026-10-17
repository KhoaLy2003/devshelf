# Project unregister and vault delete: spec

Status: DRAFT, awaiting approval. Items marked **[DECIDE]** need an answer before approval; the proposed default is shown.
Scope: reinstates a narrowed F15 (unregister only; name, type and tag editing stays out) and adds vault deletion. Both are requested for the first MVP release.
Spec mode: full. Constitution P1 (secrets stay in Rust), P2 (typed commands), P4 (tests), P5 (no network).

## Part A: Unregister a project

### Acceptance criteria

- **AC-A01** A project detail view has an "Unregister" action.
- **AC-A02** Unregister shows a confirmation: "Unregister this project? Its files and folders are not changed or deleted." with Cancel and Unregister.
- **AC-A03** Confirming removes the project registration. The project disappears from Projects, Home, and every project picker.
- **AC-A04** Unregister never deletes, moves, or modifies any file or folder on disk. This is a hard rule, tested by checking the disk before and after.
- **AC-A05** Config catalog entries for the project are removed. **[DECIDE: remove them (proposed), or keep them as "Project removed"?]** Proposed: removed, because the current schema already cascades them and their meaning depends on the project.
- **AC-A06** Workspaces for the project are removed with their items. **[DECIDE: remove them (proposed), or keep them as "Project removed"?]** Proposed: removed, because a workspace is a project-scoped setup and restoring it without the project has no meaning.
- **AC-A07** Notes linked to the project keep their content and show "Project removed" (consistent with approved F09 AC-12). **[DECIDE: confirm.]**
- **AC-A08** Secrets linked to the project keep their values and show "Project removed" (approved F09 AC-12).
- **AC-A09** The confirmation states how many config entries and workspaces will be removed (for example "This also removes 2 config entries and 1 workspace.").
- **AC-A10** A project that is pinned is unregistered like any other; the pin goes with it.
- **AC-A11** Unregistering when the project no longer exists shows "This project is already removed." and changes nothing.
- **AC-A12** Editor profiles are not affected.

## Part B: Delete the vault

### Acceptance criteria

- **AC-B01** The Vault status view has a "Delete vault" action, reachable only when the vault is set up.
- **AC-B02** Deleting requires the current master password, even if the vault is unlocked. The password is checked with the same verifier as unlock. **[DECIDE: require password (proposed), or allow when unlocked?]** Proposed: require password, so a passerby at an unlocked screen cannot delete it.
- **AC-B03** The confirmation states: "Delete the vault and all N secrets? The secrets cannot be recovered. Notes are not affected." It requires typing DELETE to enable the final button.
- **AC-B04** A wrong password shows "Wrong master password." and deletes nothing.
- **AC-B05** On success, all secret rows and the vault row are removed in one transaction. The app returns to the vault setup form.
- **AC-B06** Notes are unaffected; they keep their own key (F07).
- **AC-B07** The vault key is cleared from memory on delete.
- **AC-B08** Deleting a vault is never logged and no secret value is written anywhere.

## Edge cases

- Unregister while a note for that project is open: the editor shows the project as removed on next load; text is not lost.
- Vault delete while a secret is revealed: the reveal is cleared (vault state changes to not set up).
- Unregister from a second window: not applicable (single window).
- Deleting a vault with zero secrets: confirmation says "Delete the vault? No secrets are stored."

## Non-goals

- Editing project name, type or tags (F15 narrowed; still deferred).
- Archiving a project.
- Recovering a deleted vault or deleted secrets.
- Export before delete (F12 remains deferred).

## Constitution check

- P1: the password is compared in Rust; nothing secret reaches the webview. Deletion clears the key.
- P2: project delete goes through the projects module; config and workspace cascades are performed by the owning modules, not by SQL in the projects module.
- P4: tests cover AC-A04 (disk unchanged), AC-B02 and AC-B04 (wrong password), AC-B05 (transaction).
- P5: no network.

## Decisions needed

1. AC-A05: config entries removed with the project (proposed) or kept as "Project removed"?
2. AC-A06: workspaces removed with the project (proposed) or kept as "Project removed"?
3. AC-A07: notes kept as "Project removed" (proposed).
4. AC-B02: vault delete requires the master password (proposed) or only an unlocked vault?
