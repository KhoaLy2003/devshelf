use rusqlite_migration::{Migrations, M};

pub const MIGRATIONS_COUNT: usize = 8;

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up("CREATE TABLE app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);"),
        M::up(
            "CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  display_name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  path_key TEXT NOT NULL UNIQUE,
  drive TEXT NOT NULL,
  project_type TEXT NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE project_tags (
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY (project_id, tag)
);",
        ),
        M::up(
            "CREATE TABLE editor_profiles (
  kind TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  exe_path TEXT NOT NULL DEFAULT '',
  is_default INTEGER NOT NULL DEFAULT 0
);
INSERT INTO editor_profiles (kind, name, exe_path, is_default) VALUES
  ('windows_default', 'Windows default', '', 1),
  ('notepad_plus_plus', 'Notepad++', '', 0),
  ('vs_code', 'VS Code', '', 0),
  ('custom', 'Custom', '', 0);
CREATE UNIQUE INDEX editor_one_default ON editor_profiles(is_default) WHERE is_default = 1;",
        ),
        M::up(
            "CREATE TABLE workspaces (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  name_key TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
-- foreign_keys is off, so ON DELETE CASCADE is not relied on; the domain deletes items explicitly.
CREATE TABLE workspace_items (
  id INTEGER PRIMARY KEY,
  workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('file','folder')),
  path TEXT NOT NULL,
  path_key TEXT NOT NULL,
  project_id INTEGER,
  editor_kind TEXT NOT NULL,
  UNIQUE (workspace_id, position),
  UNIQUE (workspace_id, path_key)
);",
        ),
        M::up("ALTER TABLE workspaces ADD COLUMN last_restored_at TEXT;"),
        M::up("ALTER TABLE workspaces ADD COLUMN last_restore_missing INTEGER NOT NULL DEFAULT 0;"),
        M::up(
            "CREATE TABLE config_files (
  id INTEGER PRIMARY KEY,
  project_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  path_key TEXT NOT NULL,
  environment TEXT NOT NULL,
  kind TEXT NOT NULL,
  UNIQUE (project_id, path_key)
);
-- foreign_keys is off; project existence is checked by the command, and entries survive project removal (F15) by design.",
        ),
        M::up(
            "CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  project_id INTEGER,
  title TEXT NOT NULL,
  body_enc BLOB NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
-- project_id is validated by the notes module (projects::get); no foreign key enforcement.",
        ),
    ])
}
