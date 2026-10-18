use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroize;

use crate::commands::notes::NoteKeys;
use crate::commands::storage::StorageState;
use crate::export::{self, ExportCode};
use crate::notes;
use crate::vault::{self, VaultCode, VaultError};

/// Wire form of `export::ExportCode` and the export gate errors; the strings are the contract
/// with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportErrorCode {
    ExportAckRequired,
    ExportPasswordRequired,
    ExportWrongPassword,
    ExportSecretUnreadable,
    ExportNoteUnreadable,
    ExportNoteKeyMissing,
    ExportKeyringUnavailable,
    ExportWriteFailed,
    ExportStorageUnavailable,
}

impl From<ExportCode> for ExportErrorCode {
    fn from(code: ExportCode) -> Self {
        match code {
            ExportCode::NoteKeyMissing => Self::ExportNoteKeyMissing,
            ExportCode::KeyringUnavailable => Self::ExportKeyringUnavailable,
            ExportCode::NoteUnreadable => Self::ExportNoteUnreadable,
            ExportCode::SecretUnreadable => Self::ExportSecretUnreadable,
            ExportCode::WriteFailed => Self::ExportWriteFailed,
            ExportCode::StorageUnavailable => Self::ExportStorageUnavailable,
        }
    }
}

fn vault_error(e: VaultError) -> ExportErrorCode {
    match e.code {
        VaultCode::PasswordRequired => ExportErrorCode::ExportPasswordRequired,
        VaultCode::WrongPassword => ExportErrorCode::ExportWrongPassword,
        _ => ExportErrorCode::ExportStorageUnavailable,
    }
}

#[derive(Deserialize)]
pub struct DataExportArgs {
    pub password: String,
    pub acknowledged: bool,
}

impl std::fmt::Debug for DataExportArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataExportArgs")
            .field("password", &format_args!("<redacted>"))
            .field("acknowledged", &self.acknowledged)
            .finish()
    }
}

#[derive(Debug, Serialize)]
pub struct ExportFileView {
    pub name: String,
    pub rows: u32,
}

#[derive(Debug, Serialize)]
pub struct DataExportResult {
    pub cancelled: bool,
    pub folder: Option<String>,
    pub files: Vec<ExportFileView>,
    pub error: Option<ExportErrorCode>,
}

impl DataExportResult {
    fn failed(code: ExportErrorCode) -> Self {
        Self {
            cancelled: false,
            folder: None,
            files: Vec::new(),
            error: Some(code),
        }
    }
}

/// Exports every table as plaintext CSV into a folder the user picks. Order: acknowledgement,
/// master password (before the dialog, so a wrong password never shows it), key load, folder
/// dialog, then the storage lock for reads only, then the write. Runs off the main thread because
/// of the blocking dialog and the password KDF.
#[tauri::command(async)]
pub fn data_export(
    mut args: DataExportArgs,
    app: AppHandle,
    state: State<'_, StorageState>,
    keys: State<'_, NoteKeys>,
) -> DataExportResult {
    let outcome = run(&args, &app, &state, &keys);
    args.password.zeroize();
    match outcome {
        Ok(result) => result,
        Err(code) => DataExportResult::failed(code),
    }
}

fn run(
    args: &DataExportArgs,
    app: &AppHandle,
    state: &StorageState,
    keys: &NoteKeys,
) -> Result<DataExportResult, ExportErrorCode> {
    if !args.acknowledged {
        return Err(ExportErrorCode::ExportAckRequired);
    }
    let storage = state
        .result
        .as_ref()
        .map_err(|_| ExportErrorCode::ExportStorageUnavailable)?;
    let vault_key = vault::verified_key(storage, &args.password).map_err(vault_error)?;
    let note_key = notes::load_key(keys.0.as_ref())
        .map_err(|_| ExportErrorCode::ExportKeyringUnavailable)?;

    let picked = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok());
    let Some(parent) = picked else {
        return Ok(DataExportResult {
            cancelled: true,
            folder: None,
            files: Vec::new(),
            error: None,
        });
    };

    let dataset = {
        let guard = storage
            .lock()
            .map_err(|_| ExportErrorCode::ExportStorageUnavailable)?;
        export::collect(&guard, keys.0.as_ref(), note_key, vault_key.as_deref())
            .map_err(|e| ExportErrorCode::from(e.code))?
    };
    let (dir, files) =
        export::write(&dataset, &parent).map_err(|e| ExportErrorCode::from(e.code))?;
    Ok(DataExportResult {
        cancelled: false,
        folder: Some(dir.to_string_lossy().into_owned()),
        files: files
            .into_iter()
            .map(|file| ExportFileView {
                name: file.name.to_string(),
                rows: u32::try_from(file.rows).unwrap_or(u32::MAX),
            })
            .collect(),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_error_codes_serialize_to_contract_strings() {
        let cases = [
            (ExportErrorCode::ExportAckRequired, "export_ack_required"),
            (ExportErrorCode::ExportWrongPassword, "export_wrong_password"),
            (ExportErrorCode::ExportNoteKeyMissing, "export_note_key_missing"),
            (ExportErrorCode::ExportWriteFailed, "export_write_failed"),
        ];
        for (code, text) in cases {
            assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{text}\""));
        }
    }

    #[test]
    fn args_debug_never_shows_the_password() {
        let args = DataExportArgs {
            password: "hunter2-master".to_string(),
            acknowledged: true,
        };
        let shown = format!("{args:?}");
        assert!(!shown.contains("hunter2-master"));
        assert!(shown.contains("<redacted>"));
    }

    #[test]
    fn wrong_password_maps_to_its_own_code() {
        let err = VaultError {
            code: VaultCode::WrongPassword,
        };
        assert_eq!(vault_error(err), ExportErrorCode::ExportWrongPassword);
    }
}
