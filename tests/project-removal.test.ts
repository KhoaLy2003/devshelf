import { describe, expect, it } from "vitest";
import {
  canConfirmVaultDelete,
  formatProjectRemovalError,
  formatVaultDeleteError,
  removalSummary,
  vaultDeleteMessage,
} from "../src/lib/project-removal";

describe("removalSummary", () => {
  it("is null when both counts are zero", () => {
    expect(removalSummary(0, 0)).toBeNull();
  });

  it("uses singular and plural forms", () => {
    expect(removalSummary(1, 1)).toBe("This also removes 1 config entry and 1 workspace.");
    expect(removalSummary(2, 1)).toBe("This also removes 2 config entries and 1 workspace.");
    expect(removalSummary(1, 3)).toBe("This also removes 1 config entry and 3 workspaces.");
  });

  it("omits the zero part", () => {
    expect(removalSummary(2, 0)).toBe("This also removes 2 config entries.");
    expect(removalSummary(0, 1)).toBe("This also removes 1 workspace.");
    expect(removalSummary(0, 4)).toBe("This also removes 4 workspaces.");
  });
});

describe("vaultDeleteMessage", () => {
  it("states the zero-secret variant", () => {
    expect(vaultDeleteMessage(0)).toBe("Delete the vault? No secrets are stored.");
  });

  it("states the count", () => {
    expect(vaultDeleteMessage(1)).toBe(
      "Delete the vault and its 1 secret? The secret cannot be recovered. Notes are not affected.",
    );
    expect(vaultDeleteMessage(5)).toBe(
      "Delete the vault and all 5 secrets? The secrets cannot be recovered. Notes are not affected.",
    );
  });
});

describe("canConfirmVaultDelete", () => {
  it("is true only for exactly DELETE", () => {
    expect(canConfirmVaultDelete("DELETE")).toBe(true);
    expect(canConfirmVaultDelete("delete")).toBe(false);
    expect(canConfirmVaultDelete("DELETE ")).toBe(false);
    expect(canConfirmVaultDelete(" DELETE")).toBe(false);
    expect(canConfirmVaultDelete("")).toBe(false);
  });
});

describe("error copy", () => {
  it("maps project removal codes", () => {
    expect(formatProjectRemovalError("project_not_found")).toBe("This project is already removed.");
    expect(formatProjectRemovalError("project_storage_unavailable")).toContain("data folder");
  });

  it("maps vault delete codes", () => {
    expect(formatVaultDeleteError("vault_wrong_password")).toBe("Wrong master password.");
    expect(formatVaultDeleteError("vault_password_required")).toBe("Enter your master password.");
    expect(formatVaultDeleteError("vault_not_set_up")).toBe("The vault is not set up yet.");
    expect(formatVaultDeleteError("vault_storage_unavailable")).toContain("unavailable");
  });
});
