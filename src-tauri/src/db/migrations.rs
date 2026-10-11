use rusqlite_migration::{Migrations, M};

pub const MIGRATIONS_COUNT: usize = 3;

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
    ])
}
