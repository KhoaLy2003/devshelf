import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  EditorOpenArgs,
  EditorOpenResult,
  EditorPickExecutableArgs,
  EditorPickExecutableResult,
  EditorPickItemArgs,
  EditorPickItemResult,
  EditorProfileInputArgs,
  EditorProfilesListArgs,
  EditorProfilesListResult,
  EditorProfilesSaveArgs,
  EditorProfilesSaveResult,
  EditorSetDefaultArgs,
  EditorSetDefaultResult,
  AppInfoArgs,
  ProjectCheckPathArgs,
  ProjectCheckPathResult,
  ProjectGetArgs,
  ProjectGetResult,
  ProjectPickFolderArgs,
  ProjectPickFolderResult,
  ProjectRegisterArgs,
  ProjectRegisterResult,
  ProjectSetPinnedArgs,
  ProjectsListArgs,
  ProjectsListResult,
  StorageStatus,
  StorageStatusArgs,
  ItemAddArgs,
  ItemMoveArgs,
  ItemRemoveArgs,
  ItemUpdateArgs,
  ItemRelinkArgs,
  WorkspaceCreateArgs,
  WorkspaceDeleteArgs,
  WorkspaceDeleteResult,
  WorkspaceGetArgs,
  WorkspacePickItemArgs,
  WorkspacePickResult,
  WorkspaceRenameArgs,
  WorkspaceResult,
  WorkspaceRestoreArgs,
  WorkspaceRestoreResult,
  WorkspacesListArgs,
  WorkspacesListResult,
} from "../types/commands";

export function appInfo(): Promise<AppInfo> {
  const args: AppInfoArgs = {};
  return invoke<AppInfo>("app_info", { args });
}

export function storageStatus(): Promise<StorageStatus> {
  const args: StorageStatusArgs = {};
  return invoke<StorageStatus>("storage_status", { args });
}

export function listProjects(): Promise<ProjectsListResult> {
  const args: ProjectsListArgs = {};
  return invoke<ProjectsListResult>("projects_list", { args });
}

export function getProject(id: number): Promise<ProjectGetResult> {
  const args: ProjectGetArgs = { id };
  return invoke<ProjectGetResult>("projects_get", { args });
}

export function registerProject(args: ProjectRegisterArgs): Promise<ProjectRegisterResult> {
  return invoke<ProjectRegisterResult>("projects_register", { args });
}

export function checkProjectPath(rootPath: string): Promise<ProjectCheckPathResult> {
  const args: ProjectCheckPathArgs = { root_path: rootPath };
  return invoke<ProjectCheckPathResult>("projects_check_path", { args });
}

export function setProjectPinned(id: number, pinned: boolean): Promise<ProjectGetResult> {
  const args: ProjectSetPinnedArgs = { id, pinned };
  return invoke<ProjectGetResult>("projects_set_pinned", { args });
}

export function pickProjectFolder(): Promise<ProjectPickFolderResult> {
  const args: ProjectPickFolderArgs = {};
  return invoke<ProjectPickFolderResult>("projects_pick_folder", { args });
}

export function listEditorProfiles(): Promise<EditorProfilesListResult> {
  const args: EditorProfilesListArgs = {};
  return invoke<EditorProfilesListResult>("editor_profiles_list", { args });
}

export function saveEditorProfiles(
  profiles: EditorProfileInputArgs[],
): Promise<EditorProfilesSaveResult> {
  const args: EditorProfilesSaveArgs = { profiles };
  return invoke<EditorProfilesSaveResult>("editor_profiles_save", { args });
}

export function setDefaultEditor(kind: string): Promise<EditorSetDefaultResult> {
  const args: EditorSetDefaultArgs = { kind };
  return invoke<EditorSetDefaultResult>("editor_profiles_set_default", { args });
}

export function openInEditor(itemPath: string, kind: string | null): Promise<EditorOpenResult> {
  const args: EditorOpenArgs = { item_path: itemPath, kind };
  return invoke<EditorOpenResult>("editor_open", { args });
}

export function pickExecutable(): Promise<EditorPickExecutableResult> {
  const args: EditorPickExecutableArgs = {};
  return invoke<EditorPickExecutableResult>("editor_pick_executable", { args });
}

export function pickItem(folder: boolean): Promise<EditorPickItemResult> {
  const args: EditorPickItemArgs = { folder };
  return invoke<EditorPickItemResult>("editor_pick_item", { args });
}

export function listWorkspaces(): Promise<WorkspacesListResult> {
  const args: WorkspacesListArgs = {};
  return invoke<WorkspacesListResult>("workspaces_list", { args });
}

export function getWorkspace(id: number): Promise<WorkspaceResult> {
  const args: WorkspaceGetArgs = { id };
  return invoke<WorkspaceResult>("workspaces_get", { args });
}

export function createWorkspace(name: string): Promise<WorkspaceResult> {
  const args: WorkspaceCreateArgs = { name };
  return invoke<WorkspaceResult>("workspaces_create", { args });
}

export function renameWorkspace(id: number, name: string): Promise<WorkspaceResult> {
  const args: WorkspaceRenameArgs = { id, name };
  return invoke<WorkspaceResult>("workspaces_rename", { args });
}

export function deleteWorkspace(id: number): Promise<WorkspaceDeleteResult> {
  const args: WorkspaceDeleteArgs = { id };
  return invoke<WorkspaceDeleteResult>("workspaces_delete", { args });
}

export function addWorkspaceItem(args: ItemAddArgs): Promise<WorkspaceResult> {
  return invoke<WorkspaceResult>("workspace_items_add", { args });
}

export function updateWorkspaceItem(args: ItemUpdateArgs): Promise<WorkspaceResult> {
  return invoke<WorkspaceResult>("workspace_items_update", { args });
}

export function removeWorkspaceItem(itemId: number): Promise<WorkspaceResult> {
  const args: ItemRemoveArgs = { item_id: itemId };
  return invoke<WorkspaceResult>("workspace_items_remove", { args });
}

export function moveWorkspaceItem(itemId: number, up: boolean): Promise<WorkspaceResult> {
  const args: ItemMoveArgs = { item_id: itemId, up };
  return invoke<WorkspaceResult>("workspace_items_move", { args });
}

export function pickWorkspaceItem(folder: boolean): Promise<WorkspacePickResult> {
  const args: WorkspacePickItemArgs = { folder };
  return invoke<WorkspacePickResult>("workspaces_pick_item", { args });
}

export function restoreWorkspace(id: number): Promise<WorkspaceRestoreResult> {
  const args: WorkspaceRestoreArgs = { id };
  return invoke<WorkspaceRestoreResult>("workspace_restore", { args });
}

export function relinkWorkspaceItem(itemId: number, path: string): Promise<WorkspaceResult> {
  const args: ItemRelinkArgs = { item_id: itemId, path };
  return invoke<WorkspaceResult>("workspace_items_relink", { args });
}
