# idea-brief.md

## Metadata

- **Status**: `approved`
- **Last updated**: 2026-10-10
- **Owner**: Solo developer (project author); sole user

---

## 1. Problem

1. Work spans many repositories on different drives. The working set of files (`.env`, `application*.yml`, Docker Compose, Caddy/Nginx configs, deployment notes) lives as open editor tabs. When the editor session closes or becomes unwieldy, each file must be found and reopened by hand.
2. Credentials, API keys, connection strings and tokens are scattered across plain-text files, a password manager and pasted snippets, with no link to the project or environment they belong to.
3. Project knowledge (setup guides, deployment notes, debugging records, checklists) is kept in text files or a general notes app, disconnected from the project, and at risk of landing in a repository.

**Existing alternatives**

- Editor tabs and loose text files (Notepad++ / VS Code sessions): fragile, no environment or project metadata, nothing restorable across restarts.
- A general password manager plus a general notes app: not project- or environment-aware, and separate from the files that use the secrets.
- Nothing structured: memory and directory navigation.

---

## 2. Target Users

- **Primary user**: The author, a developer who works on several active projects at once, uses Notepad++, VS Code and command-line tools on Windows, and handles local configuration and credentials daily.
- **Context of use** (when/where they hit this problem): Switching between tasks or projects, after restarting the machine or closing the editor, and when looking for the right environment's config or credential.
- **Early adopter profile** (who would try this first, before it's polished): The author. The first release is for personal use only. Team or company rollout is not a goal.

---

## 3. Proposed Solution

**One-sentence description**: A local-first Windows desktop app that registers projects, saves and restores named sets of file references, catalogs config files by environment, stores secrets in an encrypted vault, and keeps private project notes outside repositories, while editing stays in external tools.

**Why this approach**: It is a management layer over the existing filesystem and editors, so nothing already on disk is copied, rewritten or replaced. Encrypting protected data at rest keeps it safe in a copied database while keeping everything offline.

---

## 4. Rough Scope

**In scope for MVP** (directional; `roadmap.md` formalizes it):

- Project registry, tags, file and folder references, cross-project workspace collections, restore with missing-path reporting
- External editor profiles (Windows default, Notepad++, VS Code) and quick open
- Config file catalog by project, environment and kind, with bounded discovery
- Private project notes, Markdown, autosave, stored outside project roots and encrypted at rest
- Encrypted secrets vault (master password, mask by default, explicit reveal/copy, auto-lock)
- Encrypted backup and restore that is tested by restoring
- Delivery order as in the kickoff: Foundation, Project & Workspace, Config Catalog & Notes, Vault, Backup & hardening

**Explicitly out of scope / no-gos** (prevents scope creep later):

- Embedded code editor or terminal; replacing any editor
- Arbitrary command execution; automatic secret injection into processes or OS environment variables
- Automatic config rewriting, synchronization or deployment
- Cloud sync, team sharing, multi-user access, remote secrets service
- Repository-wide content indexing by default
- Git operations or a source-control client
- Restoring unsaved editor buffers or internal editor tab state

---

## 5. Known Constraints

- Platform (verbatim from the kickoff): "Windows desktop"; offline by default with no account or cloud dependency.
- Stack (stated by the user as a hard constraint): Tauri 2, React, TypeScript, Rust, SQLite.
- Solo developer; single user.
- Product principle: never silently modify project or config files; never log or leak secret or note content.
- Rust/cargo is not currently installed on the development machine.

---

## 6. Open Questions

- Final product name (working title: DevShelf) and supported Windows versions.
- Vault crypto parameters (KDF/AEAD) and which metadata is encrypted vs. visible in a copied database.
- Auto-lock timeout, clipboard-clearing default, and re-authentication before reveal/copy.
- Whether encrypted backup is mandatory in the first usable release.
- Initial config-discovery patterns and whether discovery may include user-selected folders.
- Performance targets and expected maximum project/file counts.
- Whether any company policy restricts storing production credentials in a local vault.
