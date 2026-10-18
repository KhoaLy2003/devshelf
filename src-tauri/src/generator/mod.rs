//! Random password generation. Randomness comes only from `getrandom`; the output never
//! touches disk. The caller decides where the returned value goes.

use zeroize::Zeroizing;

pub const MIN_LENGTH: usize = 8;
pub const MAX_LENGTH: usize = 128;

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*-_=+?";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorCode {
    LengthInvalid,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratorError {
    pub code: GeneratorCode,
}

/// Generates a password of `length` ASCII characters. Every enabled class is represented at
/// least once; the remaining characters come from the union of enabled classes; the result is
/// shuffled with Fisher-Yates.
pub fn generate(length: usize, include_symbols: bool) -> Result<Zeroizing<String>, GeneratorError> {
    if !(MIN_LENGTH..=MAX_LENGTH).contains(&length) {
        return Err(GeneratorError {
            code: GeneratorCode::LengthInvalid,
        });
    }
    let mut classes: Vec<&[u8]> = vec![LOWER, UPPER, DIGITS];
    if include_symbols {
        classes.push(SYMBOLS);
    }
    let union: Vec<u8> = classes.concat();

    let mut chars: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::with_capacity(length));
    for class in &classes {
        chars.push(class[random_below(class.len())?]);
    }
    while chars.len() < length {
        chars.push(union[random_below(union.len())?]);
    }
    for i in (1..chars.len()).rev() {
        let j = random_below(i + 1)?;
        chars.swap(i, j);
    }
    // All bytes come from ASCII constants, so each byte maps to one char.
    let out: String = chars.iter().map(|&b| b as char).collect();
    Ok(Zeroizing::new(out))
}

/// Unbiased uniform draw from `0..n` by rejecting the top of the `u32` range.
fn random_below(n: usize) -> Result<usize, GeneratorError> {
    debug_assert!(n > 0 && n <= MAX_LENGTH * 4);
    let n = n as u32;
    let zone = u32::MAX - (u32::MAX % n);
    loop {
        let mut buf = [0u8; 4];
        getrandom::fill(&mut buf).map_err(|_| GeneratorError {
            code: GeneratorCode::Unavailable,
        })?;
        let v = u32::from_le_bytes(buf);
        if v < zone {
            return Ok((v % n) as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_lower(c: char) -> bool {
        c.is_ascii_lowercase()
    }
    fn is_upper(c: char) -> bool {
        c.is_ascii_uppercase()
    }
    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }
    fn is_symbol(c: char) -> bool {
        SYMBOLS.contains(&(c as u8)) && c.is_ascii()
    }

    #[test]
    fn lengths_outside_range_are_rejected() {
        for len in [0, 7, 129, 10_000] {
            let err = generate(len, true).unwrap_err();
            assert_eq!(err.code, GeneratorCode::LengthInvalid, "length {len}");
        }
    }

    #[test]
    fn boundary_lengths_produce_exact_char_count() {
        assert_eq!(generate(MIN_LENGTH, true).unwrap().chars().count(), 8);
        assert_eq!(generate(MAX_LENGTH, false).unwrap().chars().count(), 128);
    }

    #[test]
    fn symbols_on_every_result_has_all_four_classes() {
        for _ in 0..200 {
            let pw = generate(MIN_LENGTH, true).unwrap();
            assert!(pw.chars().any(is_lower), "{pw:?}");
            assert!(pw.chars().any(is_upper), "{pw:?}");
            assert!(pw.chars().any(is_digit), "{pw:?}");
            assert!(pw.chars().any(is_symbol), "{pw:?}");
            assert!(pw.chars().all(|c| is_lower(c) || is_upper(c) || is_digit(c) || is_symbol(c)));
        }
    }

    #[test]
    fn symbols_off_uses_only_letters_and_digits() {
        for _ in 0..200 {
            let pw = generate(MAX_LENGTH, false).unwrap();
            assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()), "{pw:?}");
        }
    }

    #[test]
    fn random_below_stays_in_range() {
        for n in [1usize, 2, 7, 13, 64] {
            for _ in 0..500 {
                assert!(random_below(n).unwrap() < n);
            }
        }
    }
}
