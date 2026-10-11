use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::editors;
use crate::projects;
use crate::workspaces::{
    self, Item, Workspace, WorkspaceCode, WorkspaceError, WorkspaceSummary,
};

/// Wire form of `workspaces::WorkspaceCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceErrorCode {
    NameEmpty,
    NameTooLong,
    NameTaken,
    WorkspaceNotFound,
    ItemPathEmpty,
    ItemPathTooLong,
    ItemRelativePath,
    ItemPathNotFound,
    ItemDuplicate,
    ItemNotFound,
    ProjectNotFound,
    ProfileNotFound,
    MoveBlocked,
    StorageUnavailable,
}

impl From<WorkspaceCode> for WorkspaceErrorCode {
    fn from(code: WorkspaceCode) -> Self {
        match code {
            WorkspaceCode::NameEmpty => Self::NameEmpty,
            WorkspaceCode::NameTooLong => Self::NameTooLong,
            WorkspaceCode::NameTaken => Self::NameTaken,
            WorkspaceCode::WorkspaceNotFound => Self::WorkspaceNotFound,
            WorkspaceCode::ItemPathEmpty => Self::ItemPathEmpty,
            WorkspaceCode::ItemPathTooLong => Self::ItemPathTooLong,
            WorkspaceCode::ItemRelativePath => Self::ItemRelativePath,
            WorkspaceCode::ItemPathNotFound => Self::ItemPathNotFound,
            WorkspaceCode::ItemDuplicate => Self::ItemDuplicate,
            WorkspaceCode::ItemNotFound => Self::ItemNotFound,
            WorkspaceCode::ProjectNotFound => Self::ProjectNotFound,
            WorkspaceCode::ProfileNotFound => Self::ProfileNotFound,
            WorkspaceCode::MoveBlocked => Self::MoveBlocked,
            WorkspaceCode::StorageUnavailable => Self::StorageUnavailable,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceItemView {
    pub id: i64,
    pub position: i64,
    pub kind: String,
    pub path: String,
    pub project_id: Option<i64>,
    pub editor_kind: String,
}

impl From<Item> for WorkspaceItemView {
    fn from(i: Item) -> Self {
        Self {
            id: i.id,
            position: i.position,
            kind: i.kind,
            path: i.path,
            project_id: i.project_id,
            editor_kind: i.editor_kind,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceView {
    pub id: i64,
    pub name: String,
    pub items: Vec<WorkspaceItemView>,
}

impl From<Workspace> for WorkspaceView {
    fn from(w: Workspace) -> Self {
        Self {
            id: w.id,
            name: w.name,
            items: w.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceSummaryView {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
    pub project_names: Vec<String>,
}

impl From<WorkspaceSummary> for WorkspaceSummaryView {
    fn from(s: WorkspaceSummary) -> Self {
        Self {
            id: s.id,
            name: s.name,
            item_count: s.item_count,
            project_names: s.project_names,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WorkspacesListArgs {}

#[derive(Debug, Deserialize)]
pub struct WorkspaceGetArgs {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceCreateArgs {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceRenameArgs {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceDeleteArgs {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ItemAddArgs {
    pub workspace_id: i64,
    pub path: String,
    pub project_id: Option<i64>,
    pub editor_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ItemUpdateArgs {
    pub item_id: i64,
    pub project_id: Option<i64>,
    pub editor_kind: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemRemoveArgs {
    pub item_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ItemMoveArgs {
    pub item_id: i64,
    pub up: bool,
}

#[derive(Debug, Deserialize)]
pub struct WorkspacePickItemArgs {
    pub folder: bool,
}

#[derive(Debug, Serialize)]
pub struct WorkspacesListResult {
    pub workspaces: Vec<WorkspaceSummaryView>,
    pub error: Option<WorkspaceErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceResult {
    pub workspace: Option<WorkspaceView>,
    pub error: Option<WorkspaceErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceDeleteResult {
    pub error: Option<WorkspaceErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspacePickResult {
    pub path: Option<String>,
}

fn failure(code: WorkspaceCode) -> WorkspaceError {
    WorkspaceError { code, detail: None }
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
/// `f` MUST do database work only.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, WorkspaceError>,
) -> Result<T, WorkspaceError> {
    let mutex = state
        .result
        .as_ref()
        .map_err(|_| failure(WorkspaceCode::StorageUnavailable))?;
    let guard = mutex
        .lock()
        .map_err(|_| failure(WorkspaceCode::StorageUnavailable))?;
    f(&guard)
}

fn list_result(r: Result<Vec<WorkspaceSummary>, WorkspaceError>) -> WorkspacesListResult {
    match r {
        Ok(list) => WorkspacesListResult {
            workspaces: list.into_iter().map(Into::into).collect(),
            error: None,
            detail: None,
        },
        Err(e) => WorkspacesListResult {
            workspaces: Vec::new(),
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn workspace_result(r: Result<Workspace, WorkspaceError>) -> WorkspaceResult {
    match r {
        Ok(w) => WorkspaceResult {
            workspace: Some(w.into()),
            error: None,
            detail: None,
        },
        Err(e) => WorkspaceResult {
            workspace: None,
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

/// Checks the project reference and editor profile. Reads the database, so it runs inside the
/// single lock taken for the write.
fn check_references(
    storage: &Storage,
    project_id: Option<i64>,
    editor_kind: Option<&str>,
) -> Result<String, WorkspaceError> {
    if let Some(id) = project_id {
        match projects::get(storage, id) {
            Ok(Some(_)) => {}
            Ok(None) => return Err(failure(WorkspaceCode::ProjectNotFound)),
            Err(_) => return Err(failure(WorkspaceCode::StorageUnavailable)),
        }
    }
    match editors::resolve_profile(storage, editor_kind) {
        Ok(profile) => Ok(profile.kind),
        Err(e) => match e.code {
            editors::EditorCode::StorageUnavailable => {
                Err(failure(WorkspaceCode::StorageUnavailable))
            }
            _ => Err(failure(WorkspaceCode::ProfileNotFound)),
        },
    }
}

#[tauri::command]
pub fn workspaces_list(
    _args: WorkspacesListArgs,
    state: State<'_, StorageState>,
) -> WorkspacesListResult {
    list_result(with_storage(&state, workspaces::list))
}

#[tauri::command]
pub fn workspaces_get(
    args: WorkspaceGetArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| {
        workspaces::get(s, args.id)?.ok_or_else(|| failure(WorkspaceCode::WorkspaceNotFound))
    }))
}

#[tauri::command]
pub fn workspaces_create(
    args: WorkspaceCreateArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| workspaces::create(s, &args.name)))
}

#[tauri::command]
pub fn workspaces_rename(
    args: WorkspaceRenameArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| {
        workspaces::rename(s, args.id, &args.name)
    }))
}

#[tauri::command]
pub fn workspaces_delete(
    args: WorkspaceDeleteArgs,
    state: State<'_, StorageState>,
) -> WorkspaceDeleteResult {
    match with_storage(&state, |s| workspaces::delete(s, args.id)) {
        Ok(()) => WorkspaceDeleteResult {
            error: None,
            detail: None,
        },
        Err(e) => WorkspaceDeleteResult {
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

/// Runs off the main thread. The path check touches the file system and runs first with no lock
/// held; the lock is then taken once for the reference checks and the insert.
#[tauri::command(async)]
pub fn workspace_items_add(
    args: ItemAddArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    let checked = workspaces::validate_item_path(&args.path);
    workspace_result(checked.and_then(|(path, kind)| {
        with_storage(&state, |s| {
            let editor_kind = check_references(s, args.project_id, args.editor_kind.as_deref())?;
            workspaces::add_item(s, args.workspace_id, &path, &kind, args.project_id, &editor_kind)
        })
    }))
}

/// Runs off the main thread. One lock covers the reference checks and the update.
#[tauri::command(async)]
pub fn workspace_items_update(
    args: ItemUpdateArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| {
        let editor_kind = check_references(s, args.project_id, Some(&args.editor_kind))?;
        workspaces::update_item(s, args.item_id, args.project_id, &editor_kind)
    }))
}

#[tauri::command]
pub fn workspace_items_remove(
    args: ItemRemoveArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| {
        workspaces::remove_item(s, args.item_id)
    }))
}

#[tauri::command]
pub fn workspace_items_move(
    args: ItemMoveArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(with_storage(&state, |s| {
        workspaces::move_item(s, args.item_id, args.up)
    }))
}

fn path_string(p: tauri_plugin_dialog::FilePath) -> Option<String> {
    p.into_path()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Async so the blocking dialog does not run on the main thread.
#[tauri::command]
pub async fn workspaces_pick_item(
    args: WorkspacePickItemArgs,
    app: AppHandle,
) -> WorkspacePickResult {
    let dialog = app.dialog().file();
    let picked = if args.folder {
        dialog.blocking_pick_folder()
    } else {
        dialog.blocking_pick_file()
    };
    WorkspacePickResult {
        path: picked.and_then(path_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(WorkspaceCode, &str); 14] = [
        (WorkspaceCode::NameEmpty, "name_empty"),
        (WorkspaceCode::NameTooLong, "name_too_long"),
        (WorkspaceCode::NameTaken, "name_taken"),
        (WorkspaceCode::WorkspaceNotFound, "workspace_not_found"),
        (WorkspaceCode::ItemPathEmpty, "item_path_empty"),
        (WorkspaceCode::ItemPathTooLong, "item_path_too_long"),
        (WorkspaceCode::ItemRelativePath, "item_relative_path"),
        (WorkspaceCode::ItemPathNotFound, "item_path_not_found"),
        (WorkspaceCode::ItemDuplicate, "item_duplicate"),
        (WorkspaceCode::ItemNotFound, "item_not_found"),
        (WorkspaceCode::ProjectNotFound, "project_not_found"),
        (WorkspaceCode::ProfileNotFound, "profile_not_found"),
        (WorkspaceCode::MoveBlocked, "move_blocked"),
        (WorkspaceCode::StorageUnavailable, "storage_unavailable"),
    ];

    #[test]
    fn every_code_maps_to_its_snake_case_string() {
        for (code, expected) in ALL {
            let json = serde_json::to_string(&WorkspaceErrorCode::from(code)).unwrap();
            assert_eq!(json, format!("\"{expected}\""));
        }
    }

    fn temp_storage() -> (tempfile::TempDir, Storage) {
        let tmp = tempfile::tempdir().unwrap();
        let storage = crate::db::open_in(&tmp.path().join("data")).unwrap();
        (tmp, storage)
    }

    #[test]
    fn check_references_unknown_project_is_project_not_found() {
        let (_t, st) = temp_storage();
        let err = check_references(&st, Some(424242), None).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ProjectNotFound);
    }

    #[test]
    fn check_references_unknown_editor_kind_is_profile_not_found() {
        let (_t, st) = temp_storage();
        let err = check_references(&st, None, Some("emacs")).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ProfileNotFound);
    }

    #[test]
    fn check_references_none_resolves_to_the_default_kind() {
        let (_t, st) = temp_storage();
        assert_eq!(check_references(&st, None, None).unwrap(), "windows_default");
    }

    #[test]
    fn check_references_valid_kind_is_returned() {
        let (_t, st) = temp_storage();
        assert_eq!(check_references(&st, None, Some("vs_code")).unwrap(), "vs_code");
    }

    #[test]
    fn error_detail_is_carried_into_the_result() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let file = t.path().join("a.txt");
        std::fs::write(&file, b"").unwrap();
        let (path, kind) = workspaces::validate_item_path(&file.to_string_lossy()).unwrap();
        workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default").unwrap();

        let dup = workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default");
        let r = workspace_result(dup);
        assert!(r.workspace.is_none());
        assert_eq!(r.error, Some(WorkspaceCode::ItemDuplicate.into()));
        assert_eq!(r.detail.as_deref(), Some(path.as_str()));

        let item_id = workspaces::get(&st, w.id).unwrap().unwrap().items[0].id;
        let r = workspace_result(workspaces::move_item(&st, item_id, true));
        assert!(r.workspace.is_none());
        assert_eq!(r.error, Some(WorkspaceCode::MoveBlocked.into()));
        assert_eq!(r.detail, None);
    }
}
