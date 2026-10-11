# DevShelf — Product Kickoff & Technical Brief

**Document type:** Product and Engineering Kickoff  
**Status:** Proposed baseline for product, engineering, security, and QA review  
**Version:** 0.1  
**Date:** 10 October 2026  
**Working title:** DevShelf (name to be confirmed)  
**Initial platform:** Windows desktop  
**Primary language:** English  

---

## 1. Executive Summary

DevShelf is a local-first desktop application for internal developer use. It brings together project discovery, workspace restoration, configuration-file references, environment organization, encrypted secrets, and private project notes in one place. Users continue to edit files in the external tools they already use—such as Notepad++, Visual Studio Code, or an IDE—rather than switching to a new embedded editor.

The product addresses a recurring workflow problem: developers work across many repositories and configuration files, lose track of open tabs, repeatedly navigate directories to reopen files, and keep project notes and credentials in disconnected locations. DevShelf should reduce this friction without changing repository contents or becoming another source of truth for files that already exist on disk.

The proposed implementation is a Windows desktop app built with **Tauri 2, React, TypeScript, Rust, and SQLite**. The application is local-first and works offline. Its architecture should allow optional synchronization to be added later without requiring a rewrite of the core data model.

The product has four main capabilities:

1. **Project & Workspace Management** — register projects, create file collections, pin frequently used resources, and restore a saved set of files.
2. **Configuration Catalog** — register and categorize configuration files by project, type, and environment, then open them in an external editor.
3. **Encrypted Secrets Vault** — store credentials, API keys, tokens, connection strings, and environment values encrypted at rest; reveal or copy plaintext only through an explicit user action.
4. **Private Project Notes** — create searchable notes, checklists, command references, and technical records associated with a project but stored outside the project repository.

The first release should prioritize safe, reliable local workflows. It should not attempt to become a full IDE, a general-purpose remote secrets platform, or a deployment automation tool.

## 2. Background and Problem Statement

Developers often use a text editor as a makeshift dashboard for operational knowledge. A large number of tabs may include `.env` files, Spring configuration, Docker Compose files, reverse-proxy configuration, deployment notes, and miscellaneous text files. When an editor session closes or becomes unwieldy, users must remember and navigate to each file again.

The underlying needs are broader than tab management:

- Find a project and its important files quickly, regardless of disk location.
- Group files from multiple directories and repositories into a task-specific workspace.
- Reopen a previous workspace after closing the application or restarting the machine.
- Distinguish configuration files by environment and purpose.
- Store credentials and secrets in a protected local vault instead of scattered plain-text notes or copied values.
- Keep technical notes linked to projects without adding files to source repositories or accidentally committing notes.
- Preserve the ability to use existing editors and tools.

## 3. Product Vision

**Vision:** Give developers a secure, local-first home for the projects, configuration references, secrets, and notes they need to work effectively across multiple codebases.

**Product principle:** DevShelf is a management layer over the user's filesystem and existing tools—not a replacement for them.

### 3.1 Product Goals

- Reduce the time and effort needed to locate and reopen frequently used project files.
- Make workspace collections persistent, reusable, and easy to restore.
- Provide a clear catalog of configuration files and their environment associations.
- Protect secret values and private notes with application-level encryption at rest.
- Make plaintext access deliberate, short-lived where feasible, and auditable at the application-event level without recording secret contents.
- Keep project notes outside project roots by default.
- Work offline without requiring an account, cloud service, or local server.
- Establish stable IDs, schema migrations, and export/import foundations for future synchronization.

### 3.2 Non-Goals for the Initial Release

- Replacing Notepad++, VS Code, IntelliJ IDEA, or any other editor.
- Running arbitrary shell commands or acting as a general-purpose terminal.
- Automatically injecting secrets into every process or modifying OS environment variables.
- Automatically rewriting, synchronizing, or deploying configuration files.
- Storing, rotating, or distributing production secrets for an organization as a centralized server product.
- Cloud sync, team sharing, multi-user access control, or a remote API in the MVP.
- Indexing the contents of every file in every repository by default.
- Managing Git operations or replacing a source-control client.

## 4. Target Users and Primary Use Cases

### 4.1 Primary User

A developer who works on several active projects, uses multiple editors and command-line tools, and needs quick access to each project's local configuration, environment-specific files, credentials, and technical notes.

### 4.2 Core Use Cases

