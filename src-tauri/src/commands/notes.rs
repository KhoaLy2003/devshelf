use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::storage::StorageState;
use crate::db::Storage;
use crate::notes::{self, KeyStore, Note, NoteCode, NoteError, NoteSummary};

/// Managed Tauri state holding the key store: `CredentialKeyStore` in production.
pub struct NoteKeys(pub Box<dyn KeyStore>);

/// Wire form of `notes::NoteCode`; the strings are the contract with the TypeScript union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteErrorCode {
    NoteNotFound,
    TitleTooLong,
    BodyTooLarge,
    ProjectNotFound,
    KeyMissing,
    KeyringUnavailable,
    StorageUnavailable,
    BodyUnreadable,
}

impl From<NoteCode> for NoteErrorCode {
    fn from(code: NoteCode) -> Self {
        match code {
            NoteCode::NoteNotFound => Self::NoteNotFound,
            NoteCode::TitleTooLong => Self::TitleTooLong,
            NoteCode::BodyTooLarge => Self::BodyTooLarge,
            NoteCode::ProjectNotFound => Self::ProjectNotFound,
            NoteCode::KeyMissing => Self::KeyMissing,
            NoteCode::KeyringUnavailable => Self::KeyringUnavailable,
            NoteCode::StorageUnavailable => Self::StorageUnavailable,
            NoteCode::BodyUnreadable => Self::BodyUnreadable,
        }
    }
}

/// A note without its body.
#[derive(Debug, Serialize)]
pub struct NoteSummaryView {
    pub id: i64,
    pub title: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub project_missing: bool,
    pub updated_at: String,
}

impl From<NoteSummary> for NoteSummaryView {
    fn from(s: NoteSummary) -> Self {
        Self {
            id: s.id,
            title: s.title,
            project_id: s.project_id,
            project_name: s.project_name,
            project_missing: s.project_missing,
            updated_at: s.updated_at,
        }
    }
}

/// A note with its body; returned only by `notes_get` and `notes_create`.
#[derive(Serialize)]
pub struct NoteView {
    pub id: i64,
    pub title: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub project_missing: bool,
    pub updated_at: String,
    pub body: String,
}

impl std::fmt::Debug for NoteView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteView")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("body", &format_args!("<redacted>"))
            .finish()
    }
}

