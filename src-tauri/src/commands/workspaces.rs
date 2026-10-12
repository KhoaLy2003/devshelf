use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::commands::editors::{AppOpener, EditorErrorCode};
use crate::editors;
use crate::projects;
use crate::workspaces::restore::{
    self, FsProbe, ItemOutcome, MissingReason, PathProbe, ProbeResult, RestoreReport,
};
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
    ItemPathUnreachable,
    ItemDuplicate,
    ItemNotFound,
    ProjectNotFound,
    ProfileNotFound,
    MoveBlocked,
    StorageUnavailable,
    ItemKindMismatch,
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
            WorkspaceCode::ItemPathUnreachable => Self::ItemPathUnreachable,
            WorkspaceCode::ItemDuplicate => Self::ItemDuplicate,
            WorkspaceCode::ItemNotFound => Self::ItemNotFound,
            WorkspaceCode::ProjectNotFound => Self::ProjectNotFound,
            WorkspaceCode::ProfileNotFound => Self::ProfileNotFound,
            WorkspaceCode::MoveBlocked => Self::MoveBlocked,
            WorkspaceCode::StorageUnavailable => Self::StorageUnavailable,
            WorkspaceCode::ItemKindMismatch => Self::ItemKindMismatch,
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
    pub last_restored_at: Option<String>,
}