1. **Register a project:** select a folder, assign a display name and type, and optionally add tags.
2. **Build a workspace:** select configuration files from one or more projects and save them as a named collection.
3. **Restore a workspace:** reopen registered files in the configured external editor and report missing paths without deleting metadata.
4. **Catalog configuration:** register `.env`, `application*.yml`, Docker Compose, Caddy, Nginx, and other files manually or via user-defined patterns.
5. **Store a secret:** add a name, project, environment, type, value, and optional tags; encrypt the value before persisting it.
6. **Copy a secret:** unlock the vault if required, explicitly copy the value, and apply best-effort clipboard hygiene.
7. **Write a note:** create a project-linked note without adding it to the repository.
8. **Search:** locate projects, workspace names, file paths, tags, and—while unlocked—secret and note information within the agreed privacy model.
9. **Recover:** back up and restore local metadata and encrypted records, with validation and clear warnings for files missing on the current machine.

## 5. Product Scope and Priorities

Priority definitions:

- **P0 — MVP required:** necessary to deliver the core product promise.
- **P1 — Next increment:** valuable, but can follow after the core workflows are dependable.
- **P2 — Later:** deferred until requirements and security design are validated.

### 5.1 P0 — Project & Workspace Management

- Add, edit, pin, tag, archive, and remove a project registration.
- Store a project root path; do not copy or move project files.
- Support projects on different local drives and under different parent directories.
- Validate whether a registered path currently exists and is accessible.
- Register files or folders individually, including references outside the project root.
- Create, rename, duplicate, and delete workspace collections.
- Store ordered workspace items and allow an item to be a project, folder, or file reference.
- Restore a workspace by attempting to open existing file/folder references in the configured external application.
- Report unavailable paths and let users relink or ignore them.
- Preserve metadata when a path is temporarily missing.

### 5.2 P0 — External Editor Integration

- Support the Windows default application for a file or folder.
- Allow a user to configure one or more editor profiles, such as Notepad++ and VS Code.
- Allow a default editor profile and, where practical, file-type-specific preferences.
- Validate that the configured executable exists before attempting to launch it.
- Pass executable arguments safely as discrete arguments; do not concatenate arbitrary user content into a shell command.
- Provide actionable errors when an editor cannot be found or a launch fails.
- Do not claim to restore internal editor state such as unsaved buffers, cursor position, split panes, or editor-specific tabs unless a future integration explicitly supports it.

### 5.3 P0 — Configuration Catalog

- Register configuration files manually.
- Optionally discover files using user-configurable patterns, for example:
  - `.env*`
  - `application*.yml` and `application*.yaml`
  - `application*.properties`
  - `docker-compose*.yml` and `docker-compose*.yaml`
  - `Caddyfile`, `nginx.conf`, and similar explicitly configured names
- Associate each registered file with a project where applicable, environment, file kind, tags, and notes/description.
- Display whether a path exists and when it was last checked or detected as changed, where reliable.
- Keep file references distinct from Vault-managed secret values.
- Do not read, index, or persist the content of registered config files unless a user explicitly invokes a feature that requires it.
- Do not automatically edit configuration files or synchronize file contents with the Vault.

### 5.4 P0 — Project Notes

- Create, edit, rename, pin, tag, search, and delete notes.
- Associate a note with a project or leave it unassociated as a general note.
- Support Markdown-oriented content: headings, lists, checklists, links, and code blocks.
- Provide autosave with an observable saved/unsaved state and safe handling of write failures.
- Store notes in DevShelf's application data area, not in project roots by default.
- Encrypt note content at rest using the Vault's data-protection architecture. The proposed default is that all note content is protected, including notes that appear operational rather than secret-bearing.
- Keep note data independent of project path existence so that moving or removing a project does not automatically delete its notes.

### 5.5 P0 — Encrypted Secrets Vault

- Create and unlock a local Vault using a master password.
- Store secret records with a label, value, type, associated project (optional), associated environment (optional), tags, and timestamps.
- Support initial secret types such as API key, access token, password, database credential, connection string, OAuth client secret, cloud credential, and generic secret.
- Encrypt secret values before they reach persistent storage.
- Keep plaintext secrets out of logs, crash messages, analytics, telemetry, and ordinary UI state persistence.
- Mask values by default; require an explicit reveal or copy action.
- Copy values only after the Vault is unlocked and the user explicitly requests the action.
- Provide lock, manual lock, inactivity timeout, and lock-on-application-exit behavior.
- Provide an explicit delete flow with a clear warning; remove records transactionally and document the limits of secure deletion on SSDs and database/WAL files.
- Store only the minimum metadata necessary in plaintext. Assess whether labels, note titles, environment names, and associations should also be encrypted; this trade-off must be documented before implementation.
- Provide encrypted backup/export and a recovery procedure. A backup is not successful unless it can be restored and unlocked in a test.

