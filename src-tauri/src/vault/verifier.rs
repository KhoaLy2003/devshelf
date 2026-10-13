//! Password verifier: a fixed 32-byte constant sealed with XChaCha20-Poly1305 under the derived
//! key and a fresh random nonce. No password hash is stored. Opening it succeeds only with the
//! key from the right password, salt and cost; any failure is the same `Invalid`, so a wrong
//! password and a tampered verifier or salt cannot be told apart (AC-25).

use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

use super::kdf::KEY_LEN;

const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

/// Not secret; it only has to be fixed so a successful open proves the key.
const CONSTANT: [u8; 32] = *b"devshelf.vault.verifier.v1......";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierError {
    Invalid,
    Rng,
}

/// Returns `nonce(24) || ciphertext+tag`.
pub fn seal(key: &[u8; KEY_LEN]) -> Result<Vec<u8>, VerifierError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce_bytes).map_err(|_| VerifierError::Rng)?;
    let sealed = XChaCha20Poly1305::new(key.into())
        .encrypt(&XNonce::from(nonce_bytes), CONSTANT.as_slice())
        .map_err(|_| VerifierError::Invalid)?;
    let mut blob = Vec::with_capacity(NONCE_LEN + sealed.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&sealed);
    Ok(blob)
}

/// `Ok` only if `blob` opens under `key` to the fixed constant.
pub fn open(key: &[u8; KEY_LEN], blob: &[u8]) -> Result<(), VerifierError> {
    if blob.len() < NONCE_LEN + TAG_LEN {
        return Err(VerifierError::Invalid);
    }
    let (nonce_bytes, sealed) = blob.split_at(NONCE_LEN);
    let nonce = <&XNonce>::try_from(nonce_bytes).map_err(|_| VerifierError::Invalid)?;
    let plain = XChaCha20Poly1305::new(key.into())
        .decrypt(nonce, sealed)
        .map_err(|_| VerifierError::Invalid)?;
    if plain.as_slice() == CONSTANT.as_slice() {
        Ok(())
    } else {
        Err(VerifierError::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_with_the_same_key_only() {
        let blob = seal(&[1u8; 32]).unwrap();
        assert_eq!(open(&[1u8; 32], &blob), Ok(()));
        assert_eq!(open(&[2u8; 32], &blob), Err(VerifierError::Invalid));
    }

    #[test]
    fn nonce_is_fresh_each_time() {
        assert_ne!(seal(&[1u8; 32]).unwrap(), seal(&[1u8; 32]).unwrap());
    }

    #[test]
    fn flipped_byte_and_short_blob_are_invalid() {
        let mut blob = seal(&[1u8; 32]).unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 1;
        assert_eq!(open(&[1u8; 32], &blob), Err(VerifierError::Invalid));
        assert_eq!(open(&[1u8; 32], &blob[..10]), Err(VerifierError::Invalid));
    }
}
