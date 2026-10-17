//! Secret entries: label, type, project and environment in plaintext columns, the value
//! encrypted under the vault key with the row id as associated data. Every function first
//! requires an unlocked vault. Nothing here logs, and no error or Debug output carries a value.

mod clipboard;
mod crypto;

use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};
use zeroize::Zeroizing;

use crate::db::Storage;
use crate::projects;
use crate::vault::{self, VaultSession};

pub use clipboard::{Clipboard, SystemClipboard};

pub const MAX_LABEL_CHARS: usize = 120;
pub const MAX_VALUE_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretCode {
    LabelRequired,
    ValueRequired,
    TypeRequired,
    EnvironmentRequired,
    LabelTooLong,
    ValueTooLong,
    DuplicateLabel,
    NotFound,
    Unreadable,
    Locked,
    StorageUnavailable,
    ClipboardUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretError {
    pub code: SecretCode,
}

impl SecretError {
    fn new(code: SecretCode) -> Self {
        Self { code }
    }
}

fn storage_err<E>(_: E) -> SecretError {
    SecretError::new(SecretCode::StorageUnavailable)
}

/// A unique-index violation is the duplicate label; anything else is storage trouble.
fn write_err(e: rusqlite::Error) -> SecretError {
    if let rusqlite::Error::SqliteFailure(f, _) = &e {
        if f.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE {
            return SecretError::new(SecretCode::DuplicateLabel);
        }
    }
    storage_err(e)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretType {
    ApiKey,
    Password,
    Token,
    ConnectionString,
    Other,
}

impl SecretType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::Password => "password",
            Self::Token => "token",
            Self::ConnectionString => "connection_string",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "api_key" => Self::ApiKey,
            "password" => Self::Password,
            "token" => Self::Token,
            "connection_string" => Self::ConnectionString,
            "other" => Self::Other,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Dev,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "local" => Self::Local,
            "dev" => Self::Dev,
            "staging" => Self::Staging,
            "production" => Self::Production,
            _ => return None,
        })
    }
}

/// A secret without its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretSummary {
    pub id: i64,
    pub label: String,
    pub secret_type: SecretType,
    pub project_id: Option<i64>,
    /// The entry names a project that is no longer registered.
    pub project_missing: bool,
    pub environment: Environment,
    pub updated_at: String,
}

/// Fields of a create or update. `value` is `None` on update to keep the stored value.
pub struct SecretDraft<'a> {
    pub label: &'a str,
    pub secret_type: &'a str,
    pub project_id: Option<i64>,
    pub environment: &'a str,
    pub value: Option<&'a str>,
}

impl std::fmt::Debug for SecretDraft<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretDraft")
            .field("label", &self.label)
            .field("value", &format_args!("<redacted>"))
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SecretFilter {
    /// `None` = any project; `Some(None)` = entries with no project; `Some(Some(id))` = one project.
    pub project: Option<Option<i64>>,
    pub secret_type: Option<SecretType>,
}

fn require_unlocked(session: &VaultSession) -> Result<(), SecretError> {
    vault::with_key(session, |_| ()).map_err(|_| SecretError::new(SecretCode::Locked))
}

struct Meta {
    label: String,
    secret_type: SecretType,
    environment: Environment,
}

fn validate_meta(draft: &SecretDraft<'_>) -> Result<Meta, SecretError> {
    let label = draft.label.trim();
    if label.is_empty() {
        return Err(SecretError::new(SecretCode::LabelRequired));
    }
    if label.chars().count() > MAX_LABEL_CHARS {
        return Err(SecretError::new(SecretCode::LabelTooLong));
    }
    let secret_type = SecretType::parse(draft.secret_type)
        .ok_or_else(|| SecretError::new(SecretCode::TypeRequired))?;
    let environment = Environment::parse(draft.environment)
        .ok_or_else(|| SecretError::new(SecretCode::EnvironmentRequired))?;
    Ok(Meta {
        label: label.to_string(),
        secret_type,
        environment,
    })
}

fn validate_value(value: &str) -> Result<(), SecretError> {
    if value.is_empty() {
        return Err(SecretError::new(SecretCode::ValueRequired));
    }
    if value.len() > MAX_VALUE_BYTES {
        return Err(SecretError::new(SecretCode::ValueTooLong));
    }
    Ok(())
}

