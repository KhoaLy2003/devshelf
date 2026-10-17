//! Workspaces: named, ordered lists of file and folder references.
//!
//! This module owns workspace names, item paths, ordering, and persistence. It only reads
//! `std::fs::metadata` of item paths (kind and existence); nothing inside an item is read or
//! changed. `project_id` and `editor_kind` are opaque here: callers validate them.

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::db::Storage;
use crate::projects::normalize_path;

pub mod restore;

pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_PATH_CHARS: usize = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCode {
    NameEmpty,
    NameTooLong,
    NameTaken,
    WorkspaceNotFound,
    ItemPathEmpty,
    ItemPathTooLong,
    ItemRelativePath,
    ItemPathNotFound,
    ItemPathUnreachable,
    ItemDuplicate,
    ItemNotFound,
    ProjectNotFound,
    ProfileNotFound,
    MoveBlocked,
    StorageUnavailable,
    ItemKindMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceError {
    pub code: WorkspaceCode,
    pub detail: Option<String>,
}

impl WorkspaceError {
    fn new(code: WorkspaceCode) -> Self {
        Self { code, detail: None }
    }

    fn with_detail(code: WorkspaceCode, detail: &str) -> Self {
        Self { code, detail: Some(detail.to_string()) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: i64,
    pub position: i64,
    /// `"file"` or `"folder"`.
    pub kind: String,
    pub path: String,
    pub project_id: Option<i64>,
    pub editor_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub items: Vec<Item>,
    /// UTC ISO timestamp of the last completed restore.
    pub last_restored_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
    pub last_restored_at: Option<String>,
    pub project_names: Vec<String>,
}

fn storage_failure<E>(_: E) -> WorkspaceError {
    WorkspaceError::new(WorkspaceCode::StorageUnavailable)
}

/// Trims; rejects empty and over-long names.
pub fn validate_name(raw: &str) -> Result<String, WorkspaceCode> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(WorkspaceCode::NameEmpty);
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(WorkspaceCode::NameTooLong);
    }
    Ok(name.to_string())
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
            // `\\?\` and `\\.\` are extended-length and device prefixes, not servers.
            !server.is_empty() && server != "?" && server != "." && !share.is_empty()
        }
        None => false,
    }
}

/// Normalises and checks shape only, in order: empty, too long, absolute, no `..`. Touches no
/// file system.
pub(crate) fn validate_item_shape(raw: &str) -> Result<String, WorkspaceError> {
    let normalized = normalize_path(raw);
    if normalized.is_empty() {
        return Err(WorkspaceError::new(WorkspaceCode::ItemPathEmpty));
    }
    if normalized.chars().count() > MAX_PATH_CHARS {
        return Err(WorkspaceError::new(WorkspaceCode::ItemPathTooLong));
    }
    if !is_absolute_shape(&normalized) {
        return Err(WorkspaceError::with_detail(WorkspaceCode::ItemRelativePath, &normalized));
    }
    if normalized.split('\\').any(|part| part == "..") {
        return Err(WorkspaceError::with_detail(WorkspaceCode::ItemPathNotFound, &normalized));
    }
    Ok(normalized)
}

/// Returns `(normalised path, kind)`. Checks in order: empty, too long, absolute, exists.
/// Reads only `std::fs::metadata`; run before taking the storage lock.
pub fn validate_item_path(raw: &str) -> Result<(String, String), WorkspaceError> {
    let normalized = validate_item_shape(raw)?;
    match std::fs::metadata(&normalized) {
        Ok(meta) if meta.is_dir() => Ok((normalized, "folder".to_string())),
        Ok(_) => Ok((normalized, "file".to_string())),
        Err(_) => Err(WorkspaceError::with_detail(WorkspaceCode::ItemPathNotFound, &normalized)),
    }
}

fn path_key(normalized: &str) -> String {
    normalized.to_lowercase()
}

fn load_items(conn: &Connection, workspace_id: i64) -> Result<Vec<Item>, WorkspaceError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, position, kind, path, project_id, editor_kind FROM workspace_items \
             WHERE workspace_id = ?1 ORDER BY position",
        )
        .map_err(storage_failure)?;
    let rows = stmt
        .query_map([workspace_id], |r| {
            Ok(Item {
                id: r.get(0)?,
                position: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                project_id: r.get(4)?,
                editor_kind: r.get(5)?,
            })
        })
        .map_err(storage_failure)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(storage_failure)
}

