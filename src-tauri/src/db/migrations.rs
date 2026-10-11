use rusqlite_migration::{Migrations, M};

pub const MIGRATIONS_COUNT: usize = 1;

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        "CREATE TABLE app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )])
}
