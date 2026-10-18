//! Plaintext export: every table as CSV, written into a new folder the user picked. The files hold
//! decrypted note bodies and secret values, so callers gate this behind the master password and an
//! acknowledgement. `collect` reads everything before anything is written, so a failure leaves no
//! partial backup on disk.

mod csv;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

use crate::config;
use crate::db::Storage;
use crate::editors;
use crate::notes::{self, KeyBytes, KeyStore, NoteCode, NoteError};
use crate::projects;
use crate::secrets::{self, SecretCode, SecretError};
use crate::workspaces;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportCode {
    NoteKeyMissing,
    KeyringUnavailable,
    NoteUnreadable,
    SecretUnreadable,
    WriteFailed,
    StorageUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportError {
    pub code: ExportCode,
}

impl From<NoteError> for ExportError {
    fn from(e: NoteError) -> Self {
        let code = match e.code {
            NoteCode::KeyMissing => ExportCode::NoteKeyMissing,
            NoteCode::KeyringUnavailable => ExportCode::KeyringUnavailable,
            NoteCode::BodyUnreadable => ExportCode::NoteUnreadable,
            _ => ExportCode::StorageUnavailable,
        };
        Self { code }
    }
}

impl From<SecretError> for ExportError {
    fn from(e: SecretError) -> Self {
        let code = match e.code {
            SecretCode::Unreadable => ExportCode::SecretUnreadable,
            _ => ExportCode::StorageUnavailable,
        };
        Self { code }
    }
}

fn unavailable() -> ExportError {
    ExportError {
        code: ExportCode::StorageUnavailable,
    }
}

fn write_failed() -> ExportError {
    ExportError {
        code: ExportCode::WriteFailed,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFile {
    pub name: &'static str,
    pub rows: usize,
}

/// Every file's bytes, held in memory until `write` succeeds or the value is dropped.
pub struct Dataset {
    files: Vec<(ExportFile, Zeroizing<Vec<u8>>)>,
    stamp: String,
}

/// One CSV file under construction: header written at creation, rows appended.
struct Table {
    name: &'static str,
    buf: Zeroizing<Vec<u8>>,
    rows: usize,
}

impl Table {
    fn new(name: &'static str, header: &[&str]) -> Self {
        let mut buf = csv::new_csv();
        csv::write_row(&mut buf, header);
        Self { name, buf, rows: 0 }
    }

    fn row(&mut self, fields: &[&str]) {
        csv::write_row(&mut self.buf, fields);
        self.rows += 1;
    }

    fn finish(self) -> (ExportFile, Zeroizing<Vec<u8>>) {
        (
            ExportFile {
                name: self.name,
                rows: self.rows,
            },
            self.buf,
        )
    }
}

fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn opt_id(value: Option<i64>) -> String {
    value.map(|id| id.to_string()).unwrap_or_default()
}

/// Empty when the id is null or names a project that is no longer registered.
fn project_name(names: &HashMap<i64, String>, id: Option<i64>) -> String {
    id.and_then(|id| names.get(&id).cloned()).unwrap_or_default()
}

fn stamp(storage: &Storage) -> Result<String, ExportError> {
    storage
        .connection()
        .query_row("SELECT strftime('%Y%m%d-%H%M%S','now')", [], |r| r.get::<_, String>(0))
        .map_err(|_| unavailable())
}

/// Reads every table. Nothing is written to disk here; any failure aborts the whole export.
/// `note_key` is `notes::load_key` taken before the lock; `vault_key` is `None` when the vault was
/// never set up, which leaves `secrets.csv` with its header only.
pub fn collect(
    storage: &Storage,
    keys: &dyn KeyStore,
    note_key: Option<KeyBytes>,
    vault_key: Option<&[u8; 32]>,
) -> Result<Dataset, ExportError> {
    let stamp = stamp(storage)?;
    let project_list = projects::list(storage).map_err(|_| unavailable())?;
    let names: HashMap<i64, String> = project_list
        .iter()
        .map(|p| (p.id, p.display_name.clone()))
        .collect();

    let mut projects_table = Table::new(
        "projects.csv",
        &[
            "id",
            "display_name",
            "root_path",
            "drive",
            "project_type",
            "tags",
            "pinned",
            "created_at",
        ],
    );
    for p in &project_list {
        let id = p.id.to_string();
        let tags = p.tags.join(";");
        projects_table.row(&[
            id.as_str(),
            p.display_name.as_str(),
            p.root_path.as_str(),
            p.drive.as_str(),
            p.project_type.as_str(),
            tags.as_str(),
            bool_text(p.pinned),
            p.created_at.as_str(),
        ]);
    }

    let mut workspaces_table =
        Table::new("workspaces.csv", &["id", "name", "last_restored_at"]);
    let mut items_table = Table::new(
        "workspace_items.csv",
        &[
            "id",
            "workspace_id",
            "workspace_name",
            "position",
            "kind",
            "path",
            "project_id",
            "project_name",
            "editor_kind",
        ],
    );
    let summaries = workspaces::list(storage).map_err(|_| unavailable())?;
    for summary in &summaries {
        let workspace = workspaces::get(storage, summary.id)
            .map_err(|_| unavailable())?
            .ok_or_else(unavailable)?;
        let workspace_id = workspace.id.to_string();
        let last_restored = workspace.last_restored_at.clone().unwrap_or_default();
        workspaces_table.row(&[
            workspace_id.as_str(),
            workspace.name.as_str(),
            last_restored.as_str(),
        ]);
        for item in &workspace.items {
            let item_id = item.id.to_string();
            let position = item.position.to_string();
            let item_project = opt_id(item.project_id);
            let item_project_name = project_name(&names, item.project_id);
            items_table.row(&[
                item_id.as_str(),
                workspace_id.as_str(),
                workspace.name.as_str(),
                position.as_str(),
                item.kind.as_str(),
                item.path.as_str(),
                item_project.as_str(),
                item_project_name.as_str(),
                item.editor_kind.as_str(),
            ]);
        }
    }

    let mut config_table = Table::new(
        "config_files.csv",
        &["id", "project_id", "project_name", "path", "environment", "kind"],
    );
    for row in config::load_rows(storage).map_err(|_| unavailable())? {
        let id = row.id.to_string();
        let project_id = row.project_id.to_string();
        let config_project_name = project_name(&names, Some(row.project_id));
        config_table.row(&[
            id.as_str(),
            project_id.as_str(),
            config_project_name.as_str(),
            row.path.as_str(),
            row.environment.as_str(),
            row.kind.as_str(),
        ]);
    }

    let mut editors_table = Table::new(
        "editor_profiles.csv",
        &["kind", "name", "exe_path", "is_default"],
    );
    for profile in editors::list(storage).map_err(|_| unavailable())? {
        editors_table.row(&[
            profile.kind.as_str(),
            profile.name.as_str(),
            profile.exe_path.as_str(),
            bool_text(profile.is_default),
        ]);
    }

    let exported_notes = notes::export_all(storage, keys, note_key)?;
    let mut notes_table = Table::new(
        "notes.csv",
        &[
            "id",
            "project_id",
            "project_name",
            "title",
            "body",
            "created_at",
            "updated_at",
        ],
    );
    for note in &exported_notes {
        let id = note.id.to_string();
        let note_project = opt_id(note.project_id);
        let note_project_name = project_name(&names, note.project_id);
        notes_table.row(&[
            id.as_str(),
            note_project.as_str(),
            note_project_name.as_str(),
            note.title.as_str(),
            note.body.as_str(),
            note.created_at.as_str(),
            note.updated_at.as_str(),
        ]);
    }

    let exported_secrets = match vault_key {
        Some(key) => secrets::export_all(storage, key)?,
        None => Vec::new(),
    };
    let mut secrets_table = Table::new(
        "secrets.csv",
        &[
            "id",
            "label",
            "secret_type",
            "project_id",
            "project_name",
            "environment",
            "value",
            "created_at",
            "updated_at",
        ],
    );
    for secret in &exported_secrets {
        let id = secret.id.to_string();
        let secret_project = opt_id(secret.project_id);
        let secret_project_name = project_name(&names, secret.project_id);
        secrets_table.row(&[
            id.as_str(),
            secret.label.as_str(),
            secret.secret_type.as_str(),
            secret_project.as_str(),
            secret_project_name.as_str(),
            secret.environment.as_str(),
            secret.value.as_str(),
            secret.created_at.as_str(),
            secret.updated_at.as_str(),
        ]);
    }

    let files = [
        projects_table,
        workspaces_table,
        items_table,
        config_table,
        editors_table,
        notes_table,
        secrets_table,
    ]
    .into_iter()
    .map(Table::finish)
    .collect();
    Ok(Dataset { files, stamp })
}

/// Creates `devshelf-export-<stamp>` inside `parent` and writes each file. `create_dir` refuses an
/// existing folder, so an earlier export is never overwritten or removed. If a file write fails,
/// only the folder created here is removed.
pub fn write(dataset: &Dataset, parent: &Path) -> Result<(PathBuf, Vec<ExportFile>), ExportError> {
    let dir = parent.join(format!("devshelf-export-{}", dataset.stamp));
    std::fs::create_dir(&dir).map_err(|_| write_failed())?;
    let mut written = Vec::with_capacity(dataset.files.len());
    for (file, bytes) in &dataset.files {
        if std::fs::write(dir.join(file.name), bytes.as_slice()).is_err() {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(write_failed());
        }
        written.push(file.clone());
    }
    Ok((dir, written))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::db::open_in;
    use crate::notes::MemoryKeyStore;
    use crate::projects::NewProject;
    use crate::secrets::SecretDraft;
    use crate::vault::{self, VaultSession};
    use std::sync::Mutex;

    const PW: &str = "export test password";
    const NOTE_BODY: &str = "plain body line one\nline two";
    const SECRET_VALUE: &str = "sk-PLAIN-export-value-42";

    struct Fixture {
        data: tempfile::TempDir,
        out: tempfile::TempDir,
        db: Mutex<Storage>,
        session: VaultSession,
        keys: MemoryKeyStore,
    }

    /// A set-up, unlocked vault with no other data. Editor profiles come from the migrations.
    fn vault_only() -> Fixture {
        let data = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let db = Mutex::new(open_in(data.path()).unwrap());
        let session = VaultSession::new();
        vault::setup(&db, &session, PW, PW, true).unwrap();
        vault::unlock(&db, &session, PW).unwrap();
        Fixture {
            data,
            out,
            db,
            session,
            keys: MemoryKeyStore::new(),
        }
    }

    /// Adds one project with tags, a workspace item, a config file, a note and a secret.
    fn populated() -> Fixture {
        let f = vault_only();
        let project_dir = f.data.path().join("alpha");
        std::fs::create_dir_all(&project_dir).unwrap();
        let storage = f.db.lock().unwrap();
        let project = projects::register(
            &storage,
            projects::validate_path(&project_dir.to_string_lossy()).unwrap(),
            NewProject {
                display_name: "alpha".into(),
                project_type: "Node".into(),
                tags_raw: "web, api".into(),
            },
        )
        .unwrap();
        let workspace = workspaces::create(&storage, "daily").unwrap();
        workspaces::add_item(
            &storage,
            workspace.id,
            "F:\\work\\alpha\\src",
            "folder",
            Some(project.id),
            "windows_default",
        )
        .unwrap();
        config::add(&storage, project.id, "F:\\work\\alpha\\.env", "dev", "file").unwrap();
        let note = notes::create(&storage, &f.keys, None, Some(project.id)).unwrap();
        notes::save(
            &storage,
            &f.keys,
            notes::load_key(&f.keys).unwrap(),
            note.summary.id,
            "Ops",
            Some(project.id),
            NOTE_BODY,
        )
        .unwrap();
        secrets::create(
            &storage,
            &f.session,
            &SecretDraft {
                label: "api token",
                secret_type: "api_key",
                project_id: Some(project.id),
                environment: "dev",
                value: Some(SECRET_VALUE),
            },
        )
        .unwrap();
        drop(storage);
        f
    }

    fn read(dir: &Path, name: &str) -> String {
        std::fs::read_to_string(dir.join(name)).unwrap()
    }

    #[test]
    fn export_writes_seven_files_with_plaintext_notes_and_secrets() {
        let f = populated();
        let vault_key = vault::verified_key(&f.db, PW).unwrap().unwrap();
        let storage = f.db.lock().unwrap();
        let dataset = collect(
            &storage,
            &f.keys,
            notes::load_key(&f.keys).unwrap(),
            Some(&vault_key),
        )
        .unwrap();
        drop(storage);

        let (dir, files) = write(&dataset, f.out.path()).unwrap();

        let names: Vec<&str> = files.iter().map(|file| file.name).collect();
        assert_eq!(
            names,
            [
                "projects.csv",
                "workspaces.csv",
                "workspace_items.csv",
                "config_files.csv",
                "editor_profiles.csv",
                "notes.csv",
                "secrets.csv",
            ]
        );
        let rows: Vec<usize> = files.iter().map(|file| file.rows).collect();
        assert_eq!(rows, [1, 1, 1, 1, 4, 1, 1]);
        let notes_bytes = std::fs::read(dir.join("notes.csv")).unwrap();
        assert!(notes_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert!(read(&dir, "notes.csv").contains(NOTE_BODY));
        assert!(read(&dir, "secrets.csv").contains(SECRET_VALUE));
        assert!(read(&dir, "projects.csv").contains("api;web"));
        assert!(read(&dir, "secrets.csv").contains(",alpha,"));
    }

    #[test]
    fn notes_without_their_key_abort_the_export() {
        let f = populated();
        let vault_key = vault::verified_key(&f.db, PW).unwrap().unwrap();
        let storage = f.db.lock().unwrap();
        let err = collect(&storage, &MemoryKeyStore::new(), None, Some(&vault_key))
            .err()
            .unwrap();
        assert_eq!(err.code, ExportCode::NoteKeyMissing);
    }

    #[test]
    fn empty_note_store_mints_no_key_and_no_vault_leaves_secrets_header_only() {
        let f = vault_only();
        let storage = f.db.lock().unwrap();
        let dataset = collect(&storage, &f.keys, None, None).unwrap();
        drop(storage);
        assert!(f.keys.is_empty());

        let (dir, files) = write(&dataset, f.out.path()).unwrap();

        let secret_file = files.iter().find(|file| file.name == "secrets.csv").unwrap();
        assert_eq!(secret_file.rows, 0);
        assert!(read(&dir, "secrets.csv").starts_with(
            "\u{feff}id,label,secret_type,project_id,project_name,environment,value,created_at,updated_at\r\n"
        ));
        assert_eq!(read(&dir, "notes.csv").lines().count(), 1);
    }

    #[test]
    fn a_secret_that_cannot_be_decrypted_aborts_and_writes_nothing() {
        let f = populated();
        let vault_key = vault::verified_key(&f.db, PW).unwrap().unwrap();
        let storage = f.db.lock().unwrap();
        storage
            .connection()
            .execute("UPDATE secrets SET body_enc = zeroblob(8)", [])
            .unwrap();
        let err = collect(
            &storage,
            &f.keys,
            notes::load_key(&f.keys).unwrap(),
            Some(&vault_key),
        )
        .err()
        .unwrap();
        drop(storage);
        assert_eq!(err.code, ExportCode::SecretUnreadable);
        assert_eq!(std::fs::read_dir(f.out.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_second_write_with_the_same_stamp_never_overwrites_the_first() {
        let f = populated();
        let vault_key = vault::verified_key(&f.db, PW).unwrap().unwrap();
        let storage = f.db.lock().unwrap();
        let dataset = collect(
            &storage,
            &f.keys,
            notes::load_key(&f.keys).unwrap(),
            Some(&vault_key),
        )
        .unwrap();
        drop(storage);
        let (dir, _) = write(&dataset, f.out.path()).unwrap();
        let before = std::fs::read(dir.join("notes.csv")).unwrap();

        let err = write(&dataset, f.out.path()).err().unwrap();

        assert_eq!(err.code, ExportCode::WriteFailed);
        assert_eq!(std::fs::read(dir.join("notes.csv")).unwrap(), before);
    }
}