fn load_workspace(conn: &Connection, id: i64) -> Result<Option<Workspace>, WorkspaceError> {
    let row: Option<(String, Option<String>)> = conn
        .query_row("SELECT name, last_restored_at FROM workspaces WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()
        .map_err(storage_failure)?;
    match row {
        Some((name, last_restored_at)) => {
            Ok(Some(Workspace { id, name, items: load_items(conn, id)?, last_restored_at }))
        }
        None => Ok(None),
    }
}

fn require_workspace(conn: &Connection, id: i64) -> Result<Workspace, WorkspaceError> {
    load_workspace(conn, id)?.ok_or_else(|| WorkspaceError::new(WorkspaceCode::WorkspaceNotFound))
}

/// Only a UNIQUE failure means "already exists"; NOT NULL, CHECK and the rest are storage faults.
fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(f, _)
            if f.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    )
}

/// Summaries sorted by name (case-insensitive). `project_names` lists the distinct names of
/// registered projects referenced by items, sorted case-insensitively.
pub fn list(storage: &Storage) -> Result<Vec<WorkspaceSummary>, WorkspaceError> {
    Ok(load_summaries(storage, "w.name_key, w.id")?.into_iter().map(|(s, _)| s).collect())
}

/// A workspace as the Home overview shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeWorkspace {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
    pub project_names: Vec<String>,
    pub last_restored_at: Option<String>,
    pub last_restore_missing: i64,
}

/// Recent-first: restored workspaces by `last_restored_at` descending, then never-restored
/// ones by name key.
pub fn list_for_home(storage: &Storage) -> Result<Vec<HomeWorkspace>, WorkspaceError> {
    let rows = load_summaries(
        storage,
        "(w.last_restored_at IS NULL), w.last_restored_at DESC, w.name_key, w.id",
    )?;
    Ok(rows
        .into_iter()
        .map(|(s, last_restore_missing)| HomeWorkspace {
            id: s.id,
            name: s.name,
            item_count: s.item_count,
            project_names: s.project_names,
            last_restored_at: s.last_restored_at,
            last_restore_missing,
        })
        .collect())
}

/// Stores the outcome of a completed restore: `last_restored_at` = now (UTC, ISO 8601) and the
/// count of missing items. Call only when a run completed.
pub fn set_restore_outcome(
    storage: &Storage,
    workspace_id: i64,
    missing: i64,
) -> Result<(), WorkspaceError> {
    let changed = storage
        .connection()
        .execute(
            "UPDATE workspaces SET last_restored_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), \
             last_restore_missing = ?2 WHERE id = ?1",
            params![workspace_id, missing],
        )
        .map_err(storage_failure)?;
    if changed == 0 {
        return Err(WorkspaceError::new(WorkspaceCode::WorkspaceNotFound));
    }
    Ok(())
}

/// `order_by` is a compile-time constant from this module, never user input.
fn load_summaries(
    storage: &Storage,
    order_by: &'static str,
) -> Result<Vec<(WorkspaceSummary, i64)>, WorkspaceError> {
    let conn = storage.connection();
    let sql = format!(
        "SELECT w.id, w.name, (SELECT COUNT(*) FROM workspace_items i WHERE i.workspace_id = w.id), \
         w.last_restored_at, w.last_restore_missing FROM workspaces w ORDER BY {order_by}"
    );
    let mut stmt = conn.prepare(&sql).map_err(storage_failure)?;
    let base = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(storage_failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_failure)?;
    let mut item_stmt = conn
        .prepare("SELECT DISTINCT project_id FROM workspace_items WHERE workspace_id = ?1 AND project_id IS NOT NULL")
        .map_err(storage_failure)?;
    let projects = crate::projects::list(storage).map_err(storage_failure)?;
    let mut out = Vec::with_capacity(base.len());
    for (id, name, item_count, last_restored_at, missing) in base {
        let ids = item_stmt
            .query_map([id], |r| r.get::<_, i64>(0))
            .map_err(storage_failure)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_failure)?;
        let mut project_names: Vec<String> = projects
            .iter()
            .filter(|p| ids.contains(&p.id))
            .map(|p| p.display_name.clone())
            .collect();
        project_names.sort_by_key(|n| n.to_lowercase());
        out.push((WorkspaceSummary { id, name, item_count, last_restored_at, project_names }, missing));
    }
    Ok(out)
}

pub fn get(storage: &Storage, id: i64) -> Result<Option<Workspace>, WorkspaceError> {
    load_workspace(storage.connection(), id)
}

pub fn create(storage: &Storage, name: &str) -> Result<Workspace, WorkspaceError> {
    let name = validate_name(name).map_err(WorkspaceError::new)?;
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let inserted = tx.execute(
        "INSERT INTO workspaces (name, name_key) VALUES (?1, ?2)",
        params![name, name.to_lowercase()],
    );
    match inserted {
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::NameTaken, &name));
        }
        Err(e) => return Err(storage_failure(e)),
    }
    let id = tx.last_insert_rowid();
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), id)
}