fn encrypt_value(session: &VaultSession, id: i64, value: &str) -> Result<Vec<u8>, SecretError> {
    vault::with_key(session, |key| crypto::encrypt(key, id, value.as_bytes()))
        .map_err(|_| SecretError::new(SecretCode::Locked))?
        .map_err(storage_err)
}

fn project_names(storage: &Storage) -> Result<HashMap<i64, String>, SecretError> {
    let all = projects::list(storage).map_err(storage_err)?;
    Ok(all.into_iter().map(|p| (p.id, p.display_name)).collect())
}

type Row = (i64, String, String, Option<i64>, String, String);

fn to_summary(row: Row, names: &HashMap<i64, String>) -> Result<SecretSummary, SecretError> {
    let (id, label, secret_type, project_id, environment, updated_at) = row;
    Ok(SecretSummary {
        id,
        label,
        secret_type: SecretType::parse(&secret_type).ok_or_else(|| storage_err(()))?,
        project_id,
        project_missing: project_id.is_some_and(|p| !names.contains_key(&p)),
        environment: Environment::parse(&environment).ok_or_else(|| storage_err(()))?,
        updated_at,
    })
}

fn read_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
}

const SELECT_SUMMARY: &str =
    "SELECT id, label, secret_type, project_id, environment, updated_at FROM secrets";

fn summary_by_id(storage: &Storage, id: i64) -> Result<SecretSummary, SecretError> {
    let row = storage
        .connection()
        .query_row(&format!("{SELECT_SUMMARY} WHERE id = ?1"), [id], read_row)
        .optional()
        .map_err(storage_err)?
        .ok_or_else(|| SecretError::new(SecretCode::NotFound))?;
    to_summary(row, &project_names(storage)?)
}

fn blob_by_id(storage: &Storage, id: i64) -> Result<Vec<u8>, SecretError> {
    storage
        .connection()
        .query_row("SELECT body_enc FROM secrets WHERE id = ?1", [id], |r| r.get(0))
        .optional()
        .map_err(storage_err)?
        .ok_or_else(|| SecretError::new(SecretCode::NotFound))
}

/// Entries ordered by label then environment, case-insensitively. Needs the unlocked vault but
/// never decrypts.
pub fn list(
    storage: &Storage,
    session: &VaultSession,
    filter: SecretFilter,
) -> Result<Vec<SecretSummary>, SecretError> {
    require_unlocked(session)?;
    let names = project_names(storage)?;
    let conn = storage.connection();
    let mut stmt = conn
        .prepare(&format!(
            "{SELECT_SUMMARY} ORDER BY label COLLATE NOCASE, environment COLLATE NOCASE, id"
        ))
        .map_err(storage_err)?;
    let rows = stmt.query_map([], read_row).map_err(storage_err)?;
    let mut out = Vec::new();
    for row in rows {
        let item = to_summary(row.map_err(storage_err)?, &names)?;
        if filter.project.is_some_and(|p| p != item.project_id) {
            continue;
        }
        if filter.secret_type.is_some_and(|t| t != item.secret_type) {
            continue;
        }
        out.push(item);
    }
    Ok(out)
}

/// The row is inserted with an empty body, then the value is encrypted with the new id as
/// associated data, in one transaction. A failed encrypt rolls the insert back.
pub fn create(
    storage: &Storage,
    session: &VaultSession,
    draft: &SecretDraft<'_>,
) -> Result<SecretSummary, SecretError> {
    require_unlocked(session)?;
    let meta = validate_meta(draft)?;
    let value = draft.value.unwrap_or("");
    validate_value(value)?;
    let tx = storage.connection().unchecked_transaction().map_err(storage_err)?;
    tx.execute(
        "INSERT INTO secrets (label, secret_type, project_id, environment, body_enc)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            meta.label,
            meta.secret_type.as_str(),
            draft.project_id,
            meta.environment.as_str(),
            Vec::<u8>::new()
        ],
    )
    .map_err(write_err)?;
    let id = tx.last_insert_rowid();
    let blob = encrypt_value(session, id, value)?;
    tx.execute(
        "UPDATE secrets SET body_enc = ?1 WHERE id = ?2",
        params![blob, id],
    )
    .map_err(storage_err)?;
    tx.commit().map_err(storage_err)?;
    summary_by_id(storage, id)
}

