use serde::{Deserialize, Serialize};
use tauri::State;
use zeroize::Zeroize;

use crate::commands::storage::StorageState;
use crate::vault::{
    self, LockReason, VaultCode, VaultError, VaultSession, VaultState, VaultStatus,
};

/// Wire form of `vault::VaultCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultErrorCode {
    VaultPasswordRequired,
    VaultPasswordMismatch,
    VaultPasswordTooShort,
    VaultWrongPassword,
    VaultNotSetUp,
    VaultAlreadySetUp,
    VaultInvalidTimeout,
    VaultStorageUnavailable,
}

impl From<VaultCode> for VaultErrorCode {
    fn from(code: VaultCode) -> Self {
        match code {
            VaultCode::PasswordRequired => Self::VaultPasswordRequired,
            VaultCode::PasswordMismatch => Self::VaultPasswordMismatch,
            VaultCode::PasswordTooShort => Self::VaultPasswordTooShort,
            VaultCode::WrongPassword => Self::VaultWrongPassword,
            VaultCode::NotSetUp => Self::VaultNotSetUp,
            VaultCode::AlreadySetUp => Self::VaultAlreadySetUp,
            VaultCode::InvalidTimeout => Self::VaultInvalidTimeout,
            VaultCode::StorageUnavailable => Self::VaultStorageUnavailable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultStateView {
    NotSetUp,
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultLockReasonView {
    Launch,
    Manual,
    Inactivity,
}

#[derive(Debug, Serialize)]
pub struct VaultStatusView {
    pub state: VaultStateView,
    pub lock_reason: Option<VaultLockReasonView>,
    pub auto_lock_minutes: u8,
    pub seconds_until_lock: Option<u64>,
}

impl From<VaultStatus> for VaultStatusView {
    fn from(s: VaultStatus) -> Self {
        Self {
            state: match s.state {
                VaultState::NotSetUp => VaultStateView::NotSetUp,
                VaultState::Locked => VaultStateView::Locked,
                VaultState::Unlocked => VaultStateView::Unlocked,
            },
            lock_reason: s.lock_reason.map(|r| match r {
                LockReason::Launch => VaultLockReasonView::Launch,
                LockReason::Manual => VaultLockReasonView::Manual,
                LockReason::Inactivity => VaultLockReasonView::Inactivity,
            }),
            auto_lock_minutes: s.auto_lock_minutes,
            seconds_until_lock: s.seconds_until_lock,
        }
    }
}

/// Shared result of every vault command.
#[derive(Debug, Serialize)]
pub struct VaultStatusResult {
    pub status: Option<VaultStatusView>,
    pub error: Option<VaultErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VaultStatusArgs {}

#[derive(Deserialize)]
pub struct VaultSetupArgs {
    pub password: String,
    pub confirmation: String,
    pub acknowledged: bool,
}

impl std::fmt::Debug for VaultSetupArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultSetupArgs")
            .field("password", &format_args!("<redacted>"))
            .field("acknowledged", &self.acknowledged)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
pub struct VaultUnlockArgs {
    pub password: String,
}

impl std::fmt::Debug for VaultUnlockArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultUnlockArgs")
            .field("password", &format_args!("<redacted>"))
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct VaultLockArgs {}

#[derive(Debug, Deserialize)]
pub struct VaultTouchArgs {}

#[derive(Debug, Deserialize)]
pub struct VaultSetAutoLockArgs {
    pub minutes: u8,
}

fn to_result(r: Result<VaultStatus, VaultError>) -> VaultStatusResult {
    match r {
        Ok(status) => VaultStatusResult {
            status: Some(status.into()),
            error: None,
            detail: None,
        },
        Err(e) => VaultStatusResult {
            status: None,
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

/// Runs a domain call against the vault storage. Failed storage is `StorageUnavailable`.
fn run(
    state: &StorageState,
    f: impl FnOnce(&std::sync::Mutex<crate::db::Storage>) -> Result<VaultStatus, VaultError>,
) -> VaultStatusResult {
    match state.result.as_ref() {
        Ok(storage) => to_result(f(storage)),
        Err(_) => to_result(Err(VaultError {
            code: VaultCode::StorageUnavailable,
        })),
    }
}

#[tauri::command(async)]
pub fn vault_status(
    _args: VaultStatusArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    run(&state, |s| vault::status(s, &session))
}

#[tauri::command(async)]
pub fn vault_setup(
    mut args: VaultSetupArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    let result = run(&state, |s| {
        vault::setup(s, &session, &args.password, &args.confirmation, args.acknowledged)
    });
    args.password.zeroize();
    args.confirmation.zeroize();
    result
}

#[tauri::command(async)]
pub fn vault_unlock(
    mut args: VaultUnlockArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    let result = run(&state, |s| vault::unlock(s, &session, &args.password));
    args.password.zeroize();
    result
}

#[tauri::command(async)]
pub fn vault_lock(
    _args: VaultLockArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    run(&state, |s| vault::lock(s, &session))
}

#[tauri::command(async)]
pub fn vault_touch(
    _args: VaultTouchArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    run(&state, |s| vault::touch(s, &session))
}

#[tauri::command(async)]
pub fn vault_set_auto_lock(
    args: VaultSetAutoLockArgs,
    state: State<'_, StorageState>,
    session: State<'_, VaultSession>,
) -> VaultStatusResult {
    run(&state, |s| vault::set_auto_lock(s, &session, args.minutes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::StorageError;
    use std::path::PathBuf;

    fn json<T: Serialize>(v: T) -> String {
        serde_json::to_string(&v).unwrap()
    }

    #[test]
    fn every_vault_code_maps_to_its_snake_case_string() {
        let all = [
            (VaultCode::PasswordRequired, "vault_password_required"),
            (VaultCode::PasswordMismatch, "vault_password_mismatch"),
            (VaultCode::PasswordTooShort, "vault_password_too_short"),
            (VaultCode::WrongPassword, "vault_wrong_password"),
            (VaultCode::NotSetUp, "vault_not_set_up"),
            (VaultCode::AlreadySetUp, "vault_already_set_up"),
            (VaultCode::InvalidTimeout, "vault_invalid_timeout"),
            (VaultCode::StorageUnavailable, "vault_storage_unavailable"),
        ];
        for (code, text) in all {
            assert_eq!(json(VaultErrorCode::from(code)), format!("\"{text}\""));
        }
    }

    #[test]
    fn status_result_serialises_to_the_ts_shape() {
        let ok = to_result(Ok(VaultStatus {
            state: VaultState::Locked,
            lock_reason: Some(LockReason::Inactivity),
            auto_lock_minutes: 15,
            seconds_until_lock: None,
        }));
        assert_eq!(
            json(&ok),
            r#"{"status":{"state":"locked","lock_reason":"inactivity","auto_lock_minutes":15,"seconds_until_lock":null},"error":null,"detail":null}"#
        );
        let bad = to_result(Err(VaultError {
            code: VaultCode::WrongPassword,
        }));
        assert_eq!(
            json(&bad),
            r#"{"status":null,"error":"vault_wrong_password","detail":null}"#
        );
    }

    #[test]
    fn failed_storage_is_vault_storage_unavailable() {
        let state = StorageState {
            folder: PathBuf::from("x"),
            result: Err(StorageError::Unavailable),
        };
        let r = run(&state, |_| unreachable!());
        assert_eq!(r.error, Some(VaultErrorCode::VaultStorageUnavailable));
        assert!(r.status.is_none());
    }

    #[test]
    fn args_debug_output_hides_the_password() {
        let setup = VaultSetupArgs {
            password: "hunter2hunter2".into(),
            confirmation: "hunter2hunter2".into(),
            acknowledged: true,
        };
        let unlock = VaultUnlockArgs {
            password: "hunter2hunter2".into(),
        };
        assert!(!format!("{setup:?}").contains("hunter2"));
        assert!(!format!("{unlock:?}").contains("hunter2"));
    }
}
