import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, AppInfoArgs, StorageStatus, StorageStatusArgs } from "../types/commands";

export function appInfo(): Promise<AppInfo> {
  const args: AppInfoArgs = {};
  return invoke<AppInfo>("app_info", { args });
}

export function storageStatus(): Promise<StorageStatus> {
  const args: StorageStatusArgs = {};
  return invoke<StorageStatus>("storage_status", { args });
}
