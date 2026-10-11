# F03 — Editor profiles and open in editor: Spec

- **Status**: `draft — awaiting user approval (Phase 2 Step 1 hard gate)`
- **Roadmap entry**: F03 (Must, size M, depends on F01 — shipped)
- **Feature slug**: `editor-profiles`
- **Spec mode**: Full. No designs in `docs/designs/editor-profiles/`; Phase 1 Step 5 was skipped. Screens follow `docs/DESIGN.md` (dark theme only) and `docs/preview/index.html` (Settings: editor profile list, default marker, executable path fields, Save and Reset), `design_md_fallback`.

---

## 1. Goal

The user defines editor profiles (Windows default, Notepad++, VS Code, custom path), chooses one as the default, and can open a file or folder in a chosen profile. If the file or folder cannot be opened, the user sees the reason and the fix. Later features (F04 restore, F06 config open) call the same open capability.

## 2. Screens

| ID | Screen | Route | Notes |
|---|---|---|---|
| E1 | Settings: Editor profiles section | `/settings` | Added to the F01 Settings screen above the privacy text |
| E2 | Open test | in E1 | Lets the user open a file or folder in a profile, to verify it works before F04/F06 use it |

Auto-lock settings (F08) are not in this feature. The Save and Reset controls in the preview apply to editor profiles only in F03.

## 3. Acceptance Criteria

### Profiles

- **AC-01** There are four profiles: **Windows default**, **Notepad++**, **VS Code**, **Custom**. They are created once when storage is first set up and cannot be deleted.
- **AC-02** Windows default opens the item with the file association the user already has.
- **AC-03** Notepad++, VS Code, and Custom each have one executable path field. Notepad++ and VS Code are pre-named; Custom has an editable display name (max 40 characters).
- **AC-04** Exactly one profile is the default. Windows default is the default on first launch.
- **AC-05** "Set as default" on a profile makes it the default and clears the previous default. Setting the default is saved immediately.
- **AC-06** Executable path fields accept an absolute path to a `.exe` file. Empty is allowed for a profile that is not the default.

### Saving

- **AC-10** Editing an executable path shows Save and Reset changes. Save persists all profile paths and the display name. Reset restores the last saved values.
- **AC-11** After Save, a confirmation "Settings saved" is shown until the user edits again.
- **AC-12** Saving an invalid path (AC-20 to AC-22) is refused and the field shows the error; no other profile changes are saved in that action.

### Path validation (inline, on Save)

- **AC-20** Default profile with empty path (not Windows default): "Enter the path to the editor executable."
- **AC-21** Path does not exist: "Can't find `<path>`. Check the path or pick the editor again."
- **AC-22** Path does not end in `.exe`, or is a folder: "Pick an editor program (.exe)."
- **AC-23** Path contains shell metacharacters or quotes (for example `&`, `|`, `;`, `"`, `^`, `%`): "This path has characters that are not allowed in an editor path."

### Opening

- **AC-30** "Open in editor" takes an item path (file or folder) and a profile. The item is opened with that profile.
- **AC-31** Windows default opens files and folders through the operating system's default association.
- **AC-32** Notepad++, VS Code, and Custom launch the stored executable with the item path as the single argument. No other arguments, no shell.
- **AC-33** If the item path does not exist, opening fails with: "`<path>` was not found. It may have moved or been renamed." Nothing is launched.
- **AC-34** If the profile has no executable path, opening fails with: "Set the path to <profile name> in Settings to open files with it." Nothing is launched.
- **AC-35** If the executable cannot be launched, opening fails with: "<profile name> could not start. Check the path in Settings." Nothing else is tried.
- **AC-36** Opening succeeds silently in the UI with the message "Opened <file name> in <profile name>." Opening never blocks the app; the editor runs as a separate process.

### Open test (E2)

- **AC-40** Settings has an "Open test" section: a path field with "Browse..." (file or folder picker), a profile selector (defaults to the default profile), and an "Open" button.
- **AC-41** The result shows AC-36 or the matching error (AC-33 to AC-35). The path is not stored.

### Safety and boundaries

- **AC-50** Executables are launched only through `std::process::Command` with a fixed program path and an argument list. Shell invocation (`cmd /c`, `powershell`, string concatenation) is forbidden.
- **AC-51** No user-supplied argument template or extra argument is stored or accepted. Each profile stores only its name and executable path.
- **AC-52** The React layer never launches processes or touches the file system beyond the native picker result, which is passed back to Rust.
- **AC-53** Paths and profiles are stored in SQLite through migration 3 with bound parameters.

## 4. Edge Cases

- Executable path with spaces (for example `C:\Program Files\Notepad++\notepad++.exe`): launched correctly, verified by test (constitution, architecture section 8).
- Item path with spaces or Unicode characters: passed as one argument, not split.
- Default profile's executable is deleted after it was saved: opening shows AC-35; the profile is not changed automatically.
- Two Save clicks in a row: idempotent.
- Reset with no unsaved changes: no-op.
- Setting the default while another save is pending: the default change is saved on its own; pending path edits remain unsaved.

## 5. Non-goals (F03)

- Custom command templates or arguments (AC-51).
- Opening from the Workspace, Config, or Home screens (F04, F05, F06, F10). F03 provides the capability and E2.
- Auto-lock settings (F08).
- Creating or deleting profiles beyond the four built-ins.
- Detecting installed editors automatically.
- Watching the launched editor's process or reading its output.

## 6. Decisions Needing Approval

- **D1 — Where the user can open an item in F03.** (a) **Open test section in Settings (E2).** (b) Defer any UI to F04 and keep F03 Rust-only. *Recommended: (a). Without a UI, AC-30 to AC-36 cannot be checked by a user before F04 exists; the test section is small and is removed or kept deliberately later.*
- **D2 — Windows default opener.** (a) **Use `tauri-plugin-opener`**, which the architecture names for default-association opening. (b) Launch `explorer.exe <path>` through Command. *Recommended: (a), as the architecture decided.*
- **D3 — No custom arguments.** (a) **Each profile stores only a program path; the item path is the only argument.** (b) Allow an argument template per profile. *Recommended: (a). Roadmap section 4 excludes arbitrary command execution, and an argument template is a step toward it.*

## 7. Constitution Check

- **P1** No secrets. Pass.
- **P2** Launch and path checks in the `editors` module; migration 3 adds `editor_profiles`; bound parameters; no shell. Pass.
- **P3** Commands have one Args and one Result each; error codes such as `item_not_found`, `profile_path_empty`, `executable_not_found`, `executable_invalid`, `path_unsafe`, `launch_failed`, `profile_not_found`. UI maps codes to copy. Pass.
- **P4** Rust tests: path validation rules (AC-20 to AC-23), command-line construction without a shell (assert the program and args vector), spaces in paths, default switching, migration 3 reopen. Launch tests use a fake executable in a temp dir (a small script or the test binary itself), never a real editor. Vitest for error copy mapping. E2E smoke on the built app. Pass (planned).
- **P5** No network. The opener plugin is local. Pass.

## 8. Open Questions

None beyond Section 6.
