use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::editors::{
    self, EditorCode, EditorError, LaunchPlan, Launcher, Profile, ProfileInput, SystemLauncher,
};

/// Wire form of `editors::EditorCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorErrorCode {
    ProfileNotFound,
    ProfilePathEmpty,
    ExecutableNotFound,
    ExecutableInvalid,
    PathUnsafe,
    NameTooLong,
    ItemNotFound,
    LaunchFailed,
    StorageUnavailable,
}

impl From<EditorCode> for EditorErrorCode {
    fn from(code: EditorCode) -> Self {
        match code {
            EditorCode::ProfileNotFound => Self::ProfileNotFound,
            EditorCode::ProfilePathEmpty => Self::ProfilePathEmpty,
            EditorCode::ExecutableNotFound => Self::ExecutableNotFound,
            EditorCode::ExecutableInvalid => Self::ExecutableInvalid,
            EditorCode::PathUnsafe => Self::PathUnsafe,
            EditorCode::NameTooLong => Self::NameTooLong,
            EditorCode::ItemNotFound => Self::ItemNotFound,
            EditorCode::LaunchFailed => Self::LaunchFailed,
            EditorCode::StorageUnavailable => Self::StorageUnavailable,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EditorProfileView {
    pub kind: String,
    pub name: String,
    pub exe_path: String,
    pub is_default: bool,
}

impl From<Profile> for EditorProfileView {
    fn from(p: Profile) -> Self {
        Self {
            kind: p.kind,
            name: p.name,
            exe_path: p.exe_path,
            is_default: p.is_default,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EditorProfileInputArgs {
    pub kind: String,
    pub name: String,
    pub exe_path: String,
}

#[derive(Debug, Deserialize)]
pub struct EditorProfilesListArgs {}

#[derive(Debug, Deserialize)]
pub struct EditorProfilesSaveArgs {
    pub profiles: Vec<EditorProfileInputArgs>,
}

#[derive(Debug, Deserialize)]
pub struct EditorSetDefaultArgs {
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct EditorProfilesResult {
    pub profiles: Vec<EditorProfileView>,
    pub error: Option<EditorErrorCode>,
    pub kind: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EditorOpenArgs {
    pub item_path: String,
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EditorOpenResult {
    pub opened_name: Option<String>,
    pub error: Option<EditorErrorCode>,
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EditorPickExecutableArgs {}

#[derive(Debug, Deserialize)]
pub struct EditorPickItemArgs {
    pub folder: bool,
}

#[derive(Debug, Serialize)]
pub struct EditorPickResult {
    pub path: Option<String>,
}

fn unavailable() -> EditorError {
    EditorError {
        code: EditorCode::StorageUnavailable,
        kind: None,
        detail: None,
    }
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
/// `f` MUST do database work only.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, EditorError>,
) -> Result<T, EditorError> {
    let mutex = state.result.as_ref().map_err(|_| unavailable())?;
    let guard = mutex.lock().map_err(|_| unavailable())?;
    f(&guard)
}

fn profiles_result(r: Result<Vec<Profile>, EditorError>) -> EditorProfilesResult {
    match r {
        Ok(profiles) => EditorProfilesResult {
            profiles: profiles.into_iter().map(EditorProfileView::from).collect(),
            error: None,
            kind: None,
            detail: None,
        },
        Err(e) => EditorProfilesResult {
            profiles: Vec::new(),
            error: Some(e.code.into()),
            kind: e.kind,
            detail: e.detail,
        },
    }
}

fn open_result(r: Result<Profile, EditorError>) -> EditorOpenResult {
    match r {
        Ok(profile) => EditorOpenResult {
            opened_name: Some(profile.name),
            error: None,
            kind: Some(profile.kind),
        },
        Err(e) => EditorOpenResult {
            opened_name: None,
            error: Some(e.code.into()),
            kind: e.kind,
        },
    }
}

/// Launches through the real app: processes via `SystemLauncher`, the Windows default via the
/// opener plugin.
struct AppOpener<'a>(&'a AppHandle);

impl Launcher for AppOpener<'_> {
    fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()> {
        SystemLauncher.spawn(plan)
    }

    fn open_default(&self, item_path: &str) -> Result<(), String> {
        self.0
            .opener()
            .open_path(item_path, None::<&str>)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn editor_profiles_list(
    _args: EditorProfilesListArgs,
    state: State<'_, StorageState>,
) -> EditorProfilesResult {
    profiles_result(with_storage(&state, editors::list))
}

/// Runs off the main thread. Path checks run first with no lock held; the storage lock is then
/// taken once, for the save transaction only.
#[tauri::command(async)]
pub fn editor_profiles_save(
    args: EditorProfilesSaveArgs,
    state: State<'_, StorageState>,
) -> EditorProfilesResult {
    let inputs: Vec<ProfileInput> = args
        .profiles
        .into_iter()
        .map(|p| ProfileInput {
            kind: p.kind,
            name: p.name,
            exe_path: p.exe_path,
        })
        .collect();
    let result = editors::validate_profiles(&inputs)
        .and_then(|valid| with_storage(&state, |s| editors::save(s, valid)));
    profiles_result(result)
}

#[tauri::command(async)]
pub fn editor_profiles_set_default(
    args: EditorSetDefaultArgs,
    state: State<'_, StorageState>,
) -> EditorProfilesResult {
    profiles_result(with_storage(&state, |s| editors::set_default(s, &args.kind)))
}

/// Runs off the main thread. The lock covers only resolving the profile; the item check and the
/// launch run after it is released.
#[tauri::command(async)]
pub fn editor_open(
    args: EditorOpenArgs,
    app: AppHandle,
    state: State<'_, StorageState>,
) -> EditorOpenResult {
    let result = with_storage(&state, |s| {
        editors::resolve_profile(s, args.kind.as_deref())
    })
    .and_then(|profile| editors::open_with_profile(&AppOpener(&app), profile, &args.item_path));
    open_result(result)
}

fn path_string(p: tauri_plugin_dialog::FilePath) -> Option<String> {
    p.into_path()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Async so the blocking dialog does not run on the main thread.
#[tauri::command]
pub async fn editor_pick_executable(
    _args: EditorPickExecutableArgs,
    app: AppHandle,
) -> EditorPickResult {
    let path = app
        .dialog()
        .file()
        .add_filter("Programs", &["exe"])
        .blocking_pick_file()
        .and_then(path_string);
    EditorPickResult { path }
}

/// Async so the blocking dialog does not run on the main thread.
#[tauri::command]
pub async fn editor_pick_item(args: EditorPickItemArgs, app: AppHandle) -> EditorPickResult {
    let dialog = app.dialog().file();
    let picked = if args.folder {
        dialog.blocking_pick_folder()
    } else {
        dialog.blocking_pick_file()
    };
    EditorPickResult {
        path: picked.and_then(path_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(EditorCode, &str); 9] = [
        (EditorCode::ProfileNotFound, "profile_not_found"),
        (EditorCode::ProfilePathEmpty, "profile_path_empty"),
        (EditorCode::ExecutableNotFound, "executable_not_found"),
        (EditorCode::ExecutableInvalid, "executable_invalid"),
        (EditorCode::PathUnsafe, "path_unsafe"),
        (EditorCode::NameTooLong, "name_too_long"),
        (EditorCode::ItemNotFound, "item_not_found"),
        (EditorCode::LaunchFailed, "launch_failed"),
        (EditorCode::StorageUnavailable, "storage_unavailable"),
    ];

    fn err(code: EditorCode, kind: Option<&str>, detail: Option<&str>) -> EditorError {
        EditorError {
            code,
            kind: kind.map(str::to_owned),
            detail: detail.map(str::to_owned),
        }
    }

    #[test]
    fn every_code_maps_to_its_snake_case_string() {
        for (code, expected) in ALL {
            let json = serde_json::to_value(EditorErrorCode::from(code)).unwrap();
            assert_eq!(json, serde_json::Value::String(expected.into()));
        }
    }

    #[test]
    fn profile_path_empty_result_carries_the_kind() {
        let r = profiles_result(Err(err(EditorCode::ProfilePathEmpty, Some("vs_code"), None)));
        assert_eq!(r.error, Some(EditorErrorCode::ProfilePathEmpty));
        assert_eq!(r.kind.as_deref(), Some("vs_code"));
        assert!(r.profiles.is_empty());

        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["error"], "profile_path_empty");
        assert_eq!(json["kind"], "vs_code");
    }

    #[test]
    fn open_error_carries_code_and_kind_without_a_name() {
        let r = open_result(Err(err(
            EditorCode::ProfilePathEmpty,
            Some("notepad_plus_plus"),
            None,
        )));
        assert_eq!(r.opened_name, None);
        assert_eq!(r.error, Some(EditorErrorCode::ProfilePathEmpty));
        assert_eq!(r.kind.as_deref(), Some("notepad_plus_plus"));
    }

    #[test]
    fn open_success_reports_the_profile_name() {
        let r = open_result(Ok(Profile {
            kind: "vs_code".into(),
            name: "VS Code".into(),
            exe_path: "C:\\x\\code.exe".into(),
            is_default: false,
        }));
        assert_eq!(r.opened_name.as_deref(), Some("VS Code"));
        assert_eq!(r.error, None);
    }

    #[test]
    fn list_success_has_no_error_fields() {
        let r = profiles_result(Ok(vec![Profile {
            kind: "custom".into(),
            name: "Custom".into(),
            exe_path: String::new(),
            is_default: true,
        }]));
        assert_eq!(r.profiles.len(), 1);
        assert!(r.error.is_none() && r.kind.is_none() && r.detail.is_none());
    }

    #[test]
    fn storage_failure_maps_to_storage_unavailable() {
        let r = profiles_result(Err(unavailable()));
        assert_eq!(r.error, Some(EditorErrorCode::StorageUnavailable));
    }
}
