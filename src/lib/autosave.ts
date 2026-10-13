// Pure autosave state machine for the note editor (no React, no timers).

export const AUTOSAVE_DELAY_MS = 1000;

export type AutosavePhase = "idle" | "unsaved" | "saving" | "saved" | "error";

export interface AutosaveState {
  phase: AutosavePhase;
  savedAt: string | null;
  pendingVersion: number;
  savedVersion: number;
  // Caller-supplied time (ms) of the last edit; drives the 1-second wait.
  lastEditAt: number;
}

export type AutosaveEvent =
  | { type: "edit"; now: number }
  | { type: "saveStart" }
  | { type: "saveOk"; savedAt: string; version: number }
  | { type: "saveFail" }
  | { type: "retry"; now: number };

export type AutosaveAction = { kind: "wait" } | { kind: "save" };

export const initialAutosaveState: AutosaveState = {
  phase: "idle",
  savedAt: null,
  pendingVersion: 0,
  savedVersion: 0,
  lastEditAt: 0,
};

export function autosaveReducer(state: AutosaveState, event: AutosaveEvent): AutosaveState {
  switch (event.type) {
    case "edit":
      return {
        ...state,
        // An edit during a save keeps "saving"; saveOk then resolves to a follow-up save.
        phase: state.phase === "saving" ? "saving" : "unsaved",
        pendingVersion: state.pendingVersion + 1,
        lastEditAt: event.now,
      };
    case "saveStart":
      return state.phase === "unsaved" ? { ...state, phase: "saving" } : state;
    case "saveOk": {
      const savedVersion = Math.max(state.savedVersion, event.version);
      return {
        ...state,
        phase: state.pendingVersion > savedVersion ? "unsaved" : "saved",
        savedAt: event.savedAt,
        savedVersion,
      };
    }
    case "saveFail":
      return state.phase === "saving" ? { ...state, phase: "error" } : state;
    case "retry":
      // Retry saves immediately: back-date the last edit past the wait.
      return state.phase === "error"
        ? { ...state, phase: "unsaved", lastEditAt: event.now - AUTOSAVE_DELAY_MS }
        : state;
  }
}

export function nextAction(state: AutosaveState, now: number): AutosaveAction {
  if (state.phase === "unsaved" && now - state.lastEditAt >= AUTOSAVE_DELAY_MS) {
    return { kind: "save" };
  }
  return { kind: "wait" };
}

// True while the note holds edits that are not yet stored (including a failed or in-flight save).
export function isDirty(state: AutosaveState): boolean {
  return (
    state.phase === "unsaved" ||
    state.phase === "saving" ||
    state.phase === "error" ||
    state.pendingVersion > state.savedVersion
  );
}

export function saveStateLabel(
  state: AutosaveState,
  formatTime: (savedAt: string) => string,
): string {
  switch (state.phase) {
    case "idle":
      return "";
    case "unsaved":
    case "saving":
      return "Unsaved changes";
    case "saved":
      return state.savedAt === null ? "" : `Saved at ${formatTime(state.savedAt)}`;
    case "error":
      return "Couldn't save.";
  }
}
