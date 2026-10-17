//! Config catalog: registered config files per project. Only metadata is ever read; file
//! contents are never opened, and the registry never modifies a file.

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::Storage;
use crate::projects::normalize_path;

pub const ENVIRONMENTS: [&str; 4] = ["local", "dev", "staging", "production"];
pub const KINDS: [&str; 6] = [
    "yaml",
    "properties",
    "dotenv",
    "docker_compose",
    "caddy_nginx",
    "json",
];
pub const MAX_PATH_CHARS: usize = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCode {
    ConfigNotFound,
    PathEmpty,
    PathNotFound,
    PathIsFolder,
    PathRelative,
    PathTooLong,
    DuplicatePath,
    ProjectNotFound,
    EnvironmentInvalid,
    KindInvalid,
    StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub code: ConfigCode,
    pub detail: Option<String>,
}

impl ConfigError {
    pub fn new(code: ConfigCode) -> Self {
        Self { code, detail: None }
    }

    pub fn with_detail(code: ConfigCode, detail: &str) -> Self {
        Self {
            code,
            detail: Some(detail.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult {
    Found { is_dir: bool },
    NotFound,
    Unreachable,
}

pub trait PathProbe {
    fn probe(&self, path: &str) -> ProbeResult;
}

/// Production probe: `std::fs::metadata` only.
pub struct FsProbe;

impl PathProbe for FsProbe {
    fn probe(&self, path: &str) -> ProbeResult {
        match std::fs::metadata(path) {
            Ok(meta) => ProbeResult::Found {
                is_dir: meta.is_dir(),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ProbeResult::NotFound,
            Err(_) => ProbeResult::Unreachable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Available,
    Missing,
    FolderAtPath,
    Inaccessible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    pub id: i64,
    pub project_id: i64,
    pub project_name: Option<String>,
    pub path: String,
    pub name: String,
    pub environment: String,
    pub kind: String,
    pub status: Status,
}

/// A stored entry before its status is computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: i64,
    pub project_id: i64,
    pub path: String,
    pub environment: String,
    pub kind: String,
}

fn storage_failure<E>(_: E) -> ConfigError {
    ConfigError::new(ConfigCode::StorageUnavailable)
}

/// Only a UNIQUE failure means "already registered"; other constraint failures are storage faults.
fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(f, _)
            if f.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    )
}

fn path_key(normalised: &str) -> String {
    normalised.to_lowercase()
}

/// Last non-empty path segment (for a drive root, `F:`); the whole path if there is none.
fn file_name(path: &str) -> String {
    path.rsplit('\\')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
        .to_string()
}

pub fn status_of(probe: &dyn PathProbe, path: &str) -> Status {
    match probe.probe(path) {
        ProbeResult::Found { is_dir: false } => Status::Available,
        ProbeResult::Found { is_dir: true } => Status::FolderAtPath,
        ProbeResult::NotFound => Status::Missing,
        ProbeResult::Unreachable => Status::Inaccessible,
    }
}

/// True for `X:\...` (drive letter) or `\\server\share...` (non-empty server and share).
fn is_absolute_shape(normalised: &str) -> bool {
    let b = normalised.as_bytes();
    if b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\' {
        return true;
    }
    match normalised.strip_prefix("\\\\") {
        Some(rest) => {
            let mut parts = rest.split('\\');
            let server = parts.next().unwrap_or("");
            let share = parts.next().unwrap_or("");
            // `\\?\` and `\\.\` are extended-length and device prefixes, not servers.
            !server.is_empty() && server != "?" && server != "." && !share.is_empty()
        }
        None => false,
    }
}

/// Normalises and checks shape only, in order: empty, too long, absolute, no `..`. A `..`
/// segment is reported as `PathRelative`. Touches no file system.
pub fn validate_config_path(raw: &str) -> Result<String, ConfigError> {
    let normalised = normalize_path(raw);
    if normalised.is_empty() {
        return Err(ConfigError::new(ConfigCode::PathEmpty));
    }
    if normalised.chars().count() > MAX_PATH_CHARS {
        return Err(ConfigError::new(ConfigCode::PathTooLong));
    }
    if !is_absolute_shape(&normalised) || normalised.split('\\').any(|part| part == "..") {
        return Err(ConfigError::with_detail(ConfigCode::PathRelative, &normalised));
    }
    Ok(normalised)
}

/// Path shape, environment and kind lists, then one probe: the path must be a file.
pub fn validate_entry(
    probe: &dyn PathProbe,
    raw_path: &str,
    environment: &str,
    kind: &str,
) -> Result<String, ConfigError> {
    let normalised = validate_config_path(raw_path)?;
    if !ENVIRONMENTS.contains(&environment) {
        return Err(ConfigError::new(ConfigCode::EnvironmentInvalid));
    }
    if !KINDS.contains(&kind) {
        return Err(ConfigError::new(ConfigCode::KindInvalid));
    }
    require_file(probe, normalised)
}

fn require_file(probe: &dyn PathProbe, normalised: String) -> Result<String, ConfigError> {
    match probe.probe(&normalised) {
        ProbeResult::Found { is_dir: false } => Ok(normalised),
        ProbeResult::Found { is_dir: true } => {
            Err(ConfigError::with_detail(ConfigCode::PathIsFolder, &normalised))
        }
        ProbeResult::NotFound => {
            Err(ConfigError::with_detail(ConfigCode::PathNotFound, &normalised))
        }
        ProbeResult::Unreachable => Err(ConfigError::with_detail(ConfigCode::PathNotFound, "unreachable")),
    }
}

/// Display names of registered projects by id, for `with_status`. Database read only.
pub fn project_names(storage: &Storage) -> Result<HashMap<i64, String>, ConfigError> {
    let projects = crate::projects::list(storage).map_err(storage_failure)?;
    Ok(projects.into_iter().map(|p| (p.id, p.display_name)).collect())
}

fn row_from(r: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok(Row {
        id: r.get(0)?,
        project_id: r.get(1)?,
        path: r.get(2)?,
        environment: r.get(3)?,
        kind: r.get(4)?,
    })
}

/// Database read only; compute statuses with `with_status` after releasing the storage lock.
pub fn load_rows(storage: &Storage) -> Result<Vec<Row>, ConfigError> {
    let mut stmt = storage
        .connection()
        .prepare(
            "SELECT id, project_id, path, environment, kind FROM config_files ORDER BY id",
        )
        .map_err(storage_failure)?;
    let rows = stmt
        .query_map([], row_from)
        .map_err(storage_failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_failure)?;
    Ok(rows)
}

/// One stored entry by id, or `None` when it does not exist. Database read only.
pub fn get_row(storage: &Storage, id: i64) -> Result<Option<Row>, ConfigError> {
    storage
        .connection()
        .query_row(
            "SELECT id, project_id, path, environment, kind FROM config_files WHERE id = ?1",
            params![id],
            row_from,
        )
        .optional()
        .map_err(storage_failure)
}

/// A project id absent from `names` gives `project_name: None`. No storage access.
pub fn with_status(
    rows: Vec<Row>,
    names: &HashMap<i64, String>,
    probe: &dyn PathProbe,
) -> Vec<ConfigEntry> {
    rows.into_iter()
        .map(|row| {
            let status = status_of(probe, &row.path);
            ConfigEntry {
                id: row.id,
                project_id: row.project_id,
                project_name: names.get(&row.project_id).cloned(),
                name: file_name(&row.path),
                path: row.path,
                environment: row.environment,
                kind: row.kind,
                status,
            }
        })
        .collect()
}

/// Convenience for tests: the commands split this across the storage lock instead.
#[cfg(test)]
pub fn list(storage: &Storage, probe: &dyn PathProbe) -> Result<Vec<ConfigEntry>, ConfigError> {
    Ok(with_status(load_rows(storage)?, &project_names(storage)?, probe))
}

/// Inserts a validated entry. `normalised_path` must come from `validate_entry`.
pub fn add(
    storage: &Storage,
    project_id: i64,
    normalised_path: &str,
    environment: &str,
    kind: &str,
) -> Result<i64, ConfigError> {
    let conn = storage.connection();
    let inserted = conn.execute(
        "INSERT INTO config_files (project_id, path, path_key, environment, kind) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            normalised_path,
            path_key(normalised_path),
            environment,
            kind
        ],
    );
    match inserted {
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(e) if is_unique_violation(&e) => Err(ConfigError::with_detail(
            ConfigCode::DuplicatePath,
            normalised_path,
        )),
        Err(e) => Err(storage_failure(e)),
    }
}

/// Reads an entry's environment and kind; database read only.
pub fn environment_and_kind(storage: &Storage, id: i64) -> Result<(String, String), ConfigError> {
    storage
        .connection()
        .query_row(
            "SELECT environment, kind FROM config_files WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage_failure)?
        .ok_or_else(|| ConfigError::new(ConfigCode::ConfigNotFound))
}

/// Database write only: points an entry at an already validated path. `normalised_path` must
/// come from `validate_entry`. The entry keeps its project, environment and kind.
pub fn update_path(storage: &Storage, id: i64, normalised_path: &str) -> Result<(), ConfigError> {
    let updated = storage.connection().execute(
        "UPDATE config_files SET path = ?1, path_key = ?2 WHERE id = ?3",
        params![normalised_path, path_key(normalised_path), id],
    );
    match updated {
        Ok(0) => Err(ConfigError::new(ConfigCode::ConfigNotFound)),
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => Err(ConfigError::with_detail(
            ConfigCode::DuplicatePath,
            normalised_path,
        )),
        Err(e) => Err(storage_failure(e)),
    }
}

/// Test convenience composing the three phases; the command runs the probe between two locks.
#[cfg(test)]
pub fn relink(
    storage: &Storage,
    probe: &dyn PathProbe,
    id: i64,
    raw_path: &str,
) -> Result<(), ConfigError> {
    let (environment, kind) = environment_and_kind(storage, id)?;
    let normalised = validate_entry(probe, raw_path, &environment, &kind)?;
    update_path(storage, id, &normalised)
}

/// Deletes the registry row only; the file is never touched.
pub fn remove(storage: &Storage, id: i64) -> Result<(), ConfigError> {
    let removed = storage
        .connection()
        .execute("DELETE FROM config_files WHERE id = ?1", [id])
        .map_err(storage_failure)?;
    if removed == 0 {
        return Err(ConfigError::new(ConfigCode::ConfigNotFound));
    }
    Ok(())
}

/// Number of entries registered under `project_id`; database read only.
pub fn count_for_project(conn: &Connection, project_id: i64) -> Result<i64, ConfigError> {
    conn.query_row("SELECT COUNT(*) FROM config_files WHERE project_id = ?1", [project_id], |r| r.get(0))
        .map_err(storage_failure)
}

/// Deletes every entry of `project_id` and returns how many were removed. Takes a connection so
/// the caller can run it inside its own transaction. Registry rows only; files are never touched.
pub fn remove_for_project(conn: &Connection, project_id: i64) -> Result<i64, ConfigError> {
    let removed = conn
        .execute("DELETE FROM config_files WHERE project_id = ?1", [project_id])
        .map_err(storage_failure)?;
    Ok(removed as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::time::SystemTime;

    struct FakeProbe(HashMap<String, ProbeResult>);

    impl FakeProbe {
        fn new(entries: &[(&str, ProbeResult)]) -> Self {
            Self(
                entries
                    .iter()
                    .map(|(p, r)| (p.to_lowercase(), *r))
                    .collect(),
            )
        }
    }

    impl PathProbe for FakeProbe {
        fn probe(&self, path: &str) -> ProbeResult {
            self.0
                .get(&path.to_lowercase())
                .copied()
                .unwrap_or(ProbeResult::NotFound)
        }
    }

    const FILE: ProbeResult = ProbeResult::Found { is_dir: false };
    const DIR: ProbeResult = ProbeResult::Found { is_dir: true };

    fn storage() -> (tempfile::TempDir, Storage) {
        let tmp = tempfile::tempdir().unwrap();
        let s = crate::db::open_in(tmp.path()).unwrap();
        (tmp, s)
    }

    fn insert_project(s: &Storage, id: i64, name: &str) {
        s.connection()
            .execute(
                "INSERT INTO projects (id, display_name, root_path, path_key, drive, project_type) \
                 VALUES (?1, ?2, ?3, ?4, 'F:', 'other')",
                params![id, name, format!("F:\\{name}"), format!("f:\\{}", name.to_lowercase())],
            )
            .unwrap();
    }

    fn code(r: Result<impl std::fmt::Debug, ConfigError>) -> ConfigCode {
        r.unwrap_err().code
    }

    #[test]
    fn status_of_classifies_each_probe_result() {
        let probe = FakeProbe::new(&[
            ("F:\\a.yml", FILE),
            ("F:\\dir", DIR),
            ("F:\\net", ProbeResult::Unreachable),
        ]);
        assert_eq!(status_of(&probe, "F:\\a.yml"), Status::Available);
        assert_eq!(status_of(&probe, "F:\\dir"), Status::FolderAtPath);
        assert_eq!(status_of(&probe, "F:\\gone"), Status::Missing);
        assert_eq!(status_of(&probe, "F:\\net"), Status::Inaccessible);
    }

    #[test]
    fn fs_probe_reports_file_dir_and_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.yml");
        fs::write(&f, "k: v").unwrap();
        let probe = FsProbe;
        assert_eq!(probe.probe(f.to_str().unwrap()), FILE);
        assert_eq!(probe.probe(tmp.path().to_str().unwrap()), DIR);
        assert_eq!(
            probe.probe(tmp.path().join("nope.yml").to_str().unwrap()),
            ProbeResult::NotFound
        );
    }

    #[test]
    fn validate_config_path_cases() {
        assert_eq!(code(validate_config_path("")), ConfigCode::PathEmpty);
        assert_eq!(code(validate_config_path("   ")), ConfigCode::PathEmpty);
        assert_eq!(code(validate_config_path("src\\app.yml")), ConfigCode::PathRelative);
        assert_eq!(
            code(validate_config_path("F:\\a\\..\\b.yml")),
            ConfigCode::PathRelative
        );
        let long = format!("F:\\{}", "a".repeat(258));
        assert_eq!(long.chars().count(), 261);
        assert_eq!(code(validate_config_path(&long)), ConfigCode::PathTooLong);
        assert_eq!(validate_config_path("F:\\").unwrap(), "F:\\");
        assert_eq!(validate_config_path("F:/work/app.yml").unwrap(), "F:\\work\\app.yml");
        assert_eq!(
            validate_config_path("//srv/share/app.yml").unwrap(),
            "\\\\srv\\share\\app.yml"
        );
        assert_eq!(code(validate_config_path("\\\\?\\C:\\x")), ConfigCode::PathRelative);
    }

    #[test]
    fn validate_entry_cases() {
        let probe = FakeProbe::new(&[
            ("F:\\a.yml", FILE),
            ("F:\\dir", DIR),
            ("F:\\net.yml", ProbeResult::Unreachable),
        ]);
        assert_eq!(
            validate_entry(&probe, "F:/a.yml", "dev", "yaml").unwrap(),
            "F:\\a.yml"
        );
        assert_eq!(
            code(validate_entry(&probe, "F:\\a.yml", "qa", "yaml")),
            ConfigCode::EnvironmentInvalid
        );
        assert_eq!(
            code(validate_entry(&probe, "F:\\a.yml", "dev", "toml")),
            ConfigCode::KindInvalid
        );
        assert_eq!(
            code(validate_entry(&probe, "F:\\dir", "dev", "yaml")),
            ConfigCode::PathIsFolder
        );
        let nf = validate_entry(&probe, "F:\\gone.yml", "dev", "yaml").unwrap_err();
        assert_eq!(nf.code, ConfigCode::PathNotFound);
        assert_eq!(nf.detail.as_deref(), Some("F:\\gone.yml"));
        let un = validate_entry(&probe, "F:\\net.yml", "dev", "yaml").unwrap_err();
        assert_eq!(un.code, ConfigCode::PathNotFound);
        assert_eq!(un.detail.as_deref(), Some("unreachable"));
        assert_eq!(
            code(validate_entry(&probe, "", "dev", "yaml")),
            ConfigCode::PathEmpty
        );
    }

    #[test]
    fn add_enforces_duplicates_per_project_case_insensitively() {
        let (_t, s) = storage();
        let id = add(&s, 1, "F:\\work\\app.yml", "dev", "yaml").unwrap();
        assert!(id > 0);
        let dup = add(&s, 1, "F:\\work\\app.yml", "staging", "json").unwrap_err();
        assert_eq!(dup.code, ConfigCode::DuplicatePath);
        assert_eq!(dup.detail.as_deref(), Some("F:\\work\\app.yml"));
        assert_eq!(
            code(add(&s, 1, "f:\\WORK\\App.yml", "dev", "yaml")),
            ConfigCode::DuplicatePath
        );
        assert!(add(&s, 2, "F:\\work\\app.yml", "dev", "yaml").is_ok());
        assert_eq!(load_rows(&s).unwrap().len(), 2);
    }

    #[test]
    fn list_computes_status_name_and_project_name() {
        let (_t, s) = storage();
        insert_project(&s, 1, "Billing");
        add(&s, 1, "F:\\work\\application-staging.yml", "staging", "yaml").unwrap();
        add(&s, 1, "F:\\work\\folder", "dev", "json").unwrap();
        add(&s, 99, "F:\\work\\orphan.env", "local", "dotenv").unwrap();
        add(&s, 1, "\\\\srv\\share\\net.yml", "dev", "yaml").unwrap();

        let probe = FakeProbe::new(&[
            ("F:\\work\\application-staging.yml", FILE),
            ("F:\\work\\folder", DIR),
            ("F:\\work\\orphan.env", FILE),
            ("\\\\srv\\share\\net.yml", ProbeResult::Unreachable),
        ]);
        let entries = list(&s, &probe).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].name, "application-staging.yml");
        assert_eq!(entries[0].status, Status::Available);
        assert_eq!(entries[0].project_name.as_deref(), Some("Billing"));
        assert_eq!(entries[0].environment, "staging");
        assert_eq!(entries[0].kind, "yaml");
        assert_eq!(entries[1].status, Status::FolderAtPath);
        assert_eq!(entries[2].project_name, None);
        assert_eq!(entries[2].project_id, 99);
        assert_eq!(entries[3].status, Status::Inaccessible);

        // The file is later removed: the same rows now report Missing.
        let gone = FakeProbe::new(&[]);
        let again = list(&s, &gone).unwrap();
        assert!(again.iter().all(|e| e.status == Status::Missing));
    }

    #[test]
    fn with_status_name_for_drive_root_is_drive_segment() {
        let rows = vec![Row {
            id: 1,
            project_id: 1,
            path: "F:\\".to_string(),
            environment: "dev".to_string(),
            kind: "yaml".to_string(),
        }];
        let entries = with_status(rows, &HashMap::new(), &FakeProbe::new(&[]));
        assert_eq!(entries[0].name, "F:");
        assert_eq!(entries[0].project_name, None);
    }

    #[test]
    fn with_status_absent_project_id_gives_none_and_present_gives_name() {
        let mk = |id, project_id| Row {
            id,
            project_id,
            path: "F:\\a.yml".to_string(),
            environment: "dev".to_string(),
            kind: "yaml".to_string(),
        };
        let names = HashMap::from([(1, "Billing".to_string())]);
        let entries = with_status(vec![mk(1, 1), mk(2, 7)], &names, &FakeProbe::new(&[]));
        assert_eq!(entries[0].project_name.as_deref(), Some("Billing"));
        assert_eq!(entries[1].project_name, None);
    }

    #[test]
    fn get_row_returns_entry_or_none() {
        let (_t, s) = storage();
        let id = add(&s, 1, "F:\\work\\app.yml", "dev", "yaml").unwrap();
        let row = get_row(&s, id).unwrap().unwrap();
        assert_eq!(row.id, id);
        assert_eq!(row.path, "F:\\work\\app.yml");
        assert_eq!(row.project_id, 1);
        assert!(get_row(&s, id + 100).unwrap().is_none());
    }

    #[test]
    fn relink_changes_path_and_keeps_other_fields() {
        let (_t, s) = storage();
        insert_project(&s, 1, "Billing");
        let id = add(&s, 1, "F:\\old\\app.yml", "staging", "docker_compose").unwrap();
        let probe = FakeProbe::new(&[("F:\\new\\app.yml", FILE)]);
        relink(&s, &probe, id, "F:/new/app.yml").unwrap();
        let rows = load_rows(&s).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path, "F:\\new\\app.yml");
        assert_eq!(rows[0].project_id, 1);
        assert_eq!(rows[0].environment, "staging");
        assert_eq!(rows[0].kind, "docker_compose");
        assert_eq!(list(&s, &probe).unwrap()[0].status, Status::Available);
    }

