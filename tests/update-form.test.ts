import { describe, expect, it, vi } from "vitest";
import type { UpdateErrorCode, UpdateCheckResult, UpdatePrefsResult } from "../src/types/commands";
import { formatUpdateError, startupUpdateCheck } from "../src/lib/update-form";

const enabled: UpdatePrefsResult = { auto_check: true, error: null };
const disabled: UpdatePrefsResult = { auto_check: false, error: null };
const available: UpdateCheckResult = {
  update: { version: "0.2.1", notes: "Fixes" },
  error: null,
};

describe("startupUpdateCheck", () => {
  it("does not call check when the launch check is disabled", async () => {
    const check = vi.fn<() => Promise<UpdateCheckResult>>();
    const view = await startupUpdateCheck({ getPrefs: async () => disabled, check });
    expect(view).toBeNull();
    expect(check).not.toHaveBeenCalled();
  });

  it("returns the available update when enabled", async () => {
    const view = await startupUpdateCheck({
      getPrefs: async () => enabled,
      check: async () => available,
    });
    expect(view).toEqual({ version: "0.2.1", notes: "Fixes" });
  });

  it("returns null without throwing when check rejects", async () => {
    const view = await startupUpdateCheck({
      getPrefs: async () => enabled,
      check: async () => {
        throw new Error("offline");
      },
    });
    expect(view).toBeNull();
  });

  it("returns null without throwing when check reports check_failed", async () => {
    const view = await startupUpdateCheck({
      getPrefs: async () => enabled,
      check: async () => ({ update: null, error: "check_failed" }),
    });
    expect(view).toBeNull();
  });

  it("returns null when the app is up to date", async () => {
    const view = await startupUpdateCheck({
      getPrefs: async () => enabled,
      check: async () => ({ update: null, error: null }),
    });
    expect(view).toBeNull();
  });
});

describe("formatUpdateError", () => {
  it("gives a distinct non-empty message for every code", () => {
    const codes: UpdateErrorCode[] = [
      "storage_unavailable",
      "check_failed",
      "install_failed",
      "nothing_pending",
    ];
    const messages = codes.map((code) => formatUpdateError(code));
    expect(messages.every((m) => m.length > 0)).toBe(true);
    expect(new Set(messages).size).toBe(codes.length);
  });
});
