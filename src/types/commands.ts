// Single shared file for Rust <-> TypeScript command types (constitution, principle 3).

export interface AppInfo {
  name: string;
  version: string;
}

export type ErrorCode = "internal";

export interface CommandError {
  code: ErrorCode;
}
