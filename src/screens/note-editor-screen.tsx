import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useReducer, useRef, useState } from "react";
import { Link, useBlocker, useNavigate, useParams } from "react-router";
import { deleteNote, getNote, listProjects, saveNote } from "../lib/commands";
import {
  autosaveReducer,
  initialAutosaveState,
  isDirty,
  nextAction,
  saveStateLabel,
} from "../lib/autosave";
import { formatNoteError } from "../lib/note-form";
import {
  NOTE_TITLE_MAX,
  type NoteCode,
  type NoteResult,
  type NoteSummaryView,
  type NoteView,
  type NotesListResult,
} from "../types/commands";

const linkClass =
  "text-accent hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonDanger =
  "rounded-lg border border-warning bg-surface-raised px-3 py-1.5 text-warning hover:brightness-110 focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const fieldClass =
  "rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";

const timeFormatter = new Intl.DateTimeFormat("en-GB", { timeStyle: "short" });

function formatTime(savedAt: string): string {
  const date = new Date(savedAt);
  return Number.isNaN(date.getTime()) ? savedAt : timeFormatter.format(date);
}

const AUTOSAVE_POLL_MS = 250;

export function NoteEditorScreen() {
  const params = useParams();
  const id = Number(params["id"]);
  const validId = Number.isInteger(id);

  const query = useQuery({
    queryKey: ["note", id],
    queryFn: () => getNote(id),
    enabled: validId,
    staleTime: 0,
    refetchOnMount: "always",
    refetchOnWindowFocus: false,
  });

  const back = (
    <Link to="/notes" className={linkClass}>
      Back to Notes
    </Link>
  );

  // Show the editor only for data fetched after mount, so the local text never starts from a stale cache.
  if (validId && (query.isLoading || (query.isFetching && !query.isFetchedAfterMount))) {
    return <p className="m-0 text-ink-muted">Loading note...</p>;
  }

  const note = query.data?.note ?? null;
  if (!validId || note === null) {
    const code: NoteCode = !validId
      ? "note_not_found"
      : (query.data?.error ?? "storage_unavailable");
    return (
      <div>
        <h1 className="m-0 mb-4 text-3xl font-semibold">Note unavailable</h1>
        <p role="alert" className="m-0 mb-4 text-warning">
          {formatNoteError(code)}
        </p>
        {back}
      </div>
    );
  }

  return <NoteEditor key={note.id} note={note} />;
}

