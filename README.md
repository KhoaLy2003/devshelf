# DevShelf

DevShelf is a single-user, local-first Windows desktop app that organizes your projects, editor profiles, workspaces, config references, private notes, and an encrypted secrets vault. No cloud, sync, or telemetry. The only network use is the update check.

![Home](docs/screenshots/home.png)

## Features

- **Projects**: register a project by folder and pin the ones you use most.
- **Editor profiles**: add an editor by executable and arguments. Shell syntax is rejected.
- **Workspaces**: build from projects and paths, then restore in one action. The restore result lists missing items, with Relink or Ignore for each.
- **Config catalog**: track config files per project and environment (local, dev, staging, production). Status is Available, Missing, or Inaccessible.
- **Notes**: private Markdown notes linked to a project. Bodies are encrypted and autosave one second after you stop typing.
- **Vault and secrets**: master password of at least 12 characters, with no recovery. Auto-lock after 1, 5, or 15 minutes, or never. Secrets stay masked, reveal hides again after 30 seconds, and copy puts the value on the clipboard without showing it.
- **Home**: pinned projects and recent workspaces at a glance.
- **Appearance**: Dark (default) or Light.

## Requirements

| To run the installed app | To build from source |
| --- | --- |
| Windows 10/11 x64 | Node.js 22 with npm (the version CI uses) |
| WebView2 runtime (preinstalled on Windows 11 and current Windows 10) | Rust stable with the MSVC toolchain and Visual Studio C++ Build Tools |
| | Git. Cargo is at `%USERPROFILE%\.cargo\bin` |

## Install

### Option A: download the installer

Download `DevShelf_<version>_x64-setup.exe` from the [latest GitHub Release](https://github.com/KhoaLy2003/devshelf/releases/latest) and run it.

The installer is not code-signed, so SmartScreen may warn on first run. Choose **More info**, then **Run anyway**.

DevShelf checks GitHub Releases for updates when it starts (turn this off in **Settings > Updates**), and when you click **Check for updates**. It installs an update only after you click **Install and restart**. Your data in `%APPDATA%\app.devshelf.desktop` survives updates and uninstalling, unless you tick **Delete the application data** when uninstalling.

### Option B: build the installer from source

```powershell
git clone https://github.com/KhoaLy2003/devshelf.git
cd devshelf
npm install
npm run tauri build
```

Then run `src-tauri\target\release\bundle\nsis\DevShelf_<version>_x64-setup.exe`. Building the installer needs the updater signing key (see Releasing). To build without it, skip the update artifacts:

```powershell
npm run tauri build -- --config "{\"bundle\":{\"createUpdaterArtifacts\":false}}"
```

### Option C: run in development mode

```powershell
git clone https://github.com/KhoaLy2003/devshelf.git
cd devshelf
npm install
npm run tauri dev
```

## Using DevShelf

1. **Register a project.** Open Projects, choose **Register project**, and pick the folder. Pin it to show it on Home.

   ![Projects](docs/screenshots/projects.png)

2. **Add an editor profile.** In Settings, add an editor with its executable and arguments.
3. **Build and restore a workspace.** Open Workspaces, create a workspace, add projects or paths, then choose **Restore**. Restore reopens files and folders in the order you set. Tab layout and unsaved editor buffers are not restored.

   ![Workspace](docs/screenshots/workspace-restore.png)

4. **Track config files.** In Config Catalog, choose **Add config file** and set the project, environment, and kind.

   ![Config Catalog](docs/screenshots/config-catalog.png)

5. **Write a note.** Open Notes, choose **New note**, and start typing. It saves automatically.

   ![Notes](docs/screenshots/notes.png)

6. **Create the vault, then add secrets.** Open Vault, set a master password of at least 12 characters, and confirm you understand there is no recovery. Then open Secrets and choose **Add secret**. Values stay masked until you reveal or copy them.

   ![Secrets](docs/screenshots/vault-secrets.png)

7. **Switch to Light theme.** In Settings, under Appearance, choose Light.

   ![Settings in Light theme](docs/screenshots/settings-light.png)

The in-app Help screen has full guides, troubleshooting, and the changelog.

## Your data and privacy

- DevShelf stores its data in `%APPDATA%\app.devshelf.desktop`.
- Secrets and note bodies are encrypted. The note encryption key is stored in Windows Credential Manager.
- Project names, tags, environment names, and file paths are stored unencrypted in the local database.
- If you forget the vault master password, the vault cannot be recovered.
- Unregistering a project removes only the registration. Project files are not touched.

## Development

Commands:

- `npm run tauri dev`: run the app in development mode.
- `npm run build`: typecheck and Vite build.
- `npm run test`: Vitest.
- `npm run lint`: ESLint and Prettier check. `npm run format` writes Prettier fixes.
- In `src-tauri/`: `cargo test` and `cargo build`.

Repository layout:

- `src/`: React + TypeScript UI. `src/lib/commands.ts` wraps Tauri commands; `src/types/commands.ts` holds the shared command types.
- `src-tauri/src/`: Rust core. `commands/` holds the Tauri commands. Domain modules are `projects`, `editors`, `workspaces`, `config`, `notes`, `vault`, and `secrets`. `db/` handles SQLite persistence.
- `tests/`: Vitest tests.
- `docs/`: architecture, constitution, design, roadmap, and models.
- `.github/workflows/`: CI (lint, build, Vitest, cargo test) on pushes and pull requests; Release prepare (manual, from the Actions tab) opens the release PR; Release tags the merged `main` commit and builds the draft GitHub Release when the version in `package.json` changes.

Tech stack: Tauri 2 (WebView2), React, TypeScript, Vite, Rust, and SQLite (`rusqlite` with bundled SQLite and `rusqlite_migration`). Details are in [docs/architecture.md](docs/architecture.md).

## Contributing

Coding standards: [docs/constitution.md](docs/constitution.md).

- **File naming**: kebab-case for TypeScript/TSX files and folders (`app.tsx`, `quick-open.ts`); snake_case for Rust files and modules (`project_registry.rs`). One folder per Rust domain module.
- **Tests**: TypeScript tests live in `tests/` and are named `*.test.ts` or `*.test.tsx`. Rust tests live in `#[cfg(test)]` modules beside the code, or in `src-tauri/tests/` for integration tests. Tests use temporary directories, never the real user data folder.
- **Commit messages**: Conventional Commits, `type(scope): summary` in the imperative mood, with types `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci` (for example `feat(projects): add project registry`).
- **Releasing**: in each feature PR, add a bullet for every user-facing change to `release-notes/unreleased.md` under `### Added` or `### Fixed`. Do not use `##` headings, double quotes or backslashes. To release, run the **Release prepare** workflow from the Actions tab with a bump (`patch`, `minor`, `major`, or an explicit `X.Y.Z`). It moves the notes into `release-notes/vX.Y.Z.md`, bumps the version files, regenerates `src/lib/changelog-data.ts`, and opens a release PR that auto-merges once CI passes. When that PR merges, the Release workflow tags the merged commit and builds a signed installer and `latest.json` into a **draft** GitHub Release. Install the draft's `.exe` to smoke test it, then publish it; installed apps only see published releases. Never move or reuse a published tag. Repository secrets: `RELEASE_TOKEN` (a fine-grained token so the release PR triggers CI), `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. While the version is below 1.0, a breaking change or feature is a `minor` bump and a fix is a `patch`; use `major` only from 1.0.0 onward. Preview locally with `npm run release -- <bump> --dry-run`; a local run without `--dry-run` only edits files.

## License

Private project. No license is granted.
