import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link, useNavigate } from "react-router";
import { EmptyState } from "../components/empty-state";
import { createNote, listNotes, listProjects } from "../lib/commands";
import {
  filterNotes,
  formatNoteError,
  noteProjectLabel,
  titleOrPlaceholder,
  type NoteFilter,
} from "../lib/note-form";

const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const selectClass =
  "rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";

const editedFormatter = new Intl.DateTimeFormat("en-GB", {
  dateStyle: "medium",
  timeStyle: "short",
});

function formatEdited(updatedAt: string): string {
  const date = new Date(updatedAt);
  return Number.isNaN(date.getTime()) ? updatedAt : editedFormatter.format(date);
}

function parseFilter(value: string): NoteFilter {
  if (value === "all" || value === "general") {
    return value;
  }
  const id = Number(value);
  return Number.isInteger(id) ? id : "all";
}

export function NotesScreen() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<NoteFilter>("all");
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const notesQuery = useQuery({
    queryKey: ["notes"],
    queryFn: listNotes,
    staleTime: 0,
    refetchOnMount: "always",
  });
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });

  const projects = projectsQuery.data?.projects ?? [];
  const notes = notesQuery.data?.notes ?? [];
  const listError = notesQuery.data?.error ?? (notesQuery.isError ? "storage_unavailable" : null);
  const visible = filterNotes(notes, filter);

  const newNote = async () => {
    setCreating(true);
    setCreateError(null);
    try {
      const result = await createNote(typeof filter === "number" ? filter : null);
      if (result.error !== null || result.note === null) {
        setCreateError(formatNoteError(result.error ?? "storage_unavailable"));
        return;
      }
      void queryClient.invalidateQueries({ queryKey: ["notes"] });
      navigate(`/notes/${result.note.id}`);
    } catch {
      setCreateError(formatNoteError("storage_unavailable"));
    } finally {
      setCreating(false);
    }
  };

  const newNoteButton = (
    <button
      type="button"
      className={buttonPrimary}
      disabled={creating}
      onClick={() => void newNote()}
    >
      New note
    </button>
  );

  let content;
  if (notesQuery.isLoading) {
    content = <p className="m-0 text-ink-muted">Loading notes...</p>;
  } else if (listError !== null) {
    content = (
      <p role="alert" className="m-0 text-warning">
        {formatNoteError(listError)}
      </p>
    );
  } else if (notes.length === 0) {
    content = (
      <div>
        <EmptyState heading="No notes yet" />
        <div className="mt-4 text-center">{newNoteButton}</div>
      </div>
    );
  } else if (visible.length === 0) {
    content = (
      <div>
        <EmptyState heading="No notes match this filter." />
        <div className="mt-4 text-center">
          <button type="button" className={buttonSecondary} onClick={() => setFilter("all")}>
            Show all notes
          </button>
        </div>
      </div>
    );
  } else {
    content = (
      <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
        {visible.map((note) => (
          <li key={note.id} className="border-b border-hairline last:border-b-0">
            <Link
              to={`/notes/${note.id}`}
              className="flex items-center gap-4 px-4 py-3 text-ink no-underline hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
            >
              <span className="min-w-0 flex-1">
                <span className="block truncate font-semibold">
                  {titleOrPlaceholder(note.title)}
                </span>
                <span className="block text-sm text-ink-muted">{noteProjectLabel(note)}</span>
              </span>
              <span className="text-sm text-ink-muted">Edited {formatEdited(note.updated_at)}</span>
            </Link>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Notes</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Private Markdown notes, stored outside your repositories.
      </p>
      <div className="mb-6 flex flex-wrap items-center gap-3">
        {newNoteButton}
        <div className="flex-1" />
        <select
          aria-label="Filter notes"
          className={selectClass}
          value={String(filter)}
          onChange={(event) => setFilter(parseFilter(event.target.value))}
        >
          <option value="all">All notes</option>
          <option value="general">General</option>
          {projects.map((project) => (
            <option key={project.id} value={String(project.id)}>
              {project.display_name}
            </option>
          ))}
        </select>
      </div>
      {createError !== null ? (
        <p role="alert" className="m-0 mb-4 text-warning">
          {createError}
        </p>
      ) : null}
      {content}
    </div>
  );
}
