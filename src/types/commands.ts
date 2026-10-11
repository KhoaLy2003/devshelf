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
