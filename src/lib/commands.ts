import { invoke } from "@tauri-apps/api/core";
import type { AppInfo } from "../types/commands";

export function appInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}
