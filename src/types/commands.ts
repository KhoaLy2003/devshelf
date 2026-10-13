// Single shared file for Rust <-> TypeScript command types (constitution, principle 3).

export interface AppInfo {
  name: string;
  version: string;
}

export type ErrorCode = "internal";

export interface CommandError {
  code: ErrorCode;
}

export type AppInfoArgs = Record<string, never>;

export type StorageStatusArgs = Record<string, never>;

export type StorageErrorCode = "storage_unavailable" | "schema_too_new";

export interface StorageStatus {
  ok: boolean;
  folder: string;
  error_code: StorageErrorCode | null;
}

export type ProjectErrorCode =
  | "path_empty"
  | "path_not_found"
  | "path_is_file"
  | "already_registered"
  | "name_empty"
  | "name_too_long"
  | "path_too_long"
  | "tag_invalid"
  | "too_many_tags"
  | "type_invalid"
  | "storage_unavailable"
  | "project_not_found";

// Rust (projects::PROJECT_TYPES) is authoritative; keep this list in sync.
export const PROJECT_TYPES = ["Spring Boot", "React", "Docker/Infra", "Node", "Other"] as const;

export type ProjectType = (typeof PROJECT_TYPES)[number];

export interface ProjectView {
  id: number;
  display_name: string;
  root_path: string;
  drive: string;
  project_type: ProjectType;
  tags: string[];
  pinned: boolean;
  created_at: string;
}

export type ProjectsListArgs = Record<string, never>;

export interface ProjectsListResult {
  projects: ProjectView[];
  error: ProjectErrorCode | null;
}

export interface ProjectGetArgs {
  id: number;
}

export interface ProjectGetResult {
  project: ProjectView | null;
  error: ProjectErrorCode | null;
}

export interface ProjectRegisterArgs {
  root_path: string;
  display_name: string;
  project_type: ProjectType;
  tags_raw: string;
}

export interface ProjectRegisterResult {
  project: ProjectView | null;
  error: ProjectErrorCode | null;
  detail: string | null;
}

export interface ProjectCheckPathArgs {
  root_path: string;
}

export interface ProjectCheckPathResult {
  nested_in: string | null;
  error: ProjectErrorCode | null;
}

export interface ProjectSetPinnedArgs {
  id: number;
  pinned: boolean;
}

export type ProjectPickFolderArgs = Record<string, never>;

export interface ProjectPickFolderResult {
  path: string | null;
}

export type EditorCode =
  | "profile_not_found"
  | "profile_path_empty"
  | "executable_not_found"
  | "executable_invalid"
  | "path_unsafe"
  | "name_too_long"
  | "item_not_found"
  | "launch_failed"
  | "storage_unavailable";

// Rust (editors::KINDS) is authoritative; keep this list in sync.
export const EDITOR_KINDS = ["windows_default", "notepad_plus_plus", "vs_code", "custom"] as const;

export type EditorKind = (typeof EDITOR_KINDS)[number];

export interface EditorProfileView {
  kind: string;
  name: string;
  exe_path: string;
  is_default: boolean;
}

export interface EditorProfileInputArgs {
  kind: string;
  name: string;
  exe_path: string;
}

export interface EditorProfilesResult {
  profiles: EditorProfileView[];
  error: EditorCode | null;
  kind: string | null;
  detail: string | null;
}

export type EditorProfilesListArgs = Record<string, never>;

export type EditorProfilesListResult = EditorProfilesResult;

export interface EditorProfilesSaveArgs {
  profiles: EditorProfileInputArgs[];
}

export type EditorProfilesSaveResult = EditorProfilesResult;

export interface EditorSetDefaultArgs {
  kind: string;
}

export type EditorSetDefaultResult = EditorProfilesResult;

export interface EditorOpenArgs {
  item_path: string;
  kind: string | null;
}

export interface EditorOpenResult {
  opened_name: string | null;
  error: EditorCode | null;
  kind: string | null;
}

