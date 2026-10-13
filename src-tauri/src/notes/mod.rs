//! Private notes: titles and project links in plaintext, bodies encrypted with a key held in a
//! `KeyStore`. Nothing here logs, and no error or Debug output carries a body or key bytes.

mod crypto;
mod keys;

use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};

use crate::db::Storage;
use crate::projects;

pub use keys::{CredentialKeyStore, KeyBytes, KeyStore};

pub const MAX_TITLE_CHARS: usize = 200;
pub const MAX_BODY_BYTES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteCode {
    NoteNotFound,
    TitleTooLong,
    BodyTooLarge,
    ProjectNotFound,
    KeyMissing,
    KeyringUnavailable,
    StorageUnavailable,
    BodyUnreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteError {
    pub code: NoteCode,
}

impl NoteError {
    fn new(code: NoteCode) -> Self {
        Self { code }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSummary {
    pub id: i64,
    pub title: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Note {
    pub summary: NoteSummary,
    pub body: String,
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("summary", &self.summary)
            .field("body", &format_args!("<redacted>"))
            .finish()
    }
}

fn storage_err<E>(_: E) -> NoteError {
    NoteError::new(NoteCode::StorageUnavailable)
}

fn validate_title(title: &str) -> Result<(), NoteError> {
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(NoteError::new(NoteCode::TitleTooLong));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), NoteError> {
    if body.len() > MAX_BODY_BYTES {
        return Err(NoteError::new(NoteCode::BodyTooLarge));
    }
    Ok(())
}

fn check_project(storage: &Storage, project_id: Option<i64>) -> Result<(), NoteError> {
    if let Some(pid) = project_id {
        match projects::get(storage, pid) {
            Ok(Some(_)) => {}
            Ok(None) => return Err(NoteError::new(NoteCode::ProjectNotFound)),
            Err(e) => return Err(storage_err(e)),
        }
    }
    Ok(())
}

fn note_exists(storage: &Storage, id: i64) -> Result<bool, NoteError> {
    storage
        .connection()
        .query_row("SELECT 1 FROM notes WHERE id = ?1", [id], |_| Ok(()))
        .optional()
        .map(|row| row.is_some())
        .map_err(storage_err)
}

fn any_notes(storage: &Storage) -> Result<bool, NoteError> {
    storage
        .connection()
        .query_row("SELECT EXISTS(SELECT 1 FROM notes)", [], |r| r.get::<_, i64>(0))
        .map(|n| n != 0)
        .map_err(storage_err)
}

/// Loads the data key. Touches no storage, so callers run it before taking the storage lock.
pub fn load_key(keys: &dyn KeyStore) -> Result<Option<KeyBytes>, NoteError> {
    keys.load()
        .map_err(|_| NoteError::new(NoteCode::KeyringUnavailable))
}

/// Key lifecycle, run under the storage lock with the result of `load_key`: a loaded key is
/// used; if absent and notes exist, `KeyMissing` (never mint a key over existing notes); if
/// absent and no notes exist, generate a key, store it, and use it.
fn resolve_key(
    storage: &Storage,
    keys: &dyn KeyStore,
    loaded: Option<KeyBytes>,
) -> Result<KeyBytes, NoteError> {
    if let Some(key) = loaded {
        return Ok(key);
    }
    if any_notes(storage)? {
        return Err(NoteError::new(NoteCode::KeyMissing));
    }
    let key =
        KeyBytes::generate().map_err(|_| NoteError::new(NoteCode::KeyringUnavailable))?;
    // First use only: this store is the one keyring call made under the storage lock. It cannot
    // move before the lock because it is only legal while the notes table is empty.
    keys.store(&key)
        .map_err(|_| NoteError::new(NoteCode::KeyringUnavailable))?;
    Ok(key)
}

fn project_names(storage: &Storage) -> Result<HashMap<i64, String>, NoteError> {
    let all = projects::list(storage).map_err(storage_err)?;
    Ok(all.into_iter().map(|p| (p.id, p.display_name)).collect())
}

fn project_name(storage: &Storage, project_id: Option<i64>) -> Result<Option<String>, NoteError> {
    match project_id {
        None => Ok(None),
        Some(pid) => Ok(projects::get(storage, pid)
            .map_err(storage_err)?
            .map(|p| p.display_name)),
    }
}

fn summary_by_id(storage: &Storage, id: i64) -> Result<NoteSummary, NoteError> {
    let (title, project_id, updated_at) = storage
        .connection()
        .query_row(
            "SELECT title, project_id, updated_at FROM notes WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(storage_err)?
        .ok_or_else(|| NoteError::new(NoteCode::NoteNotFound))?;
    Ok(NoteSummary {
        id,
        title,
        project_id,
        project_name: project_name(storage, project_id)?,
        updated_at,
    })
}

/// All notes, newest edit first. Needs no key.
pub fn list(storage: &Storage) -> Result<Vec<NoteSummary>, NoteError> {
    let names = project_names(storage)?;
    let conn = storage.connection();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, project_id, updated_at FROM notes
             ORDER BY updated_at DESC, id DESC",
        )
        .map_err(storage_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(storage_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, project_id, updated_at) = row.map_err(storage_err)?;
        out.push(NoteSummary {
            id,
            title,
            project_id,
            project_name: project_id.and_then(|pid| names.get(&pid).cloned()),
            updated_at,
        });
    }
    Ok(out)
}

/// `loaded` is the result of `load_key`, taken before the storage lock.
pub fn get(
    storage: &Storage,
    keys: &dyn KeyStore,
    loaded: Option<KeyBytes>,
    id: i64,
) -> Result<Note, NoteError> {
    let summary = summary_by_id(storage, id)?;
    let blob: Vec<u8> = storage
        .connection()
        .query_row("SELECT body_enc FROM notes WHERE id = ?1", [id], |r| r.get(0))
        .map_err(storage_err)?;
    let key = resolve_key(storage, keys, loaded)?;
    let plain = crypto::decrypt(&key, id, &blob)
        .map_err(|_| NoteError::new(NoteCode::BodyUnreadable))?;
    let body = String::from_utf8(plain).map_err(|_| NoteError::new(NoteCode::BodyUnreadable))?;
    Ok(Note { summary, body })
}

/// Creates an untitled note with an empty body. The row is inserted, then its body is encrypted
/// with the new id as associated data, in one transaction.
/// `loaded` is the result of `load_key`, taken before the storage lock.
pub fn create(
    storage: &Storage,
    keys: &dyn KeyStore,
    loaded: Option<KeyBytes>,
    project_id: Option<i64>,
) -> Result<Note, NoteError> {
    check_project(storage, project_id)?;
    let key = resolve_key(storage, keys, loaded)?;
    let tx = storage.connection().unchecked_transaction().map_err(storage_err)?;
    tx.execute(
        "INSERT INTO notes (project_id, title, body_enc) VALUES (?1, '', ?2)",
        params![project_id, Vec::<u8>::new()],
    )
    .map_err(storage_err)?;
    let id = tx.last_insert_rowid();
    let blob = crypto::encrypt(&key, id, b"").map_err(storage_err)?;
    tx.execute(
        "UPDATE notes SET body_enc = ?1 WHERE id = ?2",
        params![blob, id],
    )
    .map_err(storage_err)?;
    tx.commit().map_err(storage_err)?;
    Ok(Note {
        summary: summary_by_id(storage, id)?,
        body: String::new(),
    })
}

/// `loaded` is the result of `load_key`, taken before the storage lock.
pub fn save(
    storage: &Storage,
    keys: &dyn KeyStore,
    loaded: Option<KeyBytes>,
    id: i64,
    title: &str,
    project_id: Option<i64>,
    body: &str,
) -> Result<NoteSummary, NoteError> {
    validate_title(title)?;
    validate_body(body)?;
    check_project(storage, project_id)?;
    if !note_exists(storage, id)? {
        return Err(NoteError::new(NoteCode::NoteNotFound));
    }
    let key = resolve_key(storage, keys, loaded)?;
    let blob = crypto::encrypt(&key, id, body.as_bytes()).map_err(storage_err)?;
    let changed = storage
        .connection()
        .execute(
            "UPDATE notes
             SET title = ?1, project_id = ?2, body_enc = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?4",
            params![title, project_id, blob, id],
        )
        .map_err(storage_err)?;
    if changed == 0 {
        return Err(NoteError::new(NoteCode::NoteNotFound));
    }
    summary_by_id(storage, id)
}

pub fn delete(storage: &Storage, id: i64) -> Result<(), NoteError> {
    let changed = storage
        .connection()
        .execute("DELETE FROM notes WHERE id = ?1", [id])
        .map_err(storage_err)?;
    if changed == 0 {
        return Err(NoteError::new(NoteCode::NoteNotFound));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::keys::{KeyBytes, KeyStore, KeyStoreError, MemoryKeyStore};
    use super::*;
    use crate::db::{open_in, Storage};

    const SECRET: &str = "SECRET-BODY-TEXT private words";

    // The real functions take a key already loaded outside the lock; these load it the way the
    // commands do, so the tests below exercise the same path.
    fn get(storage: &Storage, keys: &dyn KeyStore, id: i64) -> Result<Note, NoteError> {
        super::get(storage, keys, load_key(keys)?, id)
    }

    fn create(
        storage: &Storage,
        keys: &dyn KeyStore,
        project_id: Option<i64>,
    ) -> Result<Note, NoteError> {
        super::create(storage, keys, load_key(keys)?, project_id)
    }

    fn save(
        storage: &Storage,
        keys: &dyn KeyStore,
        id: i64,
        title: &str,
        project_id: Option<i64>,
        body: &str,
    ) -> Result<NoteSummary, NoteError> {
        super::save(storage, keys, load_key(keys)?, id, title, project_id, body)
    }

    /// Counts `store` calls on top of a `MemoryKeyStore`.
    struct CountingStore {
        inner: MemoryKeyStore,
        stores: std::sync::atomic::AtomicUsize,
    }
    impl KeyStore for CountingStore {
        fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError> {
            self.inner.load()
        }
        fn store(&self, key: &KeyBytes) -> Result<(), KeyStoreError> {
            self.stores.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.store(key)
        }
    }

    #[test]
    fn resolve_on_empty_table_creates_a_key_once_and_reuses_it() {
        let (_t, storage) = storage();
        let keys = CountingStore {
            inner: MemoryKeyStore::new(),
            stores: std::sync::atomic::AtomicUsize::new(0),
        };
        let first = resolve_key(&storage, &keys, load_key(&keys).unwrap()).unwrap();
        assert_eq!(keys.stores.load(std::sync::atomic::Ordering::SeqCst), 1);
        let second = resolve_key(&storage, &keys, load_key(&keys).unwrap()).unwrap();
        assert_eq!(keys.stores.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    #[test]
    fn load_key_maps_keystore_failure_to_keyring_unavailable() {
        assert_eq!(
            load_key(&UnavailableStore).unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
    }

    fn storage() -> (tempfile::TempDir, Storage) {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open_in(tmp.path()).unwrap();
        (tmp, storage)
    }

    fn add_project(storage: &Storage, name: &str) -> i64 {
        let conn = storage.connection();
        conn.execute(
            "INSERT INTO projects (display_name, root_path, path_key, drive, project_type)
             VALUES (?1, ?2, ?3, 'F:', 'other')",
            rusqlite::params![name, format!("F:\\{name}"), format!("f:\\{name}")],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn set_updated_at(storage: &Storage, id: i64, value: &str) {
        storage
            .connection()
            .execute(
                "UPDATE notes SET updated_at = ?1 WHERE id = ?2",
                rusqlite::params![value, id],
            )
            .unwrap();
    }

    fn note_count(storage: &Storage) -> i64 {
        storage
            .connection()
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .unwrap()
    }

    struct UnavailableStore;
    impl KeyStore for UnavailableStore {
        fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError> {
            Err(KeyStoreError::Unavailable)
        }
        fn store(&self, _key: &KeyBytes) -> Result<(), KeyStoreError> {
            Err(KeyStoreError::Unavailable)
        }
    }

    /// Load works (no key yet) but storing fails.
    struct StoreFailsStore;
    impl KeyStore for StoreFailsStore {
        fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError> {
            Ok(None)
        }
        fn store(&self, _key: &KeyBytes) -> Result<(), KeyStoreError> {
            Err(KeyStoreError::Unavailable)
        }
    }

    #[test]
    fn create_with_no_key_and_no_notes_creates_a_key() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        assert!(!keys.is_empty());
        assert_eq!(note.summary.title, "");
        assert_eq!(note.body, "");
        assert_eq!(note.summary.project_id, None);
        assert_eq!(note.summary.project_name, None);
        assert_eq!(note_count(&storage), 1);
    }

    #[test]
    fn create_stores_body_encrypted_with_the_new_id() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let blob: Vec<u8> = storage
            .connection()
            .query_row(
                "SELECT body_enc FROM notes WHERE id = ?1",
                [note.summary.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(blob.len(), 24 + 16);
        let key = keys.load().unwrap().unwrap();
        assert_eq!(
            super::crypto::decrypt(&key, note.summary.id, &blob).unwrap(),
            b""
        );
    }

    #[test]
    fn create_with_project_links_it() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let pid = add_project(&storage, "alpha");
        let note = create(&storage, &keys, Some(pid)).unwrap();
        assert_eq!(note.summary.project_id, Some(pid));
        assert_eq!(note.summary.project_name.as_deref(), Some("alpha"));
    }

    #[test]
    fn create_with_unknown_project_is_project_not_found_and_writes_nothing() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let err = create(&storage, &keys, Some(999)).unwrap_err();
        assert_eq!(err.code, NoteCode::ProjectNotFound);
        assert_eq!(note_count(&storage), 0);
        assert!(keys.is_empty());
    }

    #[test]
    fn save_then_get_returns_the_body() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let summary = save(&storage, &keys, note.summary.id, "Title", None, SECRET).unwrap();
        assert_eq!(summary.title, "Title");
        let loaded = get(&storage, &keys, note.summary.id).unwrap();
        assert_eq!(loaded.body, SECRET);
        assert_eq!(loaded.summary, summary);
    }

    #[test]
    fn body_is_not_stored_in_plaintext() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        save(&storage, &keys, note.summary.id, "T", None, SECRET).unwrap();
        let blob: Vec<u8> = storage
            .connection()
            .query_row(
                "SELECT body_enc FROM notes WHERE id = ?1",
                [note.summary.id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!blob
            .windows(SECRET.len())
            .any(|w| w == SECRET.as_bytes()));
    }

    #[test]
    fn every_save_writes_a_fresh_nonce() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let read = || -> Vec<u8> {
            storage
                .connection()
                .query_row(
                    "SELECT body_enc FROM notes WHERE id = ?1",
                    [note.summary.id],
                    |r| r.get(0),
                )
                .unwrap()
        };
        save(&storage, &keys, note.summary.id, "T", None, SECRET).unwrap();
        let first = read();
        save(&storage, &keys, note.summary.id, "T", None, SECRET).unwrap();
        let second = read();
        assert_ne!(first[..24], second[..24]);
    }

    #[test]
    fn save_moves_note_between_project_and_general() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let pid = add_project(&storage, "alpha");
        let note = create(&storage, &keys, None).unwrap();
        let s = save(&storage, &keys, note.summary.id, "T", Some(pid), "b").unwrap();
        assert_eq!(s.project_id, Some(pid));
        assert_eq!(s.project_name.as_deref(), Some("alpha"));
        let s = save(&storage, &keys, note.summary.id, "T", None, "b").unwrap();
        assert_eq!(s.project_id, None);
        assert_eq!(s.project_name, None);
    }

    #[test]
    fn body_limit_is_inclusive_and_one_over_is_rejected() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let id = note.summary.id;
        let max = "a".repeat(MAX_BODY_BYTES);
        save(&storage, &keys, id, "T", None, &max).unwrap();
        assert_eq!(get(&storage, &keys, id).unwrap().body.len(), MAX_BODY_BYTES);

        let over = "a".repeat(MAX_BODY_BYTES + 1);
        let err = save(&storage, &keys, id, "T", None, &over).unwrap_err();
        assert_eq!(err.code, NoteCode::BodyTooLarge);
        // The stored body is unchanged by the rejected save.
        assert_eq!(get(&storage, &keys, id).unwrap().body.len(), MAX_BODY_BYTES);
    }

    #[test]
    fn body_limit_counts_bytes_not_chars() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        // 500_001 two-byte characters = 1_000_002 bytes.
        let body = "é".repeat(500_001);
        let err = save(&storage, &keys, note.summary.id, "T", None, &body).unwrap_err();
        assert_eq!(err.code, NoteCode::BodyTooLarge);
    }

    #[test]
    fn title_limit_counts_chars() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let id = note.summary.id;
        let ok = "é".repeat(MAX_TITLE_CHARS);
        save(&storage, &keys, id, &ok, None, "b").unwrap();
        assert_eq!(get(&storage, &keys, id).unwrap().summary.title, ok);

        let long = "x".repeat(MAX_TITLE_CHARS + 1);
        let err = save(&storage, &keys, id, &long, None, "b").unwrap_err();
        assert_eq!(err.code, NoteCode::TitleTooLong);
        assert_eq!(get(&storage, &keys, id).unwrap().summary.title, ok);
    }

    #[test]
    fn blank_title_is_allowed() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let s = save(&storage, &keys, note.summary.id, "", None, "b").unwrap();
        assert_eq!(s.title, "");
    }

    #[test]
    fn save_unknown_project_is_project_not_found() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let note = create(&storage, &keys, None).unwrap();
        let err = save(&storage, &keys, note.summary.id, "T", Some(999), "b").unwrap_err();
        assert_eq!(err.code, NoteCode::ProjectNotFound);
    }

    #[test]
    fn save_unknown_note_is_not_found_and_creates_no_key() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let err = save(&storage, &keys, 42, "T", None, "b").unwrap_err();
        assert_eq!(err.code, NoteCode::NoteNotFound);
        assert!(keys.is_empty());
        assert_eq!(note_count(&storage), 0);
    }

    #[test]
    fn get_unknown_note_is_not_found() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let err = get(&storage, &keys, 42).unwrap_err();
        assert_eq!(err.code, NoteCode::NoteNotFound);
        assert!(keys.is_empty());
    }

    #[test]
    fn list_orders_by_updated_at_desc_then_id_desc() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let pid = add_project(&storage, "alpha");
        let a = create(&storage, &keys, None).unwrap().summary.id;
        let b = create(&storage, &keys, Some(pid)).unwrap().summary.id;
        let c = create(&storage, &keys, None).unwrap().summary.id;
        let d = create(&storage, &keys, None).unwrap().summary.id;
        set_updated_at(&storage, a, "2026-01-03T00:00:00Z");
        set_updated_at(&storage, b, "2026-01-01T00:00:00Z");
        set_updated_at(&storage, c, "2026-01-02T00:00:00Z");
        set_updated_at(&storage, d, "2026-01-02T00:00:00Z");
        let list = list(&storage).unwrap();
        let ids: Vec<i64> = list.iter().map(|n| n.id).collect();
        assert_eq!(ids, vec![a, d, c, b]);
        assert_eq!(list[3].project_name.as_deref(), Some("alpha"));
        assert_eq!(list[0].project_name, None);
        assert_eq!(list[0].updated_at, "2026-01-03T00:00:00Z");
    }

    #[test]
    fn list_needs_no_key_and_survives_a_missing_project() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let pid = add_project(&storage, "alpha");
        let id = create(&storage, &keys, Some(pid)).unwrap().summary.id;
        storage
            .connection()
            .execute("DELETE FROM projects WHERE id = ?1", [pid])
            .unwrap();
        let list = list(&storage).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].project_name, None);
    }

    #[test]
    fn save_updates_updated_at_in_utc_iso_format() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        set_updated_at(&storage, id, "2000-01-01T00:00:00Z");
        let s = save(&storage, &keys, id, "T", None, "b").unwrap();
        assert_ne!(s.updated_at, "2000-01-01T00:00:00Z");
        assert_eq!(s.updated_at.len(), 20);
        assert!(s.updated_at.ends_with('Z'));
    }

    #[test]
    fn delete_removes_the_row_and_get_then_fails() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        delete(&storage, id).unwrap();
        assert_eq!(note_count(&storage), 0);
        assert_eq!(
            get(&storage, &keys, id).unwrap_err().code,
            NoteCode::NoteNotFound
        );
        assert_eq!(delete(&storage, id).unwrap_err().code, NoteCode::NoteNotFound);
    }

    #[test]
    fn delete_needs_no_key() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        // delete takes no key store at all; the signature is the proof.
        delete(&storage, id).unwrap();
    }

    #[test]
    fn key_missing_with_existing_notes_never_creates_a_key() {
        let (_tmp, storage) = storage();
        let with_key = MemoryKeyStore::new();
        let id = create(&storage, &with_key, None).unwrap().summary.id;
        save(&storage, &with_key, id, "T", None, SECRET).unwrap();
        let blob_before: Vec<u8> = storage
            .connection()
            .query_row("SELECT body_enc FROM notes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();

        let no_key = MemoryKeyStore::new();
        assert_eq!(get(&storage, &no_key, id).unwrap_err().code, NoteCode::KeyMissing);
        assert_eq!(
            save(&storage, &no_key, id, "T2", None, "x").unwrap_err().code,
            NoteCode::KeyMissing
        );
        assert_eq!(
            create(&storage, &no_key, None).unwrap_err().code,
            NoteCode::KeyMissing
        );
        assert!(no_key.is_empty());
        assert_eq!(note_count(&storage), 1);

        // Nothing was rewritten, and the list still works.
        let blob_after: Vec<u8> = storage
            .connection()
            .query_row("SELECT body_enc FROM notes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(blob_before, blob_after);
        assert_eq!(list(&storage).unwrap()[0].title, "T");
        // The original key still reads the note.
        assert_eq!(get(&storage, &with_key, id).unwrap().body, SECRET);
    }

    #[test]
    fn key_missing_error_carries_no_body() {
        let (_tmp, storage) = storage();
        let with_key = MemoryKeyStore::new();
        let id = create(&storage, &with_key, None).unwrap().summary.id;
        save(&storage, &with_key, id, "T", None, SECRET).unwrap();
        let err = get(&storage, &MemoryKeyStore::new(), id).unwrap_err();
        assert!(!format!("{err:?}").contains("SECRET-BODY-TEXT"));
    }

    #[test]
    fn keyring_unavailable_for_get_save_and_create() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        let bad = UnavailableStore;
        assert_eq!(
            get(&storage, &bad, id).unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
        assert_eq!(
            save(&storage, &bad, id, "T", None, "b").unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
        assert_eq!(
            create(&storage, &bad, None).unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
        assert_eq!(note_count(&storage), 1);
    }

    #[test]
    fn keyring_unavailable_on_empty_table_creates_no_note() {
        let (_tmp, storage) = storage();
        assert_eq!(
            create(&storage, &UnavailableStore, None).unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
        assert_eq!(note_count(&storage), 0);
    }

    #[test]
    fn failed_key_store_is_keyring_unavailable_and_creates_no_note() {
        let (_tmp, storage) = storage();
        assert_eq!(
            create(&storage, &StoreFailsStore, None).unwrap_err().code,
            NoteCode::KeyringUnavailable
        );
        assert_eq!(note_count(&storage), 0);
    }

    #[test]
    fn body_copied_to_another_note_is_unreadable() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let a = create(&storage, &keys, None).unwrap().summary.id;
        let b = create(&storage, &keys, None).unwrap().summary.id;
        save(&storage, &keys, a, "A", None, SECRET).unwrap();
        save(&storage, &keys, b, "B", None, "other").unwrap();
        storage
            .connection()
            .execute(
                "UPDATE notes SET body_enc = (SELECT body_enc FROM notes WHERE id = ?1) WHERE id = ?2",
                rusqlite::params![a, b],
            )
            .unwrap();
        assert_eq!(get(&storage, &keys, a).unwrap().body, SECRET);
        assert_eq!(get(&storage, &keys, b).unwrap_err().code, NoteCode::BodyUnreadable);
    }

    #[test]
    fn tampered_or_garbage_body_is_unreadable() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        save(&storage, &keys, id, "T", None, SECRET).unwrap();
        storage
            .connection()
            .execute(
                "UPDATE notes SET body_enc = X'0102' WHERE id = ?1",
                [id],
            )
            .unwrap();
        assert_eq!(get(&storage, &keys, id).unwrap_err().code, NoteCode::BodyUnreadable);
    }

    #[test]
    fn body_that_is_not_utf8_is_unreadable() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        let key = keys.load().unwrap().unwrap();
        let blob = super::crypto::encrypt(&key, id, &[0xFF, 0xFE, 0xFD]).unwrap();
        storage
            .connection()
            .execute(
                "UPDATE notes SET body_enc = ?1 WHERE id = ?2",
                rusqlite::params![blob, id],
            )
            .unwrap();
        assert_eq!(get(&storage, &keys, id).unwrap_err().code, NoteCode::BodyUnreadable);
    }

    #[test]
    fn unreadable_body_does_not_block_other_notes_or_save() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let a = create(&storage, &keys, None).unwrap().summary.id;
        let b = create(&storage, &keys, None).unwrap().summary.id;
        storage
            .connection()
            .execute("UPDATE notes SET body_enc = X'00' WHERE id = ?1", [a])
            .unwrap();
        assert_eq!(list(&storage).unwrap().len(), 2);
        assert_eq!(get(&storage, &keys, b).unwrap().body, "");
        // Saving over a damaged note replaces the body with a readable one.
        save(&storage, &keys, a, "A", None, "fresh").unwrap();
        assert_eq!(get(&storage, &keys, a).unwrap().body, "fresh");
    }

    #[test]
    fn debug_output_contains_no_body_or_key_bytes() {
        let (_tmp, storage) = storage();
        let keys = MemoryKeyStore::new();
        let id = create(&storage, &keys, None).unwrap().summary.id;
        save(&storage, &keys, id, "Title", None, SECRET).unwrap();
        let note = get(&storage, &keys, id).unwrap();
        let key = KeyBytes::from_bytes([0xAB; 32]);

        let outputs = [
            format!("{note:?}"),
            format!("{:#?}", note),
            format!("{:?}", note.summary),
            format!("{key:?}"),
            format!("{keys:?}"),
            format!("{:?}", NoteError { code: NoteCode::BodyUnreadable }),
            format!("{:?}", KeyStoreError::Unavailable),
            format!("{:?}", get(&storage, &MemoryKeyStore::new(), id).unwrap_err()),
        ];
        for text in outputs {
            assert!(!text.contains("SECRET-BODY-TEXT"), "{text}");
            assert!(!text.contains("private words"), "{text}");
            assert!(!text.contains("171"), "{text}");
        }
    }

    #[test]
    fn note_error_debug_shows_only_the_code() {
        let err = NoteError { code: NoteCode::KeyMissing };
        assert_eq!(format!("{err:?}"), "NoteError { code: KeyMissing }");
    }
}
