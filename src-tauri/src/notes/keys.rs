//! The notes data key: a zeroizing wrapper and the stores that hold it.

#[cfg(test)]
use std::sync::Mutex;

use zeroize::Zeroizing;

const KEY_LEN: usize = 32;
const SERVICE: &str = "DevShelf";
const USER: &str = "notes-data-key";

/// A 256-bit key. Zeroizes on drop; `Debug` never prints the bytes.
#[derive(Clone)]
pub struct KeyBytes(Zeroizing<[u8; KEY_LEN]>);

impl KeyBytes {
    #[cfg(test)]
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// A fresh random key from the operating system.
    pub fn generate() -> Result<Self, KeyStoreError> {
        let mut bytes = Zeroizing::new([0u8; KEY_LEN]);
        getrandom::fill(&mut bytes[..]).map_err(|_| KeyStoreError::Unavailable)?;
        Ok(Self(bytes))
    }

    pub(super) fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl std::fmt::Debug for KeyBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyBytes(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStoreError {
    Unavailable,
}

/// Where the data key lives. `load` returns `Ok(None)` when no key has been stored yet.
pub trait KeyStore: Send + Sync {
    fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError>;
    fn store(&self, key: &KeyBytes) -> Result<(), KeyStoreError>;
}

/// Windows Credential Manager, through `keyring`: service `DevShelf`, user `notes-data-key`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CredentialKeyStore;

impl CredentialKeyStore {
    fn entry() -> Result<keyring::Entry, KeyStoreError> {
        keyring::Entry::new(SERVICE, USER).map_err(|_| KeyStoreError::Unavailable)
    }
}

impl KeyStore for CredentialKeyStore {
    fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError> {
        match Self::entry()?.get_secret() {
            Ok(secret) => {
                let secret = Zeroizing::new(secret);
                let mut bytes = Zeroizing::new([0u8; KEY_LEN]);
                if secret.len() != KEY_LEN {
                    return Err(KeyStoreError::Unavailable);
                }
                bytes.copy_from_slice(&secret);
                Ok(Some(KeyBytes(bytes)))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(KeyStoreError::Unavailable),
        }
    }

    fn store(&self, key: &KeyBytes) -> Result<(), KeyStoreError> {
        Self::entry()?
            .set_secret(key.as_bytes())
            .map_err(|_| KeyStoreError::Unavailable)
    }
}

/// In-memory store for tests; never touches the Credential Manager.
#[cfg(test)]
pub struct MemoryKeyStore(Mutex<Option<KeyBytes>>);

#[cfg(test)]
impl MemoryKeyStore {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn with_key(key: KeyBytes) -> Self {
        Self(Mutex::new(Some(key)))
    }

    pub fn is_empty(&self) -> bool {
        self.0.lock().map(|g| g.is_none()).unwrap_or(false)
    }
}

#[cfg(test)]
impl Default for MemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl std::fmt::Debug for MemoryKeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MemoryKeyStore(<redacted>)")
    }
}

#[cfg(test)]
impl KeyStore for MemoryKeyStore {
    fn load(&self) -> Result<Option<KeyBytes>, KeyStoreError> {
        let guard = self.0.lock().map_err(|_| KeyStoreError::Unavailable)?;
        Ok(guard.clone())
    }

    fn store(&self, key: &KeyBytes) -> Result<(), KeyStoreError> {
        let mut guard = self.0.lock().map_err(|_| KeyStoreError::Unavailable)?;
        *guard = Some(key.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_starts_empty_and_round_trips() {
        let store = MemoryKeyStore::new();
        assert!(store.load().unwrap().is_none());
        store.store(&KeyBytes::from_bytes([0xAB; 32])).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.as_bytes(), &[0xAB; 32]);
    }

    #[test]
    fn memory_store_with_key_has_key() {
        let store = MemoryKeyStore::with_key(KeyBytes::from_bytes([1; 32]));
        assert!(!store.is_empty());
        assert!(MemoryKeyStore::new().is_empty());
    }

    #[test]
    fn generated_keys_differ() {
        let a = KeyBytes::generate().unwrap();
        let b = KeyBytes::generate().unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
        assert_ne!(a.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn key_bytes_debug_does_not_print_bytes() {
        let key = KeyBytes::from_bytes([0xAB; 32]);
        let text = format!("{key:?}");
        assert!(!text.contains("171"));
        assert!(!text.to_lowercase().contains("ab"));
        assert!(!text.contains('['));
        assert!(text.contains("redacted"));
    }

    #[test]
    fn memory_store_debug_does_not_print_bytes() {
        let store = MemoryKeyStore::with_key(KeyBytes::from_bytes([0xAB; 32]));
        let text = format!("{store:?}");
        assert!(!text.contains("171"));
        assert!(!text.to_lowercase().contains("ab"));
    }

    #[test]
    fn key_store_error_debug_is_fixed_text() {
        assert_eq!(format!("{:?}", KeyStoreError::Unavailable), "Unavailable");
    }
}
