# constitution.md

## Metadata

- **Status**: `approved`
- **Project**: DevShelf
- **Version**: 1.1.0
- **Ratified**: 2026-10-12
- **Last amended**: 2026-10-11
- **Based on**: `architecture.md`, `DESIGN.md`, `roadmap.md`

> Read by every Phase 2 step. Every principle must change at least one downstream plan or diff. Personal workflow preferences do not belong here.

---

## Principles

### Principle 1: Secrets Stay in the Rust Core

**MUST**:
- Plaintext secret values exist only in Rust memory and in the response to an explicit reveal or copy command. No secret value is written to logs, to SQLite, or to a file.
- Every type holding key material or a plaintext secret derives or implements `zeroize` on drop. Unlocked keys are cleared on vault lock and on auto-lock.
- Secret and note fields are encrypted with XChaCha20-Poly1305 using a fresh random nonce per write. Nonce reuse is a defect.
- Argon2id parameters are set in one module and are not overridden per call. Tests use the same code path with lower cost only through an explicit test-only constructor.

**SHOULD**:
- Copied secrets are cleared from the clipboard after the configured delay (F14) when that feature is enabled.

**Rationale**: The product stores real credentials. A plaintext leak through the webview, logs, or the database is the highest-cost failure. The rule keeps the crypto boundary in one language.

---

### Principle 2: Only Rust Touches the Database and the OS

**MUST**:
- The webview never runs SQL and never imports `rusqlite`-equivalent logic. All reads and writes go through typed Tauri commands.
- All filesystem access, process launch, and Credential Manager access happen in Rust modules. The React layer never calls file or process APIs directly.
- Editor launch uses `std::process::Command` with a fixed program path and an argument list. Shell invocation (`cmd /c`, `powershell -Command`, string concatenation into a command line) is forbidden.
- Every SQL statement uses bound parameters. String-built SQL is forbidden.
- Schema changes are made only through `rusqlite_migration` migrations, one numbered migration per change. Existing migrations are never edited after merge.

**SHOULD**:
- Domain logic lives in a module per feature (`projects`, `editors`, `workspaces`, `config`, `notes`, `vault`). A module may call another module's public functions; it must not reach into another module's tables directly.

**Rationale**: Keeps the attack surface small and makes the Phase 2 features independent streams, as the roadmap's `feature` scope assumes.

---

### Principle 3: Typed Boundary Between Rust and TypeScript

**MUST**:
- Every Tauri command has one Rust argument struct and one Rust return type, both deriving `serde` traits. The TypeScript types for them are kept in one generated or single shared file; no hand-copied duplicate types.
- Every command returns a user-facing error code (a small enum) and never a raw Rust error string or a stack trace.
- No command accepts an arbitrary path or command string for execution. Paths must come from the project registry or a dialog result.

**SHOULD**:
- TanStack Query is the only cache for command results in the UI. Components do not keep copies of server state in local state beyond form drafts.

**Rationale**: The vault and notes must not depend on webview trust. Typed commands also keep the 10 Must features consistent.

---

### Principle 4: Testing Standard

**MUST**:
- Crypto, path resolution, migration, and workspace restore logic have `cargo test` coverage. Each vault change has a test that a wrong master password fails and a tampered ciphertext fails.
- Each Must feature has at least one end-to-end smoke check on the built app before its phase is closed. Unit tests alone do not close a feature.
- Tests are deterministic and do not touch the real user data folder; they use a temporary directory per test.

**SHOULD**:
- UI logic that has branching (filters, save states, quick open ranking) has Vitest tests.

**Rationale**: The vault is the highest-risk area (architecture.md section 8). A green build does not prove a secret is protected.

---

### Principle 5: Local-First and Offline

**MUST**:
- No network call is made by the application. Adding one requires a Constitution amendment (MAJOR).
- No telemetry, crash reporting, or auto-update. The app opens and works with the network disabled.
- Data stays in the user's application data folder. No file is written into a project repository.

**SHOULD**:
- Errors that a user can fix (missing path, missing editor) show the fix in the message, not a generic failure.

**Rationale**: Single user, offline product (idea brief no-go list). Network code would add a dependency the product does not need.

---

## Additional Constraints

- Stack is fixed by `architecture.md`: Tauri 2, React, TypeScript, Vite, Rust, SQLite via `rusqlite`, Tailwind v4, shadcn/ui, React Router, TanStack Query, `cmdk`. Adding a runtime dependency that changes this list requires an amendment (MINOR).
- Windows 10 and 11 are the only supported platforms. WebView2 is required.
- Visual design follows `docs/DESIGN.md`. Screens built without a per-screen spec use `docs/preview/index.html` as the reference. The dark theme is the only theme.
- Code and comments are English.

---

## Governance

### Amendment Procedure

Solo project: update this file directly, bump `Version` per the Versioning Policy, and set `Last amended`. Before amending, check the change against one recent plan or diff to confirm it changes a real decision.

### Versioning Policy

- **MAJOR**: removes or redefines a principle, or permits network access.
- **MINOR**: adds a principle or materially expands a rule.
- **PATCH**: wording or clarification with no semantic change.

### Compliance Review

- The review step flags any violation explicitly. Violations are not silently fixed.

---

## Spec Evolution

**Model: Flow-forward.** Shipped feature directories under `docs/specs/<feature-slug>/` are immutable historical records. A change to a shipped feature gets a new directory (`docs/specs/<feature-slug>-v2/`) and its own Phase 2 cycle from Step 1, which references the original spec and states what changed and why. Original directories are not edited after ship.
