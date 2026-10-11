import { describe, expect, it } from "vitest";
import { formatWorkspaceError, kindLabel } from "../src/lib/workspace-form";
import { WORKSPACE_ITEM_KINDS } from "../src/types/commands";

describe("formatWorkspaceError", () => {
  it("name_empty", () => {
    expect(formatWorkspaceError("name_empty", null, "")).toBe("Enter a workspace name.");
  });

  it("name_too_long", () => {
    expect(formatWorkspaceError("name_too_long", null, "x")).toBe("Use at most 100 characters.");
  });

  it("name_taken inserts the name", () => {
    expect(formatWorkspaceError("name_taken", null, "Ops")).toBe(
      "A workspace named `Ops` already exists.",
    );
  });

  it("workspace_not_found", () => {
    expect(formatWorkspaceError("workspace_not_found", null, "Ops")).toBe(
      "This workspace no longer exists.",
    );
  });

  it("item_path_empty", () => {
    expect(formatWorkspaceError("item_path_empty", null, "")).toBe("Enter a file or folder path.");
  });

  it("item_path_too_long", () => {
    expect(formatWorkspaceError("item_path_too_long", null, "")).toBe(
      "This path is too long for Windows.",
    );
  });

  it("item_relative_path", () => {
    expect(formatWorkspaceError("item_relative_path", null, "")).toBe(
      "Enter an absolute path, for example `F:\\work\\notes.txt`.",
    );
  });

  it("item_path_not_found inserts the detail path", () => {
    expect(formatWorkspaceError("item_path_not_found", "F:\\missing\\a.txt", "Ops")).toBe(
      "`F:\\missing\\a.txt` was not found.",
    );
  });

  it("item_duplicate inserts the detail path", () => {
    expect(formatWorkspaceError("item_duplicate", "F:\\work\\notes.txt", "Ops")).toBe(
      "`F:\\work\\notes.txt` is already in this workspace.",
    );
  });

  it("path codes tolerate a missing detail", () => {
    expect(formatWorkspaceError("item_path_not_found", null, "Ops")).toBe("`` was not found.");
    expect(formatWorkspaceError("item_duplicate", null, "Ops")).toBe(
      "`` is already in this workspace.",
    );
  });

  it("item_not_found", () => {
    expect(formatWorkspaceError("item_not_found", null, "Ops")).toBe("This item no longer exists.");
  });

  it("project_not_found", () => {
    expect(formatWorkspaceError("project_not_found", null, "Ops")).toBe(
      "That project no longer exists. Choose another project or No project.",
    );
  });

  it("profile_not_found", () => {
    expect(formatWorkspaceError("profile_not_found", null, "Ops")).toBe(
      "That editor profile no longer exists.",
    );
  });

  it("move_blocked", () => {
    expect(formatWorkspaceError("move_blocked", null, "Ops")).toBe(
      "This item is already at that end of the list.",
    );
  });

  it("storage_unavailable", () => {
    expect(formatWorkspaceError("storage_unavailable", null, "Ops")).toBe(
      "DevShelf cannot reach its data folder. Restart DevShelf and try again.",
    );
  });
});

describe("kindLabel", () => {
  it("labels file and folder", () => {
    expect(kindLabel("file")).toBe("File");
    expect(kindLabel("folder")).toBe("Folder");
  });

  it("covers every WORKSPACE_ITEM_KINDS entry", () => {
    expect(WORKSPACE_ITEM_KINDS.map(kindLabel)).toEqual(["File", "Folder"]);
  });
});
