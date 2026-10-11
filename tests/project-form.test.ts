import { describe, expect, it } from "vitest";
import { formatProjectError } from "../src/lib/project-form";

describe("formatProjectError", () => {
  const path = "F:\\work\\app";

  it("path_empty (AC-10)", () => {
    expect(formatProjectError("path_empty", null, "")).toBe("Enter or pick a project folder.");
  });

  it("path_not_found uses the form path (AC-11)", () => {
    expect(formatProjectError("path_not_found", null, path)).toBe(
      "Folder `F:\\work\\app` was not found.",
    );
  });

  it("path_not_found prefers detail when present", () => {
    expect(formatProjectError("path_not_found", "G:\\x", path)).toBe(
      "Folder `G:\\x` was not found.",
    );
  });

  it("path_is_file uses the form path when detail is absent (AC-11)", () => {
    expect(formatProjectError("path_is_file", null, path)).toBe(
      "`F:\\work\\app` is a file. Pick the project's root folder.",
    );
  });

  it("path_is_file prefers detail when present", () => {
    expect(formatProjectError("path_is_file", "G:\\x", path)).toBe(
      "`G:\\x` is a file. Pick the project's root folder.",
    );
  });

  it("already_registered uses the existing name from detail (AC-12)", () => {
    expect(formatProjectError("already_registered", "Billing API", path)).toBe(
      "`F:\\work\\app` is already registered as Billing API.",
    );
  });

  it("name_empty (AC-13)", () => {
    expect(formatProjectError("name_empty", null, path)).toBe("Enter a display name.");
  });

  it("name_too_long (AC-14)", () => {
    expect(formatProjectError("name_too_long", null, path)).toBe("Use at most 100 characters.");
  });

  it("path_too_long (AC-14)", () => {
    expect(formatProjectError("path_too_long", null, path)).toBe(
      "This path is too long for Windows.",
    );
  });

  it("tag_invalid uses the tag from detail (AC-15)", () => {
    expect(formatProjectError("tag_invalid", "bad tag!", path)).toBe(
      "Tag `bad tag!` can use letters, digits, - _ and . only (max 32).",
    );
  });

  it("too_many_tags (AC-15)", () => {
    expect(formatProjectError("too_many_tags", null, path)).toBe("Use at most 10 tags.");
  });

  it("project_not_found (AC-43)", () => {
    expect(formatProjectError("project_not_found", null, "")).toBe(
      "This project no longer exists.",
    );
  });

  it("type_invalid and storage_unavailable return non-empty copy", () => {
    expect(formatProjectError("type_invalid", null, path)).not.toBe("");
    expect(formatProjectError("storage_unavailable", null, path)).not.toBe("");
  });
});
