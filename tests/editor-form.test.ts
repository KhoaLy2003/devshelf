import { describe, expect, it } from "vitest";
import {
  fileNameOf,
  formatEditorError,
  formatOpenError,
  openSuccessMessage,
} from "../src/lib/editor-form";

describe("formatEditorError (save context)", () => {
  it("profile_path_empty", () => {
    expect(formatEditorError("profile_path_empty", "vs_code", "VS Code", null)).toBe(
      "Enter the path to the editor executable.",
    );
  });

  it("executable_not_found names the path from detail", () => {
    expect(formatEditorError("executable_not_found", "vs_code", "VS Code", "C:\\x\\code.exe")).toBe(
      "Can't find `C:\\x\\code.exe`. Check the path or pick the editor again.",
    );
  });

  it("executable_invalid", () => {
    expect(formatEditorError("executable_invalid", "custom", "Custom", null)).toBe(
      "Pick an editor program (.exe).",
    );
  });

  it("path_unsafe", () => {
    expect(formatEditorError("path_unsafe", "custom", "Custom", null)).toBe(
      "This path has characters that are not allowed in an editor path.",
    );
  });

  it("name_too_long", () => {
    expect(formatEditorError("name_too_long", "custom", "Custom", null)).toBe(
      "Use at most 40 characters.",
    );
  });

  it("item_not_found names the path from detail", () => {
    expect(formatEditorError("item_not_found", null, "", "D:\\notes\\a.txt")).toBe(
      "`D:\\notes\\a.txt` was not found. It may have moved or been renamed.",
    );
  });

  it("launch_failed names the profile", () => {
    expect(formatEditorError("launch_failed", "vs_code", "VS Code", null)).toBe(
      "VS Code could not start. Check the path in Settings.",
    );
  });

  it("profile_not_found", () => {
    expect(formatEditorError("profile_not_found", null, "", null)).toBe(
      "That editor profile no longer exists.",
    );
  });

  it("storage_unavailable", () => {
    expect(formatEditorError("storage_unavailable", null, "", null)).toBe(
      "DevShelf cannot reach its data folder. Restart DevShelf and try again.",
    );
  });
});

describe("formatOpenError (open context)", () => {
  it("profile_path_empty names the profile", () => {
    expect(formatOpenError("profile_path_empty", "notepad_plus_plus", "Notepad++", null)).toBe(
      "Set the path to Notepad++ in Settings to open files with it.",
    );
  });

  it("differs from the save-time copy", () => {
    expect(formatOpenError("profile_path_empty", "vs_code", "VS Code", null)).not.toBe(
      formatEditorError("profile_path_empty", "vs_code", "VS Code", null),
    );
  });

  it("other codes match the save-time copy", () => {
    expect(formatOpenError("launch_failed", "vs_code", "VS Code", null)).toBe(
      formatEditorError("launch_failed", "vs_code", "VS Code", null),
    );
    expect(formatOpenError("item_not_found", null, "", "C:\\a.txt")).toBe(
      formatEditorError("item_not_found", null, "", "C:\\a.txt"),
    );
  });

  it.each(["executable_not_found", "executable_invalid", "path_unsafe", "launch_failed"] as const)(
    "%s shows the launch-failed copy",
    (code) => {
      expect(formatOpenError(code, "vs_code", "VS Code", "C:\\x\\code.exe")).toBe(
        "VS Code could not start. Check the path in Settings.",
      );
    },
  );

  it("item_not_found names the item path", () => {
    expect(formatOpenError("item_not_found", null, "VS Code", "D:\\a.txt")).toBe(
      "`D:\\a.txt` was not found. It may have moved or been renamed.",
    );
  });
});

describe("openSuccessMessage", () => {
  it("names file and profile", () => {
    expect(openSuccessMessage("a.txt", "VS Code")).toBe("Opened a.txt in VS Code.");
  });
});

describe("fileNameOf", () => {
  it("splits on backslash", () => {
    expect(fileNameOf("D:\\my notes\\a b.txt")).toBe("a b.txt");
  });

  it("splits on slash", () => {
    expect(fileNameOf("D:/my notes/a b.txt")).toBe("a b.txt");
  });

  it("splits on mixed separators", () => {
    expect(fileNameOf("D:\\my/notes\\a.txt")).toBe("a.txt");
  });

  it("returns the input when there is no separator", () => {
    expect(fileNameOf("a.txt")).toBe("a.txt");
  });

  it("drops trailing separators", () => {
    expect(fileNameOf("D:\\proj\\")).toBe("proj");
    expect(fileNameOf("D:/proj/")).toBe("proj");
  });
});