impl From<Note> for NoteView {
    fn from(n: Note) -> Self {
        Self {
            id: n.summary.id,
            title: n.summary.title,
            project_id: n.summary.project_id,
            project_name: n.summary.project_name,
            project_missing: n.summary.project_missing,
            updated_at: n.summary.updated_at,
            body: n.body,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NotesListArgs {}

#[derive(Debug, Serialize)]
pub struct NotesListResult {
    pub notes: Vec<NoteSummaryView>,
    pub error: Option<NoteErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NoteGetArgs {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct NoteCreateArgs {
    pub project_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct NoteResult {
    pub note: Option<NoteView>,
    pub error: Option<NoteErrorCode>,
    pub detail: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteSaveArgs {
    pub id: i64,
    pub title: String,
    pub project_id: Option<i64>,
    pub body: String,
}

impl std::fmt::Debug for NoteSaveArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteSaveArgs")
            .field("id", &self.id)
            .field("project_id", &self.project_id)
            .field("body", &format_args!("<redacted>"))
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Serialize)]
pub struct NoteSaveResult {
    pub summary: Option<NoteSummaryView>,
    pub error: Option<NoteErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NoteDeleteArgs {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct NoteDeleteResult {
    pub error: Option<NoteErrorCode>,
    pub detail: Option<String>,
}

fn unavailable() -> NoteError {
    NoteError {
        code: NoteCode::StorageUnavailable,
    }
}

/// Locks the storage and runs `f`. A failed storage or poisoned lock is `StorageUnavailable`.
/// `f` MUST be the notes domain call only.
fn with_storage<T>(
    state: &StorageState,
    f: impl FnOnce(&Storage) -> Result<T, NoteError>,
) -> Result<T, NoteError> {
    let mutex = state.result.as_ref().map_err(|_| unavailable())?;
    let guard = mutex.lock().map_err(|_| unavailable())?;
    f(&guard)
}

fn list_result(r: Result<Vec<NoteSummary>, NoteError>) -> NotesListResult {
    match r {
        Ok(list) => NotesListResult {
            notes: list.into_iter().map(Into::into).collect(),
            error: None,
            detail: None,
        },
        Err(e) => NotesListResult {
            notes: Vec::new(),
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

fn note_result(r: Result<Note, NoteError>) -> NoteResult {
    match r {
        Ok(note) => NoteResult {
            note: Some(note.into()),
            error: None,
            detail: None,
        },
        Err(e) => NoteResult {
            note: None,
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

fn save_result(r: Result<NoteSummary, NoteError>) -> NoteSaveResult {
    match r {
        Ok(summary) => NoteSaveResult {
            summary: Some(summary.into()),
            error: None,
            detail: None,
        },
        Err(e) => NoteSaveResult {
            summary: None,
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

fn delete_result(r: Result<(), NoteError>) -> NoteDeleteResult {
    match r {
        Ok(()) => NoteDeleteResult {
            error: None,
            detail: None,
        },
        Err(e) => NoteDeleteResult {
            error: Some(e.code.into()),
            detail: None,
        },
    }
}

#[tauri::command(async)]
pub fn notes_list(_args: NotesListArgs, state: State<'_, StorageState>) -> NotesListResult {
    list_result(with_storage(&state, notes::list))
}

#[tauri::command(async)]
pub fn notes_get(
    args: NoteGetArgs,
    state: State<'_, StorageState>,
    keys: State<'_, NoteKeys>,
) -> NoteResult {
    let loaded = match notes::load_key(keys.0.as_ref()) {
        Ok(k) => k,
        Err(e) => return note_result(Err(e)),
    };
    note_result(with_storage(&state, |s| {
        notes::get(s, keys.0.as_ref(), loaded, args.id)
    }))
}

#[tauri::command(async)]
pub fn notes_create(
    args: NoteCreateArgs,
    state: State<'_, StorageState>,
    keys: State<'_, NoteKeys>,
) -> NoteResult {
    let loaded = match notes::load_key(keys.0.as_ref()) {
        Ok(k) => k,
        Err(e) => return note_result(Err(e)),
    };
    note_result(with_storage(&state, |s| {
        notes::create(s, keys.0.as_ref(), loaded, args.project_id)
    }))
}

#[tauri::command(async)]
pub fn notes_save(
    args: NoteSaveArgs,
    state: State<'_, StorageState>,
    keys: State<'_, NoteKeys>,
) -> NoteSaveResult {
    let loaded = match notes::load_key(keys.0.as_ref()) {
        Ok(k) => k,
        Err(e) => return save_result(Err(e)),
    };
    save_result(with_storage(&state, |s| {
        notes::save(
            s,
            keys.0.as_ref(),
            loaded,
            args.id,
            &args.title,
            args.project_id,
            &args.body,
        )
    }))
}

#[tauri::command(async)]
pub fn notes_delete(args: NoteDeleteArgs, state: State<'_, StorageState>) -> NoteDeleteResult {
    delete_result(with_storage(&state, |s| notes::delete(s, args.id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json<T: Serialize>(v: T) -> String {
        serde_json::to_string(&v).unwrap()
    }

    fn summary() -> NoteSummary {
        NoteSummary {
            id: 7,
            title: "T".into(),
            project_id: Some(2),
            project_name: Some("P".into()),
            project_missing: false,
            updated_at: "2026-10-12T00:00:00Z".into(),
        }
    }

    #[test]
    fn every_note_code_maps_to_its_snake_case_string() {
        let all = [
            (NoteCode::NoteNotFound, "note_not_found"),
            (NoteCode::TitleTooLong, "title_too_long"),
            (NoteCode::BodyTooLarge, "body_too_large"),
            (NoteCode::ProjectNotFound, "project_not_found"),
            (NoteCode::KeyMissing, "key_missing"),
            (NoteCode::KeyringUnavailable, "keyring_unavailable"),
            (NoteCode::StorageUnavailable, "storage_unavailable"),
            (NoteCode::BodyUnreadable, "body_unreadable"),
        ];
        for (code, text) in all {
            assert_eq!(json(NoteErrorCode::from(code)), format!("\"{text}\""));
        }
    }

    #[test]
    fn save_result_has_summary_fields_and_no_body() {
        let ok: serde_json::Value = serde_json::from_str(&json(save_result(Ok(summary())))).unwrap();
        let mut top: Vec<_> = ok.as_object().unwrap().keys().cloned().collect();
        top.sort();
        assert_eq!(top, ["detail", "error", "summary"]);
        let mut inner: Vec<_> = ok["summary"].as_object().unwrap().keys().cloned().collect();
        inner.sort();
        assert_eq!(
            inner,
            ["id", "project_id", "project_missing", "project_name", "title", "updated_at"]
        );
    }

    #[test]
    fn note_view_is_flat() {
        let view = NoteView::from(Note {
            summary: summary(),
            body: "secret body".into(),
        });
        let v: serde_json::Value = serde_json::from_str(&json(&view)).unwrap();
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(
            keys,
            ["body", "id", "project_id", "project_missing", "project_name", "title", "updated_at"]
        );
        assert_eq!(v["id"], 7);
        assert_eq!(v["project_name"], "P");
        assert_eq!(v["body"], "secret body");
        assert!(!format!("{view:?}").contains("secret body"));
    }

    #[test]
    fn error_results_carry_no_note_or_body() {
        let key_missing = NoteError {
            code: NoteCode::KeyMissing,
        };
        let r = note_result(Err(key_missing.clone()));
        assert!(r.note.is_none());
        assert_eq!(r.error, Some(NoteErrorCode::KeyMissing));
        assert!(!json(&r).contains("\"body\""));
        let s = save_result(Err(key_missing));
        assert!(s.summary.is_none());
        assert_eq!(s.error, Some(NoteErrorCode::KeyMissing));
    }

    #[test]
    fn save_args_debug_hides_body() {
        let args = NoteSaveArgs {
            id: 1,
            title: "t".into(),
            project_id: None,
            body: "secret body".into(),
        };
        assert!(!format!("{args:?}").contains("secret body"));
    }
}