### 5.6 P0 — Local Search and Navigation

- Search projects, workspace names, tags, and registered paths.
- Provide keyboard-friendly quick open and recent items.
- When the Vault is unlocked, optionally search secret labels and note content according to a defined in-memory search policy.
- Do not create a plaintext on-disk search index for encrypted notes or secrets.
- Clear any transient decrypted search index on lock.

### 5.7 P1 — Config Inspection and Quality Checks

- Text diff between two registered configuration files.
- Validation for supported formats (initial candidates: JSON, YAML, and Java properties, subject to library evaluation).
- Compare declared variable names in `.env.example` with a selected `.env` file, without logging values.
- Detect duplicate variable names and unresolved placeholders where format semantics are understood.
- Show file-change status and allow users to relink missing or moved files.
- Clearly distinguish a syntax check from semantic validation; do not imply that a syntactically valid configuration is production-safe.

### 5.8 P1/P2 — Backup, Import, and Future Sync

- Versioned JSON export/import for non-secret metadata.
- Encrypted Vault backup with integrity checks and a documented recovery flow.
- Optional export of notes with an explicit choice about whether content is decrypted; encrypted export is the safe default for private data.
- Future device synchronization, only after conflict handling, key management, recovery, and threat modeling are designed.
- Do not synchronize plaintext secrets or notes by default.

## 6. Proposed User Experience

### 6.1 Primary Navigation

- **Home:** pinned projects, recent workspaces, recent files, and quick actions.
- **Projects:** project list, tags, root path, registered files, and associated notes/secrets.
- **Workspaces:** named collections with ordered file/folder references and restore actions.
- **Config Catalog:** filters for project, environment, file kind, and status.
- **Vault:** lock/unlock, secret list, detail/reveal/copy, and secret organization.
- **Notes:** project notes and general notes with Markdown-oriented editing.
- **Settings:** editor profiles, startup behavior, auto-lock timeout, backup location, and privacy/security preferences.

### 6.2 Workspace Restoration Behavior

1. User selects a workspace and requests Restore/Open.
2. DevShelf validates every registered path.
3. Valid paths are grouped by destination/editor profile.
4. The application opens the selected paths using the appropriate OS/editor integration.
5. Missing or inaccessible references are reported in a summary.
6. The application does not delete stale references or silently change their path.
7. The workspace's last-used timestamp and non-sensitive status metadata are updated.

### 6.3 Secret Copy Behavior

1. User selects a secret and clicks Copy.
2. If locked, the application requests unlock or appropriate re-authentication.
3. Rust verifies that the operation is allowed and decrypts only the selected value.
4. The value is copied to the OS clipboard through a controlled operation.
5. The UI shows a neutral confirmation; it must not display or log the value.
6. A configurable best-effort clipboard-clearing timer may clear the clipboard only if the clipboard still contains the copied value, to avoid overwriting newer user content. This cannot remove clipboard-history entries, synced clipboard copies, or copies already pasted elsewhere.
7. Plaintext buffers should be kept short-lived and cleared on a best-effort basis.

### 6.4 Lock and Shutdown Behavior

- Locking invalidates the active in-memory Vault key and clears decrypted caches/search indexes as far as the platform and runtime allow.
- Application close must trigger the lock path.
- The app must not promise guaranteed memory erasure; operating-system/runtime copies and crash dumps may be outside the app's control.
- Startup should be locked by default unless a separately reviewed Windows-protected convenience-unlock option is deliberately implemented.

## 7. Technical Architecture

### 7.1 Proposed Stack

| Layer | Proposed technology | Responsibility |
|---|---|---|
| Desktop runtime | Tauri 2 | Windows packaging, native capabilities, secure IPC boundary |
| Frontend | React + TypeScript + Vite | UI and presentation |
| Styling/components | Tailwind CSS + shadcn/ui or equivalent | Consistent accessible UI |
| UI state | Zustand (or equivalent) | Transient UI/session state |
| Async data/query state | TanStack Query (optional) | Loading, invalidation, command lifecycle |
| Native/backend layer | Rust | Domain operations, validation, filesystem and editor integration, Vault cryptography |
| Local metadata store | SQLite | Project, workspace, reference, and encrypted-record metadata |
| Cryptography | Maintained Rust crates with reviewed algorithms | Key derivation, authenticated encryption, random number generation |
| File monitoring | `notify` crate or equivalent | Best-effort external file-change detection |
| Frontend tests | Vitest + Testing Library | UI and frontend logic |
| Native tests | Rust unit/integration tests | Domain, storage, security and filesystem behavior |
| Desktop E2E | Playwright or WebdriverIO-compatible Tauri setup, subject to tool evaluation | Critical user journeys |

