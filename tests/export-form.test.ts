import { describe, expect, it } from "vitest";
import { canSubmitExport, exportSuccessText, formatExportError } from "../src/lib/export-form";
import type { ExportErrorCode } from "../src/types/commands";

describe("canSubmitExport", () => {
  it("needs only the acknowledgement when no vault exists", () => {
    expect(canSubmitExport(false, "", false)).toBe(false);
    expect(canSubmitExport(false, "", true)).toBe(true);
    expect(canSubmitExport(false, "ignored", true)).toBe(true);
  });

  it("also needs a non-empty password when a vault exists", () => {
    expect(canSubmitExport(true, "", true)).toBe(false);
    expect(canSubmitExport(true, "pw", false)).toBe(false);
    expect(canSubmitExport(true, "", false)).toBe(false);
    expect(canSubmitExport(true, "pw", true)).toBe(true);
  });
});

describe("formatExportError", () => {
  it("names a wrong master password", () => {
    expect(formatExportError("export_wrong_password")).toBe("Wrong master password.");
  });

  it("gives non-empty copy for every code", () => {
    const codes: ExportErrorCode[] = [
      "export_ack_required",
      "export_password_required",
      "export_wrong_password",
      "export_secret_unreadable",
      "export_note_unreadable",
      "export_note_key_missing",
      "export_keyring_unavailable",
      "export_write_failed",
      "export_storage_unavailable",
    ];
    for (const code of codes) {
      expect(formatExportError(code)).not.toBe("");
    }
  });
});

describe("exportSuccessText", () => {
  it("names the folder and file count and states the files are not encrypted", () => {
    expect(exportSuccessText("C:\\out\\devshelf-export-20260101-000000", 7)).toBe(
      "Exported 7 files to C:\\out\\devshelf-export-20260101-000000. These files are not encrypted.",
    );
  });
});