function NoteEditor({ note }: { note: NoteView }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const id = note.id;

  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const projects = projectsQuery.data?.projects ?? [];

  const [title, setTitle] = useState(note.title);
  const [projectId, setProjectId] = useState<number | null>(note.project_id);
  const [body, setBody] = useState(note.body);
  const [titleLimit, setTitleLimit] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [state, dispatch] = useReducer(autosaveReducer, initialAutosaveState);

  const latest = useRef({ title, projectId, body, state });
  latest.current = { title, projectId, body, state };
  const inFlight = useRef<Promise<void> | null>(null);
  const savedVersion = useRef(0);
  const deletingRef = useRef(false);

  const storeSummary = (summary: NoteSummaryView, savedBody: string) => {
    queryClient.setQueryData<NoteResult>(["note", id], {
      note: { ...summary, body: savedBody },
      error: null,
      detail: null,
    });
    queryClient.setQueryData<NotesListResult>(["notes"], (old) =>
      old === undefined
        ? old
        : {
            ...old,
            notes: [summary, ...old.notes.filter((entry) => entry.id !== summary.id)],
          },
    );
  };

  // Resolves true when every edit made so far is stored. Never overlaps another save.
  // Rechecks the pending version after each save so edits typed during a save are saved too.
  const runSave = async (): Promise<boolean> => {
    let first = true;
    for (;;) {
      while (inFlight.current !== null) {
        await inFlight.current;
      }
      if (deletingRef.current) {
        return false;
      }
      const snapshot = latest.current;
      const version = snapshot.state.pendingVersion;
      if (savedVersion.current >= version && !(first && snapshot.state.phase === "error")) {
        return true;
      }
      first = false;
      let ok = false;
      const run = (async () => {
        dispatch({ type: "saveStart" });
        try {
          const result = await saveNote({
            id,
            title: snapshot.title,
            project_id: snapshot.projectId,
            body: snapshot.body,
          });
          if (result.error !== null || result.summary === null) {
            setSaveError(formatNoteError(result.error ?? "storage_unavailable"));
            dispatch({ type: "saveFail" });
            return;
          }
          savedVersion.current = Math.max(savedVersion.current, version);
          setSaveError(null);
          storeSummary(result.summary, snapshot.body);
          dispatch({ type: "saveOk", savedAt: result.summary.updated_at, version });
          ok = true;
        } catch {
          setSaveError(formatNoteError("storage_unavailable"));
          dispatch({ type: "saveFail" });
        }
      })();
      inFlight.current = run;
      await run;
      inFlight.current = null;
      if (!ok) {
        return false;
      }
    }
  };
  const runSaveRef = useRef(runSave);
  runSaveRef.current = runSave;

  // Leaving this note with unsaved work is blocked until a save succeeds (AC-21, AC-23).
  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) =>
      isDirty(latest.current.state) &&
      !deletingRef.current &&
      currentLocation.pathname !== nextLocation.pathname,
  );
  const blockerRef = useRef(blocker);
  blockerRef.current = blocker;
  const blockerState = blocker.state;
  useEffect(() => {
    if (blockerState !== "blocked") {
      return;
    }
    void (async () => {
      if (latest.current.state.phase === "error") {
        dispatch({ type: "retry", now: Date.now() });
      }
      const clean = await runSaveRef.current();
      const current = blockerRef.current;
      if (current.state !== "blocked") {
        return;
      }
      if (clean) {
        current.proceed();
      } else {
        current.reset();
      }
    })();
  }, [blockerState]);

  const unsaved = state.phase === "unsaved";
  useEffect(() => {
    if (!unsaved) {
      return;
    }
    const timer = window.setInterval(() => {
      if (inFlight.current !== null || deletingRef.current) {
        return;
      }
      if (nextAction(latest.current.state, Date.now()).kind === "save") {
        void runSaveRef.current();
      }
    }, AUTOSAVE_POLL_MS);
    return () => window.clearInterval(timer);
  }, [unsaved]);

  const touch = () => {
    setSaveError(null);
    dispatch({ type: "edit", now: Date.now() });
  };

  const onTitleChange = (value: string) => {
    if ([...value].length > NOTE_TITLE_MAX) {
      setTitleLimit(true);
      return;
    }
    setTitleLimit(false);
    setTitle(value);
    touch();
  };

  const onProjectChange = (value: string) => {
    setProjectId(value === "" ? null : Number(value));
    touch();
  };

  const onBodyChange = (value: string) => {
    setBody(value);
    touch();
  };

  const onRetry = () => {
    dispatch({ type: "retry", now: Date.now() });
    void runSave();
  };

  const onDelete = async () => {
    deletingRef.current = true;
    setDeleting(true);
    setDeleteError(null);
    while (inFlight.current !== null) {
      await inFlight.current;
    }
    try {
      const result = await deleteNote(id);
      if (result.error !== null && result.error !== "note_not_found") {
        setDeleteError(formatNoteError(result.error));
        deletingRef.current = false;
        setDeleting(false);
        return;
      }
      queryClient.removeQueries({ queryKey: ["note", id] });
      void queryClient.invalidateQueries({ queryKey: ["notes"] });
      navigate("/notes");
    } catch {
      setDeleteError(formatNoteError("storage_unavailable"));
      deletingRef.current = false;
      setDeleting(false);
    }
  };

  const label = saveStateLabel(state, formatTime);

  return (
    <div>
      <div className="mb-4">
        <Link to="/notes" className={linkClass}>
          Back to Notes
        </Link>
      </div>
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <div className="min-w-0 max-w-[560px] flex-1">
          <input
            type="text"
            aria-label="Note title"
            aria-invalid={titleLimit}
            placeholder="Untitled note"
            className={`${fieldClass} w-full text-xl font-semibold`}
            value={title}
            onChange={(event) => onTitleChange(event.target.value)}
          />
          {titleLimit ? (
            <p role="alert" className="m-0 mt-1 text-sm text-warning">
              {formatNoteError("title_too_long")}
            </p>
          ) : null}
        </div>
        <div className="flex-1" />
        <div className="flex items-center gap-3" aria-live="polite">
          <span className={state.phase === "error" ? "text-warning" : "text-sm text-ink-muted"}>
            {label}
          </span>
          {state.phase === "error" ? (
            <button type="button" className={buttonSecondary} onClick={onRetry}>
              Retry
            </button>
          ) : null}
        </div>
      </div>
      {saveError !== null ? (
        <p role="alert" className="m-0 mb-4 text-warning">
          {saveError}
        </p>
      ) : null}
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <select
          aria-label="Project"
          className={fieldClass}
          value={projectId === null ? "" : String(projectId)}
          onChange={(event) => onProjectChange(event.target.value)}
        >
          <option value="">General</option>
          {note.project_missing && note.project_id !== null ? (
            <option value={String(note.project_id)}>Project removed</option>
          ) : null}
          {projects.map((project) => (
            <option key={project.id} value={String(project.id)}>
              {project.display_name}
            </option>
          ))}
        </select>
        <span className="text-sm text-ink-muted">
          Stored in DevShelf on this computer, outside project folders. Note text is encrypted.
        </span>
        <div className="flex-1" />
        {confirmDelete ? (
          <div
            className="flex flex-wrap items-center gap-2"
            role="alertdialog"
            aria-label="Delete note"
          >
            <span>Delete this note? This cannot be undone.</span>
            <button
              type="button"
              className={buttonSecondary}
              disabled={deleting}
              onClick={() => setConfirmDelete(false)}
            >
              Cancel
            </button>
            <button
              type="button"
              className={buttonDanger}
              disabled={deleting}
              onClick={() => void onDelete()}
            >
              Delete
            </button>
          </div>
        ) : (
          <button type="button" className={buttonDanger} onClick={() => setConfirmDelete(true)}>
            Delete
          </button>
        )}
      </div>
      {deleteError !== null ? (
        <p role="alert" className="m-0 mb-4 text-warning">
          {deleteError}
        </p>
      ) : null}
      <textarea
        aria-label="Note text"
        placeholder="Start with a heading, a checklist, or a command you keep forgetting."
        className={`${fieldClass} min-h-[420px] w-full resize-y font-mono text-sm leading-[1.7]`}
        value={body}
        onChange={(event) => onBodyChange(event.target.value)}
      />
    </div>
  );
}
