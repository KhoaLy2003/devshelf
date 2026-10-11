import type { ProjectErrorCode } from "../types/commands";

// `detail` is the existing name or invalid tag from Rust; `path` is the form's path.
export function formatProjectError(
  code: ProjectErrorCode,
  detail: string | null,
  path: string,
): string {
  switch (code) {
    case "path_empty":
      return "Enter or pick a project folder.";
    case "path_not_found":
      return `Folder \`${detail ?? path}\` was not found.`;
    case "path_is_file":
      return `\`${detail ?? path}\` is a file. Pick the project's root folder.`;
    case "already_registered":
      return `\`${path}\` is already registered as ${detail ?? "another project"}.`;
    case "name_empty":
      return "Enter a display name.";
    case "name_too_long":
      return "Use at most 100 characters.";
    case "path_too_long":
      return "This path is too long for Windows.";
    case "tag_invalid":
      return `Tag \`${detail ?? ""}\` can use letters, digits, - _ and . only (max 32).`;
    case "too_many_tags":
      return "Use at most 10 tags.";
    case "type_invalid":
      return "Pick a project type from the list.";
    case "project_not_found":
      return "This project no longer exists.";
    case "storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}
