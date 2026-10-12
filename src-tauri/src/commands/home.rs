use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::storage::StorageState;
use crate::projects::{self, Project};
use crate::workspaces::{self, HomeWorkspace};

/// Wire strings are the contract with the TypeScript `HomeErrorCode` union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeErrorCode {
    StorageUnavailable,
}

#[derive(Debug, Deserialize)]
pub struct HomeOverviewArgs {}

#[derive(Debug, Serialize)]
pub struct HomeProjectView {
    pub id: i64,
    pub display_name: String,
    pub root_path: String,
    pub drive: String,
}

#[derive(Debug, Serialize)]
pub struct HomeWorkspaceView {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
    pub project_names: Vec<String>,
    pub last_restored_at: Option<String>,
    pub last_restore_missing: i64,
}

#[derive(Debug, Serialize)]
pub struct HomeOverviewResult {
    pub pinned_projects: Vec<HomeProjectView>,
    pub recent_workspaces: Vec<HomeWorkspaceView>,
    pub project_count: i64,
    pub workspace_count: i64,
    pub error: Option<HomeErrorCode>,
    pub detail: Option<String>,
}

impl From<Project> for HomeProjectView {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            display_name: p.display_name,
            root_path: p.root_path,
            drive: p.drive,
        }
    }
}

impl From<HomeWorkspace> for HomeWorkspaceView {
    fn from(w: HomeWorkspace) -> Self {
        Self {
            id: w.id,
            name: w.name,
            item_count: w.item_count,
            project_names: w.project_names,
            last_restored_at: w.last_restored_at,
            last_restore_missing: w.last_restore_missing,
        }
    }
}

/// Keeps only pinned projects, in the order given (`projects::list` already puts pinned first).
fn pinned_views(all: Vec<Project>) -> Vec<HomeProjectView> {
    all.into_iter()
        .filter(|p| p.pinned)
        .map(Into::into)
        .collect()
}

fn unavailable(detail: Option<String>) -> HomeOverviewResult {
    HomeOverviewResult {
        pinned_projects: Vec::new(),
        recent_workspaces: Vec::new(),
        project_count: 0,
        workspace_count: 0,
        error: Some(HomeErrorCode::StorageUnavailable),
        detail,
    }
}

/// One lock covers both reads; it is released before the result is mapped.
fn read_overview(state: &StorageState) -> HomeOverviewResult {
    let Ok(mutex) = state.result.as_ref() else {
        return unavailable(None);
    };
    let Ok(guard) = mutex.lock() else {
        return unavailable(None);
    };
    let loaded = projects::list(&guard)
        .map_err(|e| e.detail)
        .and_then(|p| workspaces::list_for_home(&guard).map(|w| (p, w)).map_err(|e| e.detail));
    drop(guard);
    match loaded {
        Ok((all_projects, recent)) => overview_from(all_projects, recent),
        Err(detail) => unavailable(detail),
    }
}

/// Counts cover every project and workspace; the lists carry only what Home displays.
fn overview_from(all_projects: Vec<Project>, recent: Vec<HomeWorkspace>) -> HomeOverviewResult {
    let project_count = all_projects.len() as i64;
    let workspace_count = recent.len() as i64;
    HomeOverviewResult {
        pinned_projects: pinned_views(all_projects),
        recent_workspaces: recent.into_iter().map(Into::into).collect(),
        project_count,
        workspace_count,
        error: None,
        detail: None,
    }
}

/// Runs off the main thread. Read only.
#[tauri::command(async)]
pub fn home_overview(
    _args: HomeOverviewArgs,
    state: State<'_, StorageState>,
) -> HomeOverviewResult {
    read_overview(&state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::StorageError;
    use std::path::PathBuf;

    fn project(id: i64, name: &str, pinned: bool) -> Project {
        Project {
            id,
            display_name: name.to_string(),
            root_path: format!("C:\\{name}"),
            drive: "C:".to_string(),
            project_type: "other".to_string(),
            tags: Vec::new(),
            pinned,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn error_code_serialises_as_snake_case() {
        assert_eq!(
            serde_json::to_value(HomeErrorCode::StorageUnavailable).unwrap(),
            serde_json::json!("storage_unavailable")
        );
    }

    #[test]
    fn counts_cover_all_projects_and_workspaces() {
        let r = overview_from(vec![project(1, "a", true), project(2, "b", false)], Vec::new());
        assert_eq!(r.project_count, 2);
        assert_eq!(r.pinned_projects.len(), 1);
        assert_eq!(r.workspace_count, 0);
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["project_count"], 2);
        assert_eq!(v["workspace_count"], 0);
    }

    #[test]
    fn pinned_filter_keeps_only_pinned_projects_in_order() {
        let views = pinned_views(vec![
            project(3, "c", true),
            project(1, "a", true),
            project(2, "b", false),
            project(4, "d", false),
        ]);
        let ids: Vec<i64> = views.iter().map(|v| v.id).collect();
        assert_eq!(ids, vec![3, 1]);
        assert_eq!(views[0].display_name, "c");
        assert_eq!(views[0].root_path, "C:\\c");
        assert_eq!(views[0].drive, "C:");
    }

    #[test]
    fn unavailable_storage_gives_storage_unavailable() {
        let state = StorageState {
            folder: PathBuf::from("x"),
            result: Err(StorageError::Unavailable),
        };
        let r = read_overview(&state);
        assert_eq!(r.error, Some(HomeErrorCode::StorageUnavailable));
        assert!(r.pinned_projects.is_empty());
        assert!(r.recent_workspaces.is_empty());
    }

    #[test]
    fn overview_reads_workspaces_with_stored_restore_outcome() {
        let t = tempfile::tempdir().unwrap();
        let st = crate::db::open_in(t.path()).unwrap();
        let w = workspaces::create(&st, "W").unwrap();
        workspaces::set_restore_outcome(&st, w.id, 2).unwrap();
        let state = StorageState {
            folder: t.path().to_path_buf(),
            result: Ok(std::sync::Mutex::new(st)),
        };
        let r = read_overview(&state);
        assert_eq!(r.error, None);
        assert!(r.pinned_projects.is_empty());
        assert_eq!(r.recent_workspaces.len(), 1);
        assert_eq!(r.recent_workspaces[0].last_restore_missing, 2);
        assert!(r.recent_workspaces[0].last_restored_at.is_some());
        assert_eq!(r.workspace_count, 1);
        assert_eq!(r.project_count, 0);
    }
}
