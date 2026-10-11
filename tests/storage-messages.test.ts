import { describe, expect, it } from "vitest";
import { storageErrorMessage } from "../src/lib/storage-messages";

describe("storageErrorMessage", () => {
  it("returns the AC-24 sentence for schema_too_new", () => {
    expect(storageErrorMessage("schema_too_new", "C:\\data")).toBe(
      "This data was created by a newer version of DevShelf. Update DevShelf to open it.",
    );
  });

  it("returns the AC-25 sentence with the folder for storage_unavailable", () => {
    const folder = "C:\\Users\\x\\AppData\\Roaming\\devshelf";
    expect(storageErrorMessage("storage_unavailable", folder)).toBe(
      `DevShelf cannot write to ${folder}. Check that the folder exists and that your Windows account can write to it.`,
    );
  });

  it("uses 'the data folder' when the folder is empty", () => {
    expect(storageErrorMessage("storage_unavailable", "")).toBe(
      "DevShelf cannot write to the data folder. Check that the folder exists and that your Windows account can write to it.",
    );
  });
});
