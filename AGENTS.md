# AGENTS.md

## Purpose

DevShelf is a single-user, offline, Windows 10/11 desktop app for organizing projects, editors, workspaces, config, notes, and an encrypted secrets vault. No cloud, sync, telemetry, or network calls.

## Stack

Tauri 2 (WebView2) shell; React + TypeScript + Vite UI; Rust core; SQLite via `rusqlite` (bundled) and `rusqlite_migration`. Planned, not yet installed: Tailwind v4, shadcn/ui, React Router, TanStack Query, cmdk, crypto crates. Do not substitute libraries. See `docs/architecture.md`.

## Layout

- `src/` - React UI (`app.tsx`, `lib/commands.ts` invoke wrappers, `types/commands.ts` shared command types).
- `src-tauri/src/` - Rust core: `commands/` (Tauri commands), domain modules `projects`, `editors`, `workspaces`, `config`, `notes`, `vault`, and persistence `db`. The `projects` domain module is implemented; the others remain empty placeholders.
- `tests/` - Vitest tests. Rust tests are `#[cfg(test)]` modules.
- `docs/` - approved docs: architecture, constitution, DESIGN (dark theme only), roadmap, models. `kit/` is gitignored tooling.

## Commands

- `npm run tauri dev` - run the app.
- `npm run build` - tsc typecheck + Vite build.
- `npm run test` - Vitest. `npm run lint` - ESLint + Prettier check. `npm run format` - Prettier write.
- In `src-tauri/`: `cargo test`, `cargo build`. Cargo is at `%USERPROFILE%\.cargo\bin`.

## Conventions

- TS files kebab-case; Rust files snake_case; one folder per Rust module.
- Every Tauri command has one serde argument struct and one serde return type, returns a user-facing error code (never raw error strings), and its TS types live in `src/types/commands.ts` only.
- Only Rust touches the DB, filesystem, processes, and secrets. SQL uses bound parameters; schema changes only via numbered `rusqlite_migration` migrations.
- Never write files into user repos; tests use temp dirs; no network calls.
- Commits: Conventional Commits (see README).

Rules in full: [docs/constitution.md](docs/constitution.md).
