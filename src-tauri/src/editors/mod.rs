//! Editor profiles: storage, path validation, launch planning, and launching.
//!
//! Nothing here uses a shell. The only argument passed to an editor is the item path,
//! as a single element of the argument vector.

use std::path::Path;

use rusqlite::{params, Connection};

use crate::db::Storage;

pub const KINDS: [&str; 4] = ["windows_default", "notepad_plus_plus", "vs_code", "custom"];
pub const MAX_NAME_CHARS: usize = 40;
pub const UNSAFE_CHARS: &[char] = &['&', '|', ';', '"', '^', '%', '<', '>'];

const WINDOWS_DEFAULT: &str = "windows_default";
const CUSTOM_NAME: &str = "Custom";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCode {
    ProfileNotFound,
    ProfilePathEmpty,
    ExecutableNotFound,
    ExecutableInvalid,
    PathUnsafe,
    NameTooLong,
    ItemNotFound,
    LaunchFailed,
    StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorError {
    pub code: EditorCode,
    pub kind: Option<String>,
    pub detail: Option<String>,
}

impl EditorError {
    fn new(code: EditorCode, kind: Option<&str>, detail: Option<&str>) -> Self {
        Self {
            code,
            kind: kind.map(str::to_owned),
            detail: detail.map(str::to_owned),
        }
    }

    fn storage() -> Self {
        Self::new(EditorCode::StorageUnavailable, None, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub kind: String,
    pub name: String,
    pub exe_path: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInput {
    pub kind: String,
    pub name: String,
    pub exe_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub program: String,
    pub args: Vec<String>,
}

/// Returns the trimmed path when valid. Empty is valid unless `is_default`.
pub fn validate_exe_path(raw: &str, is_default: bool) -> Result<String, EditorCode> {
    let path = raw.trim();
    if path.is_empty() {
        return if is_default {
            Err(EditorCode::ProfilePathEmpty)
        } else {
            Ok(String::new())
        };
    }
    if path.contains(UNSAFE_CHARS) {
        return Err(EditorCode::PathUnsafe);
    }
    // Relative paths would resolve against the process working directory, so they are never
    // an acceptable executable location. UNC paths are absolute and pass.
    if !Path::new(path).is_absolute() {
        return Err(EditorCode::ExecutableInvalid);
    }
    let meta = std::fs::metadata(path).map_err(|_| EditorCode::ExecutableNotFound)?;
    let is_exe = Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    if meta.is_dir() || !is_exe {
        return Err(EditorCode::ExecutableInvalid);
    }
    Ok(path.to_owned())
}

/// Inputs that passed `validate_profiles`: names and paths trimmed, executables checked on disk,
/// a blank custom name replaced by `"Custom"`. Only `validate_profiles` builds it.
#[derive(Debug)]
pub struct ValidatedProfiles(Vec<ProfileInput>);

/// Name, unsafe-character and executable checks, including the file-system ones. Needs no
/// storage, so the caller runs it outside its lock. Whether a profile may have an empty path
/// depends on the stored default, which only `save` can read consistently; `save` enforces it.
pub fn validate_profiles(inputs: &[ProfileInput]) -> Result<ValidatedProfiles, EditorError> {
    let mut out = Vec::with_capacity(inputs.len());
    for input in inputs {
        let name = input.name.trim();
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(EditorError::new(
                EditorCode::NameTooLong,
                Some(&input.kind),
                None,
            ));
        }
        let name = if input.kind == "custom" && name.is_empty() {
            CUSTOM_NAME
        } else {
            name
        };
        let exe_path = if input.kind == WINDOWS_DEFAULT {
            String::new()
        } else {
            validate_exe_path(&input.exe_path, false).map_err(|code| {
                EditorError::new(code, Some(&input.kind), Some(input.exe_path.trim()))
            })?
        };
        out.push(ProfileInput {
            kind: input.kind.clone(),
            name: name.to_owned(),
            exe_path,
        });
    }
    Ok(ValidatedProfiles(out))
}

fn read_all(conn: &Connection) -> rusqlite::Result<Vec<Profile>> {
    let mut stmt =
        conn.prepare("SELECT kind, name, exe_path, is_default FROM editor_profiles")?;
    let mut profiles = stmt
        .query_map([], |r| {
            Ok(Profile {
                kind: r.get(0)?,
                name: r.get(1)?,
                exe_path: r.get(2)?,
                is_default: r.get::<_, i64>(3)? != 0,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    profiles.sort_by_key(|p| KINDS.iter().position(|k| *k == p.kind).unwrap_or(KINDS.len()));
    Ok(profiles)
}

pub fn list(storage: &Storage) -> Result<Vec<Profile>, EditorError> {
    read_all(storage.connection()).map_err(|_| EditorError::storage())
}

/// Writes validated profiles in one transaction. Takes `ValidatedProfiles`, so unchecked input
/// cannot be persisted, and does no file-system work: the caller's storage lock covers database
/// access only. Everything that depends on stored state is decided inside the transaction, on
/// the same connection, so it cannot race with a concurrent `set_default`: every kind must be
/// known (`ProfileNotFound`), and the stored default may not receive an empty path
/// (`ProfilePathEmpty`).
pub fn save(
    storage: &Storage,
    validated: ValidatedProfiles,
) -> Result<Vec<Profile>, EditorError> {
    let inputs = validated.0;
    let conn = storage.connection();
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| EditorError::storage())?;
    let current = read_all(&tx).map_err(|_| EditorError::storage())?;
    for input in &inputs {
        if !current.iter().any(|p| p.kind == input.kind) {
            return Err(EditorError::new(
                EditorCode::ProfileNotFound,
                Some(&input.kind),
                None,
            ));
        }
    }
    let default_kind = current.iter().find(|p| p.is_default).map(|p| p.kind.as_str());
    for input in &inputs {
        if input.kind != WINDOWS_DEFAULT
            && default_kind == Some(input.kind.as_str())
            && input.exe_path.is_empty()
        {
            return Err(EditorError::new(
                EditorCode::ProfilePathEmpty,
                Some(&input.kind),
                None,
            ));
        }
    }
    for input in &inputs {
        let result = if input.kind == WINDOWS_DEFAULT {
            // The Windows default profile has a fixed name and no executable.
            Ok(0)
        } else if input.kind == "custom" {
            tx.execute(
                "UPDATE editor_profiles SET name = ?1, exe_path = ?2 WHERE kind = ?3",
                params![input.name, input.exe_path, input.kind],
            )
        } else {
            tx.execute(
                "UPDATE editor_profiles SET exe_path = ?1 WHERE kind = ?2",
                params![input.exe_path, input.kind],
            )
        };
        result.map_err(|_| EditorError::storage())?;
    }
    tx.commit().map_err(|_| EditorError::storage())?;
    list(storage)
}

pub fn set_default(storage: &Storage, kind: &str) -> Result<Vec<Profile>, EditorError> {
    let target = resolve_profile(storage, Some(kind))?;
    if target.kind != WINDOWS_DEFAULT && target.exe_path.trim().is_empty() {
        return Err(EditorError::new(EditorCode::ProfilePathEmpty, Some(kind), None));
    }
    let conn = storage.connection();
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| EditorError::storage())?;
    tx.execute("UPDATE editor_profiles SET is_default = 0 WHERE is_default = 1", [])
        .map_err(|_| EditorError::storage())?;
    let changed = tx
        .execute(
            "UPDATE editor_profiles SET is_default = 1 WHERE kind = ?1",
            params![kind],
        )
        .map_err(|_| EditorError::storage())?;
    if changed == 0 {
        // Dropping the transaction rolls back the cleared default.
        return Err(EditorError::new(EditorCode::ProfileNotFound, Some(kind), None));
    }
    tx.commit().map_err(|_| EditorError::storage())?;
    list(storage)
}

pub fn resolve_profile(storage: &Storage, kind: Option<&str>) -> Result<Profile, EditorError> {
    let profiles = list(storage)?;
    let found = match kind {
        Some(k) => profiles.into_iter().find(|p| p.kind == k),
        None => profiles.into_iter().find(|p| p.is_default),
    };
    found.ok_or_else(|| EditorError::new(EditorCode::ProfileNotFound, kind, None))
}

/// `Ok(None)` means the Windows default: the caller uses the opener instead of a process.
pub fn launch_plan(profile: &Profile, item_path: &str) -> Result<Option<LaunchPlan>, EditorError> {
    if profile.kind == WINDOWS_DEFAULT {
        return Ok(None);
    }
    let program = validate_exe_path(&profile.exe_path, true).map_err(|code| {
        EditorError::new(code, Some(&profile.kind), Some(profile.exe_path.trim()))
    })?;
    Ok(Some(LaunchPlan {
        program,
        args: vec![item_path.to_owned()],
    }))
}

pub trait Launcher {
    fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()>;
    fn open_default(&self, item_path: &str) -> Result<(), String>;
}

pub struct SystemLauncher;

impl SystemLauncher {
    /// Starts the program and drops the child without waiting.
    pub fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()> {
        std::process::Command::new(&plan.program)
            .args(&plan.args)
            .spawn()
            .map(drop)
    }
}

/// Resolves and opens in one call. The command layer resolves under its lock and calls
/// `open_with_profile` after releasing it, so this convenience form is test-only.
#[cfg(test)]
pub fn open_item(
    storage: &Storage,
    launcher: &dyn Launcher,
    kind: Option<&str>,
    item_path: &str,
) -> Result<Profile, EditorError> {
    let profile = resolve_profile(storage, kind)?;
    open_with_profile(launcher, profile, item_path)
}

/// Checks the item and launches with an already resolved profile. Touches no storage, so a
/// caller can release its database lock before the file-system and spawn work.
pub fn open_with_profile(
    launcher: &dyn Launcher,
    profile: Profile,
    item_path: &str,
) -> Result<Profile, EditorError> {
    if std::fs::metadata(item_path).is_err() {
        return Err(EditorError::new(
            EditorCode::ItemNotFound,
            Some(&profile.kind),
            Some(item_path),
        ));
    }
    let launch_failed = |detail: Option<&str>| {
        EditorError::new(EditorCode::LaunchFailed, Some(&profile.kind), detail)
    };
    match launch_plan(&profile, item_path)? {
        None => launcher
            .open_default(item_path)
            .map_err(|e| launch_failed(Some(&e)))?,
        Some(plan) => launcher
            .spawn(&plan)
            .map_err(|e| launch_failed(Some(&e.to_string())))?,
    }
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use super::*;
    use crate::db;

    fn fake_exe(root: &Path, dirs: &str, file: &str) -> String {
        let dir = root.join(dirs);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(file);
        std::fs::write(&path, b"").unwrap();
        path.to_string_lossy().into_owned()
    }

    fn input(kind: &str, name: &str, exe: &str) -> ProfileInput {
        ProfileInput {
            kind: kind.into(),
            name: name.into(),
            exe_path: exe.into(),
        }
    }

    /// Validates like the command layer, then saves.
    fn save_validated(
        storage: &Storage,
        inputs: Vec<ProfileInput>,
    ) -> Result<Vec<Profile>, EditorError> {
        save(storage, validate_profiles(&inputs)?)
    }

    fn open(root: &Path) -> Storage {
        db::open_in(&root.join("data")).unwrap()
    }

    #[derive(Default)]
    struct Recorder {
        spawned: RefCell<Vec<LaunchPlan>>,
        defaults: RefCell<Vec<String>>,
        spawn_fails: bool,
    }

    impl Launcher for Recorder {
        fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()> {
            if self.spawn_fails {
                return Err(std::io::Error::other("boom"));
            }
            self.spawned.borrow_mut().push(plan.clone());
            Ok(())
        }
        fn open_default(&self, item_path: &str) -> Result<(), String> {
            self.defaults.borrow_mut().push(item_path.to_owned());
            Ok(())
        }
    }

    #[test]
    fn exe_path_empty_rules() {
        assert_eq!(validate_exe_path("", false), Ok(String::new()));
        assert_eq!(validate_exe_path("   ", false), Ok(String::new()));
        assert_eq!(validate_exe_path("", true), Err(EditorCode::ProfilePathEmpty));
    }

    #[test]
    fn exe_path_unsafe_chars() {
        for c in UNSAFE_CHARS {
            let raw = format!("C:\\x\\a{c}b.exe");
            assert_eq!(validate_exe_path(&raw, false), Err(EditorCode::PathUnsafe), "{c}");
        }
    }

    #[test]
    fn exe_path_existence_and_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing.exe");
        assert_eq!(
            validate_exe_path(&missing.to_string_lossy(), false),
            Err(EditorCode::ExecutableNotFound)
        );
        let txt = tmp.path().join("notes.txt");
        std::fs::write(&txt, b"x").unwrap();
        assert_eq!(
            validate_exe_path(&txt.to_string_lossy(), false),
            Err(EditorCode::ExecutableInvalid)
        );
        let dir = tmp.path().join("x.exe");
        std::fs::create_dir(&dir).unwrap();
        assert_eq!(
            validate_exe_path(&dir.to_string_lossy(), false),
            Err(EditorCode::ExecutableInvalid)
        );
    }

    #[test]
    fn exe_path_relative_is_invalid() {
        assert_eq!(validate_exe_path("code.exe", false), Err(EditorCode::ExecutableInvalid));
        assert_eq!(validate_exe_path("code.exe", true), Err(EditorCode::ExecutableInvalid));
        assert_eq!(validate_exe_path("..\\bin\\code.exe", false), Err(EditorCode::ExecutableInvalid));
    }

    #[test]
    fn exe_path_with_spaces_is_ok_and_trimmed() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(tmp.path(), "Program Files\\x", "fake editor.exe");
        assert_eq!(validate_exe_path(&format!("  {exe}  "), true), Ok(exe));
    }

    #[test]
    fn profiles_name_length_and_windows_default_path_ignored() {
        let long = "a".repeat(41);
        let err = validate_profiles(&[input("custom", &long, "")]).unwrap_err();
        assert_eq!(err.code, EditorCode::NameTooLong);
        assert_eq!(err.kind.as_deref(), Some("custom"));
        let ok = "a".repeat(40);
        assert!(validate_profiles(&[input("custom", &ok, "")]).is_ok());
        assert!(validate_profiles(&[input("windows_default", "Windows default", "C:\\a&b")]).is_ok());
        let err = validate_profiles(&[input("vs_code", "VS Code", "C:\\a&b.exe")]).unwrap_err();
        assert_eq!(err.code, EditorCode::PathUnsafe);
        assert_eq!(err.kind.as_deref(), Some("vs_code"));
    }

    #[test]
    fn list_returns_four_in_order_with_one_default() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let profiles = list(&storage).unwrap();
        let kinds: Vec<&str> = profiles.iter().map(|p| p.kind.as_str()).collect();
        assert_eq!(kinds, KINDS);
        let defaults: Vec<&str> = profiles
            .iter()
            .filter(|p| p.is_default)
            .map(|p| p.kind.as_str())
            .collect();
        assert_eq!(defaults, ["windows_default"]);
        assert_eq!(profiles[2].name, "VS Code");
    }

    #[test]
    fn save_persists_and_reopens() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(tmp.path(), "Program Files\\vs", "code.exe");
        {
            let storage = open(tmp.path());
            let saved = save_validated(
                &storage,
                vec![
                    input("vs_code", "ignored", &format!(" {exe} ")),
                    input("custom", "My Editor", ""),
                ],
            )
            .unwrap();
            assert_eq!(saved[2].exe_path, exe);
            assert_eq!(saved[2].name, "VS Code");
            assert_eq!(saved[3].name, "My Editor");
        }
        let storage = open(tmp.path());
        let profiles = list(&storage).unwrap();
        assert_eq!(profiles[2].exe_path, exe);
        assert_eq!(profiles[3].name, "My Editor");
    }

    #[test]
    fn invalid_path_is_rejected_before_save() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "n", "npp.exe");
        let err = validate_profiles(&[
            input("notepad_plus_plus", "Notepad++", &exe),
            input("vs_code", "VS Code", "C:\\nope\\code.exe"),
        ])
        .unwrap_err();
        assert_eq!(err.code, EditorCode::ExecutableNotFound);
        assert_eq!(err.kind.as_deref(), Some("vs_code"));
        assert_eq!(list(&storage).unwrap()[1].exe_path, "");
    }

    #[test]
    fn save_with_unknown_kind_changes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "n", "npp.exe");
        let err = save_validated(
            &storage,
            vec![
                input("notepad_plus_plus", "Notepad++", &exe),
                input("emacs", "Emacs", ""),
            ],
        )
        .unwrap_err();
        assert_eq!(err.code, EditorCode::ProfileNotFound);
        assert_eq!(err.kind.as_deref(), Some("emacs"));
        assert_eq!(list(&storage).unwrap()[1].exe_path, "");
    }

    #[test]
    fn validate_profiles_leaves_the_empty_path_decision_to_save() {
        // Whether an empty path is allowed depends on the stored default, which only `save`
        // reads (inside its transaction); see `default_profile_with_empty_path_is_refused_on_save`.
        assert!(validate_profiles(&[input("vs_code", "VS Code", "")]).is_ok());
        assert!(validate_profiles(&[input("vs_code", "VS Code", "  ")]).is_ok());
    }

    #[test]
    fn custom_name_is_trimmed_and_blank_falls_back() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let padded = format!("  {}  ", "a".repeat(40));
        let saved = save_validated(&storage, vec![input("custom", &padded, "")]).unwrap();
        assert_eq!(saved[3].name, "a".repeat(40));
        let saved = save_validated(&storage, vec![input("custom", "   ", "")]).unwrap();
        assert_eq!(saved[3].name, "Custom");
    }

    #[test]
    fn default_profile_with_empty_path_is_refused_on_save() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "v", "code.exe");
        save_validated(&storage, vec![input("vs_code", "VS Code", &exe)]).unwrap();
        set_default(&storage, "vs_code").unwrap();
        let err = save_validated(&storage, vec![input("vs_code", "VS Code", "")]).unwrap_err();
        assert_eq!(err.code, EditorCode::ProfilePathEmpty);
        assert_eq!(err.kind.as_deref(), Some("vs_code"));
        assert_eq!(list(&storage).unwrap()[2].exe_path, exe);
    }

    #[test]
    fn set_default_refuses_empty_path_except_windows_default() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let err = set_default(&storage, "notepad_plus_plus").unwrap_err();
        assert_eq!(err.code, EditorCode::ProfilePathEmpty);
        assert_eq!(err.kind.as_deref(), Some("notepad_plus_plus"));
        assert_eq!(resolve_profile(&storage, None).unwrap().kind, "windows_default");
        set_default(&storage, "vs_code").unwrap_err();
        set_default(&storage, "windows_default").unwrap();
    }

    #[test]
    fn save_unknown_kind_is_profile_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let err = save_validated(&storage, vec![input("emacs", "Emacs", "")]).unwrap_err();
        assert_eq!(err.code, EditorCode::ProfileNotFound);
    }

    #[test]
    fn set_default_switches_exactly_one() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "v", "code.exe");
        save_validated(&storage, vec![input("vs_code", "VS Code", &exe)]).unwrap();
        let profiles = set_default(&storage, "vs_code").unwrap();
        let defaults: Vec<&str> = profiles
            .iter()
            .filter(|p| p.is_default)
            .map(|p| p.kind.as_str())
            .collect();
        assert_eq!(defaults, ["vs_code"]);
        let err = set_default(&storage, "nope").unwrap_err();
        assert_eq!(err.code, EditorCode::ProfileNotFound);
        // Failed call rolled back: vs_code is still the only default.
        let still: Vec<String> = list(&storage)
            .unwrap()
            .into_iter()
            .filter(|p| p.is_default)
            .map(|p| p.kind)
            .collect();
        assert_eq!(still, ["vs_code"]);
    }

    #[test]
    fn resolve_profile_none_is_default() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        assert_eq!(resolve_profile(&storage, None).unwrap().kind, "windows_default");
        let exe = fake_exe(tmp.path(), "c", "custom.exe");
        save_validated(&storage, vec![input("custom", "Custom", &exe)]).unwrap();
        set_default(&storage, "custom").unwrap();
        assert_eq!(resolve_profile(&storage, None).unwrap().kind, "custom");
        assert_eq!(resolve_profile(&storage, Some("vs_code")).unwrap().kind, "vs_code");
        assert_eq!(
            resolve_profile(&storage, Some("zzz")).unwrap_err().code,
            EditorCode::ProfileNotFound
        );
    }

    fn profile(kind: &str, exe: &str) -> Profile {
        Profile {
            kind: kind.into(),
            name: kind.into(),
            exe_path: exe.into(),
            is_default: false,
        }
    }

    #[test]
    fn launch_plan_is_program_plus_single_arg() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(tmp.path(), "Program Files\\Notepad++", "notepad++.exe");
        let item = "D:\\my notes\\a b.txt";
        let plan = launch_plan(&profile("notepad_plus_plus", &exe), item)
            .unwrap()
            .unwrap();
        assert_eq!(plan.program, exe);
        assert_eq!(plan.args, vec![item.to_owned()]);
    }

    #[test]
    fn launch_plan_errors_and_windows_default() {
        let err = launch_plan(&profile("vs_code", ""), "a.txt").unwrap_err();
        assert_eq!(err.code, EditorCode::ProfilePathEmpty);
        assert_eq!(err.kind.as_deref(), Some("vs_code"));
        let err = launch_plan(&profile("custom", "C:\\gone\\x.exe"), "a.txt").unwrap_err();
        assert_eq!(err.code, EditorCode::ExecutableNotFound);
        assert_eq!(launch_plan(&profile("windows_default", ""), "a.txt"), Ok(None));
    }

    fn item_file(root: &Path) -> String {
        let p: PathBuf = root.join("item.txt");
        std::fs::write(&p, b"x").unwrap();
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn open_item_missing_item_does_not_launch() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let rec = Recorder::default();
        let missing = tmp.path().join("nope.txt").to_string_lossy().into_owned();
        let err = open_item(&storage, &rec, None, &missing).unwrap_err();
        assert_eq!(err.code, EditorCode::ItemNotFound);
        assert_eq!(err.detail.as_deref(), Some(missing.as_str()));
        assert!(rec.defaults.borrow().is_empty() && rec.spawned.borrow().is_empty());
    }

    #[test]
    fn open_item_windows_default_uses_opener_once() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let item = item_file(tmp.path());
        let rec = Recorder::default();
        let used = open_item(&storage, &rec, None, &item).unwrap();
        assert_eq!(used.kind, "windows_default");
        assert_eq!(*rec.defaults.borrow(), vec![item]);
        assert!(rec.spawned.borrow().is_empty());
    }

    #[test]
    fn open_item_spawns_planned_program() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "Program Files\\vs", "code.exe");
        save_validated(&storage, vec![input("vs_code", "VS Code", &exe)]).unwrap();
        let item = item_file(tmp.path());
        let rec = Recorder::default();
        open_item(&storage, &rec, Some("vs_code"), &item).unwrap();
        assert_eq!(
            *rec.spawned.borrow(),
            vec![LaunchPlan {
                program: exe,
                args: vec![item]
            }]
        );
        assert!(rec.defaults.borrow().is_empty());
    }

    #[test]
    fn open_item_deleted_exe_is_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "e", "code.exe");
        save_validated(&storage, vec![input("vs_code", "VS Code", &exe)]).unwrap();
        std::fs::remove_file(&exe).unwrap();
        let item = item_file(tmp.path());
        let rec = Recorder::default();
        let err = open_item(&storage, &rec, Some("vs_code"), &item).unwrap_err();
        assert_eq!(err.code, EditorCode::ExecutableNotFound);
        assert!(rec.spawned.borrow().is_empty());
    }

    #[test]
    fn open_item_spawn_error_is_launch_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let exe = fake_exe(tmp.path(), "e", "code.exe");
        save_validated(&storage, vec![input("vs_code", "VS Code", &exe)]).unwrap();
        let item = item_file(tmp.path());
        let rec = Recorder {
            spawn_fails: true,
            ..Default::default()
        };
        let err = open_item(&storage, &rec, Some("vs_code"), &item).unwrap_err();
        assert_eq!(err.code, EditorCode::LaunchFailed);
        assert_eq!(err.kind.as_deref(), Some("vs_code"));
    }

    #[test]
    fn system_launcher_missing_program_errors_without_spawning() {
        let tmp = tempfile::tempdir().unwrap();
        let plan = LaunchPlan {
            program: tmp.path().join("absent.exe").to_string_lossy().into_owned(),
            args: vec!["x".into()],
        };
        assert!(SystemLauncher.spawn(&plan).is_err());
    }

    /// Spawns a copy of the test binary: it reads the single argument as a test filter and
    /// exits, so no editor runs. Covers a program path and an argument that both contain spaces.
    #[cfg(windows)]
    #[test]
    fn system_launcher_spawns_program_with_spaces_and_single_spaced_arg() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Program Files").join("x");
        std::fs::create_dir_all(&dir).unwrap();
        let program = dir.join("fake editor.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &program).unwrap();
        let plan = LaunchPlan {
            program: program.to_string_lossy().into_owned(),
            args: vec![tmp.path().join("my notes").join("a b.txt").to_string_lossy().into_owned()],
        };
        // Run the copy directly as well: `spawn` drops the child, so only this shows that the
        // binary loads and exits cleanly with the spaced path and argument.
        let out = std::process::Command::new(&plan.program)
            .args(&plan.args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("running 0 tests"), "{stdout}");
        assert!(SystemLauncher.spawn(&plan).is_ok());
    }

    #[test]
    fn partial_unique_index_blocks_second_default() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = open(tmp.path());
        let conn = storage.connection();
        let err = conn
            .execute("UPDATE editor_profiles SET is_default = 1 WHERE kind = 'vs_code'", [])
            .unwrap_err();
        assert!(err.to_string().contains("UNIQUE"), "{err}");
        // Non-default rows (is_default = 0) may repeat.
        let zero: i64 = conn
            .query_row("SELECT COUNT(*) FROM editor_profiles WHERE is_default = 0", [], |r| r.get(0))
            .unwrap();
        assert_eq!(zero, 3);
    }
}
