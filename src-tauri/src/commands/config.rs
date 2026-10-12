use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::editors::{AppOpener, EditorErrorCode};
use crate::commands::storage::StorageState;
use crate::config::{
    self, ConfigCode, ConfigEntry, ConfigError, FsProbe, Row, Status,
};
use crate::db::Storage;
use crate::editors::{self, EditorCode, EditorError};
use crate::projects;

/// Wire form of `config::ConfigCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigErrorCode {
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

impl From<ConfigCode> for ConfigErrorCode {
    fn from(code: ConfigCode) -> Self {
        match code {
            ConfigCode::ConfigNotFound => Self::ConfigNotFound,
            ConfigCode::PathEmpty => Self::PathEmpty,
            ConfigCode::PathNotFound => Self::PathNotFound,
            ConfigCode::PathIsFolder => Self::PathIsFolder,
            ConfigCode::PathRelative => Self::PathRelative,
            ConfigCode::PathTooLong => Self::PathTooLong,
            ConfigCode::DuplicatePath => Self::DuplicatePath,
            ConfigCode::ProjectNotFound => Self::ProjectNotFound,
            ConfigCode::EnvironmentInvalid => Self::EnvironmentInvalid,
            ConfigCode::KindInvalid => Self::KindInvalid,
            ConfigCode::StorageUnavailable => Self::StorageUnavailable,
        }
    }
}

/// Wire form of `config::Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigStatusView {
    Available,
    Missing,
    FolderAtPath,
    Inaccessible,
}

