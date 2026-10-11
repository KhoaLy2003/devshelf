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
