import { describe, expect, it } from "vitest";
import {
  formatCountdown,
  formatVaultError,
  lockReasonText,
  validateSetup,
} from "../src/lib/vault-form";

describe("validateSetup", () => {
  const good = "correct horse battery";

  it("requires a password", () => {
    expect(validateSetup("", "", true)).toBe("vault_password_required");
  });

  it("rejects a mismatch (AC-02)", () => {
    expect(validateSetup(good, good + "x", true)).toBe("vault_password_mismatch");
  });

  it("rejects short passwords (AC-03)", () => {
    expect(validateSetup("a".repeat(11), "a".repeat(11), true)).toBe("vault_password_too_short");
  });

  it("accepts exactly 12 characters", () => {
    expect(validateSetup("a".repeat(12), "a".repeat(12), true)).toBeNull();
  });

  it("requires the acknowledgement (AC-04)", () => {
    expect(validateSetup(good, good, false)).toBe("vault_password_required");
  });

  it("accepts a valid form", () => {
    expect(validateSetup(good, good, true)).toBeNull();
  });
});

describe("lockReasonText", () => {
  it("names the launch reason", () => {
    expect(lockReasonText("launch", 5)).toBe("Locked because DevShelf just started.");
  });

  it("names the manual reason", () => {
    expect(lockReasonText("manual", 5)).toBe("You locked the vault.");
  });

  it("states the configured duration for inactivity (AC-18)", () => {
    expect(lockReasonText("inactivity", 5)).toBe("Locked after 5 minutes of inactivity.");
    expect(lockReasonText("inactivity", 1)).toBe("Locked after 1 minute of inactivity.");
    expect(lockReasonText("inactivity", 15)).toBe("Locked after 15 minutes of inactivity.");
  });

  it("falls back to launch copy for a null reason", () => {
    expect(lockReasonText(null, 5)).toBe("Locked because DevShelf just started.");
  });
});

describe("formatCountdown", () => {
  it("formats boundaries", () => {
    expect(formatCountdown(59)).toBe("0:59");
    expect(formatCountdown(60)).toBe("1:00");
    expect(formatCountdown(0)).toBe("0:00");
    expect(formatCountdown(5)).toBe("0:05");
  });

  it("clamps negatives", () => {
    expect(formatCountdown(-3)).toBe("0:00");
  });
});

describe("formatVaultError", () => {
  it("uses the spec copy", () => {
    expect(formatVaultError("vault_password_mismatch")).toBe("Passwords don't match.");
    expect(formatVaultError("vault_password_too_short")).toBe("Use at least 12 characters.");
    expect(formatVaultError("vault_password_required")).toBe("Enter your master password.");
    expect(formatVaultError("vault_wrong_password")).toBe("Wrong master password.");
  });
});