/// `draft.value == None` keeps the stored ciphertext untouched.
pub fn update(
    storage: &Storage,
    session: &VaultSession,
    id: i64,
    draft: &SecretDraft<'_>,
) -> Result<SecretSummary, SecretError> {
    require_unlocked(session)?;
    let meta = validate_meta(draft)?;
    if let Some(v) = draft.value {
        validate_value(v)?;
    }
    // Existence first so an unknown id is NotFound, not a silent no-op.
    summary_by_id(storage, id)?;
    let blob = match draft.value {
        Some(v) => Some(encrypt_value(session, id, v)?),
        None => None,
    };
    let conn = storage.connection();
    let changed = match blob {
        Some(blob) => conn.execute(
            "UPDATE secrets
             SET label = ?1, secret_type = ?2, project_id = ?3, environment = ?4, body_enc = ?5,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?6",
            params![
                meta.label,
                meta.secret_type.as_str(),
                draft.project_id,
                meta.environment.as_str(),
                blob,
                id
            ],
        ),
        None => conn.execute(
            "UPDATE secrets
             SET label = ?1, secret_type = ?2, project_id = ?3, environment = ?4,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?5",
            params![
                meta.label,
                meta.secret_type.as_str(),
                draft.project_id,
                meta.environment.as_str(),
                id
            ],
        ),
    }
    .map_err(write_err)?;
    if changed == 0 {
        return Err(SecretError::new(SecretCode::NotFound));
    }
    summary_by_id(storage, id)
}

/// Decrypts one value. Any failure (wrong key, moved or tampered ciphertext, bad UTF-8) is
/// `Unreadable` and carries no partial value.
fn decrypt_value(
    storage: &Storage,
    session: &VaultSession,
    id: i64,
) -> Result<Zeroizing<Vec<u8>>, SecretError> {
    let blob = blob_by_id(storage, id)?;
    vault::with_key(session, |key| crypto::decrypt(key, id, &blob))
        .map_err(|_| SecretError::new(SecretCode::Locked))?
        .map_err(|_| SecretError::new(SecretCode::Unreadable))
}

/// Returns one value on explicit request.
pub fn reveal(
    storage: &Storage,
    session: &VaultSession,
    id: i64,
) -> Result<Zeroizing<String>, SecretError> {
    require_unlocked(session)?;
    let plain = decrypt_value(storage, session, id)?;
    let text = String::from_utf8(plain.to_vec())
        .map_err(|_| SecretError::new(SecretCode::Unreadable))?;
    Ok(Zeroizing::new(text))
}

/// Decrypts and writes the value to the clipboard; the value never leaves this function.
pub fn copy(
    storage: &Storage,
    session: &VaultSession,
    clipboard: &dyn Clipboard,
    id: i64,
) -> Result<(), SecretError> {
    require_unlocked(session)?;
    let plain = decrypt_value(storage, session, id)?;
    let text = std::str::from_utf8(&plain).map_err(|_| SecretError::new(SecretCode::Unreadable))?;
    clipboard
        .set_text(text)
        .map_err(|_| SecretError::new(SecretCode::ClipboardUnavailable))
}

pub fn delete(storage: &Storage, session: &VaultSession, id: i64) -> Result<(), SecretError> {
    require_unlocked(session)?;
    let changed = storage
        .connection()
        .execute("DELETE FROM secrets WHERE id = ?1", [id])
        .map_err(storage_err)?;
    if changed == 0 {
        return Err(SecretError::new(SecretCode::NotFound));
    }
    Ok(())
}

/// Number of stored secrets. Database read only; needs no unlocked vault and never decrypts.
pub fn count_all(conn: &rusqlite::Connection) -> Result<i64, SecretError> {
    conn.query_row("SELECT COUNT(*) FROM secrets", [], |r| r.get(0)).map_err(storage_err)
}

