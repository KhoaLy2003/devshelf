//! Project registry: validation, path normalisation, persistence, and pinning.
//!
//! Project folders are only checked for existence and type; nothing is read or
//! written inside them.

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::Storage;

pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_PATH_CHARS: usize = 260;
pub const MAX_TAG_CHARS: usize = 32;
pub const MAX_TAGS: usize = 10;
pub const PROJECT_TYPES: [&str; 5] = ["Spring Boot", "React", "Docker/Infra", "Node", "Other"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCode {
    PathEmpty,
    PathNotFound,
    PathIsFile,
    AlreadyRegistered,
    NameEmpty,
    NameTooLong,
    PathTooLong,
    TagInvalid,
    TooManyTags,
    TypeInvalid,
    StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: ValidationCode,
    pub detail: Option<String>,
}

impl ValidationError {
    fn new(code: ValidationCode) -> Self {
        Self { code, detail: None }
    }

    fn with_detail(code: ValidationCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: Some(detail.into()),
        }
    }
}

/// Fields of a registration other than the folder path, which arrives as a [`ValidatedPath`].
/// `tags_raw` is the comma-separated tag text; it is parsed after every other check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProject {
    pub display_name: String,
    pub project_type: String,
    pub tags_raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
    pub display_name: String,
    pub root_path: String,
    pub drive: String,
    pub project_type: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_at: String,
}

/// Trims, converts `/` to `\`, collapses repeated `\` (keeping a leading UNC `\\`), turns a bare
/// `X:` into `X:\`, and strips trailing `\` unless the result is a drive root like `F:\`.
/// `..` segments are left untouched.
pub fn normalize_path(raw: &str) -> String {
    let slashed = raw.trim().replace('/', "\\");
    let (prefix, rest) = match slashed.strip_prefix("\\\\") {
        Some(rest) => ("\\\\", rest),
        None => ("", slashed.as_str()),
    };
    let mut path = String::with_capacity(slashed.len() + 1);
    path.push_str(prefix);
    let mut previous_separator = !prefix.is_empty();
    for c in rest.chars() {
        if c == '\\' {
            if !previous_separator {
                path.push(c);
            }
            previous_separator = true;
        } else {
            path.push(c);
            previous_separator = false;
        }
    }
    let b = path.as_bytes();
    if b.len() == 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
        path.push('\\');
    }
    while path.len() > prefix.len() && path.ends_with('\\') && !is_drive_root(&path) {
        path.pop();
    }
    path
}

fn is_drive_root(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() == 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\'
}

/// Returns `"F:"` for a path starting with a drive letter, otherwise `""`.
pub fn drive_of(normalized: &str) -> String {
    let b = normalized.as_bytes();
    if b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
        format!("{}:", (b[0] as char).to_ascii_uppercase())
    } else {
        String::new()
    }
}

fn path_key(normalized: &str) -> String {
    normalized.to_lowercase()
}

fn tag_is_valid(tag: &str) -> bool {
    !tag.is_empty()
        && tag.chars().count() <= MAX_TAG_CHARS
        && tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    for tag in tags {
        if !tag_is_valid(tag) {
            return Err(ValidationError::with_detail(ValidationCode::TagInvalid, tag.as_str()));
        }
    }
    if tags.len() > MAX_TAGS {
        return Err(ValidationError::new(ValidationCode::TooManyTags));
    }
    Ok(())
}

