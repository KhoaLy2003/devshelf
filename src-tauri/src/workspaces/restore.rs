//! Restore: classify each workspace item by metadata, open the available ones through their
//! stored editor profile, and report per-item outcomes in position order.
//!
//! Only `std::fs::metadata` (through [`PathProbe`]) touches item paths; nothing inside an item
//! is read or written. Storage is read once up front ([`load_plan`]); probing and opening
//! ([`run_plan`]) need no storage, so a caller can release its database lock in between.

use rusqlite::params;

use super::{
    is_unique_violation, locate, path_key, storage_failure, validate_item_shape, Item, Workspace,
    WorkspaceCode, WorkspaceError,
};
use crate::db::Storage;
use crate::editors::{self, EditorCode, Profile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult {
    Found { is_dir: bool },
    NotFound,
    Unreachable,
}

pub trait PathProbe {
    fn probe(&self, path: &str) -> ProbeResult;
}

/// Real probe: `std::fs::metadata`. `NotFound` is `NotFound`; any other error is `Unreachable`.
pub struct FsProbe;

impl PathProbe for FsProbe {
    fn probe(&self, path: &str) -> ProbeResult {
        match std::fs::metadata(path) {
            Ok(meta) => ProbeResult::Found { is_dir: meta.is_dir() },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ProbeResult::NotFound,
            Err(_) => ProbeResult::Unreachable,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    Available,
    Missing,
    KindMismatch,
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingReason {
    NotFound,
    NowFolder,
    NowFile,
}

/// `classify` with the reason kept: what the probe says about a path stored with `stored_kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Available,
    Missing(MissingReason),
    Unreachable,
}

pub fn verdict(probe: &dyn PathProbe, path: &str, stored_kind: &str) -> Verdict {
    match probe.probe(path) {
        ProbeResult::NotFound => Verdict::Missing(MissingReason::NotFound),
        ProbeResult::Unreachable => Verdict::Unreachable,
        ProbeResult::Found { is_dir } => match (stored_kind == "folder", is_dir) {
            (true, true) | (false, false) => Verdict::Available,
            (false, true) => Verdict::Missing(MissingReason::NowFolder),
            (true, false) => Verdict::Missing(MissingReason::NowFile),
        },
    }
}

#[cfg(test)]
pub fn classify(probe: &dyn PathProbe, path: &str, stored_kind: &str) -> ItemStatus {
    match verdict(probe, path, stored_kind) {
        Verdict::Available => ItemStatus::Available,
        Verdict::Missing(MissingReason::NotFound) => ItemStatus::Missing,
        Verdict::Missing(_) => ItemStatus::KindMismatch,
        Verdict::Unreachable => ItemStatus::Unreachable,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOutcome {
    Opened { item_id: i64 },
    Missing { item_id: i64, reason: MissingReason },
    Unreachable { item_id: i64 },
    CouldNotOpen { item_id: i64, code: EditorCode },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport {
    pub workspace_id: i64,
    pub outcomes: Vec<ItemOutcome>,
    pub completed: bool,
}

struct PlanItem {
    item: Item,
    /// The stored editor profile, or the code explaining why it could not be resolved.
    profile: Result<Profile, EditorCode>,
}

/// Items in position order with their editor profiles, read under one storage access.
pub struct RestorePlan {
    workspace_id: i64,
    items: Vec<PlanItem>,
}

/// Reads the workspace items and the editor profiles. Run under the storage lock; everything
/// after it ([`run_plan`]) needs no storage.
pub fn load_plan(storage: &Storage, workspace_id: i64) -> Result<RestorePlan, WorkspaceError> {
    let workspace = super::get(storage, workspace_id)?
        .ok_or_else(|| WorkspaceError::new(WorkspaceCode::WorkspaceNotFound))?;
    let profiles = if workspace.items.is_empty() {
        Vec::new()
    } else {
        editors::list(storage).map_err(storage_failure)?
    };
    let items = workspace
        .items
        .into_iter()
        .map(|item| {
            let profile = profiles
                .iter()
                .find(|p| p.kind == item.editor_kind)
                .cloned()
                .ok_or(EditorCode::ProfileNotFound);
            PlanItem { item, profile }
        })
        .collect();
    Ok(RestorePlan { workspace_id, items })
}

/// Probes and opens each item in order. A failure on one item never stops the rest.
/// `completed` is true after the loop; with no items it is true and nothing is launched.
pub fn run_plan(
    probe: &dyn PathProbe,
    launcher: &dyn editors::Launcher,
    plan: RestorePlan,
) -> RestoreReport {
    let mut outcomes = Vec::with_capacity(plan.items.len());
    for PlanItem { item, profile } in plan.items {
        let item_id = item.id;
        let outcome = match verdict(probe, &item.path, &item.kind) {
            Verdict::Missing(reason) => ItemOutcome::Missing { item_id, reason },
            Verdict::Unreachable => ItemOutcome::Unreachable { item_id },
            Verdict::Available => match profile {
                Err(code) => ItemOutcome::CouldNotOpen { item_id, code },
                Ok(profile) => match editors::open_with_profile(launcher, profile, &item.path) {
                    Ok(_) => ItemOutcome::Opened { item_id },
                    Err(e) => ItemOutcome::CouldNotOpen { item_id, code: e.code },
                },
            },
        };
        outcomes.push(outcome);
    }
    RestoreReport { workspace_id: plan.workspace_id, outcomes, completed: true }
}

/// Convenience for callers that hold storage for the whole run (tests). Does not write.
#[cfg(test)]
pub fn restore(
    storage: &Storage,
    probe: &dyn PathProbe,
    launcher: &dyn editors::Launcher,
    workspace_id: i64,
) -> Result<RestoreReport, WorkspaceError> {
    Ok(run_plan(probe, launcher, load_plan(storage, workspace_id)?))
}

/// Points an existing item at a new path. Shape rules as `validate_item_path`; existence and
/// kind come from `probe`; the kind must equal the stored kind. Opens nothing; id and
/// position are unchanged.
pub fn relink(
    storage: &Storage,
    probe: &dyn PathProbe,
    item_id: i64,
    raw_path: &str,
) -> Result<Workspace, WorkspaceError> {
    let normalized = validate_item_shape(raw_path)?;
    let (workspace_id, stored_kind): (i64, String) = storage
        .connection()
        .query_row(
            "SELECT workspace_id, kind FROM workspace_items WHERE id = ?1",
            [item_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => WorkspaceError::new(WorkspaceCode::ItemNotFound),
            other => storage_failure(other),
        })?;
    match probe.probe(&normalized) {
        ProbeResult::Found { is_dir } => {
            if (stored_kind == "folder") != is_dir {
                return Err(WorkspaceError::with_detail(
                    WorkspaceCode::ItemKindMismatch,
                    &stored_kind,
                ));
            }
        }
        ProbeResult::NotFound => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::ItemPathNotFound, &normalized));
        }
        ProbeResult::Unreachable => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::ItemPathUnreachable, &normalized));
        }
    }
    let tx = storage.connection().unchecked_transaction().map_err(storage_failure)?;
    locate(&tx, item_id)?;
    match tx.execute(
        "UPDATE workspace_items SET path = ?1, path_key = ?2 WHERE id = ?3",
        params![normalized, path_key(&normalized), item_id],
    ) {
        Ok(_) => {}
        Err(e) if is_unique_violation(&e) => {
            return Err(WorkspaceError::with_detail(WorkspaceCode::ItemDuplicate, &normalized));
        }
        Err(e) => return Err(storage_failure(e)),
    }
    tx.commit().map_err(storage_failure)?;
    super::require_workspace(storage.connection(), workspace_id)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    use super::*;
    use crate::db::open_in;
    use crate::editors::{LaunchPlan, Launcher, ProfileInput};
    use crate::workspaces::{add_item, create, get};

    #[derive(Default)]
    struct FakeProbe(HashMap<String, ProbeResult>);

    impl FakeProbe {
        fn with(mut self, path: &str, r: ProbeResult) -> Self {
            self.0.insert(path.to_lowercase(), r);
            self
        }
    }

    impl PathProbe for FakeProbe {
        fn probe(&self, path: &str) -> ProbeResult {
            self.0.get(&path.to_lowercase()).copied().unwrap_or(ProbeResult::NotFound)
        }
    }

    /// Records calls; never starts anything.
    #[derive(Default)]
    struct RecordingLauncher {
        spawned: RefCell<Vec<LaunchPlan>>,
        defaults: RefCell<Vec<String>>,
    }

    impl RecordingLauncher {
        fn calls(&self) -> usize {
            self.spawned.borrow().len() + self.defaults.borrow().len()
        }
    }

    impl Launcher for RecordingLauncher {
        fn spawn(&self, plan: &LaunchPlan) -> std::io::Result<()> {
            self.spawned.borrow_mut().push(plan.clone());
            Ok(())
        }
        fn open_default(&self, item_path: &str) -> Result<(), String> {
            self.defaults.borrow_mut().push(item_path.to_owned());
            Ok(())
        }
    }

    const FILE: ProbeResult = ProbeResult::Found { is_dir: false };
    const DIR: ProbeResult = ProbeResult::Found { is_dir: true };

    fn s(p: &Path) -> String {
        p.to_string_lossy().into_owned()
    }

    fn store(tmp: &Path) -> Storage {
        open_in(&tmp.join("data")).unwrap()
    }

    fn empty_file(dir: &Path, name: &str) -> String {
        let p = dir.join(name);
        std::fs::write(&p, b"").unwrap();
        s(&p)
    }

    fn put(storage: &Storage, ws: i64, path: &str, kind: &str, editor: &str) -> i64 {
        add_item(storage, ws, path, kind, None, editor)
            .unwrap()
            .items
            .last()
            .unwrap()
            .id
    }

    #[test]
    fn classify_variants() {
        let p = FakeProbe::default()
            .with("f", FILE)
            .with("d", DIR)
            .with("u", ProbeResult::Unreachable);
        assert_eq!(classify(&p, "f", "file"), ItemStatus::Available);
        assert_eq!(classify(&p, "d", "folder"), ItemStatus::Available);
        assert_eq!(classify(&p, "d", "file"), ItemStatus::KindMismatch);
        assert_eq!(verdict(&p, "d", "file"), Verdict::Missing(MissingReason::NowFolder));
        assert_eq!(classify(&p, "f", "folder"), ItemStatus::KindMismatch);
        assert_eq!(verdict(&p, "f", "folder"), Verdict::Missing(MissingReason::NowFile));
        assert_eq!(classify(&p, "gone", "file"), ItemStatus::Missing);
        assert_eq!(verdict(&p, "gone", "file"), Verdict::Missing(MissingReason::NotFound));
        assert_eq!(classify(&p, "u", "file"), ItemStatus::Unreachable);
        assert_eq!(verdict(&p, "u", "folder"), Verdict::Unreachable);
    }

    #[test]
    fn fs_probe_reports_file_folder_and_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let f = empty_file(tmp.path(), "a.txt");
        assert_eq!(FsProbe.probe(&f), FILE);
        assert_eq!(FsProbe.probe(&s(tmp.path())), DIR);
        assert_eq!(FsProbe.probe(&s(&tmp.path().join("nope"))), ProbeResult::NotFound);
    }

    #[test]
    fn restore_reports_in_position_order_and_opens_available_only() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let f = empty_file(tmp.path(), "a.txt");
        let gone = s(&tmp.path().join("gone.txt"));
        let folder = s(tmp.path());
        let i1 = put(&storage, ws, &f, "file", "windows_default");
        let i2 = put(&storage, ws, &gone, "file", "windows_default");
        let i3 = put(&storage, ws, &folder, "folder", "windows_default");
        let probe = FakeProbe::default().with(&f, FILE).with(&folder, DIR);
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &probe, &launcher, ws).unwrap();

        assert_eq!(report.workspace_id, ws);
        assert!(report.completed);
        assert_eq!(
            report.outcomes,
            vec![
                ItemOutcome::Opened { item_id: i1 },
                ItemOutcome::Missing { item_id: i2, reason: MissingReason::NotFound },
                ItemOutcome::Opened { item_id: i3 },
            ]
        );
        assert_eq!(*launcher.defaults.borrow(), vec![f, folder]);
        assert!(launcher.spawned.borrow().is_empty());
    }

    #[test]
    fn restore_unknown_workspace_is_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let err = restore(&storage, &FakeProbe::default(), &RecordingLauncher::default(), 99)
            .unwrap_err();
        assert_eq!(err.code, WorkspaceCode::WorkspaceNotFound);
    }

    #[test]
    fn removed_executable_is_could_not_open_and_run_continues() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let exe_dir = tmp.path().join("bin");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let exe = empty_file(&exe_dir, "code.exe");
        let validated = editors::validate_profiles(&[ProfileInput {
            kind: "vs_code".into(),
            name: "VS Code".into(),
            exe_path: exe.clone(),
        }])
        .unwrap();
        editors::save(&storage, validated).unwrap();
        std::fs::remove_file(&exe).unwrap();

        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        let b = empty_file(tmp.path(), "b.txt");
        let i1 = put(&storage, ws, &a, "file", "vs_code");
        let i2 = put(&storage, ws, &b, "file", "windows_default");
        let probe = FakeProbe::default().with(&a, FILE).with(&b, FILE);
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &probe, &launcher, ws).unwrap();

        assert_eq!(
            report.outcomes,
            vec![
                ItemOutcome::CouldNotOpen { item_id: i1, code: EditorCode::ExecutableNotFound },
                ItemOutcome::Opened { item_id: i2 },
            ]
        );
        assert!(report.completed);
        assert!(launcher.spawned.borrow().is_empty());
        assert_eq!(*launcher.defaults.borrow(), vec![b]);
    }

    #[test]
    fn unknown_editor_kind_is_profile_not_found_and_run_continues() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        let b = empty_file(tmp.path(), "b.txt");
        let i1 = put(&storage, ws, &a, "file", "windows_default");
        let i2 = put(&storage, ws, &b, "file", "windows_default");
        storage
            .connection()
            .execute(
                "UPDATE workspace_items SET editor_kind = 'no_such_editor' WHERE id = ?1",
                [i1],
            )
            .unwrap();
        let probe = FakeProbe::default().with(&a, FILE).with(&b, FILE);
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &probe, &launcher, ws).unwrap();

        assert_eq!(
            report.outcomes,
            vec![
                ItemOutcome::CouldNotOpen { item_id: i1, code: EditorCode::ProfileNotFound },
                ItemOutcome::Opened { item_id: i2 },
            ]
        );
        assert!(report.completed);
        assert_eq!(launcher.calls(), 1);
        assert_eq!(*launcher.defaults.borrow(), vec![b]);
    }

    #[test]
    fn unreachable_item_is_reported_and_not_opened() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        let id = put(&storage, ws, &a, "file", "windows_default");
        let probe = FakeProbe::default().with(&a, ProbeResult::Unreachable);
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &probe, &launcher, ws).unwrap();

        assert_eq!(report.outcomes, vec![ItemOutcome::Unreachable { item_id: id }]);
        assert!(report.completed);
        assert_eq!(launcher.calls(), 0);
    }

    #[test]
    fn kind_mismatch_is_missing_with_reason_and_not_opened() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        let id = put(&storage, ws, &a, "folder", "windows_default");
        let probe = FakeProbe::default().with(&a, FILE);
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &probe, &launcher, ws).unwrap();

        assert_eq!(
            report.outcomes,
            vec![ItemOutcome::Missing { item_id: id, reason: MissingReason::NowFile }]
        );
        assert_eq!(launcher.calls(), 0);
    }

    #[test]
    fn empty_workspace_completes_with_no_outcomes_and_launches_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Empty").unwrap().id;
        let launcher = RecordingLauncher::default();

        let report = restore(&storage, &FakeProbe::default(), &launcher, ws).unwrap();

        assert!(report.completed);
        assert!(report.outcomes.is_empty());
        assert_eq!(launcher.calls(), 0);
    }

    #[test]
    fn set_restore_outcome_is_the_only_writer_of_last_restored_at() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        put(&storage, ws, &a, "file", "windows_default");
        let probe = FakeProbe::default().with(&a, FILE);

        let report = restore(&storage, &probe, &RecordingLauncher::default(), ws).unwrap();
        assert!(report.completed);
        assert_eq!(get(&storage, ws).unwrap().unwrap().last_restored_at, None);
        assert_eq!(crate::workspaces::list(&storage).unwrap()[0].last_restored_at, None);

        crate::workspaces::set_restore_outcome(&storage, ws, 0).unwrap();
        let stamp = get(&storage, ws).unwrap().unwrap().last_restored_at.unwrap();
        assert_eq!(stamp.len(), 20, "{stamp}");
        assert!(stamp.ends_with('Z') && stamp.contains('T'), "{stamp}");
        assert_eq!(
            crate::workspaces::list(&storage).unwrap()[0].last_restored_at.as_deref(),
            Some(stamp.as_str())
        );

        assert_eq!(
            crate::workspaces::set_restore_outcome(&storage, 99, 0).unwrap_err().code,
            WorkspaceCode::WorkspaceNotFound
        );
    }

    #[test]
    fn last_restored_at_survives_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let ws;
        {
            let storage = store(tmp.path());
            ws = create(&storage, "Ops").unwrap().id;
            crate::workspaces::set_restore_outcome(&storage, ws, 0).unwrap();
        }
        let reopened = store(tmp.path());
        assert!(get(&reopened, ws).unwrap().unwrap().last_restored_at.is_some());
    }

    #[test]
    fn relink_kind_mismatch_reports_stored_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let gone = s(&tmp.path().join("gone.txt"));
        let file_item = put(&storage, ws, &gone, "file", "windows_default");
        let folder_item = put(&storage, ws, &s(&tmp.path().join("gone-dir")), "folder", "windows_default");
        let f = empty_file(tmp.path(), "new.txt");
        let d = s(tmp.path());
        let probe = FakeProbe::default().with(&f, FILE).with(&d, DIR);

        let err = relink(&storage, &probe, file_item, &d).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemKindMismatch);
        assert_eq!(err.detail.as_deref(), Some("file"));
        let err = relink(&storage, &probe, folder_item, &f).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemKindMismatch);
        assert_eq!(err.detail.as_deref(), Some("folder"));
        assert_eq!(get(&storage, ws).unwrap().unwrap().items[0].path, gone);
    }

    #[test]
    fn relink_validates_shape_existence_and_item() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let id = put(&storage, ws, &s(&tmp.path().join("gone.txt")), "file", "windows_default");
        let probe = FakeProbe::default()
            .with("F:\\unreach\\x.txt", ProbeResult::Unreachable);

        let code = |raw: &str| relink(&storage, &probe, id, raw).unwrap_err().code;
        assert_eq!(code("  "), WorkspaceCode::ItemPathEmpty);
        assert_eq!(code("rel\\x.txt"), WorkspaceCode::ItemRelativePath);
        assert_eq!(code("F:\\nowhere\\x.txt"), WorkspaceCode::ItemPathNotFound);
        assert_eq!(code("F:\\unreach\\x.txt"), WorkspaceCode::ItemPathUnreachable);
        assert_eq!(
            relink(&storage, &probe, 999, "F:\\a.txt").unwrap_err().code,
            WorkspaceCode::ItemNotFound
        );
    }

    #[test]
    fn relink_to_an_unreachable_path_is_item_path_unreachable_and_keeps_the_stored_path() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let old = s(&tmp.path().join("gone.txt"));
        let id = put(&storage, ws, &old, "file", "windows_default");
        let target = "F:\\unreach\\x.txt";
        let probe = FakeProbe::default().with(target, ProbeResult::Unreachable);

        let err = relink(&storage, &probe, id, target).unwrap_err();

        assert_eq!(err.code, WorkspaceCode::ItemPathUnreachable);
        assert_eq!(err.detail.as_deref(), Some(target));
        assert_eq!(get(&storage, ws).unwrap().unwrap().items[0].path, old);
    }

    #[test]
    fn relink_duplicate_path_is_item_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let a = empty_file(tmp.path(), "a.txt");
        put(&storage, ws, &a, "file", "windows_default");
        let second = put(&storage, ws, &s(&tmp.path().join("gone.txt")), "file", "windows_default");
        let probe = FakeProbe::default().with(&a, FILE);

        let err = relink(&storage, &probe, second, &a.to_uppercase()).unwrap_err();
        assert_eq!(err.code, WorkspaceCode::ItemDuplicate);
    }

    #[test]
    fn relink_updates_path_keeps_id_and_position_and_launches_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let first = put(&storage, ws, &s(&tmp.path().join("gone.txt")), "file", "windows_default");
        let second = put(&storage, ws, &empty_file(tmp.path(), "b.txt"), "file", "windows_default");
        let new = empty_file(tmp.path(), "new.txt");
        let probe = FakeProbe::default().with(&new, FILE);

        let updated = relink(&storage, &probe, first, &new).unwrap();

        assert_eq!(updated.items[0].id, first);
        assert_eq!(updated.items[0].position, 0);
        assert_eq!(updated.items[0].path, new);
        assert_eq!(updated.items[1].id, second);
        // Same path again on the same item is not a duplicate of itself.
        relink(&storage, &probe, first, &new).unwrap();
        // The new path_key now blocks adding it again.
        let dup = add_item(&storage, ws, &new, "file", None, "windows_default").unwrap_err();
        assert_eq!(dup.code, WorkspaceCode::ItemDuplicate);
    }

    fn snapshot(dir: &Path) -> Vec<(String, std::time::SystemTime)> {
        let mut out = vec![(".".to_string(), std::fs::metadata(dir).unwrap().modified().unwrap())];
        for e in std::fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            out.push((
                e.file_name().to_string_lossy().into_owned(),
                e.metadata().unwrap().modified().unwrap(),
            ));
        }
        out.sort();
        out
    }

    #[test]
    fn restore_and_relink_write_nothing_inside_item_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let storage = store(tmp.path());
        let ws = create(&storage, "Ops").unwrap().id;
        let item_dir = tmp.path().join("proj");
        let other_dir = tmp.path().join("proj2");
        std::fs::create_dir_all(&item_dir).unwrap();
        std::fs::create_dir_all(&other_dir).unwrap();
        empty_file(&item_dir, "a.txt");
        empty_file(&other_dir, "b.txt");
        let (dir_s, other_s) = (s(&item_dir), s(&other_dir));
        let id = put(&storage, ws, &dir_s, "folder", "windows_default");
        let before = (snapshot(&item_dir), snapshot(&other_dir));

        let launcher = RecordingLauncher::default();
        restore(&storage, &FsProbe, &launcher, ws).unwrap();
        relink(&storage, &FsProbe, id, &other_s).unwrap();

        assert_eq!((snapshot(&item_dir), snapshot(&other_dir)), before);
        assert_eq!(*launcher.defaults.borrow(), vec![dir_s]);
    }
}
