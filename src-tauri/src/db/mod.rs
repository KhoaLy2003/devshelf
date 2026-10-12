//! SQLite storage: open, version check, and migrations.

mod migrations;

use std::path::Path;

use rusqlite::Connection;

pub const DB_FILE_NAME: &str = "devshelf.sqlite3";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Unavailable,
    SchemaTooNew,
}

pub struct Storage {
    conn: Connection,
    schema_version: i64,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("schema_version", &self.schema_version)
            .finish()
    }
}

impl Storage {
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    #[cfg(test)]
    pub fn schema_version(&self) -> i64 {
        self.schema_version
    }
}

fn read_user_version(conn: &Connection) -> Result<i64, StorageError> {
    conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
        .map_err(|_| StorageError::Unavailable)
}

/// Creates `folder` if missing, opens `folder/devshelf.sqlite3`, refuses to write
/// when user_version is newer than this build, otherwise applies pending migrations.
pub fn open_in(folder: &Path) -> Result<Storage, StorageError> {
    std::fs::create_dir_all(folder).map_err(|_| StorageError::Unavailable)?;
    let mut conn = Connection::open(folder.join(DB_FILE_NAME))
        .map_err(|_| StorageError::Unavailable)?;

    let version = read_user_version(&conn)?;
    if version > migrations::MIGRATIONS_COUNT as i64 {
        return Err(StorageError::SchemaTooNew);
    }

    migrations::migrations()
        .to_latest(&mut conn)
        .map_err(|_| StorageError::Unavailable)?;

    let schema_version = read_user_version(&conn)?;
    Ok(Storage {
        conn,
        schema_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_missing_folder_and_applies_migrations() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("a").join("b");
        let storage = open_in(&folder).unwrap();
        assert!(folder.join(DB_FILE_NAME).is_file());
        assert_eq!(storage.schema_version(), 5);
    }

    #[test]
    fn reopen_does_not_reapply_migrations() {
        let tmp = tempfile::tempdir().unwrap();
        drop(open_in(tmp.path()).unwrap());
        {
            let raw = Connection::open(tmp.path().join(DB_FILE_NAME)).unwrap();
            raw.execute("INSERT INTO app_meta (key, value) VALUES ('marker', 'kept')", [])
                .unwrap();
        }

        // A reopen must skip migration 1 (user_version is already 1) and keep existing data.
        let second = open_in(tmp.path()).unwrap();
        assert_eq!(second.schema_version(), migrations::MIGRATIONS_COUNT as i64);
        let value: String = second
            .conn
            .query_row("SELECT value FROM app_meta WHERE key = 'marker'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(value, "kept");
    }

    #[test]
    fn schema_too_new_is_refused_without_writing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(DB_FILE_NAME);
        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", 99).unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        let result = open_in(tmp.path());
        assert_eq!(result.unwrap_err(), StorageError::SchemaTooNew);
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn unwritable_folder_is_storage_unavailable() {
        let tmp = tempfile::tempdir().unwrap();
        let blocked = tmp.path().join("blocked");
        std::fs::write(&blocked, b"x").unwrap();
        assert_eq!(open_in(&blocked).unwrap_err(), StorageError::Unavailable);
    }

    #[test]
    fn corrupt_file_is_storage_unavailable_and_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(DB_FILE_NAME);
        std::fs::write(&path, b"not a database").unwrap();
        assert_eq!(open_in(tmp.path()).unwrap_err(), StorageError::Unavailable);
        assert_eq!(std::fs::read(&path).unwrap(), b"not a database");
    }
}