/// Splits on `,`, trims, lowercases, dedupes (first occurrence wins), then validates.
/// Tags allow `a-z`, `0-9`, `-`, `_`, `.`, up to 32 characters; at most 10 tags.
pub fn parse_tags(raw: &str) -> Result<Vec<String>, ValidationError> {
    let mut tags: Vec<String> = Vec::new();
    for part in raw.split(',') {
        let tag = part.trim().to_lowercase();
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    validate_tags(&tags)?;
    Ok(tags)
}

/// A folder path that passed [`validate_path`]: normalised, absolute, and an existing directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPath(String);

impl ValidatedPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// True for `X:\...` (drive letter) or `\\server\share...` (non-empty server and share).
fn is_absolute_shape(normalized: &str) -> bool {
    let b = normalized.as_bytes();
    if b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\' {
        return true;
    }
    match normalized.strip_prefix("\\\\") {
        Some(rest) => {
            let mut parts = rest.split('\\');
            let server = parts.next().unwrap_or("");
            let share = parts.next().unwrap_or("");
            !server.is_empty() && !share.is_empty()
        }
        None => false,
    }
}

/// Path checks in order: empty, too long, shape, not found, is file. `..` components are not
/// resolved and are rejected as not found. This is the only validation that touches the
/// filesystem; run it before taking the storage lock.
pub fn validate_path(raw: &str) -> Result<ValidatedPath, ValidationError> {
    let normalized = normalize_path(raw);
    if normalized.is_empty() {
        return Err(ValidationError::new(ValidationCode::PathEmpty));
    }
    if normalized.chars().count() > MAX_PATH_CHARS {
        return Err(ValidationError::new(ValidationCode::PathTooLong));
    }
    if !is_absolute_shape(&normalized) || normalized.split('\\').any(|part| part == "..") {
        return Err(ValidationError::new(ValidationCode::PathNotFound));
    }
    match std::fs::metadata(&normalized) {
        Ok(meta) if meta.is_dir() => Ok(ValidatedPath(normalized)),
        Ok(_) => Err(ValidationError::new(ValidationCode::PathIsFile)),
        Err(_) => Err(ValidationError::new(ValidationCode::PathNotFound)),
    }
}

/// Remaining checks after [`validate_path`]; `existing` holds `(id, display_name, path_key)`
/// rows. Order: already registered, name empty, name too long, type, tags. Returns the parsed tags.
fn validate_rest(
    path: &ValidatedPath,
    input: &NewProject,
    existing: &[(i64, String, String)],
) -> Result<Vec<String>, ValidationError> {
    let key = path_key(&path.0);
    if let Some((_, name, _)) = existing.iter().find(|(_, _, k)| *k == key) {
        return Err(ValidationError::with_detail(
            ValidationCode::AlreadyRegistered,
            name.as_str(),
        ));
    }
    let name = input.display_name.trim();
    if name.is_empty() {
        return Err(ValidationError::new(ValidationCode::NameEmpty));
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(ValidationError::new(ValidationCode::NameTooLong));
    }
    if !PROJECT_TYPES.contains(&input.project_type.as_str()) {
        return Err(ValidationError::with_detail(
            ValidationCode::TypeInvalid,
            input.project_type.as_str(),
        ));
    }
    parse_tags(&input.tags_raw)
}

// Database failures surface as `StorageUnavailable` so callers never see raw SQLite text.
fn storage_failure(_: rusqlite::Error) -> ValidationError {
    ValidationError::new(ValidationCode::StorageUnavailable)
}

fn existing_rows(conn: &Connection) -> Result<Vec<(i64, String, String)>, ValidationError> {
    let mut stmt = conn
        .prepare("SELECT id, display_name, path_key FROM projects")
        .map_err(storage_failure)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(storage_failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_failure)?;
    Ok(rows)
}

fn tags_of(conn: &Connection, id: i64) -> Result<Vec<String>, ValidationError> {
    let mut stmt = conn
        .prepare("SELECT tag FROM project_tags WHERE project_id = ?1 ORDER BY tag")
        .map_err(storage_failure)?;
    let tags = stmt
        .query_map([id], |r| r.get(0))
        .map_err(storage_failure)?
        .collect::<Result<Vec<String>, _>>()
        .map_err(storage_failure)?;
    Ok(tags)
}

const SELECT_PROJECT: &str =
    "SELECT id, display_name, root_path, drive, project_type, pinned, created_at FROM projects";