These are recommended starting choices, not immutable requirements. The engineering team should validate current library maintenance, Windows support, licensing, and Tauri compatibility before dependency approval.

### 7.2 Architectural Style

Use a modular monolith. Do not introduce microservices or a local HTTP server for the MVP.

```text
React UI
  └─ Feature services
       └─ Tauri IPC / commands
            └─ Rust application services
                 ├─ Project and workspace domain
                 ├─ Config catalog domain
                 ├─ Notes service
                 ├─ Vault service
                 ├─ Filesystem adapter
                 ├─ External editor launcher
                 └─ Repositories / SQLite migrations
```

### 7.3 Suggested Frontend Structure

```text
src/
  app/                 # Router, providers, global app composition
  features/
    projects/
    workspaces/
    config-catalog/
    environments/
    vault/
    notes/
    search/
    settings/
  components/
    layout/
    ui/
  lib/
    tauri/
    validation/
    utils/
  main.tsx
```

### 7.4 Suggested Rust Structure

```text
src-tauri/
  capabilities/
  migrations/
  src/
    commands/           # Narrow, validated IPC entry points
    domain/             # Entities and business rules
    application/        # Use cases and orchestration
    infrastructure/
      database/
      filesystem/
      editor/
      crypto/
      key_store/
    error.rs
    lib.rs
```

Frontend components must not issue SQL or perform arbitrary filesystem operations. Rust commands should be narrow, typed, validated, and expose only the information needed by the UI. Do not expose a generic command-execution API to the frontend.

## 8. Data Model and Storage

### 8.1 Application Data Directory

Suggested Windows location:

```text
%LOCALAPPDATA%\DevShelf\
  database\devshelf.db
  backups\
  logs\
  cache\
```

Use Tauri's platform-aware app-data directory API rather than hard-coding the path. The precise layout should be confirmed during implementation. Logs and caches must never contain secret values or decrypted note bodies. Encrypted database files, backups, WAL files, and temporary files must be considered in the storage threat model.

### 8.2 Initial Entities

| Entity/table | Main fields | Notes |
|---|---|---|
| `projects` | `id`, `name`, `root_path`, `project_type`, `is_pinned`, timestamps | Root path is a reference, not a managed copy |
| `tags` | `id`, `name` | Shared tags |
| `project_tags` | `project_id`, `tag_id` | Many-to-many association |
| `environments` | `id`, `project_id` nullable, `name`, timestamps | Local/dev/staging/production or user-defined |
| `config_files` | `id`, `project_id` nullable, `path`, `environment_id` nullable, `file_kind`, `description`, timestamps | Path reference only by default |
| `workspaces` | `id`, `name`, `description`, `editor_profile_id` nullable, timestamps, `last_used_at` | Named collections/snapshots |
| `workspace_items` | `id`, `workspace_id`, `target_type`, `target_id` or path reference, `sort_order` | Must support stable ordering and references outside a project |
| `editor_profiles` | `id`, `name`, `executable_path`, safe argument configuration, `is_default` | No shell command interpolation |
| `vault_metadata` | `id`, KDF parameters, salt, wrapped vault key, format version | Never store the master password or derived key |
| `secrets` | `id`, project/environment references, encrypted label/value/metadata as decided, nonce(s), algorithm/version, timestamps | No plaintext secret value at rest |
| `notes` | `id`, project_id nullable, encrypted title/content metadata, timestamps | Content encrypted at rest by default |
| `audit_events` | event type, timestamp, record ID if safe, result | No plaintext values, passwords, tokens, or note bodies |

Use stable random IDs (UUIDs or an equivalent design), foreign keys where applicable, database migrations, transaction boundaries, and documented deletion rules. Schema design should avoid assumptions that all referenced paths exist on every device.

### 8.3 Storage Principles

