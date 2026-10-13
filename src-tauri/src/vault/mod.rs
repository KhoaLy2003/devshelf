//! Vault access control: a master-password gate in front of secret storage. This module owns the
//! `vault_meta` table (salt, cost parameters, verifier, auto-lock setting) and the in-memory
//! session. It stores no secrets itself. The password and derived key never leave Rust and are
//! never logged; no error or Debug output carries them.

mod kdf;
mod session;
mod verifier;

use std::sync::Mutex;

use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use zeroize::Zeroizing;

use crate::db::Storage;

use kdf::KdfParams;
pub use session::{LockReason, VaultSession, DEFAULT_AUTO_LOCK_MINUTES};

pub const MIN_PASSWORD_CHARS: usize = 12;
const ALLOWED_TIMEOUTS: [u8; 4] = [0, 1, 5, 15];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultCode {
    PasswordRequired,
    PasswordMismatch,
    PasswordTooShort,
    WrongPassword,
    NotSetUp,
    AlreadySetUp,
    InvalidTimeout,
    StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultError {
    pub code: VaultCode,
}

impl VaultError {
    fn new(code: VaultCode) -> Self {
        Self { code }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultState {
    NotSetUp,
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultStatus {
    pub state: VaultState,
    /// Set only while locked.
    pub lock_reason: Option<LockReason>,
    /// 0 means Never.
    pub auto_lock_minutes: u8,
    /// `Some(1..=60)` only while unlocked, auto-lock is on and at most 60 s remain.
    pub seconds_until_lock: Option<u64>,
}

struct Meta {
    salt: Vec<u8>,
    kdf: KdfParams,
    verifier: Vec<u8>,
    auto_lock_minutes: u8,
}

fn unavailable() -> VaultError {
    VaultError::new(VaultCode::StorageUnavailable)
}

/// Locks the storage and runs `f` on its connection. A poisoned lock or SQL failure is
/// `StorageUnavailable`. The lock is never held across key derivation.
fn with_conn<T>(
    storage: &Mutex<Storage>,
    f: impl FnOnce(&Connection) -> rusqlite::Result<T>,
) -> Result<T, VaultError> {
    let guard = storage.lock().map_err(|_| unavailable())?;
    f(guard.connection()).map_err(|_| unavailable())
}

fn read_meta(storage: &Mutex<Storage>) -> Result<Option<Meta>, VaultError> {
    with_conn(storage, |conn| {
        conn.query_row(
            "SELECT salt, m_cost, t_cost, p_cost, verifier, auto_lock_minutes FROM vault_meta WHERE id = 1",
            [],
            |r| {
                let cost = |i: usize| -> rusqlite::Result<u32> {
                    // Out-of-range values become 0, which `KdfParams::is_acceptable` rejects.
                    Ok(u32::try_from(r.get::<_, i64>(i)?).unwrap_or(0))
                };
                Ok(Meta {
                    salt: r.get(0)?,
                    kdf: KdfParams {
                        m_cost: cost(1)?,
                        t_cost: cost(2)?,
                        p_cost: cost(3)?,
                    },
                    verifier: r.get(4)?,
                    auto_lock_minutes: u8::try_from(r.get::<_, i64>(5)?)
                        .unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
                })
            },
        )
        .optional()
    })
}

fn status_from(meta: Option<&Meta>, session: &VaultSession) -> VaultStatus {
    let Some(meta) = meta else {
        return VaultStatus {
            state: VaultState::NotSetUp,
            lock_reason: None,
            auto_lock_minutes: DEFAULT_AUTO_LOCK_MINUTES,
            seconds_until_lock: None,
        };
    };
    let snap = session.snapshot();
    if snap.unlocked {
        VaultStatus {
            state: VaultState::Unlocked,
            lock_reason: None,
            auto_lock_minutes: meta.auto_lock_minutes,
            seconds_until_lock: snap.seconds_until_lock,
        }
    } else {
        VaultStatus {
            state: VaultState::Locked,
            lock_reason: Some(snap.reason),
            auto_lock_minutes: meta.auto_lock_minutes,
            seconds_until_lock: None,
        }
    }
}

fn current_status(
    storage: &Mutex<Storage>,
    session: &VaultSession,
) -> Result<VaultStatus, VaultError> {
    let meta = read_meta(storage)?;
    Ok(status_from(meta.as_ref(), session))
}

pub fn status(storage: &Mutex<Storage>, session: &VaultSession) -> Result<VaultStatus, VaultError> {
    current_status(storage, session)
}

/// Creates the vault, unlocked. Order of checks: required, mismatch, length, acknowledgement,
/// already set up. `acknowledged = false` has no code of its own in the contract and is reported
/// as `PasswordRequired`; the UI never submits without the checkbox.
pub fn setup(
    storage: &Mutex<Storage>,
    session: &VaultSession,
    password: &str,
    confirmation: &str,
    acknowledged: bool,
) -> Result<VaultStatus, VaultError> {
    if password.is_empty() {
        return Err(VaultError::new(VaultCode::PasswordRequired));
    }
    if password != confirmation {
        return Err(VaultError::new(VaultCode::PasswordMismatch));
    }
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(VaultError::new(VaultCode::PasswordTooShort));
    }
    if !acknowledged {
        return Err(VaultError::new(VaultCode::PasswordRequired));
    }
    if read_meta(storage)?.is_some() {
        return Err(VaultError::new(VaultCode::AlreadySetUp));
    }

    let mut salt = [0u8; kdf::SALT_LEN];
    getrandom::fill(&mut salt).map_err(|_| unavailable())?;
    let cost = kdf::setup_params();
    let key = kdf::derive(password.as_bytes(), &salt, cost).map_err(|_| unavailable())?;
    let blob = verifier::seal(&key).map_err(|_| unavailable())?;

    with_conn(storage, |conn| {
        conn.execute(
            "INSERT INTO vault_meta (id, salt, m_cost, t_cost, p_cost, verifier) VALUES (1, ?1, ?2, ?3, ?4, ?5)",
            params![
                salt.as_slice(),
                i64::from(cost.m_cost),
                i64::from(cost.t_cost),
                i64::from(cost.p_cost),
                blob
            ],
        )
        .map(|_| ())
        .or_else(|e| match e.sqlite_error_code() {
            // A concurrent setup won the single-row insert.
            Some(ErrorCode::ConstraintViolation) => Ok(()),
            _ => Err(e),
        })
    })?;
    // The constraint arm above hides which writer won; re-read to decide.
    let meta = read_meta(storage)?.ok_or_else(unavailable)?;
    if meta.salt.as_slice() != salt.as_slice() {
        return Err(VaultError::new(VaultCode::AlreadySetUp));
    }
    session.unlock(key, meta.auto_lock_minutes);
    Ok(status_from(Some(&meta), session))
}

pub fn unlock(
    storage: &Mutex<Storage>,
    session: &VaultSession,
    password: &str,
) -> Result<VaultStatus, VaultError> {
    if password.is_empty() {
        return Err(VaultError::new(VaultCode::PasswordRequired));
    }
    let meta = read_meta(storage)?.ok_or_else(|| VaultError::new(VaultCode::NotSetUp))?;
    let wrong = || VaultError::new(VaultCode::WrongPassword);

    // Bad stored salt or cost is treated like a failed verification (AC-25).
    let key: Zeroizing<[u8; kdf::KEY_LEN]> =
        kdf::derive(password.as_bytes(), &meta.salt, meta.kdf).map_err(|_| wrong())?;
    verifier::open(&key, &meta.verifier).map_err(|_| wrong())?;

    session.unlock(key, meta.auto_lock_minutes);
    Ok(status_from(Some(&meta), session))
}

pub fn lock(storage: &Mutex<Storage>, session: &VaultSession) -> Result<VaultStatus, VaultError> {
    session.lock();
    current_status(storage, session)
}

pub fn touch(storage: &Mutex<Storage>, session: &VaultSession) -> Result<VaultStatus, VaultError> {
    session.touch();
    current_status(storage, session)
}

/// Persists the timeout (0 = Never, 1, 5, 15) and applies it to the running session at once.
pub fn set_auto_lock(
    storage: &Mutex<Storage>,
    session: &VaultSession,
    minutes: u8,
) -> Result<VaultStatus, VaultError> {
    if !ALLOWED_TIMEOUTS.contains(&minutes) {
        return Err(VaultError::new(VaultCode::InvalidTimeout));
    }
    let changed = with_conn(storage, |conn| {
        conn.execute(
            "UPDATE vault_meta SET auto_lock_minutes = ?1 WHERE id = 1",
            params![i64::from(minutes)],
        )
    })?;
    if changed == 0 {
        return Err(VaultError::new(VaultCode::NotSetUp));
    }
    session.set_auto_lock_minutes(minutes);
    current_status(storage, session)
}

#[cfg(test)]
mod tests {
    use super::session::{advance_clock, fake_clock};
    use super::*;
    use crate::db::open_in;
    use std::sync::Arc;
    use std::time::Instant;

    const PW: &str = "correct horse battery";

    struct Fixture {
        dir: tempfile::TempDir,
        storage: Mutex<Storage>,
        session: VaultSession,
        now: Arc<Mutex<Instant>>,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let storage = Mutex::new(open_in(dir.path()).unwrap());
        let (now, clock) = fake_clock();
        Fixture {
            dir,
            storage,
            session: VaultSession::with_clock(clock),
            now,
        }
    }

    fn set_up(f: &Fixture) -> VaultStatus {
        setup(&f.storage, &f.session, PW, PW, true).unwrap()
    }

    fn err<T: std::fmt::Debug>(r: Result<T, VaultError>) -> VaultCode {
        r.unwrap_err().code
    }

    fn exec(f: &Fixture, sql: &str, p: impl rusqlite::Params) {
        f.storage.lock().unwrap().connection().execute(sql, p).unwrap();
    }

    #[test]
    fn fresh_vault_is_not_set_up() {
        let f = fixture();
        let s = status(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::NotSetUp);
        assert_eq!(s.lock_reason, None);
        assert_eq!(s.auto_lock_minutes, 5);
    }

    #[test]
    fn setup_creates_an_unlocked_vault_with_default_timeout() {
        let f = fixture();
        let s = set_up(&f);
        assert_eq!(s.state, VaultState::Unlocked);
        assert_eq!(s.lock_reason, None);
        assert_eq!(s.auto_lock_minutes, 5);
        assert!(f.session.key().is_ok());
    }

    #[test]
    fn setup_then_unlock_with_correct_password_succeeds() {
        let f = fixture();
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        let s = unlock(&f.storage, &f.session, PW).unwrap();
        assert_eq!(s.state, VaultState::Unlocked);
        assert!(f.session.key().is_ok());
    }

    #[test]
    fn wrong_password_fails_and_changes_nothing() {
        let f = fixture();
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        assert_eq!(
            err(unlock(&f.storage, &f.session, "not the password")),
            VaultCode::WrongPassword
        );
        let s = status(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::Locked);
        assert_eq!(s.lock_reason, Some(LockReason::Manual));
    }

    #[test]
    fn tampered_verifier_fails_like_a_wrong_password() {
        let f = fixture();
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        exec(&f, "UPDATE vault_meta SET verifier = ?1", params![vec![0u8; 72]]);
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::WrongPassword);
        // A truncated verifier gives the same code.
        exec(&f, "UPDATE vault_meta SET verifier = X'0102'", []);
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::WrongPassword);
    }

    #[test]
    fn tampered_salt_fails_like_a_wrong_password() {
        let f = fixture();
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        exec(&f, "UPDATE vault_meta SET salt = ?1", params![vec![0xAAu8; 16]]);
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::WrongPassword);
        exec(&f, "UPDATE vault_meta SET salt = X'0102'", []);
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::WrongPassword);
    }

    #[test]
    fn tampered_cost_is_wrong_password_not_a_huge_allocation() {
        let f = fixture();
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        exec(&f, "UPDATE vault_meta SET m_cost = 4000000000", []);
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::WrongPassword);
    }

    #[test]
    fn setup_rejects_empty_mismatched_short_and_unacknowledged() {
        let f = fixture();
        assert_eq!(err(setup(&f.storage, &f.session, "", "", true)), VaultCode::PasswordRequired);
        assert_eq!(
            err(setup(&f.storage, &f.session, PW, "different pass phrase", true)),
            VaultCode::PasswordMismatch
        );
        assert_eq!(
            err(setup(&f.storage, &f.session, "short", "short", true)),
            VaultCode::PasswordTooShort
        );
        assert_eq!(
            err(setup(&f.storage, &f.session, PW, PW, false)),
            VaultCode::PasswordRequired
        );
        // Nothing was created by any rejected attempt.
        assert_eq!(status(&f.storage, &f.session).unwrap().state, VaultState::NotSetUp);
    }

    #[test]
    fn length_rule_counts_characters_and_has_no_composition_rules() {
        let f = fixture();
        let eleven = "ééééééééééé";
        assert_eq!(eleven.chars().count(), 11);
        assert_eq!(
            err(setup(&f.storage, &f.session, eleven, eleven, true)),
            VaultCode::PasswordTooShort
        );
        let spaces = " ".repeat(12);
        assert!(setup(&f.storage, &f.session, &spaces, &spaces, true).is_ok());
    }

    #[test]
    fn repeated_setup_is_rejected_and_keeps_the_first_password() {
        let f = fixture();
        set_up(&f);
        let other = "another long password";
        assert_eq!(
            err(setup(&f.storage, &f.session, other, other, true)),
            VaultCode::AlreadySetUp
        );
        lock(&f.storage, &f.session).unwrap();
        assert!(unlock(&f.storage, &f.session, PW).is_ok());
    }

    #[test]
    fn empty_unlock_password_is_required_and_skips_the_kdf() {
        let f = fixture();
        // Not set up: validation comes before the storage read.
        assert_eq!(err(unlock(&f.storage, &f.session, "")), VaultCode::PasswordRequired);
        set_up(&f);
        lock(&f.storage, &f.session).unwrap();
        // Were the KDF reached, a corrupt cost would turn this into WrongPassword.
        exec(&f, "UPDATE vault_meta SET m_cost = 4000000000", []);
        assert_eq!(err(unlock(&f.storage, &f.session, "")), VaultCode::PasswordRequired);
    }

    #[test]
    fn unlock_without_a_vault_is_not_set_up() {
        let f = fixture();
        assert_eq!(err(unlock(&f.storage, &f.session, PW)), VaultCode::NotSetUp);
    }

    #[test]
    fn lock_drops_the_key() {
        let f = fixture();
        set_up(&f);
        let s = lock(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::Locked);
        assert_eq!(s.lock_reason, Some(LockReason::Manual));
        assert!(f.session.key().is_err());
        // Idempotent.
        assert_eq!(lock(&f.storage, &f.session).unwrap().state, VaultState::Locked);
    }

    #[test]
    fn restart_shows_launch_reason() {
        let f = fixture();
        set_up(&f);
        drop(f.session);
        let fresh = VaultSession::new();
        let s = status(&f.storage, &fresh).unwrap();
        assert_eq!(s.state, VaultState::Locked);
        assert_eq!(s.lock_reason, Some(LockReason::Launch));
    }

    #[test]
    fn idle_past_the_timeout_locks_with_inactivity_and_touch_resets() {
        let f = fixture();
        set_up(&f);
        advance_clock(&f.now, 200);
        touch(&f.storage, &f.session).unwrap();
        advance_clock(&f.now, 200);
        assert_eq!(status(&f.storage, &f.session).unwrap().state, VaultState::Unlocked);
        advance_clock(&f.now, 101);
        let s = status(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::Locked);
        assert_eq!(s.lock_reason, Some(LockReason::Inactivity));
        assert!(f.session.key().is_err());
        // Stays locked; touching does not revive it.
        let s = touch(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::Locked);
    }

    #[test]
    fn never_setting_does_not_lock() {
        let f = fixture();
        set_up(&f);
        set_auto_lock(&f.storage, &f.session, 0).unwrap();
        advance_clock(&f.now, 7 * 24 * 3600);
        let s = status(&f.storage, &f.session).unwrap();
        assert_eq!(s.state, VaultState::Unlocked);
        assert_eq!(s.auto_lock_minutes, 0);
        assert_eq!(s.seconds_until_lock, None);
    }

    #[test]
    fn countdown_is_none_above_60_seconds_and_some_inside_the_window() {
        let f = fixture();
        set_up(&f);
        advance_clock(&f.now, 239);
        assert_eq!(status(&f.storage, &f.session).unwrap().seconds_until_lock, None);
        advance_clock(&f.now, 1);
        let secs = status(&f.storage, &f.session).unwrap().seconds_until_lock;
        assert_eq!(secs, Some(60));
        advance_clock(&f.now, 30);
        let secs = status(&f.storage, &f.session).unwrap().seconds_until_lock.unwrap();
        assert!(secs <= 60 && secs == 30);
        touch(&f.storage, &f.session).unwrap();
        assert_eq!(status(&f.storage, &f.session).unwrap().seconds_until_lock, None);
    }

    #[test]
    fn timeout_persists_across_a_reopened_database() {
        let f = fixture();
        set_up(&f);
        let s = set_auto_lock(&f.storage, &f.session, 15).unwrap();
        assert_eq!(s.auto_lock_minutes, 15);
        let Fixture { dir, storage, .. } = f;
        drop(storage);
        let reopened = Mutex::new(open_in(dir.path()).unwrap());
        let s = status(&reopened, &VaultSession::new()).unwrap();
        assert_eq!(s.auto_lock_minutes, 15);
        // Unlock applies the persisted value to the new session.
        let session = VaultSession::new();
        let s = unlock(&reopened, &session, PW).unwrap();
        assert_eq!(s.auto_lock_minutes, 15);
    }

    #[test]
    fn changing_the_timeout_takes_effect_immediately() {
        let f = fixture();
        set_up(&f);
        advance_clock(&f.now, 120);
        set_auto_lock(&f.storage, &f.session, 1).unwrap();
        assert_eq!(status(&f.storage, &f.session).unwrap().state, VaultState::Locked);
    }

    #[test]
    fn timeout_rejects_values_outside_the_allowed_set() {
        let f = fixture();
        set_up(&f);
        for bad in [2u8, 10, 30, 60, 255] {
            assert_eq!(
                err(set_auto_lock(&f.storage, &f.session, bad)),
                VaultCode::InvalidTimeout
            );
        }
        for ok in [0u8, 1, 5, 15] {
            assert_eq!(set_auto_lock(&f.storage, &f.session, ok).unwrap().auto_lock_minutes, ok);
        }
    }

    #[test]
    fn timeout_without_a_vault_is_not_set_up() {
        let f = fixture();
        assert_eq!(err(set_auto_lock(&f.storage, &f.session, 5)), VaultCode::NotSetUp);
    }

    #[test]
    fn no_password_or_key_bytes_reach_the_database_files() {
        let f = fixture();
        set_up(&f);
        let key = *f.session.key().unwrap();
        let (salt, verifier): (Vec<u8>, Vec<u8>) = f
            .storage
            .lock()
            .unwrap()
            .connection()
            .query_row("SELECT salt, verifier FROM vault_meta", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(salt.len(), 16);
        assert_eq!(verifier.len(), 24 + 32 + 16);
        let contains = |hay: &[u8], needle: &[u8]| hay.windows(needle.len()).any(|w| w == needle);
        assert!(!contains(&verifier, PW.as_bytes()));
        assert!(!contains(&verifier, &key));
        for entry in std::fs::read_dir(f.dir.path()).unwrap() {
            let bytes = std::fs::read(entry.unwrap().path()).unwrap();
            assert!(!contains(&bytes, PW.as_bytes()));
            assert!(!contains(&bytes, &key));
        }
    }
}
