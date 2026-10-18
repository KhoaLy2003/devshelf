use serde::{Deserialize, Serialize};
use tauri::State;
use zeroize::Zeroize;

use crate::commands::secrets::SecretClipboard;
use crate::generator::{self, GeneratorCode};
use crate::secrets::MAX_VALUE_BYTES;

/// Wire form of `generator::GeneratorCode` plus the clipboard failure; the strings are the
/// contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorErrorCode {
    GeneratorLengthInvalid,
    GeneratorUnavailable,
    GeneratorCopyFailed,
}

impl From<GeneratorCode> for GeneratorErrorCode {
    fn from(code: GeneratorCode) -> Self {
        match code {
            GeneratorCode::LengthInvalid => Self::GeneratorLengthInvalid,
            GeneratorCode::Unavailable => Self::GeneratorUnavailable,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PasswordGenerateArgs {
    pub length: u32,
    pub include_symbols: bool,
}

#[derive(Serialize)]
pub struct PasswordGenerateResult {
    pub value: Option<String>,
    pub error: Option<GeneratorErrorCode>,
}

impl std::fmt::Debug for PasswordGenerateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordGenerateResult")
            .field("value", &format_args!("<redacted>"))
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
pub struct PasswordCopyArgs {
    pub value: String,
}

impl std::fmt::Debug for PasswordCopyArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordCopyArgs")
            .field("value", &format_args!("<redacted>"))
            .finish()
    }
}

#[derive(Debug, Serialize)]
pub struct PasswordCopyResult {
    pub error: Option<GeneratorErrorCode>,
}

#[tauri::command(async)]
pub fn password_generate(args: PasswordGenerateArgs) -> PasswordGenerateResult {
    // A length that does not fit `usize` is out of range by definition.
    let length = usize::try_from(args.length).unwrap_or(0);
    match generator::generate(length, args.include_symbols) {
        Ok(value) => PasswordGenerateResult {
            value: Some(value.to_string()),
            error: None,
        },
        Err(e) => PasswordGenerateResult {
            value: None,
            error: Some(e.code.into()),
        },
    }
}

#[tauri::command(async)]
pub fn password_copy(
    mut args: PasswordCopyArgs,
    clipboard: State<'_, SecretClipboard>,
) -> PasswordCopyResult {
    let copied = !args.value.is_empty()
        && args.value.len() <= MAX_VALUE_BYTES
        && clipboard.0.set_text(&args.value).is_ok();
    args.value.zeroize();
    PasswordCopyResult {
        error: if copied {
            None
        } else {
            Some(GeneratorErrorCode::GeneratorCopyFailed)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_serialize_to_contract_strings() {
        let cases = [
            (GeneratorErrorCode::GeneratorLengthInvalid, "generator_length_invalid"),
            (GeneratorErrorCode::GeneratorUnavailable, "generator_unavailable"),
            (GeneratorErrorCode::GeneratorCopyFailed, "generator_copy_failed"),
        ];
        for (code, text) in cases {
            assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{text}\""));
        }
    }

    #[test]
    fn generate_result_debug_never_shows_the_value() {
        let result = PasswordGenerateResult {
            value: Some("s3cret-Value!".to_string()),
            error: None,
        };
        let shown = format!("{result:?}");
        assert!(!shown.contains("s3cret-Value!"));
        assert!(shown.contains("<redacted>"));
    }

    #[test]
    fn copy_args_debug_never_shows_the_value() {
        let args = PasswordCopyArgs {
            value: "s3cret-Value!".to_string(),
        };
        assert!(!format!("{args:?}").contains("s3cret-Value!"));
    }

    #[test]
    fn generator_length_error_maps_to_wire_code() {
        assert_eq!(
            GeneratorErrorCode::from(GeneratorCode::LengthInvalid),
            GeneratorErrorCode::GeneratorLengthInvalid
        );
    }
}
