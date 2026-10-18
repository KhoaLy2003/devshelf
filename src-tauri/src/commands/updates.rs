use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::updates::{auto_check_enabled, set_auto_check};

use super::storage::StorageState;

/// Wire form of update failures; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateErrorCode {
    StorageUnavailable,
    CheckFailed,
    InstallFailed,
    NothingPending,
}

/// The update found by the last check, kept in Rust so the install step never trusts the webview.
pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[derive(Debug, Deserialize)]
pub struct UpdatePrefsGetArgs {}

#[derive(Debug, Deserialize)]
pub struct UpdatePrefsSetArgs {
    pub auto_check: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdatePrefsResult {
    pub auto_check: bool,
    pub error: Option<UpdateErrorCode>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCheckArgs {}

#[derive(Debug, Deserialize)]
pub struct UpdateInstallArgs {}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateView {
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateCheckResult {
    pub update: Option<UpdateView>,
    pub error: Option<UpdateErrorCode>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateInstallResult {
    pub error: Option<UpdateErrorCode>,
}

/// Failure reading or writing the preference. Falls back to the default (`true`) so the
/// launch check keeps its documented behaviour.
fn unavailable() -> UpdatePrefsResult {
    UpdatePrefsResult {
        auto_check: true,
        error: Some(UpdateErrorCode::StorageUnavailable),
    }
}

/// Reads the preference, or writes `write` first when given, then returns the stored value.
fn prefs_from(state: &StorageState, write: Option<bool>) -> UpdatePrefsResult {
    let Ok(lock) = state.result.as_ref() else {
        return unavailable();
    };
    let Ok(storage) = lock.lock() else {
        return unavailable();
    };
    let conn = storage.connection();
    if let Some(enabled) = write {
        if set_auto_check(conn, enabled).is_err() {
            return unavailable();
        }
    }
    match auto_check_enabled(conn) {
        Ok(auto_check) => UpdatePrefsResult {
            auto_check,
            error: None,
        },
        Err(_) => unavailable(),
    }
}

#[tauri::command]
pub fn update_prefs_get(
    _args: UpdatePrefsGetArgs,
    state: State<'_, StorageState>,
) -> UpdatePrefsResult {
    prefs_from(&state, None)
}

#[tauri::command]
pub fn update_prefs_set(
    args: UpdatePrefsSetArgs,
    state: State<'_, StorageState>,
) -> UpdatePrefsResult {
    prefs_from(&state, Some(args.auto_check))
}

/// Async so the network request does not run on the main thread.
#[tauri::command]
pub async fn update_check(_args: UpdateCheckArgs, app: AppHandle) -> UpdateCheckResult {
    let checked = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(err) => Err(err),
    };
    let pending = app.state::<PendingUpdate>();
    match checked {
        Ok(Some(update)) => {
            let view = UpdateView {
                version: update.version.clone(),
                notes: update.body.clone(),
            };
            *pending.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(update);
            UpdateCheckResult {
                update: Some(view),
                error: None,
            }
        }
        Ok(None) => {
            *pending.0.lock().unwrap_or_else(|e| e.into_inner()) = None;
            UpdateCheckResult {
                update: None,
                error: None,
            }
        }
        Err(_) => UpdateCheckResult {
            update: None,
            error: Some(UpdateErrorCode::CheckFailed),
        },
    }
}

/// Installs only the update the last successful check stored. The Windows installer
/// closes the app and relaunches it, so a successful return is rare.
#[tauri::command]
pub async fn update_install(_args: UpdateInstallArgs, app: AppHandle) -> UpdateInstallResult {
    let pending = app.state::<PendingUpdate>();
    let taken = pending.0.lock().unwrap_or_else(|e| e.into_inner()).take();
    let Some(update) = taken else {
        return UpdateInstallResult {
            error: Some(UpdateErrorCode::NothingPending),
        };
    };
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => UpdateInstallResult { error: None },
        Err(_) => UpdateInstallResult {
            error: Some(UpdateErrorCode::InstallFailed),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::StorageError;

    fn wire(code: UpdateErrorCode) -> String {
        serde_json::to_value(code).unwrap().as_str().unwrap().to_string()
    }

    #[test]
    fn error_codes_serialize_to_contract_strings() {
        assert_eq!(wire(UpdateErrorCode::StorageUnavailable), "storage_unavailable");
        assert_eq!(wire(UpdateErrorCode::CheckFailed), "check_failed");
        assert_eq!(wire(UpdateErrorCode::InstallFailed), "install_failed");
        assert_eq!(wire(UpdateErrorCode::NothingPending), "nothing_pending");
    }

    #[test]
    fn unavailable_storage_reports_error_and_keeps_default_on() {
        let state = StorageState {
            folder: std::path::PathBuf::new(),
            result: Err(StorageError::Unavailable),
        };
        assert_eq!(
            prefs_from(&state, None),
            UpdatePrefsResult {
                auto_check: true,
                error: Some(UpdateErrorCode::StorageUnavailable),
            }
        );
        assert_eq!(prefs_from(&state, Some(false)).error, Some(UpdateErrorCode::StorageUnavailable));
    }

    #[test]
    fn set_then_get_returns_stored_value() {
        let tmp = tempfile::tempdir().unwrap();
        let state = StorageState {
            folder: tmp.path().to_path_buf(),
            result: Ok(Mutex::new(crate::db::open_in(tmp.path()).unwrap())),
        };
        assert_eq!(
            prefs_from(&state, Some(false)),
            UpdatePrefsResult {
                auto_check: false,
                error: None,
            }
        );
        assert!(!prefs_from(&state, None).auto_check);
    }
}