- SQLite is the local system of record for DevShelf metadata.
- Actual project/config files remain in their existing locations.
- Secrets and private note content are encrypted before persistence.
- Do not store plaintext backups, search indexes, crash artifacts, or debug dumps containing decrypted records.
- Avoid relying on SQLite row deletion as a guarantee of secure physical erasure, especially on SSDs and in WAL/journal files. The Vault's encryption and key lifecycle are the primary protection.
- Export formats must be versioned and validated; import must not execute content or overwrite data silently.

## 9. Security Architecture and Requirements

The Vault is a security-sensitive component, not ordinary CRUD. Security design must be reviewed before using production credentials.

### 9.1 Threat Model (Initial)

Protect against:

- Someone obtaining a copy of the local SQLite database or an encrypted backup while the Vault is locked.
- Accidental disclosure through application logs, error messages, exports, crash diagnostics, search indexes, or UI state.
- Unauthorized or unintended use of overly broad filesystem/shell capabilities.
- Mistakenly copying or exporting a secret into a project repository.
- Accidental loss of the Vault due to a forgotten password or damaged local database.

The MVP cannot fully protect against:

- Malware or an attacker who controls the logged-in Windows user account while the Vault is unlocked.
- Keyloggers, clipboard monitoring, screen capture, or malicious processes with equivalent user privileges.
- Secrets that have already been pasted into another application, committed to source control, or exposed by an external editor.
- Forensic recovery of plaintext that an application or OS previously wrote outside the Vault.

The UI and documentation must state these boundaries clearly.

### 9.2 Cryptographic Design Proposal

Final algorithm and parameter choices must be approved by an engineer responsible for security before implementation.

1. On Vault creation, generate a cryptographically random Vault Data Key (VDK).
2. Use Argon2id with a unique random salt to derive a Key Encryption Key (KEK) from the user's master password. Parameters must be calibrated for target Windows hardware and versioned so they can be upgraded later.
3. Wrap/encrypt the VDK with the KEK using authenticated encryption.
4. Encrypt secret values and private note content with an established AEAD algorithm, such as AES-256-GCM or XChaCha20-Poly1305, through a maintained and reviewed library. The team must choose one supported design rather than improvising crypto composition.
5. Use a fresh, cryptographically random nonce per encryption operation and obey the selected algorithm's nonce requirements. Never reuse a nonce with the same key where prohibited.
6. Bind record identity and relevant format/version fields as authenticated associated data where appropriate, so ciphertext cannot be silently reassigned to another record.
7. Store ciphertext, nonce, algorithm/format version, and necessary KDF metadata. Never store the master password or KEK.
8. Change the master password by deriving a new KEK and rewrapping the VDK after successful unlock, rather than decrypting and re-encrypting every secret.
9. Provide a key-rotation/migration strategy for cryptographic version changes.

The use of authenticated encryption means that tampering should result in a failed decryption, not silently returned corrupted content.

### 9.3 Key Handling

- Perform cryptographic operations in Rust, not in React or browser JavaScript.
- Do not send the master password or Vault key to persistent frontend stores, local storage, session storage, logs, or IPC responses beyond what is strictly required by a well-defined command flow.
- Avoid long-lived plaintext caches. Clear decrypted view state and search indexes when the Vault locks.
- Use best-effort memory clearing for key and plaintext buffers where supported, while documenting that this is not a guarantee against OS/runtime copies.
- Lock the Vault on explicit request, application exit, and inactivity timeout.
- Do not offer automatic unlock by default. A future convenience unlock using Windows-protected credentials must be opt-in, threat-modeled, and clearly described as a different security/usability trade-off from requiring the master password every time.
- Recovery limitations must be explicit. If the master password is lost and no independently secured recovery mechanism exists, the encrypted Vault may be unrecoverable.

### 9.4 Metadata Protection

Even an encrypted secret value can be sensitive when its label, project association, or environment is visible. Before implementation, decide which fields remain searchable in plaintext and which are encrypted. Safer default: use random record IDs, keep only operationally necessary metadata unencrypted, encrypt secret labels and secret-specific descriptions where feasible, and do not store a plaintext content index. The product must document the residual metadata visible in a copied database.

### 9.5 Clipboard and Reveal Controls

- Secrets are masked by default.
- Reveal and Copy are explicit actions; require the Vault to be unlocked.
- Clipboard content must never be included in logs or app diagnostics.
- Optional clipboard clearing must only clear the clipboard if it still matches the value DevShelf copied, and must be described as best-effort.
- Do not promise removal from Windows clipboard history, cross-device clipboard sync, screenshots, paste targets, or third-party clipboard managers.
- Avoid showing plaintext in toast messages, tooltips, URLs, error strings, or analytics.