impl From<Status> for ConfigStatusView {
    fn from(status: Status) -> Self {
        match status {
            Status::Available => Self::Available,
            Status::Missing => Self::Missing,
            Status::FolderAtPath => Self::FolderAtPath,
            Status::Inaccessible => Self::Inaccessible,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConfigEntryView {
    pub id: i64,
    pub project_id: i64,
    pub project_name: Option<String>,
    pub path: String,
    pub name: String,
    pub environment: String,
    pub kind: String,
    pub status: ConfigStatusView,
}

impl From<ConfigEntry> for ConfigEntryView {
    fn from(e: ConfigEntry) -> Self {
        Self {
            id: e.id,
            project_id: e.project_id,
            project_name: e.project_name,
            path: e.path,
            name: e.name,
            environment: e.environment,
            kind: e.kind,
            status: e.status.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigListArgs {}

#[derive(Debug, Serialize)]
pub struct ConfigListResult {
    pub entries: Vec<ConfigEntryView>,
    pub error: Option<ConfigErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigAddArgs {
    pub project_id: i64,
    pub path: String,
    pub environment: String,
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRelinkArgs {
    pub id: i64,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ConfigEntryResult {
    pub entry: Option<ConfigEntryView>,
    pub error: Option<ConfigErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRemoveArgs {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct ConfigRemoveResult {
    pub error: Option<ConfigErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigOpenArgs {
    pub id: i64,
    pub editor_kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigOpenResult {
    pub opened_name: Option<String>,
    pub error: Option<EditorErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigPickArgs {}

#[derive(Debug, Serialize)]
pub struct ConfigPickResult {
    pub path: Option<String>,
}

fn unavailable() -> ConfigError {
    ConfigError::new(ConfigCode::StorageUnavailable)
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
/// `f` MUST do database work only.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, ConfigError>,
) -> Result<T, ConfigError> {
    let mutex = state.result.as_ref().map_err(|_| unavailable())?;
    let guard = mutex.lock().map_err(|_| unavailable())?;
    f(&guard)
}

fn entry_by_id(
    rows: Vec<Row>,
    names: &HashMap<i64, String>,
    id: i64,
) -> Result<ConfigEntry, ConfigError> {
    let row = rows
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| ConfigError::new(ConfigCode::ConfigNotFound))?;
    config::with_status(vec![row], names, &FsProbe)
        .pop()
        .ok_or_else(|| ConfigError::new(ConfigCode::ConfigNotFound))
}

fn list_result(r: Result<Vec<ConfigEntry>, ConfigError>) -> ConfigListResult {
    match r {
        Ok(entries) => ConfigListResult {
            entries: entries.into_iter().map(ConfigEntryView::from).collect(),
            error: None,
            detail: None,
        },
        Err(e) => ConfigListResult {
            entries: Vec::new(),
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn entry_result(r: Result<ConfigEntry, ConfigError>) -> ConfigEntryResult {
    match r {
        Ok(entry) => ConfigEntryResult {
            entry: Some(entry.into()),
            error: None,
            detail: None,
        },
        Err(e) => ConfigEntryResult {
            entry: None,
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn remove_result(r: Result<(), ConfigError>) -> ConfigRemoveResult {
    match r {
        Ok(()) => ConfigRemoveResult {
            error: None,
            detail: None,
        },
        Err(e) => ConfigRemoveResult {
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn open_result(r: Result<editors::Profile, EditorError>) -> ConfigOpenResult {
    match r {
        Ok(profile) => ConfigOpenResult {
            opened_name: Some(profile.name),
            error: None,
            detail: None,
        },
        Err(e) => ConfigOpenResult {
            opened_name: None,
            error: Some(e.code.into()),
            detail: e.detail,
        },
    }
}

fn item_not_found(detail: &str) -> EditorError {
    EditorError {
        code: EditorCode::ItemNotFound,
        kind: None,
        detail: Some(detail.to_string()),
    }
}

/// Maps a probed status to the open outcome: only `Available` may proceed.
fn open_gate(status: Status) -> Result<(), EditorError> {
    match status {
        Status::Available => Ok(()),
        Status::Inaccessible => Err(item_not_found("inaccessible")),
        Status::Missing | Status::FolderAtPath => Err(item_not_found("missing")),
    }
}

/// Runs off the main thread. Rows are read under the lock; statuses are probed after it is
/// released.
#[tauri::command(async)]
pub fn config_files_list(
    _args: ConfigListArgs,
    state: State<'_, StorageState>,
) -> ConfigListResult {
    let result = with_storage(&state, |s| Ok((config::load_rows(s)?, config::project_names(s)?)))
        .map(|(rows, names)| config::with_status(rows, &names, &FsProbe));
    list_result(result)
}

/// Runs off the main thread. The file check runs with no lock held; the project check and the
/// insert each take the lock briefly.
#[tauri::command(async)]
pub fn config_files_add(
    args: ConfigAddArgs,
    state: State<'_, StorageState>,
) -> ConfigEntryResult {
    let result = (|| {
        let path = config::validate_entry(&FsProbe, &args.path, &args.environment, &args.kind)?;
        with_storage(&state, |s| match projects::get(s, args.project_id) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(ConfigError::new(ConfigCode::ProjectNotFound)),
            Err(_) => Err(unavailable()),
        })?;
        let (id, rows, names) = with_storage(&state, |s| {
            let id = config::add(s, args.project_id, &path, &args.environment, &args.kind)?;
            Ok((id, config::load_rows(s)?, config::project_names(s)?))
        })?;
        entry_by_id(rows, &names, id)
    })();
    entry_result(result)
}

/// Runs off the main thread. The entry is read under the lock, the new path is checked with no
/// lock held, then the lock is taken again for the update.
#[tauri::command(async)]
pub fn config_files_relink(
    args: ConfigRelinkArgs,
    state: State<'_, StorageState>,
) -> ConfigEntryResult {
    let result = (|| {
        let (environment, kind) =
            with_storage(&state, |s| config::environment_and_kind(s, args.id))?;
        let path = config::validate_entry(&FsProbe, &args.path, &environment, &kind)?;
        let (rows, names) = with_storage(&state, |s| {
            config::update_path(s, args.id, &path)?;
            Ok((config::load_rows(s)?, config::project_names(s)?))
        })?;
        entry_by_id(rows, &names, args.id)
    })();
    entry_result(result)
}

#[tauri::command]
pub fn config_files_remove(
    args: ConfigRemoveArgs,
    state: State<'_, StorageState>,
) -> ConfigRemoveResult {
    remove_result(with_storage(&state, |s| config::remove(s, args.id)))
}

/// Runs off the main thread. The lock is taken for the database read of the entry only; the
/// profile is then resolved under its own short lock, and the file check and launch run with no
/// lock held.
#[tauri::command(async)]
pub fn config_files_open(
    args: ConfigOpenArgs,
    app: AppHandle,
    state: State<'_, StorageState>,
) -> ConfigOpenResult {
    let storage_error = || EditorError {
        code: EditorCode::StorageUnavailable,
        kind: None,
        detail: None,
    };
    let result = (|| {
        let row = with_storage(&state, |s| config::get_row(s, args.id))
            .map_err(|e| match e.code {
                ConfigCode::StorageUnavailable => storage_error(),
                _ => item_not_found("missing"),
            })?
            .ok_or_else(|| item_not_found("missing"))?;
        open_gate(config::status_of(&FsProbe, &row.path))?;
        let mutex = state.result.as_ref().map_err(|_| storage_error())?;
        let profile = {
            let guard = mutex.lock().map_err(|_| storage_error())?;
            editors::resolve_profile(&guard, args.editor_kind.as_deref())?
        };
        editors::open_with_profile(&AppOpener(&app), profile, &row.path)
    })();
    open_result(result)
}

/// Async so the blocking dialog does not run on the main thread.
#[tauri::command(async)]
pub fn config_files_pick(_args: ConfigPickArgs, app: AppHandle) -> ConfigPickResult {
    let path = app
        .dialog()
        .file()
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned());
    ConfigPickResult { path }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json<T: Serialize>(v: T) -> String {
        serde_json::to_string(&v).unwrap()
    }

    #[test]
    fn config_codes_are_snake_case() {
        let all = [
            (ConfigCode::ConfigNotFound, "config_not_found"),
            (ConfigCode::PathEmpty, "path_empty"),
            (ConfigCode::PathNotFound, "path_not_found"),
            (ConfigCode::PathIsFolder, "path_is_folder"),
            (ConfigCode::PathRelative, "path_relative"),
            (ConfigCode::PathTooLong, "path_too_long"),
            (ConfigCode::DuplicatePath, "duplicate_path"),
            (ConfigCode::ProjectNotFound, "project_not_found"),
            (ConfigCode::EnvironmentInvalid, "environment_invalid"),
            (ConfigCode::KindInvalid, "kind_invalid"),
            (ConfigCode::StorageUnavailable, "storage_unavailable"),
        ];
        for (code, text) in all {
            assert_eq!(json(ConfigErrorCode::from(code)), format!("\"{text}\""));
        }
    }

    #[test]
    fn statuses_map_to_view_strings() {
        let all = [
            (Status::Available, "available"),
            (Status::Missing, "missing"),
            (Status::FolderAtPath, "folder_at_path"),
            (Status::Inaccessible, "inaccessible"),
        ];
        for (status, text) in all {
            assert_eq!(json(ConfigStatusView::from(status)), format!("\"{text}\""));
        }
    }

    #[test]
    fn view_keeps_missing_project_name_as_none() {
        let view = ConfigEntryView::from(ConfigEntry {
            id: 1,
            project_id: 99,
            project_name: None,
            path: "F:\\a\\app.yml".into(),
            name: "app.yml".into(),
            environment: "dev".into(),
            kind: "yaml".into(),
            status: Status::Missing,
        });
        assert_eq!(view.project_name, None);
        assert_eq!(view.status, ConfigStatusView::Missing);
        assert!(json(&view).contains("\"project_name\":null"));
    }

    #[test]
    fn open_gate_allows_only_available() {
        assert!(open_gate(Status::Available).is_ok());
        for (status, detail) in [
            (Status::Missing, "missing"),
            (Status::FolderAtPath, "missing"),
            (Status::Inaccessible, "inaccessible"),
        ] {
            let e = open_gate(status).unwrap_err();
            assert_eq!(e.code, EditorCode::ItemNotFound);
            assert_eq!(e.detail.as_deref(), Some(detail));
        }
    }
}
