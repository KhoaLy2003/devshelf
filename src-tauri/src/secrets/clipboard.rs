//! Clipboard access behind a trait so tests inject a fake and never touch the real clipboard.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardError;

pub trait Clipboard: Send + Sync {
    fn set_text(&self, text: &str) -> Result<(), ClipboardError>;
}

/// The operating system clipboard through `arboard`. A handle is opened per call and dropped
/// afterwards, so nothing here retains the value.
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        arboard::Clipboard::new()
            .and_then(|mut c| c.set_text(text))
            .map_err(|_| ClipboardError)
    }
}
