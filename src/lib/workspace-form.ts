import type { WorkspaceCode, WorkspaceItemKind } from "../types/commands";

// `detail` carries the item path for path codes; `name` is the workspace name for name_taken.
export function formatWorkspaceError(
  code: WorkspaceCode,
  detail: string | null,
  name: string,
): string {
  switch (code) {
    case "name_empty":
      return "Enter a workspace name.";
    case "name_too_long":
      return "Use at most 100 characters.";
    case "name_taken":
      return `A workspace named \`${name}\` already exists.`;
    case "workspace_not_found":
      return "This workspace no longer exists.";
    case "item_path_empty":
      return "Enter a file or folder path.";
    case "item_path_too_long":
      return "This path is too long for Windows.";
    case "item_relative_path":
      return "Enter an absolute path, for example `F:\\work\\notes.txt`.";
    case "item_path_not_found":
      return `\`${detail ?? ""}\` was not found.`;
    case "item_path_unreachable":
      return `\`${detail ?? ""}\` can't be reached. The drive may not be ready or the network path is unavailable.`;
    case "item_duplicate":
      return `\`${detail ?? ""}\` is already in this workspace.`;
    case "item_not_found":
      return "This item no longer exists.";
    case "project_not_found":
      return "That project no longer exists. Choose another project or No project.";
    case "profile_not_found":
      return "That editor profile no longer exists.";
    case "move_blocked":
      return "This item is already at that end of the list.";
    case "item_kind_mismatch":
      return detail === "folder"
        ? "This is a file. Pick a folder."
        : "This is a folder. Pick a file.";
    case "storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

export function kindLabel(kind: WorkspaceItemKind): string {
  return kind === "file" ? "File" : "Folder";
}