### 9.6 File Export

- Plaintext `.env` export must be an explicit user-initiated action with a destination preview and a warning.
- Refuse silent overwrite by default; offer an explicit replace flow.
- Warn when exporting beneath a registered repository root or a path that appears to be tracked by Git, while acknowledging that path heuristics are not a security guarantee.
- Do not automatically add secrets to Git, modify `.gitignore`, or write directly into arbitrary project paths without confirmation.
- Prefer encrypted backups for Vault content; plaintext export is a deliberate exception.
- Temporary decrypted files must not be used unless unavoidable and reviewed. If introduced later, define cleanup behavior and failure handling.

### 9.7 Tauri Capability Model

- Enable only the plugins and commands required by approved workflows.
- Use Tauri v2 capabilities/permissions and scope filesystem access where applicable.
- Avoid a generic shell-execution command. Editor launching must validate configured executables and pass arguments safely.
- Treat paths, filenames, imported data, and project metadata as untrusted input.
- Review dependency permissions, plugin maintenance, and Windows packaging/signing before release.

### 9.8 Audit Events

Record security-relevant events such as vault created/unlocked/locked, failed unlock attempts, secret created/updated/deleted, copy/export requested, backup created/restored, and configuration changes. Audit data must not record secret values, master passwords, decrypted note bodies, or raw clipboard content. Audit scope and retention should remain proportionate for a local single-user tool.

## 10. Future Synchronization Design Constraints

Sync is out of scope for MVP, but design the local data model to make it possible later.

- Use stable IDs and explicit schema versions.
- Keep filesystem-specific paths separate from logical entity identity.
- Add timestamps and, if later needed, a change journal/version field without treating timestamps alone as a reliable conflict resolver.
- Separate local machine state (editor executable paths, last-opened state, local absolute paths) from portable project/workspace metadata.
- Provide versioned export/import before attempting live sync.
- Design secret sync separately from ordinary metadata sync. Do not upload the master password or plaintext Vault key.
- Future sync requires conflict policy, authentication, authorization, encryption-key distribution/recovery, deletion propagation, device revocation, and backup/restore decisions.
- Do not claim end-to-end encryption merely because HTTPS or server-side encryption is used; define where plaintext and keys exist.

## 11. Non-Functional Requirements

### 11.1 Performance

- The app should start and show local project/workspace metadata without network access.
- Search over project names, paths, tags, and registered metadata should feel immediate for a reasonable internal-user dataset; QA/product should agree on an explicit benchmark dataset and target before release.
- Opening a file delegates to the OS or configured editor and must not block the UI while waiting for editor startup.
- Directory scanning must be user-initiated or bounded, avoid scanning large dependency/build directories by default, and provide cancellation/progress for potentially long operations.

### 11.2 Reliability and Data Integrity

- Use SQLite transactions for logically related changes.
- Apply versioned migrations and test upgrades from previous schema versions.
- Handle a missing path, permission error, malformed import, database lock, and corrupted record without deleting unrelated metadata.
- Debounced autosave must expose saving/saved/error state and must not discard a failed write silently.
- Backup and restore must be tested with actual restore verification.

### 11.3 Privacy and Offline Operation

- No account or internet connection is required for core features.
- No telemetry or crash-report upload should be enabled by default without explicit company approval and a privacy review.
- The app must not send file paths, secret metadata, note content, or secrets over the network in MVP.
- If telemetry is introduced later, collect the minimum data, document it, and never include plaintext secrets or note content.

### 11.4 Accessibility and Usability

- Support keyboard navigation and discoverable shortcuts for quick open, new note, new project, lock vault, and search.
- Provide clear empty, loading, inaccessible-path, locked-vault, and error states.
- Confirm destructive operations and distinguish removing a reference from deleting the underlying file.
- Use masking, warnings, and status indicators consistently for sensitive content.

## 12. Milestones and Delivery Plan

### M0 — Foundation

**Deliver:** Tauri/React/Rust application skeleton, SQLite setup, migration framework, app data directory handling, logging policy, capability baseline, Windows development/build instructions.  
**Exit criteria:** clean setup and build on a supported Windows machine; database initializes; logs do not contain sensitive data; CI can run foundational tests.

### M1 — Project Hub and Workspace Collections

