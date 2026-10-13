//! In-memory vault session: the derived key, the input clock, the auto-lock timeout and why the
//! vault is locked. Nothing here is persisted, so a restart resets the reason to `Launch`.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use zeroize::Zeroizing;

use super::kdf::KEY_LEN;

pub const DEFAULT_AUTO_LOCK_MINUTES: u8 = 5;
pub const COUNTDOWN_WINDOW_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockReason {
    Launch,
    Manual,
    Inactivity,
}

/// Monotonic time source; replaced by a fake in tests.
pub type Clock = Box<dyn Fn() -> Instant + Send + Sync>;

struct Inner {
    key: Option<Zeroizing<[u8; KEY_LEN]>>,
    last_input: Instant,
    /// 0 means Never.
    auto_lock_minutes: u8,
    last_reason: LockReason,
}

impl Inner {
    fn timeout(&self) -> Option<Duration> {
        match self.auto_lock_minutes {
            0 => None,
            m => Some(Duration::from_secs(u64::from(m) * 60)),
        }
    }

    /// Locks if unlocked and idle past the timeout. Returns whether it is still unlocked.
    fn expire(&mut self, now: Instant) -> bool {
        if self.key.is_none() {
            return false;
        }
        if let Some(timeout) = self.timeout() {
            if now.saturating_duration_since(self.last_input) > timeout {
                self.key = None;
                self.last_reason = LockReason::Inactivity;
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub unlocked: bool,
    pub reason: LockReason,
    /// `Some(1..=60)` only while unlocked, auto-lock is on and at most 60 s remain.
    pub seconds_until_lock: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotUnlocked;

pub struct VaultSession {
    inner: Mutex<Inner>,
    clock: Clock,
}

impl VaultSession {
    pub fn new() -> Self {
        Self::with_clock(Box::new(Instant::now))
    }

    pub fn with_clock(clock: Clock) -> Self {
        let now = clock();
        Self {
            inner: Mutex::new(Inner {
                key: None,
                last_input: now,
                auto_lock_minutes: DEFAULT_AUTO_LOCK_MINUTES,
                last_reason: LockReason::Launch,
            }),
            clock,
        }
    }

    /// A poisoned lock still holds consistent data (every update is a plain assignment).
    fn guard(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Stores the key, starts the input clock, and applies the persisted timeout.
    pub fn unlock(&self, key: Zeroizing<[u8; KEY_LEN]>, auto_lock_minutes: u8) {
        let now = (self.clock)();
        let mut g = self.guard();
        g.key = Some(key);
        g.auto_lock_minutes = auto_lock_minutes;
        g.last_input = now;
    }

    /// Drops the key (zeroized). Idempotent; the reason is only recorded when it was unlocked.
    pub fn lock(&self) {
        let mut g = self.guard();
        if g.key.take().is_some() {
            g.last_reason = LockReason::Manual;
        }
    }

    /// Resets the inactivity clock. An already expired vault locks instead of resetting.
    pub fn touch(&self) {
        let now = (self.clock)();
        let mut g = self.guard();
        if g.expire(now) {
            g.last_input = now;
        }
    }

    pub fn set_auto_lock_minutes(&self, minutes: u8) {
        self.guard().auto_lock_minutes = minutes;
    }

    /// Applies the expiry rule. Called by the background thread and by every accessor.
    pub fn expire_if_due(&self) {
        let now = (self.clock)();
        self.guard().expire(now);
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        let now = (self.clock)();
        let mut g = self.guard();
        let unlocked = g.expire(now);
        let seconds_until_lock = match (unlocked, g.timeout()) {
            (true, Some(timeout)) => {
                let remaining = timeout.saturating_sub(now.saturating_duration_since(g.last_input));
                let secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
                (secs <= COUNTDOWN_WINDOW_SECS).then_some(secs)
            }
            _ => None,
        };
        SessionSnapshot {
            unlocked,
            reason: g.last_reason,
            seconds_until_lock,
        }
    }

    /// A copy of the vault key for a protected operation (F09). Checks expiry first.
    /// Used by `vault::with_key`, the only way other modules reach the key.
    pub fn key(&self) -> Result<Zeroizing<[u8; KEY_LEN]>, NotUnlocked> {
        let now = (self.clock)();
        let mut g = self.guard();
        if !g.expire(now) {
            return Err(NotUnlocked);
        }
        g.key.as_ref().map(|k| Zeroizing::new(**k)).ok_or(NotUnlocked)
    }
}

/// A clock the tests advance by hand.
#[cfg(test)]
pub fn fake_clock() -> (std::sync::Arc<Mutex<Instant>>, Clock) {
    let now = std::sync::Arc::new(Mutex::new(Instant::now()));
    let shared = std::sync::Arc::clone(&now);
    (now, Box::new(move || *shared.lock().unwrap()))
}

#[cfg(test)]
pub fn advance_clock(now: &std::sync::Arc<Mutex<Instant>>, secs: u64) {
    *now.lock().unwrap() += Duration::from_secs(secs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::advance_clock as advance;

    fn key() -> Zeroizing<[u8; KEY_LEN]> {
        Zeroizing::new([3u8; KEY_LEN])
    }

    #[test]
    fn starts_locked_with_launch_reason() {
        let s = VaultSession::new();
        let snap = s.snapshot();
        assert!(!snap.unlocked);
        assert_eq!(snap.reason, LockReason::Launch);
        assert_eq!(s.key(), Err(NotUnlocked));
    }

    #[test]
    fn lock_drops_the_key_and_records_manual() {
        let s = VaultSession::new();
        s.unlock(key(), 5);
        assert_eq!(*s.key().unwrap(), [3u8; KEY_LEN]);
        s.lock();
        assert_eq!(s.key(), Err(NotUnlocked));
        assert_eq!(s.snapshot().reason, LockReason::Manual);
    }

    #[test]
    fn locking_a_locked_vault_keeps_the_reason() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 1);
        advance(&now, 61);
        assert_eq!(s.snapshot().reason, LockReason::Inactivity);
        s.lock();
        assert_eq!(s.snapshot().reason, LockReason::Inactivity);
    }

    #[test]
    fn idle_past_timeout_locks_with_inactivity() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 5);
        advance(&now, 5 * 60);
        assert!(s.snapshot().unlocked, "exactly at the timeout is not yet idle past it");
        advance(&now, 1);
        let snap = s.snapshot();
        assert!(!snap.unlocked);
        assert_eq!(snap.reason, LockReason::Inactivity);
        assert_eq!(s.key(), Err(NotUnlocked));
    }

    #[test]
    fn expire_if_due_locks_without_any_other_call() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 1);
        advance(&now, 61);
        s.expire_if_due();
        assert_eq!(s.key(), Err(NotUnlocked));
    }

