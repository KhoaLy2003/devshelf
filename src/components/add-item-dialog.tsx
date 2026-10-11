import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  addWorkspaceItem,
  listEditorProfiles,
  listProjects,
  pickWorkspaceItem,
} from "../lib/commands";
import { formatWorkspaceError } from "../lib/workspace-form";
import type { WorkspaceView } from "../types/commands";

interface AddItemDialogProps {
  workspaceId: number;
  onClose: () => void;
  onAdded: (workspace: WorkspaceView) => void;
}

const inputClass =
  "w-full rounded-lg border bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

export function AddItemDialog({ workspaceId, onClose, onAdded }: AddItemDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const profilesQuery = useQuery({ queryKey: ["editor-profiles"], queryFn: listEditorProfiles });
  const [path, setPath] = useState("");
  const [projectId, setProjectId] = useState<number | null>(null);
  const [editorKind, setEditorKind] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [errorField, setErrorField] = useState<"path" | "project" | "profile">("path");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const projects = projectsQuery.data?.projects ?? [];
  const profiles = profilesQuery.data?.profiles ?? [];
  const defaultKind =
    profiles.find((profile) => profile.is_default)?.kind ?? profiles[0]?.kind ?? "";
  const selectedKind = editorKind ?? defaultKind;

  const browse = async (folder: boolean) => {
    try {
      const result = await pickWorkspaceItem(folder);
      if (result.path !== null) {
        setPath(result.path);
        setError(null);
      }
    } catch {
      setError(formatWorkspaceError("storage_unavailable", null, ""));
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    setBusy(true);
    setError(null);
    setErrorField("path");
    try {
      const result = await addWorkspaceItem({
        workspace_id: workspaceId,
        path,
        project_id: projectId,
        editor_kind: selectedKind === "" ? null : selectedKind,
      });
      if (result.error !== null || result.workspace === null) {
        const code = result.error ?? "storage_unavailable";
        setErrorField(
          code === "project_not_found"
            ? "project"
            : code === "profile_not_found"
              ? "profile"
              : "path",
        );
        setError(formatWorkspaceError(code, result.detail ?? path, ""));
        return;
      }
      onAdded(result.workspace);
    } catch {
      setError(formatWorkspaceError("storage_unavailable", null, ""));
    } finally {
      setBusy(false);
    }
  };

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="add-item-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(36rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <form onSubmit={(event) => void submit(event)} noValidate>
        <h2 id="add-item-title" className="m-0 mb-4 text-xl font-semibold">
          Add item
        </h2>

        <div className="mb-4">
          <label htmlFor="ai-path" className="mb-1 block text-sm text-ink-muted">
            File or folder path
          </label>
          <div className="mb-2 flex flex-wrap gap-2">
            <button type="button" onClick={() => void browse(false)} className={buttonSecondary}>
              Browse file...
            </button>
            <button type="button" onClick={() => void browse(true)} className={buttonSecondary}>
              Browse folder...
            </button>
          </div>
          <input
            id="ai-path"
            autoFocus
            value={path}
            onChange={(event) => setPath(event.target.value)}
            placeholder="F:\work\notes.txt"
            aria-invalid={error !== null && errorField === "path"}
            aria-describedby={error !== null && errorField === "path" ? "ai-path-error" : undefined}
            className={`${inputClass} font-mono ${error !== null && errorField === "path" ? "border-warning" : "border-hairline"}`}
          />
          {error !== null && errorField === "path" ? (
            <p id="ai-path-error" role="alert" className="m-0 mt-1 text-sm text-warning">
              {error}
            </p>
          ) : null}
        </div>

        <div className="mb-4">
          <label htmlFor="ai-project" className="mb-1 block text-sm text-ink-muted">
            Project
          </label>
          <select
            id="ai-project"
            value={projectId === null ? "" : String(projectId)}
            onChange={(event) =>
              setProjectId(event.target.value === "" ? null : Number(event.target.value))
            }
            aria-invalid={errorField === "project"}
            aria-describedby={errorField === "project" ? "ai-project-error" : undefined}
            className={`${inputClass} ${errorField === "project" ? "border-warning" : "border-hairline"}`}
          >
            <option value="">No project</option>
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.display_name}
              </option>
            ))}
          </select>
          {error !== null && errorField === "project" ? (
            <p id="ai-project-error" role="alert" className="m-0 mt-1 text-sm text-warning">
              {error}
            </p>
          ) : null}
        </div>

        <div className="mb-4">
          <label htmlFor="ai-profile" className="mb-1 block text-sm text-ink-muted">
            Editor profile
          </label>
          <select
            id="ai-profile"
            value={selectedKind}
            onChange={(event) => setEditorKind(event.target.value)}
            aria-invalid={errorField === "profile"}
            aria-describedby={errorField === "profile" ? "ai-profile-error" : undefined}
            className={`${inputClass} ${errorField === "profile" ? "border-warning" : "border-hairline"}`}
          >
            {profiles.map((profile) => (
              <option key={profile.kind} value={profile.kind}>
                {profile.name}
              </option>
            ))}
          </select>
          {error !== null && errorField === "profile" ? (
            <p id="ai-profile-error" role="alert" className="m-0 mt-1 text-sm text-warning">
              {error}
            </p>
          ) : null}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={buttonSecondary}>
            Cancel
          </button>
          <button
            type="submit"
            disabled={busy}
            className="rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50"
          >
            Add item
          </button>
        </div>
      </form>
    </dialog>
  );
}
