//! XChaCha20-Poly1305 for secret values. Blob = `nonce(24) || ciphertext+tag`; the associated
//! data is the row id as 8 little-endian bytes, so a ciphertext moved to another row fails.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CryptoError;

fn cipher(key: &[u8; 32]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(key.into())
}

pub fn encrypt(key: &[u8; 32], id: i64, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce_bytes).map_err(|_| CryptoError)?;
    let nonce = XNonce::from(nonce_bytes);
    let aad = id.to_le_bytes();
    let sealed = cipher(key)
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError)?;
    let mut blob = Vec::with_capacity(NONCE_LEN + sealed.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&sealed);
    Ok(blob)
}

/// Any failure (short blob, wrong key, wrong row, tampering) is `CryptoError`.
pub fn decrypt(key: &[u8; 32], id: i64, blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if blob.len() < NONCE_LEN + TAG_LEN {
        return Err(CryptoError);
    }
    let (nonce_bytes, sealed) = blob.split_at(NONCE_LEN);
    let nonce = <&XNonce>::try_from(nonce_bytes).map_err(|_| CryptoError)?;
    let aad = id.to_le_bytes();
    cipher(key)
        .decrypt(
            nonce,
            Payload {
                msg: sealed,
                aad: &aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| CryptoError)
}
