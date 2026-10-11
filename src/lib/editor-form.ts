import type { EditorCode } from "../types/commands";

// Save-time copy (Settings profile form). `detail` carries the path where the copy names it.
export function formatEditorError(
  code: EditorCode,
  _kind: string | null,
  name: string,
  detail: string | null,
): string {
  switch (code) {
    case "profile_path_empty":
      return "Enter the path to the editor executable.";
    case "executable_not_found":
      return `Can't find \`${detail ?? ""}\`. Check the path or pick the editor again.`;
    case "executable_invalid":
      return "Pick an editor program (.exe).";
    case "path_unsafe":
      return "This path has characters that are not allowed in an editor path.";
    case "name_too_long":
      return "Use at most 40 characters.";
    case "item_not_found":
      return `\`${detail ?? ""}\` was not found. It may have moved or been renamed.`;
    case "launch_failed":
      return `${name} could not start. Check the path in Settings.`;
    case "profile_not_found":
      return "That editor profile no longer exists.";
    case "storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

// Open-time copy: a stored executable that is missing, invalid or unsafe is a launch failure.
export function formatOpenError(
  code: EditorCode,
  kind: string | null,
  name: string,
  detail: string | null,
): string {
  switch (code) {
    case "profile_path_empty":
      return `Set the path to ${name} in Settings to open files with it.`;
    case "executable_not_found":
    case "executable_invalid":
    case "path_unsafe":
      return formatEditorError("launch_failed", kind, name, detail);
    default:
      return formatEditorError(code, kind, name, detail);
  }
}

export function openSuccessMessage(fileName: string, profileName: string): string {
  return `Opened ${fileName} in ${profileName}.`;
}

export function fileNameOf(path: string): string {
  const trimmed = path.trim();
  const stripped = trimmed.replace(/[\\/]+$/, "");
  const parts = stripped.split(/[\\/]/);
  return parts[parts.length - 1] || trimmed;
}
