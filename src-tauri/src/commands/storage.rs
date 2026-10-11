use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{self, Storage, StorageError};

#[derive(Debug, Deserialize)]
pub struct StorageStatusArgs {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageErrorCode {
    StorageUnavailable,
    SchemaTooNew,
}

#[derive(Debug, Serialize)]
pub struct StorageStatus {
    pub ok: bool,
    pub folder: String,
    pub error_code: Option<StorageErrorCode>,
}

/// Managed Tauri state, created once in `setup`.
/// The connection is wrapped in a `Mutex` because `Storage` is `Send` but not `Sync`.
pub struct StorageState {
    pub folder: PathBuf,
    pub result: Result<Mutex<Storage>, StorageError>,
}

impl StorageState {
    pub fn open(folder: PathBuf) -> Self {
        let result = db::open_in(&folder).map(Mutex::new);
        StorageState { folder, result }
    }
}

fn status_from_state(state: &StorageState) -> StorageStatus {
    let error_code = match &state.result {
        Ok(_) => None,
        Err(StorageError::Unavailable) => Some(StorageErrorCode::StorageUnavailable),
        Err(StorageError::SchemaTooNew) => Some(StorageErrorCode::SchemaTooNew),
    };
    StorageStatus {
        ok: error_code.is_none(),
        folder: state.folder.to_string_lossy().into_owned(),
        error_code,
    }
}

#[tauri::command]
pub fn storage_status(_args: StorageStatusArgs, state: State<'_, StorageState>) -> StorageStatus {
    status_from_state(&state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_status_maps_ready_state() {
        let dir = tempfile::tempdir().unwrap();
        let state = StorageState::open(dir.path().to_path_buf());
        let status = status_from_state(&state);
        assert!(status.ok);
        assert_eq!(status.error_code, None);
        assert_eq!(status.folder, dir.path().to_string_lossy());
    }

    #[test]
    fn storage_status_maps_unavailable() {
        let state = StorageState {
            folder: PathBuf::from("x"),
            result: Err(StorageError::Unavailable),
        };
        let status = status_from_state(&state);
        assert!(!status.ok);
        assert_eq!(status.error_code, Some(StorageErrorCode::StorageUnavailable));
    }

    #[test]
    fn storage_status_maps_schema_too_new() {
        let state = StorageState {
            folder: PathBuf::from("x"),
            result: Err(StorageError::SchemaTooNew),
        };
        let status = status_from_state(&state);
        assert!(!status.ok);
        assert_eq!(status.error_code, Some(StorageErrorCode::SchemaTooNew));
    }

    #[test]
    fn storage_error_code_serialises_snake_case() {
        assert_eq!(
            serde_json::to_string(&StorageErrorCode::StorageUnavailable).unwrap(),
            "\"storage_unavailable\""
        );
        assert_eq!(
            serde_json::to_string(&StorageErrorCode::SchemaTooNew).unwrap(),
            "\"schema_too_new\""
        );
    }
}
