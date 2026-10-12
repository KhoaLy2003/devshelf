# Phase 2 Session Log — DevShelf

Models: capable = claude-haiku-5.5 · balanced = claude-haiku-5.5 · fast = claude-haiku-5.5

Phase 2 Scope: `feature` (experimental). Build order: F01 → F02 → F03 → F04 → F05 → F10 → F06 → F07 → F08 → F09 → F11 → F13 → F12 → F14 → F15.

## foundation-and-navigation

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/foundation-and-navigation/spec.md (approved; D1–D4 recommended) |
| 2 | Plan | writing-plans | complete | docs/specs/foundation-and-navigation/plan.md |
| 3 | Implement | task, frontend-developer (no backend-developer agent configured) | complete | branch `feature/foundation-and-navigation` |
| 4 | Review | code-reviewer | complete | 1 major, 7 minor, 3 nit; major and minor fixed; nits 10 and 12 kept |
| 5 | E2E | task (desktop run) | complete | docs/E2E-TESTS.md: 18 PASS, 0 FAIL, 2 BLOCKED |
| 6 | Manual Check | Human | complete (approved "Approve for Ship") | — |
| 7 | Ship | finishing-a-development-branch | complete: local commit `617f7b6`; push and PR blocked, no git remote | roadmap F01 = shipped; constitution 1.1.0 Spec Evolution = flow-forward |

Human decisions: identifier `app.devshelf.desktop` kept; remote: keep local commit until a remote is added.

## project-registry

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/project-registry/spec.md (approved; D1 nested allowed with note; D2 preview types; D3 tauri-plugin-dialog) |
| 2 | Plan | writing-plans | complete | docs/specs/project-registry/plan.md |
| 3 | Implement | task (Rust), frontend-developer (UI) | complete | branch `feature/project-registry` (cut from 617f7b6); not yet committed |
| 4 | Review | code-reviewer | complete | 1 major (relative paths accepted), 8 minor, 3 nit; major and minor fixed (dead helpers removed, tag order, unique-violation code, async commands, stale nested note, picker copy, AGENTS.md); nits 3 and 4 kept (see notes) |
| 5 | E2E | task (desktop run) | complete | docs/E2E-TESTS-F02.md: 24 PASS, 0 FAIL, 0 BLOCKED |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Verification at Step 4 close: `npm run build` ok (161 modules); `npm run lint` ok; `npx vitest run` 5 files, 32 tests; `cargo test` 34 pass; `cargo build` 0 warnings.

Notes:
- Migration 2 applied to the existing F01 database on launch (user_version 1 → 2). Backup: `%APPDATA%\app.devshelf.desktop\devshelf.sqlite3.f01-backup`.
- E2E left test projects in the real data folder: App A, App A Sub, app b (pinned), App C. F02 has no delete (F15). To return to the F01 state, close the app and copy the backup over `devshelf.sqlite3`.
- E2E: the picker's selection path was not exercised; only cancel was. Drive `F:` was not exercised; `%TEMP%` is on `C:`.
- E2E observation: the inline error stays visible while the user edits the field until the next Register. Meets AC-16 as written; flagged for the Step 6 reviewer.
- Review nit 3: the picker has no parent window, so it is not modal to the main window. Left as is.
- Review nit 4: the 260-character limit counts chars, not UTF-16 units. Left as is.
- Spec AC-33 wording was noise; implemented as row link to detail.

## project-registry (shipped)

- Step 6 approved ("Approve for Ship"). Step 7: local commit `dd298ea` on `feature/project-registry`; roadmap F02 = shipped. Push and PR blocked: no git remote.

## editor-profiles

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/editor-profiles/spec.md (approved; D1 Open test in Settings; D2 tauri-plugin-opener; D3 no custom arguments) |
| 2 | Plan | writing-plans | pending | — |
| 3 | Implement | — | pending | — |
| 4 | Review | code-reviewer | pending | — |
| 5 | E2E | task | pending | — |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Next: F03 Step 2 (plan) on a branch cut from `feature/project-registry`, then Steps 3–5.

## editor-profiles

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/editor-profiles/spec.md (approved; D1 Open test in Settings; D2 tauri-plugin-opener; D3 item path only) |
| 2 | Plan | writing-plans | complete | docs/specs/editor-profiles/plan.md |
| 3 | Implement | task (Rust), frontend-developer (UI) | complete | branch `feature/editor-profiles` (cut from dd298ea); not committed |
| 4 | Review | code-reviewer | complete | 1 major (open-time error copy for deleted exe), 6 minor, 4 nit; major and minor fixed; nits F9 kept, F10 `_kind` kept, F11 noted for F04 |
| 5 | E2E | task (dev-mode run) | complete with 1 FAIL fixed (C17 stale error after save); 21 PASS, 2 PARTIAL, 1 BLOCKED | docs/E2E-TESTS-F03.md |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Verification at fix close: `npm run build` ok; `npm run lint` ok; `npx vitest run` 55 pass; `cargo test` 67 pass; `cargo build` 0 warnings.

Open gaps for the Step 6 reviewer:
- E2E ran against `tauri dev`, not the built app. Constitution P4 requires a smoke run on the built app before the phase closes. F01 and F02 used the same mode. A built-app run is still outstanding for all three features.
- C17 (stale error after save) is fixed but the re-check was not re-run in E2E; only the build was verified after the fix.
- C08b: the Notepad++ profile launch could not be observed. A copy of System32 notepad.exe exits with code 0 and shows no window. The spawn path is covered only by the Rust tests (argument vector, spaced path, spawned copy exits cleanly).
- Set-default with an empty path is refused with AC-20 copy. Spec AC-05 reads as if this should succeed; this is a spec clarification to confirm.

