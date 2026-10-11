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