**Deliver:** project registry, tags, file/folder references, editor profiles, workspace collections, restore flow, missing-path handling, recent/pinned items.  
**Exit criteria:** a tester can add projects from different drives, create a cross-project collection, restart the app, restore the collection, and recover cleanly from missing paths.

### M2 — Config Catalog and Private Notes

**Deliver:** manual config registration, environment labels, bounded pattern-based discovery, Markdown notes, autosave, project association, local search.  
**Exit criteria:** notes are stored outside project roots and encrypted at rest; catalog operations do not modify config files; paths can be relinked without data loss.

### M3 — Vault Core

**Deliver:** create/unlock/lock Vault, reviewed cryptographic implementation, encrypted secret values and private notes, mask/reveal/copy, inactivity timeout, minimal security audit events.  
**Exit criteria:** database and backup inspection reveals no plaintext secret/note content in protected fields; tampered ciphertext fails safely; incorrect passwords do not reveal data; lock clears app-level decrypted state to the extent testable.

### M4 — Backup, Environment Utilities, and Release Hardening

**Deliver:** verified backup/restore, versioned import/export, config diff/validation where selected, security test suite, Windows packaging/signing decision, end-user documentation.  
**Exit criteria:** restore tested from a clean installation; no unreviewed plaintext export path; critical acceptance tests and security checks pass.

### Later — Optional Sync

Requires separate product requirements, threat model, conflict-resolution policy, account/device model, key distribution/recovery design, and security review. It is not part of the MVP commitment.

## 13. Acceptance Criteria

The MVP is not complete until all relevant criteria below are demonstrated:

### Project and Workspace

- [ ] Users can register and manage multiple projects with roots on separate drives.
- [ ] Project registration never copies or changes project source files.
- [ ] A workspace can reference files from multiple projects and from outside any project root.
- [ ] Workspace item order persists across restart.
- [ ] Missing references are reported; metadata remains available for repair.
- [ ] Opening uses an approved external-editor/OS flow and never executes arbitrary commands from workspace data.

### Config Catalog

- [ ] Users can register and categorize configuration references by environment and kind.
- [ ] Pattern scanning is bounded and avoids known large/generated directories by default.
- [ ] Registering or scanning a config file does not modify its contents.
- [ ] File path and metadata remain distinct from Vault-managed secret values.

### Notes

- [ ] Notes can be associated with a project or remain general.
- [ ] Notes are stored outside project roots by default.
- [ ] Note content is encrypted at rest using the approved storage design.
- [ ] Autosave exposes unsaved/error state and protects against silent loss.
- [ ] Locking the Vault clears the app's decrypted note view and any plaintext search cache to the extent designed.

### Vault and Security

- [ ] The database never stores master passwords or derived KEKs.
- [ ] Secret values are encrypted before persistence, and ciphertext tampering is detected.
- [ ] Secrets remain masked until explicitly revealed or copied.
- [ ] Copy/reveal is unavailable while locked.
- [ ] Logs, errors, telemetry, crash reports, and audit events contain no secret values or note bodies.
- [ ] Lock, inactivity timeout, and application-exit behavior work as specified.
- [ ] The backup/restore procedure is tested, including the ability to unlock restored data.
- [ ] Plaintext export requires explicit user action and warns before overwrite or export into a repository path.
- [ ] Security limitations for clipboard, compromised host, memory, and password recovery are documented.

### Quality and Release

- [ ] Database migrations and import validation have tests.
- [ ] Critical flows have integration/E2E coverage on Windows.
- [ ] Keyboard navigation and essential accessibility states are reviewed.
- [ ] Windows packaging, update strategy, and code-signing expectations are documented.

## 14. Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Master password lost | Vault data may become unrecoverable | Explain recovery limits; test encrypted backups; do not imply recovery exists unless implemented |
| Plaintext leakage through logs/UI/search | Credential exposure | Rust-only crypto operations, strict logging rules, no persistent plaintext indexes, security tests |
| Clipboard history or third-party clipboard manager | Secret persists outside Vault | Explicit confirmation, best-effort clearing, clear user warning |
| Project path moved/deleted | Workspace references become stale | Preserve references, report status, provide manual relink |
| Overly broad Tauri/shell permissions | Local file or command abuse | Least privilege, narrow IPC, safe argument handling, dependency review |
| Plaintext export enters Git | Credential exposure | Destination preview, overwrite confirmation, repository-path warning, no automatic export |
| Corrupted or untested backup | Permanent data loss | Versioned backup, integrity validation, clean restore tests |
| Premature sync implementation | Increased security and conflict complexity | Keep sync out of MVP; conduct separate design/security review |
| Scope expansion into an IDE | Delayed delivery and higher complexity | Keep editing in external tools; prioritize navigation and references |