export type EditorPickExecutableArgs = Record<string, never>;

export interface EditorPickExecutableResult {
  path: string | null;
}

export interface EditorPickItemArgs {
  folder: boolean;
}

export interface EditorPickItemResult {
  path: string | null;
}

export type WorkspaceCode =
  | "name_empty"
  | "name_too_long"
  | "name_taken"
  | "workspace_not_found"
  | "item_path_empty"
  | "item_path_too_long"
  | "item_relative_path"
  | "item_path_not_found"
  | "item_path_unreachable"
  | "item_duplicate"
  | "item_not_found"
  | "project_not_found"
  | "profile_not_found"
  | "move_blocked"
  | "item_kind_mismatch"
  | "storage_unavailable";

// Rust (workspaces::Item.kind) is authoritative; keep this list in sync.
export const WORKSPACE_ITEM_KINDS = ["file", "folder"] as const;

export type WorkspaceItemKind = (typeof WORKSPACE_ITEM_KINDS)[number];

export interface WorkspaceItemView {
  id: number;
  position: number;
  kind: WorkspaceItemKind;
  path: string;
  project_id: number | null;
  editor_kind: string;
}

export interface WorkspaceView {
  id: number;
  name: string;
  items: WorkspaceItemView[];
  last_restored_at: string | null;
}

export interface WorkspaceSummaryView {
  id: number;
  name: string;
  item_count: number;
  project_names: string[];
  last_restored_at: string | null;
}

export type WorkspacesListArgs = Record<string, never>;

export interface WorkspacesListResult {
  workspaces: WorkspaceSummaryView[];
  error: WorkspaceCode | null;
  detail: string | null;
}

export interface WorkspaceGetArgs {
  id: number;
}

export interface WorkspaceResult {
  workspace: WorkspaceView | null;
  error: WorkspaceCode | null;
  detail: string | null;
}

export interface WorkspaceCreateArgs {
  name: string;
}

export interface WorkspaceRenameArgs {
  id: number;
  name: string;
}

export interface WorkspaceDeleteArgs {
  id: number;
}

export interface WorkspaceDeleteResult {
  error: WorkspaceCode | null;
  detail: string | null;
}

export interface ItemAddArgs {
  workspace_id: number;
  path: string;
  project_id: number | null;
  editor_kind: string | null;
}

export interface ItemUpdateArgs {
  item_id: number;
  project_id: number | null;
  editor_kind: string;
}

export interface ItemRemoveArgs {
  item_id: number;
}

export interface ItemMoveArgs {
  item_id: number;
  up: boolean;
}

export interface WorkspacePickItemArgs {
  folder: boolean;
}

export interface WorkspacePickResult {
  path: string | null;
}

export type RestoreMissingReason = "not_found" | "now_folder" | "now_file";

export type RestoreOutcomeView =
  | { kind: "opened"; item_id: number; reason: null; code: null }
  | { kind: "missing"; item_id: number; reason: RestoreMissingReason; code: null }
  | { kind: "unreachable"; item_id: number; reason: null; code: null }
  | { kind: "could_not_open"; item_id: number; reason: null; code: EditorCode };

export interface WorkspaceRestoreArgs {
  id: number;
}

export interface WorkspaceRestoreResult {
  outcomes: RestoreOutcomeView[];
  completed: boolean;
  error: WorkspaceCode | null;
  detail: string | null;
}

export interface ItemRelinkArgs {
  item_id: number;
  path: string;
}

export type HomeErrorCode = "storage_unavailable";

export type HomeOverviewArgs = Record<string, never>;

export interface HomeProjectView {
  id: number;
  display_name: string;
  root_path: string;
  drive: string;
}

export interface HomeWorkspaceView {
  id: number;
  name: string;
  item_count: number;
  project_names: string[];
  last_restored_at: string | null;
  last_restore_missing: number;
}

