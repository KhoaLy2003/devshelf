use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::secrets::{
    self, Clipboard, SecretCode, SecretDraft, SecretError, SecretFilter, SecretSummary, SecretType,
};
use crate::vault::VaultSession;

/// Managed Tauri state holding the clipboard: `SystemClipboard` in production.
pub struct SecretClipboard(pub Box<dyn Clipboard>);

/// Wire form of `secrets::SecretCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretErrorCode {
    SecretsLabelRequired,
    SecretsValueRequired,
    SecretsTypeRequired,
    SecretsEnvironmentRequired,
    SecretsLabelTooLong,
    SecretsValueTooLong,
    SecretsDuplicateLabel,
    SecretsNotFound,
    SecretsUnreadable,
    SecretsLocked,
    SecretsStorageUnavailable,
    SecretsClipboardUnavailable,
}

impl From<SecretCode> for SecretErrorCode {
    fn from(code: SecretCode) -> Self {
        match code {
            SecretCode::LabelRequired => Self::SecretsLabelRequired,
            SecretCode::ValueRequired => Self::SecretsValueRequired,
            SecretCode::TypeRequired => Self::SecretsTypeRequired,
            SecretCode::EnvironmentRequired => Self::SecretsEnvironmentRequired,
            SecretCode::LabelTooLong => Self::SecretsLabelTooLong,
            SecretCode::ValueTooLong => Self::SecretsValueTooLong,
            SecretCode::DuplicateLabel => Self::SecretsDuplicateLabel,
            SecretCode::NotFound => Self::SecretsNotFound,
            SecretCode::Unreadable => Self::SecretsUnreadable,
            SecretCode::Locked => Self::SecretsLocked,
            SecretCode::StorageUnavailable => Self::SecretsStorageUnavailable,
            SecretCode::ClipboardUnavailable => Self::SecretsClipboardUnavailable,
        }
    }
}

/// A secret without its value. `secret_type` and `environment` carry the contract strings.
#[derive(Debug, Serialize)]
pub struct SecretSummaryView {
    pub id: i64,
    pub label: String,
    pub secret_type: String,
    pub project_id: Option<i64>,
    pub project_missing: bool,
    pub environment: String,
    pub updated_at: String,
}