## 15. Open Decisions for Kickoff Review

The product and engineering leads should resolve these before or during M0:

1. Confirm final product name, internal ownership, and target Windows versions/architectures.
2. Decide whether the MVP uses master-password unlock only or also offers opt-in Windows-protected convenience unlock. Recommended default: master password only.
3. Approve the cryptographic library and exact KDF/AEAD parameters after security review.
4. Decide which Vault metadata is encrypted and what remains visible in a copied database.
5. Confirm whether all notes must always require Vault unlock. Recommended default: yes for content encryption; a separate public/non-sensitive note mode, if desired, must be explicit.
6. Confirm behavior for failed unlock attempts, auto-lock duration, clipboard clearing default, and re-authentication before reveal/copy.
7. Decide initial supported config patterns and whether discovery is project-root-only or can include user-selected folders.
8. Select the editor-profile UX and whether multiple profiles are required for MVP.
9. Approve backup format, default backup location, and whether encrypted Vault backup is mandatory in the first usable release.
10. Set product-specific performance targets and maximum expected project/file counts.
11. Confirm internal distribution, signing, update, support, and vulnerability-reporting processes.
12. Decide whether any company policy requires approval before storing corporate production credentials in a local developer vault.

## 16. Recommended Implementation Order

1. Finalize the MVP and security decisions in Section 15.
2. Build a small vertical slice: project registration → register a file → save workspace → restart → reopen file.
3. Add the config catalog and external editor profiles.
4. Add encrypted notes and establish the Vault's key-management foundation before storing real credentials.
5. Implement the Vault with security-reviewed libraries and tests before onboarding any production secrets.
6. Add backup/restore and hardening before internal rollout.
7. Gather internal feedback and only then decide whether to build config diff, secret-to-environment export, or sync.

## 17. Definition of Success

The first internal release is successful when developers can manage projects, quickly restore a curated set of files, identify the correct environment configuration, keep project notes outside repositories, and store/copy credentials through a deliberate encrypted Vault workflow—all while continuing to use their preferred external editors and without requiring a network connection.

The most important quality bar is not the number of features. It is predictable behavior, no silent file modifications, no accidental disclosure of protected content, recoverable local data, and a small enough scope that the team can validate these promises before wider internal adoption.

---

## Appendix A — Suggested First-Sprint User Stories

- **US-001:** As a developer, I can register a project by selecting its root folder so that I can access it without remembering its path.
- **US-002:** As a developer, I can register a configuration file outside the project root so that deployment files can be part of my workflow.
- **US-003:** As a developer, I can create a named workspace with ordered file references from several projects so that I can reopen a task's working set later.
- **US-004:** As a developer, I can restore a workspace and see any missing paths without losing the saved collection.
- **US-005:** As a developer, I can choose Notepad++ or VS Code as an editor so that DevShelf complements my existing tools.
- **US-006:** As a developer, I can create a project note stored outside the repository so that technical knowledge does not become an accidental Git commit.
- **US-007:** As a developer, I can store an API key in the Vault and see only a masked value in normal views.
- **US-008:** As a developer, I can unlock the Vault and explicitly copy a secret when required.
- **US-009:** As a developer, I can lock the Vault and have decrypted app-level views and search caches cleared as designed.
- **US-010:** As a developer, I can create and restore an encrypted backup so that I can recover my local data after device or database failure.

## Appendix B — Glossary

- **Project registration:** DevShelf metadata representing a project root; it does not own or copy the repository.
- **Workspace/collection:** A named and ordered set of project, folder, and file references to reopen together.
- **Config Catalog:** Metadata and classification for existing configuration files, not a copy of their contents.
- **Environment:** A label such as local, development, staging, or production used to organize configuration references and secrets.
- **Vault:** The encrypted store for credentials, secret values, and protected note content.
- **Master password:** The user-provided secret used to derive a key that unlocks the Vault's data key; it is not itself stored.
- **VDK (Vault Data Key):** A randomly generated key used to encrypt protected records.
- **KEK (Key Encryption Key):** A key derived from the master password and used to wrap/unwrap the VDK.
- **AEAD:** Authenticated encryption with associated data; protects confidentiality and detects tampering.
- **Local-first:** Core functionality and data storage work locally without a required cloud account or network connection.
