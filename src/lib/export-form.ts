import type { ExportErrorCode } from "../types/commands";

/** The acknowledgement is always required; the master password only when a vault exists. */
export function canSubmitExport(
  vaultSetUp: boolean,
  password: string,
  acknowledged: boolean,
): boolean {
  return acknowledged && (!vaultSetUp || password !== "");
}

export function formatExportError(code: ExportErrorCode): string {
  switch (code) {
    case "export_ack_required":
      return "Confirm that you understand the export is not encrypted.";
    case "export_password_required":
      return "Enter your master password.";
    case "export_wrong_password":
      return "Wrong master password.";
    case "export_secret_unreadable":
      return "A secret could not be read, so nothing was exported.";
    case "export_note_unreadable":
      return "A note could not be read, so nothing was exported.";
    case "export_note_key_missing":
      return "Notes can't be read because their key is missing from Windows Credential Manager. Nothing was exported.";
    case "export_keyring_unavailable":
      return "DevShelf can't reach Windows Credential Manager. Nothing was exported.";
    case "export_write_failed":
      return "Could not write the files. Pick a different folder and try again.";
    case "export_storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

export function exportSuccessText(folder: string, fileCount: number): string {
  return `Exported ${fileCount} files to ${folder}. These files are not encrypted.`;
}