    #[test]
    fn relink_rejects_duplicates_in_same_project_only() {
        let (_t, s) = storage();
        let a = add(&s, 1, "F:\\a.yml", "dev", "yaml").unwrap();
        add(&s, 1, "F:\\b.yml", "dev", "yaml").unwrap();
        add(&s, 2, "F:\\c.yml", "dev", "yaml").unwrap();
        let probe = FakeProbe::new(&[("F:\\b.yml", FILE), ("F:\\c.yml", FILE)]);
        let dup = relink(&s, &probe, a, "f:\\B.YML").unwrap_err();
        assert_eq!(dup.code, ConfigCode::DuplicatePath);
        // Same path under a different project is allowed.
        relink(&s, &probe, a, "F:\\c.yml").unwrap();
        let rows = load_rows(&s).unwrap();
        assert_eq!(rows[0].path, "F:\\c.yml");
    }

    #[test]
    fn relink_requires_file_and_known_id() {
        let (_t, s) = storage();
        let id = add(&s, 1, "F:\\a.yml", "dev", "yaml").unwrap();
        let probe = FakeProbe::new(&[
            ("F:\\dir", DIR),
            ("F:\\net.yml", ProbeResult::Unreachable),
        ]);
        assert_eq!(code(relink(&s, &probe, id, "F:\\dir")), ConfigCode::PathIsFolder);
        assert_eq!(code(relink(&s, &probe, id, "F:\\gone.yml")), ConfigCode::PathNotFound);
        let un = relink(&s, &probe, id, "F:\\net.yml").unwrap_err();
        assert_eq!(un.detail.as_deref(), Some("unreachable"));
        assert_eq!(code(relink(&s, &probe, id, "rel.yml")), ConfigCode::PathRelative);
        assert_eq!(code(relink(&s, &probe, 999, "F:\\dir")), ConfigCode::ConfigNotFound);
        assert_eq!(load_rows(&s).unwrap()[0].path, "F:\\a.yml");
    }

