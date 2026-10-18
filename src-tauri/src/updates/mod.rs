//! Persisted update preferences. The network check itself lives in `commands::updates`.

use rusqlite::{params, Connection, OptionalExtension};

/// `app_meta` key holding the "check for updates when DevShelf starts" flag.
pub const AUTO_CHECK_KEY: &str = "update_check_on_launch";

/// Defaults to `true` when no row exists; only the exact value `"0"` disables it.
pub fn auto_check_enabled(conn: &Connection) -> rusqlite::Result<bool> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = ?1",
            params![AUTO_CHECK_KEY],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.as_deref() != Some("0"))
}

pub fn set_auto_check(conn: &Connection, enabled: bool) -> rusqlite::Result<()> {
    let value = if enabled { "1" } else { "0" };
    conn.execute(
        "INSERT INTO app_meta (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![AUTO_CHECK_KEY, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_check_defaults_to_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = crate::db::open_in(tmp.path()).unwrap();
        assert!(auto_check_enabled(storage.connection()).unwrap());
    }

    #[test]
    fn auto_check_round_trips_and_upserts() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = crate::db::open_in(tmp.path()).unwrap();
        let conn = storage.connection();
        set_auto_check(conn, false).unwrap();
        assert!(!auto_check_enabled(conn).unwrap());
        set_auto_check(conn, true).unwrap();
        assert!(auto_check_enabled(conn).unwrap());
        set_auto_check(conn, false).unwrap();
        assert!(!auto_check_enabled(conn).unwrap());
    }

    #[test]
    fn unknown_stored_value_counts_as_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = crate::db::open_in(tmp.path()).unwrap();
        let conn = storage.connection();
        conn.execute(
            "INSERT INTO app_meta (key, value) VALUES (?1, 'garbage')",
            params![AUTO_CHECK_KEY],
        )
        .unwrap();
        assert!(auto_check_enabled(conn).unwrap());
    }
}