impl From<Workspace> for WorkspaceView {
    fn from(w: Workspace) -> Self {
        Self {
            id: w.id,
            name: w.name,
            items: w.items.into_iter().map(Into::into).collect(),
            last_restored_at: w.last_restored_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceSummaryView {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
    pub project_names: Vec<String>,
    pub last_restored_at: Option<String>,
}

impl From<WorkspaceSummary> for WorkspaceSummaryView {
    fn from(s: WorkspaceSummary) -> Self {
        Self {
            id: s.id,
            name: s.name,
            item_count: s.item_count,
            project_names: s.project_names,
            last_restored_at: s.last_restored_at,
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

#[derive(Debug, Deserialize)]
pub struct WorkspaceRestoreArgs {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ItemRelinkArgs {
    pub item_id: i64,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingReasonView {
    NotFound,
    NowFolder,
    NowFile,
}

/// `kind` is one of `opened`, `missing`, `unreachable`, `could_not_open`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestoreOutcomeView {
    pub kind: &'static str,
    pub item_id: i64,
    pub reason: Option<MissingReasonView>,
    pub code: Option<EditorErrorCode>,
}

impl From<ItemOutcome> for RestoreOutcomeView {
    fn from(o: ItemOutcome) -> Self {
        let (kind, item_id, reason, code) = match o {
            ItemOutcome::Opened { item_id } => ("opened", item_id, None, None),
            ItemOutcome::Missing { item_id, reason } => {
                let reason = match reason {
                    MissingReason::NotFound => MissingReasonView::NotFound,
                    MissingReason::NowFolder => MissingReasonView::NowFolder,
                    MissingReason::NowFile => MissingReasonView::NowFile,
                };
                ("missing", item_id, Some(reason), None)
            }
            ItemOutcome::Unreachable { item_id } => ("unreachable", item_id, None, None),
            ItemOutcome::CouldNotOpen { item_id, code } => {
                ("could_not_open", item_id, None, Some(code.into()))
            }
        };
        Self { kind, item_id, reason, code }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRestoreResult {
    pub outcomes: Vec<RestoreOutcomeView>,
    pub completed: bool,
    pub error: Option<WorkspaceErrorCode>,
    pub detail: Option<String>,
}

/// `error` with `completed: true` means the run finished and was reported, but recording the
/// last-used time failed; `error` with `completed: false` means nothing ran.
fn restore_result(r: Result<RestoreRun, WorkspaceError>) -> WorkspaceRestoreResult {
    match r {
        Ok(RestoreRun { report, save_error }) => WorkspaceRestoreResult {
            outcomes: report.outcomes.into_iter().map(Into::into).collect(),
            completed: report.completed,
            error: save_error.as_ref().map(|e| e.code.into()),
            detail: save_error.and_then(|e| e.detail),
        },
        Err(e) => WorkspaceRestoreResult {
            outcomes: Vec::new(),
            completed: false,
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
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

/// Runs off the main thread. Loads the plan under one lock, probes and launches with no lock
/// held, then locks once more to record the completed run.
#[tauri::command(async)]
pub fn workspace_restore(
    args: WorkspaceRestoreArgs,
    state: State<'_, StorageState>,
    app: AppHandle,
) -> WorkspaceRestoreResult {
    restore_result(run_restore(&state, args.id, &FsProbe, &AppOpener(&app)))
}

/// A finished run: the report always survives, even when recording the run failed.
#[derive(Debug)]
struct RestoreRun {
    report: RestoreReport,
    /// Set when the run completed but the restore outcome could not be saved.
    save_error: Option<WorkspaceError>,
}

fn run_restore(
    state: &StorageState,
    workspace_id: i64,
    probe: &dyn PathProbe,
    launcher: &dyn editors::Launcher,
) -> Result<RestoreRun, WorkspaceError> {
    let plan = with_storage(state, |s| restore::load_plan(s, workspace_id))?;
    let report = restore::run_plan(probe, launcher, plan);
    let save_error = record_restore(state, &report);
    Ok(RestoreRun { report, save_error })
}

/// Stores the outcome of a run, only when it completed: the time and the count of `Missing`
/// plus `Unreachable` items. A save failure is returned so the caller can keep the report.
fn record_restore(state: &StorageState, report: &RestoreReport) -> Option<WorkspaceError> {
    if !report.completed {
        return None;
    }
    let missing = report
        .outcomes
        .iter()
        .filter(|o| matches!(o, ItemOutcome::Missing { .. } | ItemOutcome::Unreachable { .. }))
        .count() as i64;
    with_storage(state, |s| workspaces::set_restore_outcome(s, report.workspace_id, missing)).err()
}

/// Replays a probe result taken before the lock, so the locked update does no file system work.
struct ProbedOnce(ProbeResult);

impl PathProbe for ProbedOnce {
    fn probe(&self, _path: &str) -> ProbeResult {
        self.0
    }
}

/// Runs off the main thread. Shape check and probe run with no lock held; the lock covers the
/// update only.
#[tauri::command(async)]
pub fn workspace_items_relink(
    args: ItemRelinkArgs,
    state: State<'_, StorageState>,
) -> WorkspaceResult {
    workspace_result(run_relink(&state, &FsProbe, args.item_id, &args.path))
}

fn run_relink(
    state: &StorageState,
    probe: &dyn PathProbe,
    item_id: i64,
    raw_path: &str,
) -> Result<Workspace, WorkspaceError> {
    let normalized = workspaces::validate_item_shape(raw_path)?;
    let probed = ProbedOnce(probe.probe(&normalized));
    with_storage(state, |s| restore::relink(s, &probed, item_id, &normalized))
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
    use crate::editors::EditorCode;
    use super::*;

    const ALL: [(WorkspaceCode, &str); 16] = [
        (WorkspaceCode::NameEmpty, "name_empty"),
        (WorkspaceCode::NameTooLong, "name_too_long"),
        (WorkspaceCode::NameTaken, "name_taken"),
        (WorkspaceCode::WorkspaceNotFound, "workspace_not_found"),
        (WorkspaceCode::ItemPathEmpty, "item_path_empty"),
        (WorkspaceCode::ItemPathTooLong, "item_path_too_long"),
        (WorkspaceCode::ItemRelativePath, "item_relative_path"),
        (WorkspaceCode::ItemPathNotFound, "item_path_not_found"),
        (WorkspaceCode::ItemPathUnreachable, "item_path_unreachable"),
        (WorkspaceCode::ItemDuplicate, "item_duplicate"),
        (WorkspaceCode::ItemNotFound, "item_not_found"),
        (WorkspaceCode::ProjectNotFound, "project_not_found"),
        (WorkspaceCode::ProfileNotFound, "profile_not_found"),
        (WorkspaceCode::MoveBlocked, "move_blocked"),
        (WorkspaceCode::StorageUnavailable, "storage_unavailable"),
        (WorkspaceCode::ItemKindMismatch, "item_kind_mismatch"),
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

    #[test]
    fn new_codes_serialize_snake_case() {
        let json = serde_json::to_string(&WorkspaceErrorCode::from(WorkspaceCode::ItemKindMismatch))
            .unwrap();
        assert_eq!(json, "\"item_kind_mismatch\"");
    }

    fn view_json(o: ItemOutcome) -> serde_json::Value {
        serde_json::to_value(RestoreOutcomeView::from(o)).unwrap()
    }

    #[test]
    fn opened_outcome_maps_to_opened_view() {
        assert_eq!(
            view_json(ItemOutcome::Opened { item_id: 3 }),
            serde_json::json!({"kind": "opened", "item_id": 3, "reason": null, "code": null})
        );
    }

    #[test]
    fn missing_outcomes_carry_each_reason() {
        for (reason, expected) in [
            (MissingReason::NotFound, "not_found"),
            (MissingReason::NowFolder, "now_folder"),
            (MissingReason::NowFile, "now_file"),
        ] {
            assert_eq!(
                view_json(ItemOutcome::Missing { item_id: 4, reason }),
                serde_json::json!({"kind": "missing", "item_id": 4, "reason": expected, "code": null})
            );
        }
    }

    #[test]
    fn unreachable_outcome_maps_to_unreachable_view() {
        assert_eq!(
            view_json(ItemOutcome::Unreachable { item_id: 5 }),
            serde_json::json!({"kind": "unreachable", "item_id": 5, "reason": null, "code": null})
        );
    }

    #[test]
    fn could_not_open_outcome_carries_the_editor_code() {
        assert_eq!(
            view_json(ItemOutcome::CouldNotOpen {
                item_id: 6,
                code: EditorCode::ExecutableNotFound
            }),
            serde_json::json!({
                "kind": "could_not_open", "item_id": 6, "reason": null, "code": "executable_not_found"
            })
        );
    }

    #[test]
    fn restore_result_carries_error_and_detail() {
        let r = restore_result(Err(WorkspaceError {
            code: WorkspaceCode::ItemNotFound,
            detail: Some("x".to_string()),
        }));
        assert!(r.outcomes.is_empty());
        assert!(!r.completed);
        assert_eq!(r.error, Some(WorkspaceErrorCode::ItemNotFound));
        assert_eq!(r.detail.as_deref(), Some("x"));
    }

    struct CountingLauncher(std::cell::Cell<usize>);

    impl editors::Launcher for CountingLauncher {
        fn spawn(&self, _plan: &editors::LaunchPlan) -> std::io::Result<()> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }

        fn open_default(&self, _item_path: &str) -> Result<(), String> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }

    fn state_with(storage: Storage, folder: &std::path::Path) -> StorageState {
        StorageState { folder: folder.to_path_buf(), result: Ok(std::sync::Mutex::new(storage)) }
    }

    #[test]
    fn run_restore_opens_items_and_marks_the_workspace_restored() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let file = t.path().join("a.txt");
        std::fs::write(&file, b"").unwrap();
        let (path, kind) = workspaces::validate_item_path(&file.to_string_lossy()).unwrap();
        workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default").unwrap();
        let state = state_with(st, t.path());

        let launcher = CountingLauncher(std::cell::Cell::new(0));
        let run = run_restore(&state, w.id, &FsProbe, &launcher).unwrap();
        assert!(run.report.completed);
        assert!(run.save_error.is_none());
        assert!(matches!(run.report.outcomes[..], [ItemOutcome::Opened { .. }]));
        assert_eq!(launcher.0.get(), 1);
        let got = with_storage(&state, |s| Ok(workspaces::get(s, w.id)?.unwrap())).unwrap();
        assert!(got.last_restored_at.is_some());
    }

    #[test]
    fn run_restore_on_an_empty_workspace_completes_with_no_outcomes() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let state = state_with(st, t.path());
        let launcher = CountingLauncher(std::cell::Cell::new(0));
        let run = run_restore(&state, w.id, &FsProbe, &launcher).unwrap();
        assert!(run.report.completed);
        assert!(run.save_error.is_none());
        assert!(run.report.outcomes.is_empty());
        assert_eq!(launcher.0.get(), 0);
    }

    #[test]
    fn run_restore_unknown_workspace_is_not_found() {
        let (t, st) = temp_storage();
        let state = state_with(st, t.path());
        let launcher = CountingLauncher(std::cell::Cell::new(0));
        let err = run_restore(&state, 9999, &FsProbe, &launcher).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::WorkspaceNotFound);
    }

    #[test]
    fn run_restore_keeps_the_report_when_the_last_used_time_cannot_be_saved() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let file = t.path().join("a.txt");
        std::fs::write(&file, b"").unwrap();
        let (path, kind) = workspaces::validate_item_path(&file.to_string_lossy()).unwrap();
        workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default").unwrap();
        st.connection()
            .execute_batch(
                "CREATE TRIGGER block_restore_stamp BEFORE UPDATE ON workspaces \
                 BEGIN SELECT RAISE(ABORT, 'read-only'); END;",
            )
            .unwrap();
        let state = state_with(st, t.path());

        let launcher = CountingLauncher(std::cell::Cell::new(0));
        let run = run_restore(&state, w.id, &FsProbe, &launcher).unwrap();
        assert!(run.report.completed);
        assert!(matches!(run.report.outcomes[..], [ItemOutcome::Opened { .. }]));
        assert_eq!(launcher.0.get(), 1);
        assert_eq!(run.save_error.as_ref().map(|e| e.code), Some(WorkspaceCode::StorageUnavailable));

        let wire = restore_result(Ok(run));
        assert!(wire.completed);
        assert_eq!(wire.outcomes.len(), 1);
        assert_eq!(wire.error, Some(WorkspaceErrorCode::StorageUnavailable));
        let got = with_storage(&state, |s| Ok(workspaces::get(s, w.id)?.unwrap())).unwrap();
        assert!(got.last_restored_at.is_none());
    }

    #[test]
    fn run_restore_stores_the_missing_count_of_a_completed_run() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let mut files = Vec::new();
        for name in ["a.txt", "b.txt", "c.txt"] {
            let file = t.path().join(name);
            std::fs::write(&file, b"").unwrap();
            let (path, kind) = workspaces::validate_item_path(&file.to_string_lossy()).unwrap();
            workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default").unwrap();
            files.push(file);
        }
        std::fs::remove_file(&files[0]).unwrap();
        std::fs::remove_file(&files[1]).unwrap();
        let state = state_with(st, t.path());

        let launcher = CountingLauncher(std::cell::Cell::new(0));
        let run = run_restore(&state, w.id, &FsProbe, &launcher).unwrap();
        assert!(run.report.completed);
        assert!(run.save_error.is_none());
        assert_eq!(launcher.0.get(), 1);
        let home = with_storage(&state, workspaces::list_for_home).unwrap();
        assert_eq!(home[0].last_restore_missing, 2);
        assert!(home[0].last_restored_at.is_some());
    }

    #[test]
    fn a_run_that_did_not_complete_stores_nothing() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let state = state_with(st, t.path());
        let report = RestoreReport {
            workspace_id: w.id,
            outcomes: vec![
                ItemOutcome::Missing { item_id: 1, reason: MissingReason::NotFound },
                ItemOutcome::Unreachable { item_id: 2 },
            ],
            completed: false,
        };
        assert!(record_restore(&state, &report).is_none());
        let home = with_storage(&state, workspaces::list_for_home).unwrap();
        assert_eq!(home[0].last_restored_at, None);
        assert_eq!(home[0].last_restore_missing, 0);
    }

    #[test]
    fn run_relink_updates_the_path_and_rejects_a_kind_mismatch() {
        let (t, st) = temp_storage();
        let w = workspaces::create(&st, "W").unwrap();
        let old = t.path().join("old.txt");
        let new = t.path().join("new.txt");
        let dir = t.path().join("dir");
        std::fs::write(&old, b"").unwrap();
        std::fs::write(&new, b"").unwrap();
        std::fs::create_dir(&dir).unwrap();
        let (path, kind) = workspaces::validate_item_path(&old.to_string_lossy()).unwrap();
        let ws = workspaces::add_item(&st, w.id, &path, &kind, None, "windows_default").unwrap();
        let item_id = ws.items[0].id;
        let state = state_with(st, t.path());

        let err = run_relink(&state, &FsProbe, item_id, &dir.to_string_lossy()).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemKindMismatch);
        assert_eq!(err.detail.as_deref(), Some("file"));

        let ws = run_relink(&state, &FsProbe, item_id, &new.to_string_lossy()).unwrap();
        assert_eq!(ws.items[0].id, item_id);
        assert!(ws.items[0].path.ends_with("new.txt"));

        let err = run_relink(&state, &FsProbe, item_id, "relative\\x").unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemRelativePath);
    }
}
