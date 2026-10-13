//! The single home of the vault's Argon2id cost parameters (AC-24). Setup and unlock derive
//! through `derive`; nothing passes a cost per call. Tests get a low-cost set through
//! `setup_params`, which only differs under `cfg(test)`.

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

/// Upper bounds accepted from stored parameters, so a tampered row cannot request a huge
/// allocation or runtime. Production values sit far below these.
const MAX_M_COST: u32 = 262_144;
const MAX_T_COST: u32 = 10;
const MAX_P_COST: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl KdfParams {
    /// OWASP minimum for Argon2id: 19 MiB, 2 iterations, 1 lane.
    pub const PRODUCTION: KdfParams = KdfParams {
        m_cost: 19_456,
        t_cost: 2,
        p_cost: 1,
    };

    /// Test-only low-cost set (P4).
    #[cfg(test)]
    pub const fn test_low_cost() -> KdfParams {
        KdfParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        }
    }

    /// Whether stored parameters are within the accepted range.
    pub fn is_acceptable(&self) -> bool {
        (8..=MAX_M_COST).contains(&self.m_cost)
            && (1..=MAX_T_COST).contains(&self.t_cost)
            && (1..=MAX_P_COST).contains(&self.p_cost)
    }
}

/// The parameters new vaults are created with.
pub fn setup_params() -> KdfParams {
    #[cfg(test)]
    {
        KdfParams::test_low_cost()
    }
    #[cfg(not(test))]
    {
        KdfParams::PRODUCTION
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfError;

/// Derives the 32-byte vault key. The result zeroizes on drop.
pub fn derive(
    password: &[u8],
    salt: &[u8],
    params: KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, KdfError> {
    if salt.len() != SALT_LEN || !params.is_acceptable() {
        return Err(KdfError);
    }
    let argon_params =
        Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN)).map_err(|_| KdfError)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(password, salt, key.as_mut_slice())
        .map_err(|_| KdfError)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_give_same_key_and_salt_changes_it() {
        let p = KdfParams::test_low_cost();
        let a = derive(b"password", &[1u8; 16], p).unwrap();
        let b = derive(b"password", &[1u8; 16], p).unwrap();
        let c = derive(b"password", &[2u8; 16], p).unwrap();
        assert_eq!(*a, *b);
        assert_ne!(*a, *c);
    }

    #[test]
    fn rejects_bad_salt_length_and_out_of_range_costs() {
        let p = KdfParams::test_low_cost();
        assert!(derive(b"x", &[1u8; 15], p).is_err());
        let huge = KdfParams {
            m_cost: u32::MAX,
            ..p
        };
        assert!(derive(b"x", &[1u8; 16], huge).is_err());
    }

    /// Run on demand: `cargo test --lib vault::kdf::tests::production_cost_timing -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn production_cost_timing() {
        let start = std::time::Instant::now();
        derive(b"correct horse battery", &[9u8; 16], KdfParams::PRODUCTION).unwrap();
        println!("argon2id production derive: {:?}", start.elapsed());
    }
}
