import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
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
