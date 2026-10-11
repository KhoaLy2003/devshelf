use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::projects::{self, NewProject, Project, ValidationCode, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectErrorCode {
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
    ProjectNotFound,
}

impl From<ValidationCode> for ProjectErrorCode {
    fn from(code: ValidationCode) -> Self {
        match code {
            ValidationCode::PathEmpty => Self::PathEmpty,
            ValidationCode::PathNotFound => Self::PathNotFound,
            ValidationCode::PathIsFile => Self::PathIsFile,
            ValidationCode::AlreadyRegistered => Self::AlreadyRegistered,
            ValidationCode::NameEmpty => Self::NameEmpty,
            ValidationCode::NameTooLong => Self::NameTooLong,
            ValidationCode::PathTooLong => Self::PathTooLong,
            ValidationCode::TagInvalid => Self::TagInvalid,
            ValidationCode::TooManyTags => Self::TooManyTags,
            ValidationCode::TypeInvalid => Self::TypeInvalid,
            ValidationCode::StorageUnavailable => Self::StorageUnavailable,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProjectView {
    pub id: i64,
    pub display_name: String,
    pub root_path: String,
    pub drive: String,
    pub project_type: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_at: String,
}

impl From<Project> for ProjectView {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            display_name: p.display_name,
            root_path: p.root_path,
            drive: p.drive,
            project_type: p.project_type,
            tags: p.tags,
            pinned: p.pinned,
            created_at: p.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectsListArgs {}

#[derive(Debug, Serialize)]
pub struct ProjectsListResult {
    pub projects: Vec<ProjectView>,
    pub error: Option<ProjectErrorCode>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectGetArgs {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct ProjectGetResult {
    pub project: Option<ProjectView>,
    pub error: Option<ProjectErrorCode>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectRegisterArgs {
    pub root_path: String,
    pub display_name: String,
    pub project_type: String,
    pub tags_raw: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectRegisterResult {
    pub project: Option<ProjectView>,
    pub error: Option<ProjectErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectCheckPathArgs {
    pub root_path: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectCheckPathResult {
    pub nested_in: Option<String>,
    pub error: Option<ProjectErrorCode>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSetPinnedArgs {
    pub id: i64,
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProjectPickFolderArgs {}

#[derive(Debug, Serialize)]
pub struct ProjectPickFolderResult {
    pub path: Option<String>,
}

fn unavailable() -> ValidationError {
    ValidationError {
        code: ValidationCode::StorageUnavailable,
        detail: None,
    }
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, ValidationError>,
) -> Result<T, ValidationError> {
    let mutex = state.result.as_ref().map_err(|_| unavailable())?;
    let guard = mutex.lock().map_err(|_| unavailable())?;
    f(&guard)
}

fn get_result(r: Result<Option<Project>, ValidationError>) -> ProjectGetResult {
    match r {
        Ok(Some(p)) => ProjectGetResult {
            project: Some(p.into()),
            error: None,
        },
        Ok(None) => ProjectGetResult {
            project: None,
            error: Some(ProjectErrorCode::ProjectNotFound),
        },
        Err(e) => ProjectGetResult {
            project: None,
            error: Some(e.code.into()),
        },
    }
}

fn list_result(r: Result<Vec<Project>, ValidationError>) -> ProjectsListResult {
    match r {
        Ok(v) => ProjectsListResult {
            projects: v.into_iter().map(ProjectView::from).collect(),
            error: None,
        },
        Err(e) => ProjectsListResult {
            projects: Vec::new(),
            error: Some(e.code.into()),
        },
    }
}

fn register_result(r: Result<Project, ValidationError>) -> ProjectRegisterResult {
    match r {
        Ok(p) => ProjectRegisterResult {
            project: Some(p.into()),
            error: None,
            detail: None,
        },
        Err(e) => ProjectRegisterResult {
            project: None,
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn check_path_result(r: Result<Option<String>, ValidationError>) -> ProjectCheckPathResult {
    match r {
        Ok(nested_in) => ProjectCheckPathResult {
            nested_in,
            error: None,
        },
        Err(e) => ProjectCheckPathResult {
            nested_in: None,
            error: Some(e.code.into()),
        },
    }
}

#[tauri::command]
pub fn projects_list(_args: ProjectsListArgs, state: State<'_, StorageState>) -> ProjectsListResult {
    list_result(with_storage(&state, projects::list))
}

#[tauri::command]
pub fn projects_get(args: ProjectGetArgs, state: State<'_, StorageState>) -> ProjectGetResult {
    get_result(with_storage(&state, |s| projects::get(s, args.id)))
}

/// Runs off the main thread (`async` command attribute on a sync fn; a borrowed `State` in an
/// `async fn` must return `Result`). The path is validated before the storage lock is taken;
/// the lock covers only the database work.
#[tauri::command(async)]
pub fn projects_register(
    args: ProjectRegisterArgs,
    state: State<'_, StorageState>,
) -> ProjectRegisterResult {
    let ProjectRegisterArgs {
        root_path,
        display_name,
        project_type,
        tags_raw,
    } = args;
    let result = projects::validate_path(&root_path).and_then(|path| {
        with_storage(&state, |s| {
            projects::register(
                s,
                path,
                NewProject {
                    display_name,
                    project_type,
                    tags_raw,
                },
            )
        })
    });
    register_result(result)
}

/// Runs off the main thread; the lock covers only the database query.
#[tauri::command(async)]
pub fn projects_check_path(
    args: ProjectCheckPathArgs,
    state: State<'_, StorageState>,
) -> ProjectCheckPathResult {
    let normalized = projects::normalize_path(&args.root_path);
    check_path_result(with_storage(&state, |s| projects::nested_note(s, &normalized)))
}

#[tauri::command]
pub fn projects_set_pinned(
    args: ProjectSetPinnedArgs,
    state: State<'_, StorageState>,
) -> ProjectGetResult {
    get_result(with_storage(&state, |s| {
        projects::set_pinned(s, args.id, args.pinned)
    }))
}

/// Async so the blocking dialog does not run on the main thread.
#[tauri::command]
pub async fn projects_pick_folder(
    _args: ProjectPickFolderArgs,
    app: AppHandle,
) -> ProjectPickFolderResult {
    let path = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned());
    ProjectPickFolderResult { path }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_validation_code_maps_to_matching_error_code() {
        let pairs = [
            (ValidationCode::PathEmpty, ProjectErrorCode::PathEmpty),
            (ValidationCode::PathNotFound, ProjectErrorCode::PathNotFound),
            (ValidationCode::PathIsFile, ProjectErrorCode::PathIsFile),
            (ValidationCode::AlreadyRegistered, ProjectErrorCode::AlreadyRegistered),
            (ValidationCode::NameEmpty, ProjectErrorCode::NameEmpty),
            (ValidationCode::NameTooLong, ProjectErrorCode::NameTooLong),
            (ValidationCode::PathTooLong, ProjectErrorCode::PathTooLong),
            (ValidationCode::TagInvalid, ProjectErrorCode::TagInvalid),
            (ValidationCode::TooManyTags, ProjectErrorCode::TooManyTags),
            (ValidationCode::TypeInvalid, ProjectErrorCode::TypeInvalid),
            (ValidationCode::StorageUnavailable, ProjectErrorCode::StorageUnavailable),
        ];
        for (v, p) in pairs {
            assert_eq!(ProjectErrorCode::from(v), p);
        }
    }

    #[test]
    fn error_codes_serialise_as_snake_case() {
        let cases = [
            (ProjectErrorCode::PathEmpty, "path_empty"),
            (ProjectErrorCode::PathNotFound, "path_not_found"),
            (ProjectErrorCode::PathIsFile, "path_is_file"),
            (ProjectErrorCode::AlreadyRegistered, "already_registered"),
            (ProjectErrorCode::NameEmpty, "name_empty"),
            (ProjectErrorCode::NameTooLong, "name_too_long"),
            (ProjectErrorCode::PathTooLong, "path_too_long"),
            (ProjectErrorCode::TagInvalid, "tag_invalid"),
            (ProjectErrorCode::TooManyTags, "too_many_tags"),
            (ProjectErrorCode::TypeInvalid, "type_invalid"),
            (ProjectErrorCode::StorageUnavailable, "storage_unavailable"),
            (ProjectErrorCode::ProjectNotFound, "project_not_found"),
        ];
        for (code, text) in cases {
            assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{text}\""));
        }
    }

    #[test]
    fn missing_project_maps_to_project_not_found() {
        let r = get_result(Ok(None));
        assert!(r.project.is_none());
        assert_eq!(r.error, Some(ProjectErrorCode::ProjectNotFound));
    }

    #[test]
    fn register_error_keeps_detail() {
        let r = register_result(Err(ValidationError {
            code: ValidationCode::AlreadyRegistered,
            detail: Some("api".into()),
        }));
        assert_eq!(r.error, Some(ProjectErrorCode::AlreadyRegistered));
        assert_eq!(r.detail.as_deref(), Some("api"));
    }
}