    #[test]
    fn touch_resets_the_clock() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 1);
        advance(&now, 50);
        s.touch();
        advance(&now, 50);
        assert!(s.snapshot().unlocked);
        advance(&now, 11);
        assert!(!s.snapshot().unlocked);
    }

    #[test]
    fn touch_cannot_revive_an_expired_vault() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 1);
        advance(&now, 61);
        s.touch();
        assert!(!s.snapshot().unlocked);
        assert_eq!(s.snapshot().reason, LockReason::Inactivity);
    }

    #[test]
    fn never_does_not_lock_and_has_no_countdown() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 0);
        advance(&now, 24 * 3600);
        let snap = s.snapshot();
        assert!(snap.unlocked);
        assert_eq!(snap.seconds_until_lock, None);
    }

    #[test]
    fn locked_vault_stays_locked_and_has_no_countdown() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 1);
        s.lock();
        advance(&now, 3600);
        s.touch();
        let snap = s.snapshot();
        assert!(!snap.unlocked);
        assert_eq!(snap.reason, LockReason::Manual);
        assert_eq!(snap.seconds_until_lock, None);
    }

    #[test]
    fn countdown_appears_only_inside_the_last_minute() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 5);
        advance(&now, 200);
        assert_eq!(s.snapshot().seconds_until_lock, None);
        advance(&now, 40); // 240 s idle, 60 s left
        assert_eq!(s.snapshot().seconds_until_lock, Some(60));
        advance(&now, 1);
        assert_eq!(s.snapshot().seconds_until_lock, Some(59));
        s.touch();
        assert_eq!(s.snapshot().seconds_until_lock, None);
    }

    #[test]
    fn changing_the_timeout_applies_immediately() {
        let (now, clock) = fake_clock();
        let s = VaultSession::with_clock(clock);
        s.unlock(key(), 15);
        advance(&now, 120);
        s.set_auto_lock_minutes(1);
        assert!(!s.snapshot().unlocked);
    }
}
