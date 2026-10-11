import type { StorageErrorCode } from "../types/commands";

export function storageErrorMessage(code: StorageErrorCode, folder: string): string {
  switch (code) {
    case "schema_too_new":
      return "This data was created by a newer version of DevShelf. Update DevShelf to open it.";
    case "storage_unavailable":
      return `DevShelf cannot write to ${folder === "" ? "the data folder" : folder}. Check that the folder exists and that your Windows account can write to it.`;
  }
}