    #[test]
    fn update_path_writes_without_probing_and_maps_errors() {
        let (_t, s) = storage();
        let a = add(&s, 1, "F:\\a.yml", "dev", "yaml").unwrap();
        add(&s, 1, "F:\\b.yml", "dev", "yaml").unwrap();
        assert_eq!(code(update_path(&s, a, "F:\\B.yml")), ConfigCode::DuplicatePath);
        assert_eq!(code(update_path(&s, 999, "F:\\z.yml")), ConfigCode::ConfigNotFound);
        update_path(&s, a, "F:\\c.yml").unwrap();
        assert_eq!(load_rows(&s).unwrap()[0].path, "F:\\c.yml");
    }

    #[test]
    fn remove_deletes_only_the_row() {
        let (_t, s) = storage();
        let a = add(&s, 1, "F:\\a.yml", "dev", "yaml").unwrap();
        let b = add(&s, 1, "F:\\b.yml", "dev", "yaml").unwrap();
        remove(&s, a).unwrap();
        let rows = load_rows(&s).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, b);
        assert_eq!(code(remove(&s, a)), ConfigCode::ConfigNotFound);
        assert_eq!(code(remove(&s, 12345)), ConfigCode::ConfigNotFound);
    }

    fn snapshot(dir: &Path) -> Vec<(String, SystemTime, Vec<u8>)> {
        let mut out: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                let meta = e.metadata().unwrap();
                (
                    e.file_name().to_string_lossy().into_owned(),
                    meta.modified().unwrap(),
                    fs::read(e.path()).unwrap(),
                )
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn add_relink_remove_never_change_config_files() {
        let (_t, s) = storage();
        let cfg = tempfile::tempdir().unwrap();
        let a = cfg.path().join("a.yml");
        let b = cfg.path().join("b.yml");
        fs::write(&a, "key: a").unwrap();
        fs::write(&b, "key: b").unwrap();
        let before = snapshot(cfg.path());

        let a_path = a.to_str().unwrap();
        let b_path = b.to_str().unwrap();
        let normalised = validate_entry(&FsProbe, a_path, "dev", "yaml").unwrap();
        let id = add(&s, 1, &normalised, "dev", "yaml").unwrap();
        assert_eq!(list(&s, &FsProbe).unwrap()[0].status, Status::Available);
        relink(&s, &FsProbe, id, b_path).unwrap();
        assert_eq!(list(&s, &FsProbe).unwrap()[0].name, "b.yml");
        remove(&s, id).unwrap();

        assert_eq!(snapshot(cfg.path()), before);
        assert!(load_rows(&s).unwrap().is_empty());
    }

    #[test]
    fn migration_7_reopen_keeps_rows() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let s = crate::db::open_in(tmp.path()).unwrap();
            add(&s, 1, "F:\\a.yml", "dev", "yaml").unwrap();
        }
        let s = crate::db::open_in(tmp.path()).unwrap();
        let rows = load_rows(&s).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path, "F:\\a.yml");
        assert_eq!(code(add(&s, 1, "F:\\A.yml", "dev", "yaml")), ConfigCode::DuplicatePath);
    }
}
