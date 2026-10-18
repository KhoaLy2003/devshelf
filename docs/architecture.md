# architecture.md

## Metadata

- **Status**: `approved`
- **Last updated**: 2026-10-12
- **Based on**: `idea-brief.md`, `DESIGN.md`, `roadmap.md`. `market-notes.md` does not exist (Step 2 skipped). Screen design was skipped (Step 5); screens rely on `DESIGN.md` plus `docs/preview/index.html`.

> This is the final decision. The stack was fixed by the user and confirmed at the Step 6 gate. Stack research was not repeated; the proposal in the Step 6 research output is the basis for the additions below.

---

## 1. Context

- Single user, Windows 10/11 desktop only, local-first, local data only. The only network access is the signed update check against GitHub Releases. No cloud, sync, team, or multi-user features (idea brief no-go list).
- Roadmap scope: 10 Must features (nine M, one S), 2 Should, 3 Could. Phase 2 runs as `feature` (one feature per cycle), because two domains exist (workspace/config/notes and secrets/encryption) and the dependency structure forms parallel streams after F01 to F03.
- The product stores real credentials (vault) and project file references. Security and data safety outweigh feature breadth.
- The stack is a hard constraint chosen by the user. The web-oriented catalog entries (Next.js, Supabase, Vercel, NextAuth, hosted services) do not apply and are not used.

---

## 2. Decision

**Stack**:

- Shell: Tauri 2 (WebView2 on Windows)
- UI: React + TypeScript + Vite
- Core: Rust (all file-system, database, crypto, and process access)
- Database: SQLite via `rusqlite` (bundled), with `rusqlite_migration` for schema migrations
- Styling and components: Tailwind CSS v4 and shadcn/ui; Inter stands in for the proprietary font named in `DESIGN.md`
- Client-side routing and data fetching: React Router (seven sidebar areas) and TanStack Query (cached calls to Rust commands)
- Command palette: `cmdk` (F11)

**High-level architecture**: a single desktop application with a modular core. The webview renders screens and calls typed Tauri commands. All file, database, crypto, process, and network logic lives in the Rust core, organised by feature module (projects, editors, workspaces, config, notes, vault, updates). The only network layer is the update check and download (`updates` module).

---

## 3. Options Considered

| Option | Considered | Rejected because |
|---|---|---|
| Next.js + Supabase + Vercel (catalog baseline) | Yes | Web and hosted; cannot access local files, processes, or Credential Manager; contradicts offline single-user scope. |
| Electron | Not formally compared | Not the user's chosen stack; Tauri 2 was fixed as a hard constraint. |
| `tauri-plugin-sql` (SQL from the webview) | Yes | Would expose SQL to the webview; all DB access stays in Rust. |
| `tauri-plugin-stronghold` for vault storage | Yes | Not adopted. Not researched further; not needed given the field-level design below. Its deprecation status was recalled but not verified. |
| SQLCipher (whole-database encryption) | Yes | Adds a C build dependency; field-level encryption already protects secrets and notes. |
| Redux or another global state store | Yes | Unnecessary for one user and a small number of screens; TanStack Query covers server-state caching. |
| ORM (Diesel or SeaORM) | Yes | Overhead for a small schema; raw `rusqlite` with migrations is sufficient. |
| `notify` (live file watching) | Yes | Deferred. Availability is checked on demand (F05, F06), so live watching is not needed for the MVP. |

---

## 4. Reasoning

- **Roadmap fit**: 10 Must features in one solo cycle sequence need one boring, layered stack with no extra services. The Rust core handles what a browser SPA cannot: filesystem checks, process launch, and audited crypto with memory wiping.
- **Domains**: workspace, config, and notes need native file access and path checks. Secrets need Argon2id, XChaCha20-Poly1305, and zeroization. These sit in Rust for correctness and for keeping secret material out of the webview.
- **Dependency structure**: F01 to F03 are the foundation. Config (F06), notes (F07), and the vault (F08, F09) are independent streams, so one SQLite schema with per-feature migrations fits.

**Encryption design (fixed by this document)**:

- Vault master password is run through Argon2id to derive a key that wraps a random vault data key. Each secret field is encrypted with XChaCha20-Poly1305.
- Notes use a separate data key, protected by the Windows Credential Manager (via the `keyring` crate). Notes therefore work while the vault is locked.
- Backup (F12, Could) reuses the same primitives with its own password and KDF salt.
- Plaintext metadata: labels, project names, environment names, and file paths stay unencrypted in SQLite. This is a trade-off the user accepted at the gate. It keeps filtering and quick open simple; a copied database reveals those names.

**Editors and process launch**:

- Default-association opening uses `tauri-plugin-opener`.
- Custom editors (Notepad++, VS Code, custom path) are launched from Rust with `std::process::Command` using an argument list and no shell. Arbitrary command execution is excluded (roadmap section 4).