### editor-profiles: Step 6 and Step 7

- Step 6 approved ("Approve for Ship"). Step 7: local commit `bdbb3c5` on `feature/editor-profiles`; roadmap F03 = shipped. No remote, so no push or PR.
- Built-app smoke still outstanding for F01–F03 (dev-mode runs only).
- Next pending in build order: F04 Workspace builder (depends on F02, F03). Restarts at Step 1 when the user says go.

## workspace-builder

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/workspace-builder/spec.md (approved; D1 reject duplicate paths; D2 default profile stored on the item at add time; D3 move up/down buttons) |
| 2 | Plan | writing-plans | complete | docs/specs/workspace-builder/plan.md |
| 3 | Implement | task (Rust), frontend-developer (UI) | complete | branch `feature/workspace-builder` (cut from ad0bd40); not committed |
| 4 | Review | code-reviewer | complete | 2 major (0-based numbering shown; command mapping untested), 6 minor, 4 nit; majors and minors fixed (unique-violation code, reject `?`/`.` UNC server, focus and Escape on confirmations, delete navigation order, error routing to selects, spec wording); nit N4 (N+1 query) accepted |
| 5 | E2E | task (dev-mode run) | complete | docs/E2E-TESTS-F04.md: 30 PASS, 0 FAIL, 0 BLOCKED |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Verification at fix close: `npm run build` ok; `npm run lint` ok; `npx vitest run` 72 pass; `cargo test --lib` 91 pass; `cargo build` 0 warnings.

Open gaps for the Step 6 reviewer:
- Persistence check used a webview reload, not a full app restart (AC-20). Reopen behaviour is covered by the Rust persistence test.
- E2E ran against `tauri dev`, not the built app (constitution P4). Still outstanding for F01–F04.
- Test data from earlier E2E runs remains in the real database (projects App A, App A Sub, app b, App C) — intentionally kept; workspaces were all deleted.

## workspace-restore

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/workspace-restore/spec.md (approved; D1 one by one; D2 Ignore current panel only; D3 missing and unreachable separate) |
| 2 | Plan | writing-plans | complete | docs/specs/workspace-restore/plan.md |
| 3 | Implement | task (Rust), frontend-developer (UI) | complete | branch `feature/workspace-restore` (cut from eedf4c5); not committed |
| 4 | Review | code-reviewer | complete | APPROVE; 3 minor fixed (relink to unreachable path, report lost when last-used save fails, profile-not-found test); TS handler fixed for partial completion; nits kept |
| 5 | E2E | task (dev-mode run) | complete | docs/E2E-TESTS-F05.md: 26 PASS, 0 FAIL, 2 BLOCKED (R04 covered by R04b; R23 unreachable path unavailable on this machine) |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Verification at fix close: `npx tsc --noEmit` ok; `npm run build` ok; `npm run lint` ok; `npx vitest run` 89 pass; `cargo test --lib` 119 pass; `cargo build` 0 warnings.

Open gaps for the Step 6 reviewer:
- Unreachable group never displayed in the UI E2E (no safe unreachable path). Covered by Rust tests only.
- Dev-mode run, not the built app (constitution P4), still outstanding for F01–F05.
- Two earlier advisories (M2 command-mapping tests; WorkspaceFormContract lint/tsc) were checked: the tests exist and tsc, build and lint pass.

## home-overview

| Step | Name | Agent | Status | Output |
|---|---|---|---|---|
| 1 | Spec | brainstorming | complete | docs/specs/home-overview/spec.md (approved; D1 heading only; D2 Register navigates to Projects; D3 stored missing count) |
| 2 | Plan | writing-plans | complete | docs/specs/home-overview/plan.md |
| 3 | Implement | task (Rust), frontend-developer (UI) | complete | branch `feature/home-overview` (cut from fbf18da); not committed. Red-first step skipped for the domain tests (process deviation) |
| 4 | Review | code-reviewer | complete | CHANGES REQUIRED: 1 major (Home stale after restore; fixed with refetch on visit and invalidations), 4 minor (1 spec status fixed, 1 comment fixed, 2 accepted) |
| 5 | E2E | task (dev-mode run) | complete with 1 FAIL fixed (empty state wrongly shown with unpinned projects; now counts projects and workspaces per AC-42); 19 PASS, 2 FAIL (fixed), 4 BLOCKED | docs/E2E-TESTS-F10.md + re-check section |
| 6 | Manual Check | Human | pending (hard gate) | — |
| 7 | Ship | finishing-a-development-branch | pending | — |

Verification at fix close: `npx tsc --noEmit` ok; `npm run build` ok; `npm run lint` ok; `npx vitest run` 98 pass; `cargo test --lib` 131 pass; `cargo build` 0 warnings.

Open gaps for the Step 6 reviewer:
- BLOCKED: AC-23 (more than 6 pinned), AC-35 (more than 6 recent), AC-42 full run (projects cannot be removed; the empty state was verified by logic and the re-check shows sections instead), storage-failure state (needs broken DB).
- Date format differs between Home ("12 Oct 2026, 19:37") and the Workspaces list ("10/12/2026, 7:06:37 PM"). Not fixed; decide one format at Step 6.
- Dev-mode run, not the built app (constitution P4), still outstanding for F01–F10.
- The re-check cleanup used a name-based kill of node.exe, which may have ended unrelated Node processes.
