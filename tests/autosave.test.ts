import { describe, expect, it } from "vitest";
import {
  AUTOSAVE_DELAY_MS,
  autosaveReducer,
  initialAutosaveState,
  nextAction,
  saveStateLabel,
  isDirty,
  type AutosaveState,
} from "../src/lib/autosave";

const fmt = (iso: string) => `T(${iso})`;

describe("autosave", () => {
  it("starts idle with no pending edits", () => {
    expect(initialAutosaveState).toEqual({
      phase: "idle",
      savedAt: null,
      pendingVersion: 0,
      savedVersion: 0,
      lastEditAt: 0,
    });
    expect(nextAction(initialAutosaveState, 5000)).toEqual({ kind: "wait" });
  });

  it("an edit marks unsaved and waits under 1000 ms, then saves", () => {
    const s = autosaveReducer(initialAutosaveState, { type: "edit", now: 10_000 });
    expect(s.phase).toBe("unsaved");
    expect(s.pendingVersion).toBe(1);
    expect(nextAction(s, 10_000)).toEqual({ kind: "wait" });
    expect(nextAction(s, 10_000 + AUTOSAVE_DELAY_MS - 1)).toEqual({ kind: "wait" });
    expect(nextAction(s, 10_000 + AUTOSAVE_DELAY_MS)).toEqual({ kind: "save" });
  });

  it("a later edit restarts the wait", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "edit", now: 800 });
    expect(s.pendingVersion).toBe(2);
    expect(nextAction(s, 1500)).toEqual({ kind: "wait" });
    expect(nextAction(s, 1800)).toEqual({ kind: "save" });
  });

  it("never saves while saving", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    expect(s.phase).toBe("saving");
    expect(nextAction(s, 10_000)).toEqual({ kind: "wait" });
  });

  it("a newer edit during saving schedules a follow-up save", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    const inFlight = s.pendingVersion;
    s = autosaveReducer(s, { type: "edit", now: 1200 });
    expect(s.phase).toBe("saving");
    expect(nextAction(s, 5000)).toEqual({ kind: "wait" });
    s = autosaveReducer(s, { type: "saveOk", savedAt: "2026-01-01T00:00:00Z", version: inFlight });
    expect(s.phase).toBe("unsaved");
    expect(s.savedVersion).toBe(inFlight);
    expect(nextAction(s, 1200 + AUTOSAVE_DELAY_MS - 1)).toEqual({ kind: "wait" });
    expect(nextAction(s, 1200 + AUTOSAVE_DELAY_MS)).toEqual({ kind: "save" });
  });

  it("saveOk with no newer edit becomes saved", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveOk", savedAt: "2026-01-01T10:00:00Z", version: 1 });
    expect(s.phase).toBe("saved");
    expect(s.savedAt).toBe("2026-01-01T10:00:00Z");
    expect(nextAction(s, 99_999)).toEqual({ kind: "wait" });
  });

  it("saveFail enters error and does not auto-save again", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveFail" });
    expect(s.phase).toBe("error");
    expect(nextAction(s, 99_999)).toEqual({ kind: "wait" });
  });

  it("retry from error returns to unsaved and saves right away", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveFail" });
    s = autosaveReducer(s, { type: "retry", now: 7000 });
    expect(s.phase).toBe("unsaved");
    expect(nextAction(s, 7000)).toEqual({ kind: "save" });
  });

  it("retry outside error changes nothing", () => {
    expect(autosaveReducer(initialAutosaveState, { type: "retry", now: 1 })).toBe(
      initialAutosaveState,
    );
  });

  it("saveOk clears the error", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveFail" });
    s = autosaveReducer(s, { type: "retry", now: 5000 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveOk", savedAt: "2026-01-01T10:00:00Z", version: 1 });
    expect(s.phase).toBe("saved");
    expect(saveStateLabel(s, fmt)).not.toBe("Couldn't save.");
  });

  it("an edit while in error moves to unsaved", () => {
    let s = autosaveReducer(initialAutosaveState, { type: "edit", now: 0 });
    s = autosaveReducer(s, { type: "saveStart" });
    s = autosaveReducer(s, { type: "saveFail" });
    s = autosaveReducer(s, { type: "edit", now: 3000 });
    expect(s.phase).toBe("unsaved");
  });

  it("saveStart is ignored unless unsaved", () => {
    expect(autosaveReducer(initialAutosaveState, { type: "saveStart" })).toBe(initialAutosaveState);
  });

  describe("saveStateLabel", () => {
    const base: AutosaveState = initialAutosaveState;

    it("idle is empty", () => {
      expect(saveStateLabel(base, fmt)).toBe("");
    });

    it("unsaved and saving read Unsaved changes", () => {
      const unsaved = autosaveReducer(base, { type: "edit", now: 0 });
      expect(saveStateLabel(unsaved, fmt)).toBe("Unsaved changes");
      const saving = autosaveReducer(unsaved, { type: "saveStart" });
      expect(saveStateLabel(saving, fmt)).toBe("Unsaved changes");
    });

    it("saved uses the injected formatter", () => {
      const s: AutosaveState = { ...base, phase: "saved", savedAt: "2026-01-01T10:00:00Z" };
      expect(saveStateLabel(s, fmt)).toBe("Saved at T(2026-01-01T10:00:00Z)");
    });

    it("error reads Couldn't save.", () => {
      expect(saveStateLabel({ ...base, phase: "error" }, fmt)).toBe("Couldn't save.");
    });
  });
});

describe("isDirty", () => {
  const base: AutosaveState = { ...initialAutosaveState };

  it("is clean when idle or saved with no newer edits", () => {
    expect(isDirty(base)).toBe(false);
    expect(isDirty({ ...base, phase: "saved", pendingVersion: 2, savedVersion: 2 })).toBe(false);
  });

  it("is dirty while unsaved, saving, or in error", () => {
    expect(isDirty({ ...base, phase: "unsaved", pendingVersion: 1 })).toBe(true);
    expect(isDirty({ ...base, phase: "saving", pendingVersion: 1 })).toBe(true);
    expect(isDirty({ ...base, phase: "error", pendingVersion: 1 })).toBe(true);
  });

  it("is dirty when the pending version is above the saved version", () => {
    expect(isDirty({ ...base, phase: "saved", pendingVersion: 3, savedVersion: 2 })).toBe(true);
  });
});