pub fn rename(storage: &Storage, id: i64, name: &str) -> Result<Workspace, WorkspaceError> {
    let name = validate_name(name).map_err(WorkspaceError::new)?;
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let updated = tx.execute(
        "UPDATE workspaces SET name = ?1, name_key = ?2 WHERE id = ?3",
        params![name, name.to_lowercase(), id],
    );
    match updated {
        Ok(0) => return Err(WorkspaceError::new(WorkspaceCode::WorkspaceNotFound)),
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::NameTaken, &name));
        }
        Err(e) => return Err(storage_failure(e)),
    }
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), id)
}

/// Deletes the workspace and its items in one transaction. Items are deleted explicitly so the
/// result does not depend on the connection's `foreign_keys` setting.
pub fn delete(storage: &Storage, id: i64) -> Result<(), WorkspaceError> {
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    tx.execute("DELETE FROM workspace_items WHERE workspace_id = ?1", [id])
        .map_err(storage_failure)?;
    let removed = tx.execute("DELETE FROM workspaces WHERE id = ?1", [id]).map_err(storage_failure)?;
    if removed == 0 {
        return Err(WorkspaceError::new(WorkspaceCode::WorkspaceNotFound));
    }
    tx.commit().map_err(storage_failure)
}

/// Ids of workspaces in which every item references `project_id` (and that hold at least one
/// item). Items with no project, or another project, keep a workspace out of this set.
fn fully_owned_ids(conn: &Connection, project_id: i64) -> Result<Vec<i64>, WorkspaceError> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT p.workspace_id FROM workspace_items p WHERE p.project_id = ?1 \
             AND NOT EXISTS (SELECT 1 FROM workspace_items o \
                             WHERE o.workspace_id = p.workspace_id AND o.project_id IS NOT ?1)",
        )
        .map_err(storage_failure)?;
    let ids = stmt
        .query_map([project_id], |r| r.get::<_, i64>(0))
        .map_err(storage_failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_failure)?;
    Ok(ids)
}

/// Number of workspaces `remove_for_project` would delete entirely; database read only.
/// Items removed from workspaces that are kept are not counted.
pub fn count_for_project(conn: &Connection, project_id: i64) -> Result<i64, WorkspaceError> {
    Ok(fully_owned_ids(conn, project_id)?.len() as i64)
}

/// Removes the workspaces whose items all reference `project_id` (with those items) and returns
/// how many workspaces were deleted. In every other workspace only the items referencing
/// `project_id` are removed and the remaining items are renumbered to `0..n-1`; workspaces with
/// no such items are untouched. Takes a connection so the caller can run it inside its own
/// transaction. Registry rows only; the referenced files and folders are never touched.
pub fn remove_for_project(conn: &Connection, project_id: i64) -> Result<i64, WorkspaceError> {
    let whole = fully_owned_ids(conn, project_id)?;
    for id in &whole {
        conn.execute("DELETE FROM workspace_items WHERE workspace_id = ?1", [id])
            .map_err(storage_failure)?;
        conn.execute("DELETE FROM workspaces WHERE id = ?1", [id]).map_err(storage_failure)?;
    }
    let partial: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT DISTINCT workspace_id FROM workspace_items WHERE project_id = ?1")
            .map_err(storage_failure)?;
        let rows = stmt.query_map([project_id], |r| r.get(0)).map_err(storage_failure)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_failure)?
    };
    conn.execute("DELETE FROM workspace_items WHERE project_id = ?1", [project_id])
        .map_err(storage_failure)?;
    for ws in &partial {
        let remaining: Vec<i64> = {
            let mut stmt = conn
                .prepare("SELECT id FROM workspace_items WHERE workspace_id = ?1 ORDER BY position")
                .map_err(storage_failure)?;
            let rows = stmt.query_map([ws], |r| r.get(0)).map_err(storage_failure)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(storage_failure)?
        };
        // Ascending order only moves items to vacated lower positions, so UNIQUE never trips.
        for (pos, item) in remaining.iter().enumerate() {
            conn.execute(
                "UPDATE workspace_items SET position = ?1 WHERE id = ?2",
                params![pos as i64, item],
            )
            .map_err(storage_failure)?;
        }
    }
    Ok(whole.len() as i64)
}