export interface HomeOverviewResult {
  pinned_projects: HomeProjectView[];
  recent_workspaces: HomeWorkspaceView[];
  project_count: number;
  workspace_count: number;
  error: HomeErrorCode | null;
  detail: string | null;
}

export type ConfigCode =
  | "config_not_found"
  | "path_empty"
  | "path_not_found"
  | "path_is_folder"
  | "path_relative"
  | "path_too_long"
  | "duplicate_path"
  | "project_not_found"
  | "environment_invalid"
  | "kind_invalid"
  | "storage_unavailable";

// Rust (config::ENVIRONMENTS) is authoritative; keep this list in sync.
export const CONFIG_ENVIRONMENTS = ["local", "dev", "staging", "production"] as const;

export type ConfigEnvironment = (typeof CONFIG_ENVIRONMENTS)[number];

// Rust (config::KINDS) is authoritative; keep this list in sync.
export const CONFIG_KINDS = [
  "yaml",
  "properties",
  "dotenv",
  "docker_compose",
  "caddy_nginx",
  "json",
] as const;

export type ConfigKind = (typeof CONFIG_KINDS)[number];

export type ConfigStatus = "available" | "missing" | "folder_at_path" | "inaccessible";

export interface ConfigEntryView {
  id: number;
  project_id: number;
  project_name: string | null;
  path: string;
  name: string;
  environment: string;
  kind: string;
  status: ConfigStatus;
}

export type ConfigListArgs = Record<string, never>;

export interface ConfigListResult {
  entries: ConfigEntryView[];
  error: ConfigCode | null;
  detail: string | null;
}

export interface ConfigAddArgs {
  project_id: number;
  path: string;
  environment: string;
  kind: string;
}

export interface ConfigEntryResult {
  entry: ConfigEntryView | null;
  error: ConfigCode | null;
  detail: string | null;
}

export interface ConfigRelinkArgs {
  id: number;
  path: string;
}

export interface ConfigRemoveArgs {
  id: number;
}

export interface ConfigRemoveResult {
  error: ConfigCode | null;
  detail: string | null;
}

export interface ConfigOpenArgs {
  id: number;
  editor_kind: string | null;
}

export interface ConfigOpenResult {
  opened_name: string | null;
  error: EditorCode | null;
  detail: string | null;
}

export type ConfigPickArgs = Record<string, never>;

export interface ConfigPickResult {
  path: string | null;
}

export type NoteCode =
  | "note_not_found"
  | "title_too_long"
  | "body_too_large"
  | "project_not_found"
  | "key_missing"
  | "keyring_unavailable"
  | "body_unreadable"
  | "storage_unavailable";

// Rust (notes::MAX_TITLE_CHARS, notes::MAX_BODY_BYTES) is authoritative; keep these in sync.
export const NOTE_TITLE_MAX = 200;
export const NOTE_BODY_MAX_BYTES = 1_000_000;

export interface NoteSummaryView {
  id: number;
  title: string;
  project_id: number | null;
  project_name: string | null;
  updated_at: string;
}

export interface NoteView {
  id: number;
  title: string;
  project_id: number | null;
  project_name: string | null;
  updated_at: string;
  body: string;
}

export type NotesListArgs = Record<string, never>;

export interface NotesListResult {
  notes: NoteSummaryView[];
  error: NoteCode | null;
  detail: string | null;
}

export interface NoteGetArgs {
  id: number;
}

export interface NoteResult {
  note: NoteView | null;
  error: NoteCode | null;
  detail: string | null;
}

export interface NoteCreateArgs {
  project_id: number | null;
}

export interface NoteSaveArgs {
  id: number;
  title: string;
  project_id: number | null;
  body: string;
}

export interface NoteSaveResult {
  summary: NoteSummaryView | null;
  error: NoteCode | null;
  detail: string | null;
}

export interface NoteDeleteArgs {
  id: number;
}

export interface NoteDeleteResult {
  error: NoteCode | null;
  detail: string | null;
}
