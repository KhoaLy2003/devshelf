//! XChaCha20-Poly1305 helpers. Each body gets a fresh random 24-byte nonce and is bound to its
//! note id as associated data, so a ciphertext cannot be moved to another note.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

use super::keys::KeyBytes;

const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    Invalid,
    Rng,
}

fn cipher(key: &KeyBytes) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(key.as_bytes().into())
}

/// Returns `nonce(24) || ciphertext+tag`. The AAD is the note id as 8 little-endian bytes.
///
/// `CryptoError::Rng` if the operating system cannot supply random bytes; `Invalid` if the cipher
/// rejects the plaintext (over its size limit). Never panics, so a failure cannot poison a lock.
pub fn encrypt(key: &KeyBytes, note_id: i64, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce_bytes).map_err(|_| CryptoError::Rng)?;
    let nonce = XNonce::from(nonce_bytes);
    let aad = note_id.to_le_bytes();
    let sealed = cipher(key)
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Invalid)?;
    let mut blob = Vec::with_capacity(NONCE_LEN + sealed.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&sealed);
    Ok(blob)
}

/// Any failure (short blob, wrong key, wrong note id, tampering) is `CryptoError::Invalid`.
pub fn decrypt(key: &KeyBytes, note_id: i64, blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if blob.len() < NONCE_LEN + TAG_LEN {
        return Err(CryptoError::Invalid);
    }
    let (nonce_bytes, sealed) = blob.split_at(NONCE_LEN);
    let nonce = <&XNonce>::try_from(nonce_bytes).map_err(|_| CryptoError::Invalid)?;
    let aad = note_id.to_le_bytes();
    cipher(key)
        .decrypt(
            nonce,
            Payload {
                msg: sealed,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::keys::KeyBytes;

    fn key() -> KeyBytes {
        KeyBytes::from_bytes([7u8; 32])
    }

    #[test]
    fn round_trip() {
        let blob = encrypt(&key(), 5, b"hello note").unwrap();
        assert_eq!(decrypt(&key(), 5, &blob).unwrap(), b"hello note");
    }

    #[test]
    fn empty_plaintext_round_trips() {
        let blob = encrypt(&key(), 1, b"").unwrap();
        assert_eq!(blob.len(), 24 + 16);
        assert_eq!(decrypt(&key(), 1, &blob).unwrap(), b"");
    }

    #[test]
    fn different_note_id_fails_to_decrypt() {
        let blob = encrypt(&key(), 5, b"hello note").unwrap();
        assert_eq!(decrypt(&key(), 6, &blob), Err(CryptoError::Invalid));
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let blob = encrypt(&key(), 5, b"hello note").unwrap();
        let other = KeyBytes::from_bytes([8u8; 32]);
        assert_eq!(decrypt(&other, 5, &blob), Err(CryptoError::Invalid));
    }

    #[test]
    fn flipped_ciphertext_byte_fails() {
        let mut blob = encrypt(&key(), 5, b"hello note").unwrap();
        let last = blob.len() - 1;
        blob[24] ^= 0x01;
        assert_eq!(decrypt(&key(), 5, &blob), Err(CryptoError::Invalid));
        blob[24] ^= 0x01;
        blob[last] ^= 0x80;
        assert_eq!(decrypt(&key(), 5, &blob), Err(CryptoError::Invalid));
    }

    #[test]
    fn flipped_nonce_byte_fails() {
        let mut blob = encrypt(&key(), 5, b"hello note").unwrap();
        blob[0] ^= 0x01;
        assert_eq!(decrypt(&key(), 5, &blob), Err(CryptoError::Invalid));
    }

    #[test]
    fn truncated_or_short_blob_fails() {
        let blob = encrypt(&key(), 5, b"hello note").unwrap();
        assert_eq!(
            decrypt(&key(), 5, &blob[..blob.len() - 1]),
            Err(CryptoError::Invalid)
        );
        assert_eq!(decrypt(&key(), 5, &blob[..24]), Err(CryptoError::Invalid));
        assert_eq!(decrypt(&key(), 5, &blob[..10]), Err(CryptoError::Invalid));
        assert_eq!(decrypt(&key(), 5, &[]), Err(CryptoError::Invalid));
    }

    #[test]
    fn two_encryptions_use_different_nonces() {
        let a = encrypt(&key(), 5, b"same text").unwrap();
        let b = encrypt(&key(), 5, b"same text").unwrap();
        assert_ne!(a[..24], b[..24]);
        assert_ne!(a, b);
    }

    #[test]
    fn ciphertext_does_not_contain_plaintext() {
        let blob = encrypt(&key(), 5, b"SECRET-BODY-TEXT").unwrap();
        assert!(!blob.windows(16).any(|w| w == b"SECRET-BODY-TEXT"));
    }

    #[test]
    fn crypto_error_debug_is_fixed_text() {
        assert_eq!(format!("{:?}", CryptoError::Invalid), "Invalid");
    }
}