pub fn add_item(
    storage: &Storage,
    workspace_id: i64,
    normalised_path: &str,
    kind: &str,
    project_id: Option<i64>,
    editor_kind: &str,
) -> Result<Workspace, WorkspaceError> {
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let exists: bool = tx
        .query_row("SELECT 1 FROM workspaces WHERE id = ?1", [workspace_id], |_| Ok(true))
        .optional()
        .map_err(storage_failure)?
        .unwrap_or(false);
    if !exists {
        return Err(WorkspaceError::new(WorkspaceCode::WorkspaceNotFound));
    }
    let next: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM workspace_items WHERE workspace_id = ?1",
            [workspace_id],
            |r| r.get(0),
        )
        .map_err(storage_failure)?;
    let inserted = tx.execute(
        "INSERT INTO workspace_items (workspace_id, position, kind, path, path_key, project_id, editor_kind) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![workspace_id, next, kind, normalised_path, path_key(normalised_path), project_id, editor_kind],
    );
    match inserted {
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::ItemDuplicate, normalised_path));
        }
        Err(e) => return Err(storage_failure(e)),
    }
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), workspace_id)
}

/// `(workspace_id, position)` of an item, or `ItemNotFound`.
fn locate(tx: &Transaction, item_id: i64) -> Result<(i64, i64), WorkspaceError> {
    tx.query_row(
        "SELECT workspace_id, position FROM workspace_items WHERE id = ?1",
        [item_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(storage_failure)?
    .ok_or_else(|| WorkspaceError::new(WorkspaceCode::ItemNotFound))
}

/// Changes only the project reference and editor profile; path and kind stay as stored.
pub fn update_item(
    storage: &Storage,
    item_id: i64,
    project_id: Option<i64>,
    editor_kind: &str,
) -> Result<Workspace, WorkspaceError> {
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let (workspace_id, _) = locate(&tx, item_id)?;
    tx.execute(
        "UPDATE workspace_items SET project_id = ?1, editor_kind = ?2 WHERE id = ?3",
        params![project_id, editor_kind, item_id],
    )
    .map_err(storage_failure)?;
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), workspace_id)
}

/// Removes the item and renumbers following items to keep positions `0..n-1`.
pub fn remove_item(storage: &Storage, item_id: i64) -> Result<Workspace, WorkspaceError> {
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let (workspace_id, position) = locate(&tx, item_id)?;
    tx.execute("DELETE FROM workspace_items WHERE id = ?1", [item_id]).map_err(storage_failure)?;
    let following: Vec<i64> = {
        let mut stmt = tx
            .prepare(
                "SELECT id FROM workspace_items WHERE workspace_id = ?1 AND position > ?2 \
                 ORDER BY position",
            )
            .map_err(storage_failure)?;
        let rows = stmt
            .query_map(params![workspace_id, position], |r| r.get(0))
            .map_err(storage_failure)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_failure)?
    };
    // Ascending order: each item moves into the slot just vacated, so UNIQUE holds throughout.
    for (offset, id) in following.iter().enumerate() {
        tx.execute(
            "UPDATE workspace_items SET position = ?1 WHERE id = ?2",
            params![position + offset as i64, id],
        )
        .map_err(storage_failure)?;
    }
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), workspace_id)
}

