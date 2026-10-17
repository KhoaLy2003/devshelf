import { describe, expect, it } from "vitest";
import {
  filterNotes,
  formatNoteError,
  noteProjectLabel,
  titleOrPlaceholder,
} from "../src/lib/note-form";
import type { NoteCode, NoteSummaryView } from "../src/types/commands";

const COPY: Record<NoteCode, string> = {
  note_not_found: "This note no longer exists.",
  title_too_long: "Use at most 200 characters.",
  body_too_large: "This note is too large to save (limit 1 MB).",
  project_not_found: "That project no longer exists. Choose General or another project.",
  key_missing:
    "Notes can't be read. The key that protects them is missing from Windows Credential Manager.",
  keyring_unavailable:
    "DevShelf can't reach Windows Credential Manager. Notes are unavailable until it can.",
  body_unreadable: "This note can't be read. Its text may be damaged.",
  storage_unavailable: "DevShelf cannot reach its data folder. Restart DevShelf and try again.",
};

function note(id: number, projectId: number | null, name: string | null): NoteSummaryView {
  return {
    id,
    title: `n${id}`,
    project_id: projectId,
    project_name: name,
    project_missing: false,
    updated_at: "2026-01-01T00:00:00Z",
  };
}

describe("formatNoteError", () => {
  for (const [code, text] of Object.entries(COPY)) {
    it(`maps ${code}`, () => {
      expect(formatNoteError(code as NoteCode)).toBe(text);
    });
  }
});

describe("titleOrPlaceholder", () => {
  it("returns Untitled note for blank titles", () => {
    expect(titleOrPlaceholder("")).toBe("Untitled note");
    expect(titleOrPlaceholder("   ")).toBe("Untitled note");
  });

  it("returns the title otherwise", () => {
    expect(titleOrPlaceholder("Deploy steps")).toBe("Deploy steps");
  });
});

describe("noteProjectLabel", () => {
  it("returns the project name or General", () => {
    expect(noteProjectLabel(note(1, 5, "Alpha"))).toBe("Alpha");
    expect(noteProjectLabel(note(2, null, null))).toBe("General");
    expect(noteProjectLabel({ ...note(3, 9, null), project_missing: true })).toBe(
      "Project removed",
    );
  });
});

describe("filterNotes", () => {
  const notes = [note(1, null, null), note(2, 5, "Alpha"), note(3, 6, "Beta"), note(4, 5, "Alpha")];

  it("all returns every note in order", () => {
    expect(filterNotes(notes, "all").map((n) => n.id)).toEqual([1, 2, 3, 4]);
  });

  it("general returns only notes without a project", () => {
    expect(filterNotes(notes, "general").map((n) => n.id)).toEqual([1]);
  });

  it("a project id returns that project's notes", () => {
    expect(filterNotes(notes, 5).map((n) => n.id)).toEqual([2, 4]);
    expect(filterNotes(notes, 99)).toEqual([]);
  });
});
