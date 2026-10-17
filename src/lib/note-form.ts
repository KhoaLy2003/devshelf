import type { NoteCode, NoteSummaryView } from "../types/commands";

export type NoteFilter = "all" | "general" | number;

export function formatNoteError(code: NoteCode): string {
  switch (code) {
    case "note_not_found":
      return "This note no longer exists.";
    case "title_too_long":
      return "Use at most 200 characters.";
    case "body_too_large":
      return "This note is too large to save (limit 1 MB).";
    case "project_not_found":
      return "That project no longer exists. Choose General or another project.";
    case "key_missing":
      return "Notes can't be read. The key that protects them is missing from Windows Credential Manager.";
    case "keyring_unavailable":
      return "DevShelf can't reach Windows Credential Manager. Notes are unavailable until it can.";
    case "body_unreadable":
      return "This note can't be read. Its text may be damaged.";
    case "storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

export function titleOrPlaceholder(title: string): string {
  return title.trim() === "" ? "Untitled note" : title;
}

// "Project removed" when the linked project was unregistered; "General" when none is linked.
export function noteProjectLabel(
  note: Pick<NoteSummaryView, "project_id" | "project_name" | "project_missing">,
): string {
  if (note.project_missing) return "Project removed";
  return note.project_id === null || note.project_name === null ? "General" : note.project_name;
}

export function filterNotes(notes: NoteSummaryView[], filter: NoteFilter): NoteSummaryView[] {
  if (filter === "all") return notes;
  if (filter === "general") return notes.filter((n) => n.project_id === null);
  return notes.filter((n) => n.project_id === filter);
}