impl From<SecretSummary> for SecretSummaryView {
    fn from(s: SecretSummary) -> Self {
        Self {
            id: s.id,
            label: s.label,
            secret_type: s.secret_type.as_str().to_string(),
            project_id: s.project_id,
            project_missing: s.project_missing,
            environment: s.environment.as_str().to_string(),
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SecretListArgs {
    pub project_id: Option<i64>,
    /// When false, `project_id` is ignored and every project matches.
    pub filter_by_project: bool,
    pub secret_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SecretListResult {
    pub items: Vec<SecretSummaryView>,
    pub error: Option<SecretErrorCode>,
    pub detail: Option<String>,
}

#[derive(Deserialize)]
pub struct SecretCreateArgs {
    pub label: String,
    pub secret_type: String,
    pub project_id: Option<i64>,
    pub environment: String,
    pub value: String,
}

impl std::fmt::Debug for SecretCreateArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretCreateArgs")
            .field("label", &self.label)
            .field("value", &format_args!("<redacted>"))
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
pub struct SecretUpdateArgs {
    pub id: i64,
    pub label: String,
    pub secret_type: String,
    pub project_id: Option<i64>,
    pub environment: String,
    /// `None` keeps the stored value.
    pub value: Option<String>,
}

impl std::fmt::Debug for SecretUpdateArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretUpdateArgs")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("value", &format_args!("<redacted>"))
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Serialize)]
pub struct SecretResult {
    pub summary: Option<SecretSummaryView>,
    pub error: Option<SecretErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SecretIdArgs {
    pub id: i64,
}

#[derive(Serialize)]
pub struct SecretRevealResult {
    pub value: Option<String>,
    pub error: Option<SecretErrorCode>,
    pub detail: Option<String>,
}

impl std::fmt::Debug for SecretRevealResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretRevealResult")
            .field("value", &format_args!("<redacted>"))
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Serialize)]
pub struct SecretCopyResult {
    pub error: Option<SecretErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SecretDeleteResult {
    pub error: Option<SecretErrorCode>,
    pub detail: Option<String>,
}

fn unavailable() -> SecretError {
    SecretError {
        code: SecretCode::StorageUnavailable,
    }
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
/// `f` MUST be the secrets domain call only.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, SecretError>,
) -> Result<T, SecretError> {
    let mutex = state.result.as_ref().map_err(|_| unavailable())?;
    let guard = mutex.lock().map_err(|_| unavailable())?;
    f(&guard)
}

fn list_result(r: Result<Vec<SecretSummary>, SecretError>) -> SecretListResult {
    match r {
        Ok(list) => SecretListResult {
            items: list.into_iter().map(Into::into).collect(),
            error: None,
            detail: None,
        },
        Err(e) => SecretListResult {
            items: Vec::new(),
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

fn summary_result(r: Result<SecretSummary, SecretError>) -> SecretResult {
    match r {
        Ok(s) => SecretResult {
            summary: Some(s.into()),
            error: None,
            detail: None,
        },
        Err(e) => SecretResult {
            summary: None,
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

fn unit_error(r: Result<(), SecretError>) -> Option<SecretErrorCode> {
    r.err().map(|e| e.code.into())
}

/// An empty or unknown type filter is ignored the same way as no filter.
fn filter_from(args: &SecretListArgs) -> SecretFilter {
    SecretFilter {
        project: args.filter_by_project.then_some(args.project_id),
        secret_type: args.secret_type.as_deref().and_then(SecretType::parse),
    }
}

#[tauri::command(async)]
pub fn secrets_list(
    args: SecretListArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> SecretListResult {
    let filter = filter_from(&args);
    list_result(with_storage(&state, |s| secrets::list(s, &session, filter)))
}

#[tauri::command(async)]
pub fn secrets_create(
    args: SecretCreateArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> SecretResult {
    let draft = SecretDraft {
        label: &args.label,
        secret_type: &args.secret_type,
        project_id: args.project_id,
        environment: &args.environment,
        value: Some(&args.value),
    };
    summary_result(with_storage(&state, |s| secrets::create(s, &session, &draft)))
}

#[tauri::command(async)]
pub fn secrets_update(
    args: SecretUpdateArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> SecretResult {
    let draft = SecretDraft {
        label: &args.label,
        secret_type: &args.secret_type,
        project_id: args.project_id,
        environment: &args.environment,
        value: args.value.as_deref(),
    };
    summary_result(with_storage(&state, |s| {
        secrets::update(s, &session, args.id, &draft)
    }))
}

#[tauri::command(async)]
pub fn secrets_reveal(
    args: SecretIdArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> SecretRevealResult {
    match with_storage(&state, |s| secrets::reveal(s, &session, args.id)) {
        Ok(value) => SecretRevealResult {
            value: Some(value.to_string()),
            error: None,
            detail: None,
        },
        Err(e) => SecretRevealResult {
            value: None,
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

#[tauri::command(async)]
pub fn secrets_copy(
    args: SecretIdArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
    clipboard: State<'_, SecretClipboard>,
) -> SecretCopyResult {
    let r = with_storage(&state, |s| {
        secrets::copy(s, &session, clipboard.0.as_ref(), args.id)
    });
    SecretCopyResult {
        error: unit_error(r),
        detail: None,
    }
}

#[tauri::command(async)]
pub fn secrets_delete(
    args: SecretIdArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> SecretDeleteResult {
    let r = with_storage(&state, |s| secrets::delete(s, &session, args.id));
    SecretDeleteResult {
        error: unit_error(r),
        detail: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_serialize_to_the_contract_strings() {
        let all = [
            (SecretCode::LabelRequired, "secrets_label_required"),
            (SecretCode::ValueRequired, "secrets_value_required"),
            (SecretCode::TypeRequired, "secrets_type_required"),
            (SecretCode::EnvironmentRequired, "secrets_environment_required"),
            (SecretCode::LabelTooLong, "secrets_label_too_long"),
            (SecretCode::ValueTooLong, "secrets_value_too_long"),
            (SecretCode::DuplicateLabel, "secrets_duplicate_label"),
            (SecretCode::NotFound, "secrets_not_found"),
            (SecretCode::Unreadable, "secrets_unreadable"),
            (SecretCode::Locked, "secrets_locked"),
            (SecretCode::StorageUnavailable, "secrets_storage_unavailable"),
            (SecretCode::ClipboardUnavailable, "secrets_clipboard_unavailable"),
        ];
        for (code, text) in all {
            let json = serde_json::to_string(&SecretErrorCode::from(code)).unwrap();
            assert_eq!(json, format!("\"{text}\""));
        }
    }

    #[test]
    fn list_filter_maps_args() {
        let a = SecretListArgs {
            project_id: Some(3),
            filter_by_project: false,
            secret_type: None,
        };
        assert_eq!(filter_from(&a).project, None);
        let a = SecretListArgs {
            project_id: None,
            filter_by_project: true,
            secret_type: Some("token".into()),
        };
        let f = filter_from(&a);
        assert_eq!(f.project, Some(None));
        assert_eq!(f.secret_type, Some(SecretType::Token));
    }

    #[test]
    fn results_serialize_with_contract_fields_and_no_value_on_copy() {
        let copy = serde_json::to_value(SecretCopyResult { error: None, detail: None }).unwrap();
        assert_eq!(copy, serde_json::json!({ "error": null, "detail": null }));
        let reveal = SecretRevealResult {
            value: Some("hunter2".into()),
            error: None,
            detail: None,
        };
        assert!(!format!("{reveal:?}").contains("hunter2"));
        let update: SecretUpdateArgs = serde_json::from_str(
            r#"{"id":1,"label":"a","secret_type":"token","project_id":null,"environment":"dev","value":null}"#,
        )
        .unwrap();
        assert!(update.value.is_none());
    }
}