/// Deletes every secret row and returns how many were removed. Used when the vault itself is
/// deleted, so it needs no unlocked vault; the caller proves the master password first and runs
/// this inside its own transaction.
pub fn delete_all(conn: &rusqlite::Connection) -> Result<i64, SecretError> {
    conn.execute("DELETE FROM secrets", []).map(|n| n as i64).map_err(storage_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::clipboard::ClipboardError;
    use crate::db::open_in;
    use std::sync::Mutex;

    const VALUE: &str = "sk-LIVE-very-secret-value-123";

    struct Fixture {
        dir: tempfile::TempDir,
        storage: Storage,
        session: VaultSession,
    }

    fn key(byte: u8) -> Zeroizing<[u8; 32]> {
        Zeroizing::new([byte; 32])
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let storage = open_in(dir.path()).unwrap();
        let session = VaultSession::new();
        session.unlock(key(7), 5);
        Fixture {
            dir,
            storage,
            session,
        }
    }

    #[derive(Default)]
    struct FakeClipboard {
        writes: Mutex<Vec<String>>,
        fail: bool,
    }

    impl Clipboard for FakeClipboard {
        fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
            if self.fail {
                return Err(ClipboardError);
            }
            self.writes.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    fn draft<'a>(label: &'a str, project_id: Option<i64>, value: Option<&'a str>) -> SecretDraft<'a> {
        SecretDraft {
            label,
            secret_type: "api_key",
            project_id,
            environment: "dev",
            value,
        }
    }

    fn add(f: &Fixture, label: &str, project_id: Option<i64>) -> SecretSummary {
        create(&f.storage, &f.session, &draft(label, project_id, Some(VALUE))).unwrap()
    }

    fn code<T: std::fmt::Debug>(r: Result<T, SecretError>) -> SecretCode {
        r.unwrap_err().code
    }

    fn blob(f: &Fixture, id: i64) -> Vec<u8> {
        blob_by_id(&f.storage, id).unwrap()
    }

    fn no_filter() -> SecretFilter {
        SecretFilter::default()
    }

    // AC-09, AC-16
    #[test]
    fn create_then_reveal_returns_value_and_list_holds_only_metadata() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        assert_eq!(s.label, "Stripe");
        assert_eq!(s.secret_type, SecretType::ApiKey);
        assert_eq!(s.environment, Environment::Dev);
        assert!(!s.project_missing);
        assert_eq!(&*reveal(&f.storage, &f.session, s.id).unwrap(), VALUE);

        let items = list(&f.storage, &f.session, no_filter()).unwrap();
        assert_eq!(items, vec![s]);
        assert!(!format!("{items:?}").contains(VALUE));
    }

    // AC-03, AC-04, AC-05
    #[test]
    fn validation_codes() {
        let f = fixture();
        let go = |d: SecretDraft<'_>| code(create(&f.storage, &f.session, &d));
        assert_eq!(go(draft("   ", None, Some(VALUE))), SecretCode::LabelRequired);
        assert_eq!(go(draft("a", None, Some(""))), SecretCode::ValueRequired);
        assert_eq!(go(draft("a", None, None)), SecretCode::ValueRequired);
        let mut d = draft("a", None, Some(VALUE));
        d.secret_type = "";
        assert_eq!(go(d), SecretCode::TypeRequired);
        let mut d = draft("a", None, Some(VALUE));
        d.secret_type = "bogus";
        assert_eq!(go(d), SecretCode::TypeRequired);
        let mut d = draft("a", None, Some(VALUE));
        d.environment = "";
        assert_eq!(go(d), SecretCode::EnvironmentRequired);
        let long = "x".repeat(121);
        assert_eq!(go(draft(&long, None, Some(VALUE))), SecretCode::LabelTooLong);
        let big = "v".repeat(4097);
        assert_eq!(go(draft("a", None, Some(&big))), SecretCode::ValueTooLong);
        assert!(list(&f.storage, &f.session, no_filter()).unwrap().is_empty());
    }

    // AC-04, AC-05 boundaries
    #[test]
    fn limits_are_inclusive_and_counted_in_chars_and_bytes() {
        let f = fixture();
        let label = "é".repeat(120); // 240 bytes, 120 chars
        create(&f.storage, &f.session, &draft(&label, None, Some(&"v".repeat(4096)))).unwrap();
        // 1366 three-byte chars = 4098 bytes > 4096
        let value = "€".repeat(1366);
        assert_eq!(
            code(create(&f.storage, &f.session, &draft("b", None, Some(&value)))),
            SecretCode::ValueTooLong
        );
    }

    // AC-04
    #[test]
    fn label_is_trimmed_and_unique_per_project_case_insensitively() {
        let f = fixture();
        let s = add(&f, "  Stripe  ", None);
        assert_eq!(s.label, "Stripe");
        let dup = create(&f.storage, &f.session, &draft("STRIPE", None, Some(VALUE)));
        assert_eq!(code(dup), SecretCode::DuplicateLabel);
        // A second no-project entry is a duplicate; a different project is allowed.
        add(&f, "Stripe", Some(1));
        add(&f, "Stripe", Some(2));
        assert_eq!(
            code(create(&f.storage, &f.session, &draft("stripe", Some(2), Some(VALUE)))),
            SecretCode::DuplicateLabel
        );
        assert_eq!(list(&f.storage, &f.session, no_filter()).unwrap().len(), 3);
    }

    // AC-04 edge: edit to a duplicate keeps the stored label
    #[test]
    fn update_to_duplicate_label_is_rejected_and_keeps_label() {
        let f = fixture();
        add(&f, "One", None);
        let two = add(&f, "Two", None);
        let r = update(&f.storage, &f.session, two.id, &draft("one", None, None));
        assert_eq!(code(r), SecretCode::DuplicateLabel);
        assert_eq!(summary_by_id(&f.storage, two.id).unwrap().label, "Two");
    }

    // AC-06, AC-07
    #[test]
    fn update_without_value_keeps_ciphertext_byte_identical() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let before = blob(&f, s.id);
        let mut d = draft("Stripe live", Some(5), None);
        d.secret_type = "token";
        d.environment = "production";
        let u = update(&f.storage, &f.session, s.id, &d).unwrap();
        assert_eq!(u.label, "Stripe live");
        assert_eq!(u.secret_type, SecretType::Token);
        assert_eq!(u.environment, Environment::Production);
        assert_eq!(u.project_id, Some(5));
        assert_eq!(blob(&f, s.id), before);
        assert_eq!(&*reveal(&f.storage, &f.session, s.id).unwrap(), VALUE);
    }

    #[test]
    fn update_with_value_replaces_it_with_a_fresh_nonce() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let before = blob(&f, s.id);
        update(&f.storage, &f.session, s.id, &draft("Stripe", None, Some("new-value"))).unwrap();
        let after = blob(&f, s.id);
        assert_ne!(after[..24], before[..24]);
        assert_eq!(&*reveal(&f.storage, &f.session, s.id).unwrap(), "new-value");
        assert_eq!(
            code(update(&f.storage, &f.session, s.id, &draft("Stripe", None, Some("")))),
            SecretCode::ValueRequired
        );
    }

    #[test]
    fn unknown_id_is_not_found_for_every_operation() {
        let f = fixture();
        let clip = FakeClipboard::default();
        assert_eq!(code(update(&f.storage, &f.session, 99, &draft("a", None, None))), SecretCode::NotFound);
        assert_eq!(code(reveal(&f.storage, &f.session, 99)), SecretCode::NotFound);
        assert_eq!(code(copy(&f.storage, &f.session, &clip, 99)), SecretCode::NotFound);
        assert_eq!(code(delete(&f.storage, &f.session, 99)), SecretCode::NotFound);
    }

    #[test]
    fn delete_removes_the_row() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        delete(&f.storage, &f.session, s.id).unwrap();
        assert!(list(&f.storage, &f.session, no_filter()).unwrap().is_empty());
        assert_eq!(code(reveal(&f.storage, &f.session, s.id)), SecretCode::NotFound);
    }

    // AC-11 (project separated so labels may repeat)
    #[test]
    fn list_order_is_label_then_environment() {
        let f = fixture();
        let mk = |label: &str, project: Option<i64>, env: &str| {
            let mut d = draft(label, project, Some(VALUE));
            d.environment = env;
            create(&f.storage, &f.session, &d).unwrap();
        };
        mk("beta", None, "staging");
        mk("Alpha", None, "production");
        mk("Beta", Some(1), "dev");
        mk("alpha", Some(1), "local");
        mk("Zed", None, "dev");
        let got: Vec<(String, &str)> = list(&f.storage, &f.session, no_filter())
            .unwrap()
            .into_iter()
            .map(|s| (s.label.to_lowercase(), s.environment.as_str()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("alpha".to_string(), "local"),
                ("alpha".to_string(), "production"),
                ("beta".to_string(), "dev"),
                ("beta".to_string(), "staging"),
                ("zed".to_string(), "dev"),
            ]
        );
    }

    // AC-10 filters are served by the list call
    #[test]
    fn list_filters_by_project_and_type() {
        let f = fixture();
        add(&f, "a", None);
        add(&f, "b", Some(1));
        let mut d = draft("c", Some(1), Some(VALUE));
        d.secret_type = "password";
        create(&f.storage, &f.session, &d).unwrap();
        let labels = |flt: SecretFilter| -> Vec<String> {
            list(&f.storage, &f.session, flt)
                .unwrap()
                .into_iter()
                .map(|s| s.label)
                .collect()
        };
        assert_eq!(labels(SecretFilter { project: Some(None), secret_type: None }), ["a"]);
        assert_eq!(labels(SecretFilter { project: Some(Some(1)), secret_type: None }), ["b", "c"]);
        assert_eq!(
            labels(SecretFilter { project: Some(Some(1)), secret_type: Some(SecretType::Password) }),
            ["c"]
        );
        assert!(labels(SecretFilter { project: Some(Some(2)), secret_type: None }).is_empty());
    }

    // AC-12: a project that is not registered is flagged, the entry stays
    #[test]
    fn unregistered_project_is_flagged_missing() {
        let f = fixture();
        let s = add(&f, "a", Some(42));
        assert!(s.project_missing);
        let none = add(&f, "b", None);
        assert!(!none.project_missing);
    }

    // AC-A08: a new project must not take over the secrets of an unregistered one
    #[test]
    fn secrets_of_an_unregistered_project_stay_missing_after_a_new_registration() {
        let f = fixture();
        let new_project = |name: &str| {
            let dir = f.dir.path().join(name);
            std::fs::create_dir_all(&dir).unwrap();
            let path = crate::projects::validate_path(&dir.to_string_lossy()).unwrap();
            crate::projects::register(
                &f.storage,
                path,
                crate::projects::NewProject {
                    display_name: name.to_string(),
                    project_type: "Other".to_string(),
                    tags_raw: String::new(),
                },
            )
            .unwrap()
        };
        let a = new_project("a");
        add(&f, "kept", Some(a.id));
        crate::projects::unregister(&f.storage, a.id).unwrap();
        let b = new_project("b");
        assert_ne!(b.id, a.id);

        let items = list(&f.storage, &f.session, SecretFilter { project: None, secret_type: None }).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].project_missing);
    }

    // AC-17, AC-18
    #[test]
    fn copy_writes_clipboard_and_returns_no_value() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let before = list(&f.storage, &f.session, no_filter()).unwrap();
        let clip = FakeClipboard::default();
        let r: Result<(), SecretError> = copy(&f.storage, &f.session, &clip, s.id);
        assert_eq!(r, Ok(()));
        assert_eq!(*clip.writes.lock().unwrap(), vec![VALUE.to_string()]);
        assert_eq!(list(&f.storage, &f.session, no_filter()).unwrap(), before);
    }

    #[test]
    fn copy_failure_is_clipboard_unavailable() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let clip = FakeClipboard {
            fail: true,
            ..Default::default()
        };
        assert_eq!(
            code(copy(&f.storage, &f.session, &clip, s.id)),
            SecretCode::ClipboardUnavailable
        );
    }

    // AC-22, AC-08 (storage side)
    #[test]
    fn database_files_hold_no_plaintext_value() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let stored = blob(&f, s.id);
        assert!(stored.len() >= 24 + 16 + VALUE.len());
        assert!(!stored.windows(VALUE.len()).any(|w| w == VALUE.as_bytes()));
        for entry in std::fs::read_dir(f.dir.path()).unwrap() {
            let bytes = std::fs::read(entry.unwrap().path()).unwrap();
            assert!(!bytes.windows(VALUE.len()).any(|w| w == VALUE.as_bytes()));
        }
    }

    #[test]
    fn same_value_gets_a_fresh_nonce_each_write() {
        let f = fixture();
        let a = add(&f, "a", None);
        let b = add(&f, "b", None);
        assert_ne!(blob(&f, a.id)[..24], blob(&f, b.id)[..24]);
    }

    // AC-25, AC-29: wrong key
    #[test]
    fn wrong_vault_key_is_unreadable() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        f.session.lock();
        f.session.unlock(key(8), 5);
        assert_eq!(code(reveal(&f.storage, &f.session, s.id)), SecretCode::Unreadable);
        let clip = FakeClipboard::default();
        assert_eq!(code(copy(&f.storage, &f.session, &clip, s.id)), SecretCode::Unreadable);
        assert!(clip.writes.lock().unwrap().is_empty());
    }

    // AC-25, AC-29: tampering
    #[test]
    fn tampered_or_truncated_ciphertext_is_unreadable() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        let mut bytes = blob(&f, s.id);
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        f.storage
            .connection()
            .execute("UPDATE secrets SET body_enc = ?1 WHERE id = ?2", params![bytes, s.id])
            .unwrap();
        assert_eq!(code(reveal(&f.storage, &f.session, s.id)), SecretCode::Unreadable);
        f.storage
            .connection()
            .execute("UPDATE secrets SET body_enc = X'0102' WHERE id = ?1", [s.id])
            .unwrap();
        assert_eq!(code(reveal(&f.storage, &f.session, s.id)), SecretCode::Unreadable);
    }

    // AC-24
    #[test]
    fn ciphertext_moved_to_another_row_is_unreadable() {
        let f = fixture();
        let a = add(&f, "a", None);
        let b = add(&f, "b", None);
        let a_blob = blob(&f, a.id);
        f.storage
            .connection()
            .execute("UPDATE secrets SET body_enc = ?1 WHERE id = ?2", params![a_blob, b.id])
            .unwrap();
        assert_eq!(code(reveal(&f.storage, &f.session, b.id)), SecretCode::Unreadable);
        assert_eq!(&*reveal(&f.storage, &f.session, a.id).unwrap(), VALUE);
    }

    // AC-30
    #[test]
    fn every_operation_is_locked_when_the_vault_is_locked() {
        let f = fixture();
        let s = add(&f, "Stripe", None);
        f.session.lock();
        let clip = FakeClipboard::default();
        let d = draft("x", None, Some(VALUE));
        assert_eq!(code(list(&f.storage, &f.session, no_filter())), SecretCode::Locked);
        assert_eq!(code(create(&f.storage, &f.session, &d)), SecretCode::Locked);
        assert_eq!(code(update(&f.storage, &f.session, s.id, &d)), SecretCode::Locked);
        assert_eq!(code(reveal(&f.storage, &f.session, s.id)), SecretCode::Locked);
        assert_eq!(code(copy(&f.storage, &f.session, &clip, s.id)), SecretCode::Locked);
        assert_eq!(code(delete(&f.storage, &f.session, s.id)), SecretCode::Locked);
        assert!(clip.writes.lock().unwrap().is_empty());
        // Nothing was written or removed while locked.
        f.session.unlock(key(7), 5);
        assert_eq!(list(&f.storage, &f.session, no_filter()).unwrap().len(), 1);
    }

    #[test]
    fn debug_output_redacts_the_value() {
        let d = draft("l", None, Some(VALUE));
        assert!(!format!("{d:?}").contains(VALUE));
    }

    // Migration 10
    #[test]
    fn migration_10_enforces_checks_and_unique_index() {
        let f = fixture();
        let conn = f.storage.connection();
        let ins = |t: &str, p: Option<i64>, l: &str, e: &str| {
            conn.execute(
                "INSERT INTO secrets (label, secret_type, project_id, environment, body_enc)
                 VALUES (?1, ?2, ?3, ?4, X'00')",
                params![l, t, p, e],
            )
        };
        assert!(ins("bogus", None, "a", "dev").is_err());
        assert!(ins("token", None, "a", "bogus").is_err());
        assert!(ins("token", None, "a", "dev").is_ok());
        assert!(ins("token", None, "A", "dev").is_err());
        assert!(ins("token", Some(1), "a", "dev").is_ok());
        assert_eq!(f.storage.schema_version(), 11);
    }
}