fn project_from_row(conn: &Connection, r: &rusqlite::Row<'_>) -> Result<Project, ValidationError> {
    let id: i64 = r.get(0).map_err(storage_failure)?;
    Ok(Project {
        id,
        display_name: r.get(1).map_err(storage_failure)?,
        root_path: r.get(2).map_err(storage_failure)?,
        drive: r.get(3).map_err(storage_failure)?,
        project_type: r.get(4).map_err(storage_failure)?,
        pinned: r.get::<_, i64>(5).map_err(storage_failure)? != 0,
        created_at: r.get(6).map_err(storage_failure)?,
        tags: tags_of(conn, id)?,
    })
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(err, _)
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    )
}

/// Registers a project whose folder passed [`validate_path`]. Touches only the database, so the
/// caller can hold the storage lock without blocking on the filesystem.
pub fn register(
    storage: &Storage,
    path: ValidatedPath,
    input: NewProject,
) -> Result<Project, ValidationError> {
    let conn = storage.connection();
    let normalized = path.as_str();
    let tags = validate_rest(&path, &input, &existing_rows(conn)?)?;
    let display_name = input.display_name.trim();

    let key = path_key(&normalized);
    let tx = conn.unchecked_transaction().map_err(storage_failure)?;
    let inserted = tx.execute(
        "INSERT INTO projects (display_name, root_path, path_key, drive, project_type) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            display_name,
            normalized,
            key,
            drive_of(&normalized),
            input.project_type
        ],
    );
    if let Err(e) = inserted {
        drop(tx);
        if is_unique_violation(&e) {
            let name: Option<String> = conn
                .query_row(
                    "SELECT display_name FROM projects WHERE path_key = ?1",
                    [&key],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_failure)?;
            return Err(ValidationError {
                code: ValidationCode::AlreadyRegistered,
                detail: name,
            });
        }
        return Err(storage_failure(e));
    }
    let id = tx.last_insert_rowid();
    for tag in &tags {
        tx.execute(
            "INSERT OR IGNORE INTO project_tags (project_id, tag) VALUES (?1, ?2)",
            params![id, tag],
        )
        .map_err(storage_failure)?;
    }
    tx.commit().map_err(storage_failure)?;

    get(storage, id)?.ok_or_else(|| ValidationError::new(ValidationCode::StorageUnavailable))
}

pub fn list(storage: &Storage) -> Result<Vec<Project>, ValidationError> {
    let conn = storage.connection();
    let mut stmt = conn
        .prepare(&format!(
            "{SELECT_PROJECT} ORDER BY pinned DESC, display_name COLLATE NOCASE, id"
        ))
        .map_err(storage_failure)?;
    let mut rows = stmt.query([]).map_err(storage_failure)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(storage_failure)? {
        out.push(project_from_row(conn, row)?);
    }
    Ok(out)
}

pub fn get(storage: &Storage, id: i64) -> Result<Option<Project>, ValidationError> {
    let conn = storage.connection();
    let mut stmt = conn
        .prepare(&format!("{SELECT_PROJECT} WHERE id = ?1"))
        .map_err(storage_failure)?;
    let mut rows = stmt.query([id]).map_err(storage_failure)?;
    match rows.next().map_err(storage_failure)? {
        Some(row) => Ok(Some(project_from_row(conn, row)?)),
        None => Ok(None),
    }
}

pub fn set_pinned(
    storage: &Storage,
    id: i64,
    pinned: bool,
) -> Result<Option<Project>, ValidationError> {
    storage
        .connection()
        .execute(
            "UPDATE projects SET pinned = ?1 WHERE id = ?2",
            params![pinned as i64, id],
        )
        .map_err(storage_failure)?;
    get(storage, id)
}