/// Swaps the item with its neighbour. The first item cannot move up and the last cannot move
/// down (`MoveBlocked`). The swap goes through the sentinel position `-1`.
pub fn move_item(storage: &Storage, item_id: i64, up: bool) -> Result<Workspace, WorkspaceError> {
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    let (workspace_id, position) = locate(&tx, item_id)?;
    let target = if up { position - 1 } else { position + 1 };
    let neighbour: Option<i64> = tx
        .query_row(
            "SELECT id FROM workspace_items WHERE workspace_id = ?1 AND position = ?2",
            params![workspace_id, target],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_failure)?;
    let Some(neighbour) = neighbour else {
        return Err(WorkspaceError::new(WorkspaceCode::MoveBlocked));
    };
    let set = "UPDATE workspace_items SET position = ?1 WHERE id = ?2";
    tx.execute(set, params![-1_i64, item_id]).map_err(storage_failure)?;
    tx.execute(set, params![position, neighbour]).map_err(storage_failure)?;
    tx.execute(set, params![target, item_id]).map_err(storage_failure)?;
    tx.commit().map_err(storage_failure)?;
    require_workspace(storage.connection(), workspace_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in, Storage};
    use std::path::Path;
    use rusqlite::ErrorCode;

    fn store() -> (tempfile::TempDir, Storage) {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open_in(&tmp.path().join("data")).unwrap();
        (tmp, storage)
    }

    fn s(p: &Path) -> String {
        p.to_string_lossy().into_owned()
    }

    fn file(dir: &Path, name: &str) -> String {
        let p = dir.join(name);
        std::fs::write(&p, b"").unwrap();
        s(&p)
    }

    fn add(storage: &Storage, ws: i64, path: &str) -> Result<Workspace, WorkspaceError> {
        let (norm, kind) = validate_item_path(path).unwrap();
        add_item(storage, ws, &norm, &kind, None, "windows_default")
    }

    fn positions(w: &Workspace) -> Vec<i64> {
        w.items.iter().map(|i| i.position).collect()
    }

    fn names(w: &Workspace) -> Vec<String> {
        w.items
            .iter()
            .map(|i| i.path.rsplit('\\').next().unwrap().to_string())
            .collect()
    }

    #[test]
    fn validate_name_cases() {
        assert_eq!(validate_name("  Ops  "), Ok("Ops".to_string()));
        assert_eq!(validate_name(""), Err(WorkspaceCode::NameEmpty));
        assert_eq!(validate_name("   "), Err(WorkspaceCode::NameEmpty));
        assert_eq!(validate_name(&"a".repeat(101)), Err(WorkspaceCode::NameTooLong));
        assert!(validate_name(&"a".repeat(100)).is_ok());
    }

    #[test]
    fn validate_item_path_cases() {
        let tmp = tempfile::tempdir().unwrap();
        let f = file(tmp.path(), "a.txt");
        let d = s(tmp.path());

        assert_eq!(validate_item_path("").unwrap_err().code, WorkspaceCode::ItemPathEmpty);
        assert_eq!(validate_item_path("   ").unwrap_err().code, WorkspaceCode::ItemPathEmpty);
        assert_eq!(validate_item_path(".\\src").unwrap_err().code, WorkspaceCode::ItemRelativePath);
        assert_eq!(validate_item_path("src").unwrap_err().code, WorkspaceCode::ItemRelativePath);
        assert_eq!(
            validate_item_path("F:\\devshelf-missing-xyz\\nope").unwrap_err().code,
            WorkspaceCode::ItemPathNotFound
        );
        assert_eq!(validate_item_path(&f).unwrap(), (f.clone(), "file".to_string()));
        assert_eq!(validate_item_path(&d).unwrap(), (d.clone(), "folder".to_string()));
        let long = format!("F:\\{}", "a".repeat(259));
        assert_eq!(validate_item_path(&long).unwrap_err().code, WorkspaceCode::ItemPathTooLong);
        assert_eq!(
            validate_item_path("\\\\devshelf-no-such-host-xyz\\share\\x").unwrap_err().code,
            WorkspaceCode::ItemPathNotFound
        );
    }

    #[test]
    fn validate_item_path_rejects_extended_and_device_paths() {
        for raw in ["\\\\?\\C:\\x", "\\\\.\\C:\\x"] {
            let err = validate_item_path(raw).unwrap_err();
            assert_eq!(err.code, WorkspaceCode::ItemRelativePath, "{raw}");
            assert_eq!(err.detail.as_deref(), Some(raw));
        }
    }

    #[test]
    fn validate_item_path_rejects_dot_dot_segments_as_not_found() {
        let err = validate_item_path("C:\\Windows\\..\\Windows").unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemPathNotFound);
        assert_eq!(err.detail.as_deref(), Some("C:\\Windows\\..\\Windows"));
    }

    #[test]
    fn only_unique_constraint_failures_are_unique_violations() {
        let (_t, st) = store();
        let conn = st.connection();
        conn.execute("INSERT INTO workspaces (name, name_key) VALUES ('A', 'a')", [])
            .unwrap();
        let unique = conn
            .execute("INSERT INTO workspaces (name, name_key) VALUES ('B', 'a')", [])
            .unwrap_err();
        assert!(is_unique_violation(&unique));
        let not_null = conn
            .execute("INSERT INTO workspaces (name) VALUES ('C')", [])
            .unwrap_err();
        assert!(matches!(
            &not_null,
            rusqlite::Error::SqliteFailure(f, _) if f.code == ErrorCode::ConstraintViolation
        ));
        assert!(!is_unique_violation(&not_null));
        let check = conn
            .execute(
                "INSERT INTO workspace_items (workspace_id, position, kind, path, path_key, editor_kind) \
                 VALUES (1, 0, 'socket', 'p', 'p', 'windows_default')",
                [],
            )
            .unwrap_err();
        assert!(matches!(
            &check,
            rusqlite::Error::SqliteFailure(f, _) if f.code == ErrorCode::ConstraintViolation
        ));
        assert!(!is_unique_violation(&check));
    }

    #[test]
    fn validate_item_path_normalises_bare_drive() {
        // Only run where drive C: exists; normalisation of "C:" is "C:\".
        if Path::new("C:\\").exists() {
            assert_eq!(validate_item_path("C:").unwrap(), ("C:\\".to_string(), "folder".to_string()));
        }
    }

    #[test]
    fn create_then_list_sorted_case_insensitively() {
        let (_t, st) = store();
        create(&st, "beta").unwrap();
        create(&st, "Alpha").unwrap();
        create(&st, "  Gamma ").unwrap();
        let list = list(&st).unwrap();
        let got: Vec<&str> = list.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(got, vec!["Alpha", "beta", "Gamma"]);
        assert!(list.iter().all(|w| w.item_count == 0 && w.project_names.is_empty()));
    }

    #[test]
    fn name_uniqueness_and_rename() {
        let (_t, st) = store();
        let a = create(&st, "Ops").unwrap();
        let b = create(&st, "Dev").unwrap();
        assert_eq!(create(&st, "OPS").unwrap_err().code, WorkspaceCode::NameTaken);
        assert_eq!(rename(&st, a.id, "ops").unwrap().name, "ops");
        assert_eq!(rename(&st, b.id, "OPS").unwrap_err().code, WorkspaceCode::NameTaken);
        assert_eq!(rename(&st, 9999, "x").unwrap_err().code, WorkspaceCode::WorkspaceNotFound);
        assert_eq!(rename(&st, a.id, "").unwrap_err().code, WorkspaceCode::NameEmpty);
        assert_eq!(create(&st, "").unwrap_err().code, WorkspaceCode::NameEmpty);
    }

    #[test]
    fn get_unknown_is_none() {
        let (_t, st) = store();
        assert_eq!(get(&st, 5).unwrap(), None);
    }

    #[test]
    fn add_item_appends_in_order_and_rejects_duplicates() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        let dir = t.path().join("items");
        std::fs::create_dir(&dir).unwrap();
        let f1 = file(&dir, "One.txt");
        let f2 = file(&dir, "two.txt");
        add(&st, w.id, &f1).unwrap();
        add(&st, w.id, &f2).unwrap();
        let w3 = add(&st, w.id, &s(&dir)).unwrap();
        assert_eq!(positions(&w3), vec![0, 1, 2]);
        assert_eq!(w3.items[2].kind, "folder");
        let dup = add(&st, w.id, &f1.to_uppercase()).unwrap_err();
        assert_eq!(dup.code, WorkspaceCode::ItemDuplicate);
        assert_eq!(dup.detail.as_deref(), Some(normalize_path(&f1.to_uppercase()).as_str()));
        let folder = format!("{}\\", s(&dir)).to_lowercase();
        let dup2 = add(&st, w.id, &folder).unwrap_err();
        assert_eq!(dup2.code, WorkspaceCode::ItemDuplicate);
        assert_eq!(dup2.detail.as_deref(), Some(normalize_path(&folder).as_str()));
        assert_eq!(get(&st, w.id).unwrap().unwrap().items.len(), 3);
        assert_eq!(
            add_item(&st, 9999, "F:\\x", "file", None, "windows_default").unwrap_err().code,
            WorkspaceCode::WorkspaceNotFound
        );
    }

    #[test]
    fn add_item_accepts_opaque_project_id_command_layer_checks_lookup() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        let f = file(t.path(), "a.txt");
        let (n, k) = validate_item_path(&f).unwrap();
        let out = add_item(&st, w.id, &n, &k, Some(424242), "vs_code").unwrap();
        assert_eq!(out.items[0].project_id, Some(424242));
        assert_eq!(out.items[0].editor_kind, "vs_code");
    }

    #[test]
    fn move_item_swaps_and_blocks_at_ends() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        for n in ["a", "b", "c"] {
            add(&st, w.id, &file(t.path(), n)).unwrap();
        }
        let cur = get(&st, w.id).unwrap().unwrap();
        let (ia, ib, ic) = (cur.items[0].id, cur.items[1].id, cur.items[2].id);
        assert_eq!(move_item(&st, ia, true).unwrap_err().code, WorkspaceCode::MoveBlocked);
        assert_eq!(move_item(&st, ic, false).unwrap_err().code, WorkspaceCode::MoveBlocked);
        assert_eq!(move_item(&st, 9999, true).unwrap_err().code, WorkspaceCode::ItemNotFound);

        let w = move_item(&st, ib, true).unwrap();
        assert_eq!(names(&w), vec!["b", "a", "c"]);
        assert_eq!(positions(&w), vec![0, 1, 2]);
        let w = move_item(&st, ib, false).unwrap();
        assert_eq!(names(&w), vec!["a", "b", "c"]);
        assert_eq!(positions(&w), vec![0, 1, 2]);
        let w = move_item(&st, ia, false).unwrap();
        assert_eq!(names(&w), vec!["b", "a", "c"]);
        assert_eq!(positions(&w), vec![0, 1, 2]);
        let w = move_item(&st, ic, true).unwrap();
        assert_eq!(names(&w), vec!["b", "c", "a"]);
        assert_eq!(positions(&w), vec![0, 1, 2]);
    }

    #[test]
    fn remove_item_renumbers() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        for n in ["a", "b", "c", "d"] {
            add(&st, w.id, &file(t.path(), n)).unwrap();
        }
        let cur = get(&st, w.id).unwrap().unwrap();
        let w = remove_item(&st, cur.items[1].id).unwrap();
        assert_eq!(names(&w), vec!["a", "c", "d"]);
        assert_eq!(positions(&w), vec![0, 1, 2]);
        assert_eq!(remove_item(&st, 9999).unwrap_err().code, WorkspaceCode::ItemNotFound);
        let w = add(&st, w.id, &file(t.path(), "e")).unwrap();
        assert_eq!(positions(&w), vec![0, 1, 2, 3]);
    }

    #[test]
    fn update_item_changes_project_and_editor_only() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        let f = file(t.path(), "a.txt");
        let w = add(&st, w.id, &f).unwrap();
        let before = w.items[0].clone();
        let w = update_item(&st, before.id, Some(7), "notepad_plus_plus").unwrap();
        let after = &w.items[0];
        assert_eq!(after.project_id, Some(7));
        assert_eq!(after.editor_kind, "notepad_plus_plus");
        assert_eq!(after.path, before.path);
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.position, before.position);
        let w = update_item(&st, before.id, None, "windows_default").unwrap();
        assert_eq!(w.items[0].project_id, None);
        assert_eq!(
            update_item(&st, 9999, None, "windows_default").unwrap_err().code,
            WorkspaceCode::ItemNotFound
        );
    }

    #[test]
    fn delete_cascades_items_and_leaves_disk_untouched() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        let other = create(&st, "Other").unwrap();
        let dir = t.path().join("proj");
        std::fs::create_dir(&dir).unwrap();
        let f = file(&dir, "a.txt");
        add(&st, w.id, &s(&dir)).unwrap();
        add(&st, w.id, &f).unwrap();
        add(&st, other.id, &f).unwrap();
        delete(&st, w.id).unwrap();
        let n: i64 = st
            .connection()
            .query_row("SELECT COUNT(*) FROM workspace_items WHERE workspace_id = ?1", [w.id], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
        assert_eq!(get(&st, w.id).unwrap(), None);
        assert_eq!(get(&st, other.id).unwrap().unwrap().items.len(), 1);
        assert!(dir.is_dir() && Path::new(&f).is_file());
        assert_eq!(delete(&st, w.id).unwrap_err().code, WorkspaceCode::WorkspaceNotFound);
    }

    #[test]
    fn persistence_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let items = tmp.path().join("items");
        std::fs::create_dir(&items).unwrap();
        let id;
        {
            let st = open_in(&data).unwrap();
            id = create(&st, "Keep").unwrap().id;
            add(&st, id, &file(&items, "a")).unwrap();
            add(&st, id, &file(&items, "b")).unwrap();
        }
        let st = open_in(&data).unwrap();
        let w = get(&st, id).unwrap().unwrap();
        assert_eq!(w.name, "Keep");
        assert_eq!(names(&w), vec!["a", "b"]);
    }

    #[test]
    fn summary_counts_items() {
        let (t, st) = store();
        let w = create(&st, "W").unwrap();
        add(&st, w.id, &file(t.path(), "a")).unwrap();
        let sum = list(&st).unwrap();
        assert_eq!(sum[0].item_count, 1);
    }

    fn stamp(st: &Storage, id: i64, at: &str) {
        st.connection()
            .execute("UPDATE workspaces SET last_restored_at = ?2 WHERE id = ?1", params![id, at])
            .unwrap();
    }

    #[test]
    fn set_restore_outcome_sets_both_fields_and_overwrites() {
        let (_t, st) = store();
        let w = create(&st, "W").unwrap();
        set_restore_outcome(&st, w.id, 3).unwrap();
        let first = list_for_home(&st).unwrap().remove(0);
        assert_eq!(first.last_restore_missing, 3);
        let at = first.last_restored_at.expect("stamp set");
        assert!(at.len() == 20 && at.ends_with('Z') && at.contains('T'), "{at}");

        stamp(&st, w.id, "2000-01-01T00:00:00Z");
        set_restore_outcome(&st, w.id, 0).unwrap();
        let second = list_for_home(&st).unwrap().remove(0);
        assert_eq!(second.last_restore_missing, 0);
        assert_ne!(second.last_restored_at.as_deref(), Some("2000-01-01T00:00:00Z"));
        assert_eq!(
            set_restore_outcome(&st, 99, 1).unwrap_err().code,
            WorkspaceCode::WorkspaceNotFound
        );
    }

    #[test]
    fn last_restore_missing_defaults_to_zero() {
        let (_t, st) = store();
        create(&st, "W").unwrap();
        let home = list_for_home(&st).unwrap();
        assert_eq!(home[0].last_restore_missing, 0);
        assert_eq!(home[0].last_restored_at, None);
    }

    #[test]
    fn list_for_home_orders_restored_desc_then_never_restored_by_name() {
        let (_t, st) = store();
        let never_b = create(&st, "beta").unwrap();
        let old = create(&st, "Zulu").unwrap();
        let never_a = create(&st, "Alpha").unwrap();
        let new = create(&st, "Mike").unwrap();
        stamp(&st, old.id, "2024-01-01T00:00:00Z");
        stamp(&st, new.id, "2025-06-01T12:00:00Z");
        let ids: Vec<i64> = list_for_home(&st).unwrap().iter().map(|w| w.id).collect();
        assert_eq!(ids, vec![new.id, old.id, never_a.id, never_b.id]);
    }

    #[test]
    fn list_for_home_matches_summary_counts_and_project_names() {
        let (t, st) = store();
        let proj_dir = t.path().join("projects");
        std::fs::create_dir_all(proj_dir.join("p1")).unwrap();
        std::fs::create_dir_all(proj_dir.join("p2")).unwrap();
        let register = |sub: &str, name: &str| {
            let path = crate::projects::validate_path(&s(&proj_dir.join(sub))).unwrap();
            crate::projects::register(
                &st,
                path,
                crate::projects::NewProject {
                    display_name: name.to_string(),
                    project_type: "Other".to_string(),
                    tags_raw: String::new(),
                },
            )
            .unwrap()
        };
        let p1 = register("p1", "zeta");
        let p2 = register("p2", "Alpha");
        let w = create(&st, "W").unwrap();
        let (n1, k1) = validate_item_path(&file(t.path(), "a")).unwrap();
        add_item(&st, w.id, &n1, &k1, Some(p1.id), "windows_default").unwrap();
        let (n2, k2) = validate_item_path(&file(t.path(), "b")).unwrap();
        add_item(&st, w.id, &n2, &k2, Some(p2.id), "windows_default").unwrap();
        add(&st, w.id, &file(t.path(), "c")).unwrap();

        let sum = list(&st).unwrap().remove(0);
        let home = list_for_home(&st).unwrap().remove(0);
        assert_eq!(home.item_count, 3);
        assert_eq!(home.item_count, sum.item_count);
        assert_eq!(home.project_names, vec!["Alpha", "zeta"]);
        assert_eq!(home.project_names, sum.project_names);
        assert_eq!(home.name, sum.name);
    }

    #[test]
    fn restore_missing_count_survives_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let id;
        {
            let st = open_in(&data).unwrap();
            id = create(&st, "W").unwrap().id;
            set_restore_outcome(&st, id, 4).unwrap();
        }
        let st = open_in(&data).unwrap();
        let home = list_for_home(&st).unwrap();
        assert_eq!(home[0].id, id);
        assert_eq!(home[0].last_restore_missing, 4);
    }

    fn snapshot(dir: &Path) -> Vec<(String, std::time::SystemTime)> {
        let mut v: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.file_name().to_string_lossy().into_owned(), e.metadata().unwrap().modified().unwrap())
            })
            .collect();
        v.push((".".into(), std::fs::metadata(dir).unwrap().modified().unwrap()));
        v.sort();
        v
    }

    #[test]
    fn mutations_do_not_write_to_item_paths() {
        let (t, st) = store();
        let dir = t.path().join("items");
        std::fs::create_dir(&dir).unwrap();
        let f1 = file(&dir, "a");
        let f2 = file(&dir, "b");
        let w = create(&st, "W").unwrap();
        let before = snapshot(&dir);
        add(&st, w.id, &s(&dir)).unwrap();
        add(&st, w.id, &f1).unwrap();
        let w = add(&st, w.id, &f2).unwrap();
        let id = w.items[2].id;
        move_item(&st, id, true).unwrap();
        update_item(&st, id, Some(1), "vs_code").unwrap();
        remove_item(&st, id).unwrap();
        delete(&st, w.id).unwrap();
        assert_eq!(snapshot(&dir), before);
    }
}
