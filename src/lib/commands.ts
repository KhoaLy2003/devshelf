import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ConfigAddArgs,
  ConfigEntryResult,
  ConfigListArgs,
  ConfigListResult,
  ConfigOpenArgs,
  ConfigOpenResult,
  ConfigPickArgs,
  ConfigPickResult,
  ConfigRelinkArgs,
  ConfigRemoveArgs,
  ConfigRemoveResult,
  HomeOverviewArgs,
  HomeOverviewResult,
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
  NoteCreateArgs,
  NoteDeleteArgs,
  NoteDeleteResult,
  NoteGetArgs,
  NoteResult,
  NoteSaveArgs,
  NoteSaveResult,
  NotesListArgs,
  NotesListResult,
  SecretCreateArgs,
  SecretDeleteResult,
  SecretIdArgs,
  SecretListArgs,
  SecretListResult,
  SecretResult,
  SecretRevealResult,
  SecretCopyResult,
  SecretUpdateArgs,
  VaultLockArgs,
  VaultSetAutoLockArgs,
  VaultSetupArgs,
  VaultStatusArgs,
  VaultStatusResult,
  VaultTouchArgs,
  VaultUnlockArgs,
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

export function getHomeOverview(): Promise<HomeOverviewResult> {
  const args: HomeOverviewArgs = {};
  return invoke<HomeOverviewResult>("home_overview", { args });
}

export function listConfigFiles(): Promise<ConfigListResult> {
  const args: ConfigListArgs = {};
  return invoke<ConfigListResult>("config_files_list", { args });
}

export function addConfigFile(args: ConfigAddArgs): Promise<ConfigEntryResult> {
  return invoke<ConfigEntryResult>("config_files_add", { args });
}

export function relinkConfigFile(id: number, path: string): Promise<ConfigEntryResult> {
  const args: ConfigRelinkArgs = { id, path };
  return invoke<ConfigEntryResult>("config_files_relink", { args });
}

export function removeConfigFile(id: number): Promise<ConfigRemoveResult> {
  const args: ConfigRemoveArgs = { id };
  return invoke<ConfigRemoveResult>("config_files_remove", { args });
}

export function openConfigFile(id: number, editorKind: string | null): Promise<ConfigOpenResult> {
  const args: ConfigOpenArgs = { id, editor_kind: editorKind };
  return invoke<ConfigOpenResult>("config_files_open", { args });
}

export function pickConfigFile(): Promise<ConfigPickResult> {
  const args: ConfigPickArgs = {};
  return invoke<ConfigPickResult>("config_files_pick", { args });
}

export function listNotes(): Promise<NotesListResult> {
  const args: NotesListArgs = {};
  return invoke<NotesListResult>("notes_list", { args });
}

export function getNote(id: number): Promise<NoteResult> {
  const args: NoteGetArgs = { id };
  return invoke<NoteResult>("notes_get", { args });
}

export function createNote(projectId: number | null): Promise<NoteResult> {
  const args: NoteCreateArgs = { project_id: projectId };
  return invoke<NoteResult>("notes_create", { args });
}

export function saveNote(args: NoteSaveArgs): Promise<NoteSaveResult> {
  return invoke<NoteSaveResult>("notes_save", { args });
}

export function deleteNote(id: number): Promise<NoteDeleteResult> {
  const args: NoteDeleteArgs = { id };
  return invoke<NoteDeleteResult>("notes_delete", { args });
}

export function vaultStatus(): Promise<VaultStatusResult> {
  const args: VaultStatusArgs = {};
  return invoke<VaultStatusResult>("vault_status", { args });
}

export function vaultSetup(args: VaultSetupArgs): Promise<VaultStatusResult> {
  return invoke<VaultStatusResult>("vault_setup", { args });
}

export function vaultUnlock(password: string): Promise<VaultStatusResult> {
  const args: VaultUnlockArgs = { password };
  return invoke<VaultStatusResult>("vault_unlock", { args });
}

export function vaultLock(): Promise<VaultStatusResult> {
  const args: VaultLockArgs = {};
  return invoke<VaultStatusResult>("vault_lock", { args });
}

export function vaultTouch(): Promise<VaultStatusResult> {
  const args: VaultTouchArgs = {};
  return invoke<VaultStatusResult>("vault_touch", { args });
}

export function vaultSetAutoLock(minutes: number): Promise<VaultStatusResult> {
  const args: VaultSetAutoLockArgs = { minutes };
  return invoke<VaultStatusResult>("vault_set_auto_lock", { args });
}

export function secretsList(args: SecretListArgs): Promise<SecretListResult> {
  return invoke<SecretListResult>("secrets_list", { args });
}

export function secretsCreate(args: SecretCreateArgs): Promise<SecretResult> {
  return invoke<SecretResult>("secrets_create", { args });
}

export function secretsUpdate(args: SecretUpdateArgs): Promise<SecretResult> {
  return invoke<SecretResult>("secrets_update", { args });
}

export function secretsReveal(id: number): Promise<SecretRevealResult> {
  const args: SecretIdArgs = { id };
  return invoke<SecretRevealResult>("secrets_reveal", { args });
}

export function secretsCopy(id: number): Promise<SecretCopyResult> {
  const args: SecretIdArgs = { id };
  return invoke<SecretCopyResult>("secrets_copy", { args });
}

export function secretsDelete(id: number): Promise<SecretDeleteResult> {
  const args: SecretIdArgs = { id };
  return invoke<SecretDeleteResult>("secrets_delete", { args });
}