/// Name of a registered project whose folder contains `normalized_path`, if any.
pub fn nested_note(
    storage: &Storage,
    normalized_path: &str,
) -> Result<Option<String>, ValidationError> {
    let key = path_key(&normalize_path(normalized_path));
    let rows = existing_rows(storage.connection())?;
    Ok(rows
        .into_iter()
        .filter(|(_, _, k)| {
            let prefix = if k.ends_with('\\') {
                k.clone()
            } else {
                format!("{k}\\")
            };
            key.len() > prefix.len() && key.starts_with(&prefix)
        })
        .map(|(_, name, _)| name)
        .next())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::ErrorCode;
    use crate::db;
    use std::path::Path;

    fn storage_in(dir: &Path) -> Storage {
        db::open_in(&dir.join("data")).unwrap()
    }

    fn folder(dir: &Path, name: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn input(name: &str) -> NewProject {
        NewProject {
            display_name: name.to_string(),
            project_type: "Other".to_string(),
            tags_raw: String::new(),
        }
    }

    /// Validates `path` then registers, mirroring the command flow.
    fn reg_str(storage: &Storage, path: &str, input: NewProject) -> Result<Project, ValidationError> {
        register(storage, validate_path(path)?, input)
    }

    fn reg(storage: &Storage, dir: &Path, name: &str) -> Result<Project, ValidationError> {
        reg_str(storage, &dir.to_string_lossy(), input(name))
    }

    fn code(r: Result<Project, ValidationError>) -> ValidationCode {
        r.unwrap_err().code
    }

    /// Inserts a row without touching the filesystem.
    fn insert_raw(storage: &Storage, path: &str, name: &str) {
        let normalized = normalize_path(path);
        storage
            .connection()
            .execute(
                "INSERT INTO projects (display_name, root_path, path_key, drive, project_type) \
                 VALUES (?1, ?2, ?3, ?4, 'Other')",
                params![name, normalized, path_key(&normalized), drive_of(&normalized)],
            )
            .unwrap();
    }

    #[test]
    fn normalize_path_cases() {
        assert_eq!(normalize_path("F:/a/b/"), "F:\\a\\b");
        assert_eq!(normalize_path("F:\\"), "F:\\");
        assert_eq!(normalize_path("  F:\\a  "), "F:\\a");
        assert_eq!(drive_of("F:\\a"), "F:");
        assert_eq!(drive_of("\\\\server\\share"), "");
    }

    #[test]
    fn normalize_path_bare_drive_and_repeated_separators() {
        assert_eq!(normalize_path("F:"), "F:\\");
        assert_eq!(normalize_path("f:"), "f:\\");
        assert_eq!(normalize_path("F:\\a\\\\b"), "F:\\a\\b");
        assert_eq!(normalize_path("F://a///b//"), "F:\\a\\b");
        assert_eq!(normalize_path("F:\\\\"), "F:\\");
    }

    #[test]
    fn normalize_path_keeps_unc_prefix() {
        assert_eq!(normalize_path("\\\\server\\share\\"), "\\\\server\\share");
        assert_eq!(normalize_path("//server/share/dir//"), "\\\\server\\share\\dir");
        assert_eq!(normalize_path("\\\\\\server\\\\share"), "\\\\server\\share");
    }

    #[test]
    fn relative_and_drive_relative_paths_are_rejected() {
        // `src` exists relative to the test working directory; it must still be rejected.
        for raw in [".", "src", "F:a", "F:a\\b", "\\a\\b", "\\\\server", "\\\\server\\"] {
            assert_eq!(
                validate_path(raw).unwrap_err().code,
                ValidationCode::PathNotFound,
                "{raw}"
            );
        }
    }

    #[test]
    fn parent_directory_components_are_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = folder(tmp.path(), "a");
        let base = tmp.path().to_string_lossy().into_owned();
        let with_dots = format!("{base}\\a\\..\\a");
        assert!(dir.is_dir());
        assert_eq!(validate_path(&with_dots).unwrap_err().code, ValidationCode::PathNotFound);
        assert_eq!(
            validate_path("F:\\a\\..\\a").unwrap_err().code,
            ValidationCode::PathNotFound
        );
    }

    #[test]
    fn validate_path_returns_normalized_path() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = folder(tmp.path(), "proj");
        let raw = format!("{}/", dir.to_string_lossy());
        assert_eq!(validate_path(&raw).unwrap().as_str(), normalize_path(&raw));
    }

    #[test]
    fn parse_tags_cases() {
        assert_eq!(parse_tags("API, api ,  x-1,,").unwrap(), vec!["api", "x-1"]);
        let e = parse_tags("bad tag!").unwrap_err();
        assert_eq!(e.code, ValidationCode::TagInvalid);
        assert_eq!(e.detail.as_deref(), Some("bad tag!"));
        let eleven = (0..11).map(|i| format!("t{i}")).collect::<Vec<_>>().join(",");
        assert_eq!(parse_tags(&eleven).unwrap_err().code, ValidationCode::TooManyTags);
        let long = "a".repeat(33);
        assert_eq!(parse_tags(&long).unwrap_err().code, ValidationCode::TagInvalid);
    }

    #[test]
    fn register_then_list() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        let mut new = input("  My Proj ");
        new.tags_raw = "API, web".into();
        let created = reg_str(&storage, &dir.to_string_lossy(), new).unwrap();
        let all = list(&storage).unwrap();
        assert_eq!(all, vec![created.clone()]);
        assert_eq!(created.display_name, "My Proj");
        assert_eq!(created.drive, drive_of(&normalize_path(&dir.to_string_lossy())));
        assert_eq!(created.tags, vec!["api", "web"]);
        assert!(!created.pinned);
        assert_eq!(get(&storage, created.id).unwrap(), Some(created));
        assert_eq!(get(&storage, 9999).unwrap(), None);
    }

    #[test]
    fn non_ascii_display_name_registers() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let a = folder(tmp.path(), "a");
        let b = folder(tmp.path(), "b");
        let created = reg(&storage, &a, "  Проект 日本語 ✓ ").unwrap();
        assert_eq!(created.display_name, "Проект 日本語 ✓");
        assert_eq!(get(&storage, created.id).unwrap().unwrap().display_name, "Проект 日本語 ✓");
        // The limit counts characters, not bytes.
        let max = "日".repeat(MAX_NAME_CHARS);
        assert_eq!(reg(&storage, &b, &max).unwrap().display_name, max);
    }

    #[test]
    fn duplicate_by_case_or_trailing_slash() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        reg(&storage, &dir, "First").unwrap();
        let path = dir.to_string_lossy().into_owned();
        for variant in [path.to_uppercase(), format!("{path}\\"), format!("{path}/")] {
            let e = reg_str(&storage, &variant, input("Second")).unwrap_err();
            assert_eq!(e.code, ValidationCode::AlreadyRegistered);
            assert_eq!(e.detail.as_deref(), Some("First"));
        }
        assert_eq!(list(&storage).unwrap().len(), 1);
    }

    #[test]
    fn unique_violation_maps_to_already_registered() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        reg(&storage, &dir, "First").unwrap();
        // Simulates a concurrent registration: validation sees no existing rows.
        let path = validate_path(&dir.to_string_lossy()).unwrap();
        assert!(validate_rest(&path, &input("Second"), &[]).is_ok());
        let e = register(&storage, path, input("Second")).unwrap_err();
        assert_eq!(e.code, ValidationCode::AlreadyRegistered);
        assert_eq!(e.detail.as_deref(), Some("First"));
    }

    #[test]
    fn only_unique_constraint_counts_as_unique_violation() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        insert_raw(&storage, "F:\\a", "a");
        let conn = storage.connection();
        let unique = conn
            .execute(
                "INSERT INTO projects (display_name, root_path, path_key, drive, project_type) \
                 VALUES ('b', 'F:\\a', 'f:\\a', 'F:', 'Other')",
                [],
            )
            .unwrap_err();
        assert!(is_unique_violation(&unique));
        let not_null = conn
            .execute(
                "INSERT INTO projects (display_name, root_path, path_key, drive, project_type) \
                 VALUES (NULL, 'F:\\c', 'f:\\c', 'F:', 'Other')",
                [],
            )
            .unwrap_err();
        assert!(matches!(
            &not_null,
            rusqlite::Error::SqliteFailure(err, _) if err.code == ErrorCode::ConstraintViolation
        ));
        assert!(!is_unique_violation(&not_null));
    }

    #[test]
    fn validation_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        let file = tmp.path().join("f.txt");
        std::fs::write(&file, b"x").unwrap();

        assert_eq!(code(reg_str(&storage, "   ", input("n"))), ValidationCode::PathEmpty);
        assert_eq!(code(reg(&storage, &tmp.path().join("missing"), "n")), ValidationCode::PathNotFound);
        assert_eq!(code(reg(&storage, &file, "n")), ValidationCode::PathIsFile);
        assert_eq!(code(reg(&storage, &dir, "   ")), ValidationCode::NameEmpty);
        assert_eq!(code(reg(&storage, &dir, &"x".repeat(101))), ValidationCode::NameTooLong);
        let long_path = format!("F:\\{}", "a".repeat(261));
        assert_eq!(code(reg_str(&storage, &long_path, input("n"))), ValidationCode::PathTooLong);
        let mut bad_type = input("n");
        bad_type.project_type = "Rust".into();
        assert_eq!(
            code(reg_str(&storage, &dir.to_string_lossy(), bad_type)),
            ValidationCode::TypeInvalid
        );
        let mut bad_tag = input("n");
        bad_tag.tags_raw = "Bad Tag!".into();
        assert_eq!(
            code(reg_str(&storage, &dir.to_string_lossy(), bad_tag)),
            ValidationCode::TagInvalid
        );
        assert!(list(&storage).unwrap().is_empty());
    }

    #[test]
    fn tags_are_validated_last() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        let dir_str = dir.to_string_lossy().into_owned();
        let bad = |mut n: NewProject| {
            n.tags_raw = "Bad Tag!".into();
            n
        };

        assert_eq!(code(reg_str(&storage, "", bad(input("n")))), ValidationCode::PathEmpty);
        assert_eq!(
            code(reg_str(&storage, "src", bad(input("n")))),
            ValidationCode::PathNotFound
        );
        assert_eq!(code(reg_str(&storage, &dir_str, bad(input("  ")))), ValidationCode::NameEmpty);
        assert_eq!(
            code(reg_str(&storage, &dir_str, bad(input(&"x".repeat(101))))),
            ValidationCode::NameTooLong
        );
        let mut bad_type = bad(input("n"));
        bad_type.project_type = "Rust".into();
        assert_eq!(code(reg_str(&storage, &dir_str, bad_type)), ValidationCode::TypeInvalid);
        assert_eq!(code(reg_str(&storage, &dir_str, bad(input("n")))), ValidationCode::TagInvalid);

        reg(&storage, &dir, "First").unwrap();
        assert_eq!(
            code(reg_str(&storage, &dir_str, bad(input("Second")))),
            ValidationCode::AlreadyRegistered
        );
    }

    #[test]
    fn failed_tag_insert_leaves_no_project_row() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        storage
            .connection()
            .execute_batch(
                "CREATE TRIGGER fail_tag BEFORE INSERT ON project_tags \
                 BEGIN SELECT RAISE(ABORT, 'forced'); END;",
            )
            .unwrap();
        let mut new = input("n");
        new.tags_raw = "api".into();
        let e = reg_str(&storage, &dir.to_string_lossy(), new).unwrap_err();
        assert_eq!(e.code, ValidationCode::StorageUnavailable);
        assert!(list(&storage).unwrap().is_empty());

        storage.connection().execute_batch("DROP TRIGGER fail_tag;").unwrap();
        let mut again = input("n");
        again.tags_raw = "api".into();
        assert_eq!(reg_str(&storage, &dir.to_string_lossy(), again).unwrap().tags, vec!["api"]);
    }

    #[test]
    fn nested_projects_are_allowed_in_either_order() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let parent = folder(tmp.path(), "parent");
        let child = folder(&parent, "child");
        let parent2 = folder(tmp.path(), "parent2");
        let child2 = folder(&parent2, "child2");

        reg(&storage, &parent, "parent name").unwrap();
        let child_norm = normalize_path(&child.to_string_lossy());
        assert_eq!(nested_note(&storage, &child_norm).unwrap(), Some("parent name".into()));
        assert_eq!(
            nested_note(&storage, &normalize_path(&parent.to_string_lossy())).unwrap(),
            None
        );
        reg(&storage, &child, "child name").unwrap();

        reg(&storage, &child2, "child2 name").unwrap();
        reg(&storage, &parent2, "parent2 name").unwrap();
        assert_eq!(list(&storage).unwrap().len(), 4);
    }

    #[test]
    fn nested_note_ignores_sibling_with_shared_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        insert_raw(&storage, "F:\\app", "app");
        assert_eq!(nested_note(&storage, "F:\\app2").unwrap(), None);
        assert_eq!(nested_note(&storage, "F:\\app2\\sub").unwrap(), None);
        assert_eq!(nested_note(&storage, "F:\\app\\sub").unwrap(), Some("app".into()));
        assert_eq!(nested_note(&storage, "f:\\APP\\Sub").unwrap(), Some("app".into()));
    }

    #[test]
    fn nested_note_with_drive_root_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        insert_raw(&storage, "F:\\", "root");
        assert_eq!(nested_note(&storage, "F:\\child").unwrap(), Some("root".into()));
        assert_eq!(nested_note(&storage, "F:\\a\\b").unwrap(), Some("root".into()));
        assert_eq!(nested_note(&storage, "F:\\").unwrap(), None);
        assert_eq!(nested_note(&storage, "G:\\child").unwrap(), None);
    }

    #[test]
    fn pinned_persists_and_sorts_first() {
        let tmp = tempfile::tempdir().unwrap();
        let a = folder(tmp.path(), "a");
        let b = folder(tmp.path(), "b");
        let id_b;
        {
            let storage = storage_in(tmp.path());
            reg(&storage, &a, "alpha").unwrap();
            id_b = reg(&storage, &b, "Beta").unwrap().id;
            let names: Vec<_> = list(&storage).unwrap().into_iter().map(|p| p.display_name).collect();
            assert_eq!(names, vec!["alpha", "Beta"]);
            assert!(set_pinned(&storage, id_b, true).unwrap().unwrap().pinned);
            assert_eq!(set_pinned(&storage, 9999, true).unwrap(), None);
        }
        let reopened = storage_in(tmp.path());
        let all = list(&reopened).unwrap();
        assert_eq!(all[0].display_name, "Beta");
        assert!(all[0].pinned);
        assert!(!all[1].pinned);
        assert!(!set_pinned(&reopened, id_b, false).unwrap().unwrap().pinned);
    }

    fn snapshot(dir: &Path) -> Vec<(String, std::time::SystemTime)> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.metadata().unwrap().modified().unwrap(),
                )
            })
            .collect();
        entries.sort();
        entries.push((".".into(), std::fs::metadata(dir).unwrap().modified().unwrap()));
        entries
    }

    #[test]
    fn register_does_not_write_to_project_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = storage_in(tmp.path());
        let dir = folder(tmp.path(), "proj");
        std::fs::write(dir.join("keep.txt"), b"hello").unwrap();
        folder(&dir, "sub");
        let before = snapshot(&dir);
        reg(&storage, &dir, "proj").unwrap();
        list(&storage).unwrap();
        assert_eq!(snapshot(&dir), before);
    }
}