**Other additions**:

- `tauri-plugin-dialog`: folder and file pickers (F02, F04, F13).
- `tauri-plugin-clipboard-manager`: secret copy and clipboard auto-clear (F09, F14).
- `tauri-plugin-single-instance`: prevents two instances opening the same SQLite file.
- `walkdir` and `globset` (or `ignore`): bounded config discovery only (F13).
- Testing: Vitest for the UI; `cargo test` for crypto, path, and database logic. Playwright is optional.

**Complexity deliberately avoided for MVP scope**:

- SQLCipher or whole-database encryption.
- `tauri-plugin-stronghold`.
- Redux or another global state store; ORMs; GraphQL.
- Web services, authentication, sync, and telemetry (single user, local-first).
- A full-text search engine (SQLite `LIKE` or FTS5 is enough for F11).
- A code-editor component (Markdown editing is a plain textarea).
- Live file watching (`notify`).

---

## 5. System Overview

```
[React UI (webview)]
        |  typed Tauri commands (invoke)
        v
[Rust command layer]  -- validation, errors as user-facing codes
        |
        +--> [Domain modules: projects, editors, workspaces, config, notes, vault]
        |          |                         |
        |          v                         v
        |     [SQLite via rusqlite]    [Crypto: Argon2id, XChaCha20-Poly1305, zeroize]
        |                                    |
        +--> [OS: filesystem, process launch, Credential Manager (keyring), clipboard]
```

**Key components**:

| Component | Responsibility | Technology |
|---|---|---|
| Desktop shell | Window, webview host, plugin registration | Tauri 2 |
| UI | Screens, forms, quick open, status indicators | React, TypeScript, Vite, Tailwind v4, shadcn/ui, cmdk |
| Routing and data cache | Seven sidebar areas; cached command results | React Router, TanStack Query |
| Command layer | Typed entry points; validation; error mapping | Rust, Tauri commands |
| Domain modules | Business rules per feature | Rust |
| Persistence | Schema, migrations, queries (no SQL in webview) | SQLite, `rusqlite`, `rusqlite_migration` |
| Crypto | Key derivation, field encryption, memory wiping | `argon2`, `chacha20poly1305`, `zeroize` |
| Device key | Notes key protected by Windows Credential Manager | `keyring` |
| OS integration | Open files, dialogs, clipboard, single instance | Tauri plugins (opener, dialog, clipboard-manager, single-instance) |

---

## 6. Data & Integration

- **Primary datastore**: a single SQLite file in the user's application data folder, accessed only from Rust. Secret values and note bodies are stored as encrypted fields; other metadata is plaintext.
- **External integrations/APIs**: GitHub Releases only, for the signed update check and download. Editors are external executables the user configures.
- **Auth approach**: local master password for the vault (Argon2id). No accounts, no sessions, no tokens. Auto-lock after inactivity is enforced in the UI and the Rust layer, which zeroizes the unlocked key on lock.

---

## 7. Consequences

**Easier**:

- One language boundary for all sensitive operations; the webview never receives plaintext secrets unless the user reveals or copies one.
- No server, deployment, or account infrastructure to build or maintain.
- Field-level encryption keeps filtering, quick open, and the config catalog fast without decrypting everything.
- Notes remain usable while the vault is locked.

**Harder / constraints introduced**:

- Rust is required for all backend work, and Rust/cargo is not yet installed on the development machine. Compilation time and Rust learning cost apply to every feature.
- Plaintext metadata means a copied database reveals project, label, and path names. This must be stated in the app's privacy text.
- Backup (F12) must cover every data type, so any new data type forces a backup change.
- Two encryption domains (vault key and notes device key) must be kept consistent; a key-rotation feature would touch both.
- WebView2 must be present on Windows; Windows 10 may need it installed.
- Tauri command types must be kept in sync between Rust and TypeScript.

---

## 8. Risks & Unknowns

- **Cargo and Rust not installed.** Required before Step 8 scaffold. Installation (rustup, MSVC C++ Build Tools, WebView2) is not yet done.
- **Crypto crate versions not verified.** `argon2`, `chacha20poly1305`, `zeroize`, and `keyring` were named by the research but not checked against current releases or audit status in this document. Verify before Phase 2 F08 and F09.
- **Argon2id parameters not chosen.** Memory, iteration, and parallelism values must be set and tested on this machine during F08.
- **Editor profile executables.** Launching user-defined executables must reject shell syntax and be tested against paths with spaces.
- **Clipboard auto-clear default and delay**: open question in the idea brief; affects F14 only.
- **Re-authentication before reveal or copy**: open question in the idea brief; not in MVP.
- **Vault is the highest-risk area.** A defect in F08 or F09 has high cost; allow extra verification effort.
- **Step 5 screen design skipped**: screens are built from `DESIGN.md` and the prototype, so visual decisions are not independently reviewed.
