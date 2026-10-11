# DevShelf

DevShelf is a local-first Windows desktop app that keeps your projects, editors, workspaces, config, notes, and secrets in one offline place.

## Quick Start

Prerequisites: Windows 10/11 with WebView2, Node.js and npm, and Rust (MSVC toolchain).

```powershell
git clone <repo-url> devshelf
cd devshelf
npm install
npm run tauri dev
```

The window shows a hello page with the value returned by the Rust `app_info` command.

Other commands: `npm run build` (typecheck and Vite build), `npm run test` (Vitest), `npm run lint` (ESLint and Prettier check), `cargo test` and `cargo build` inside `src-tauri/`.

## Repository Structure

- `src/` - React + TypeScript UI. `src/lib/commands.ts` wraps Tauri commands; `src/types/commands.ts` holds the shared command types.
- `src-tauri/` - Rust core. `src/commands/` holds typed Tauri commands. `src/projects`, `editors`, `workspaces`, `config`, `notes`, `vault`, and `db` are domain/persistence module folders (placeholders for now).
- `tests/` - TypeScript (Vitest) tests.
- `docs/` - approved product, architecture, design, and constitution documents.
- `.github/workflows/` - CI (lint, Vitest, cargo test).

## Tech Stack

Tauri 2, React, TypeScript, Vite, Rust, SQLite (rusqlite, rusqlite_migration). Details and the planned additions are in [docs/architecture.md](docs/architecture.md).

## Contributing

Coding standards: [docs/constitution.md](docs/constitution.md).

- **File naming**: kebab-case for TypeScript/TSX files and folders (`app.tsx`, `quick-open.ts`); snake_case for Rust files and modules (`project_registry.rs`). One folder per Rust domain module.
- **Tests**: TypeScript tests live in `tests/` and are named `*.test.ts` or `*.test.tsx`. Rust tests live in `#[cfg(test)]` modules beside the code, or in `src-tauri/tests/` for integration tests. Tests use temporary directories, never the real user data folder.
- **Commit messages**: Conventional Commits, `type(scope): summary` in the imperative mood, with types `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci` (for example `feat(projects): add project registry`).

## License

Private project. No license is granted.
