import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router";
import { projectsRemovalPreview, projectsUnregister } from "../lib/commands";
import { formatProjectRemovalError, removalSummary } from "../lib/project-removal";

interface UnregisterProjectDialogProps {
  projectId: number;
  onClose: () => void;
}

const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

const INVALIDATED_KEYS = [
  "projects",
  "project",
  "home",
  "config-files",
  "workspaces",
  "notes",
  "secrets",
];

export function UnregisterProjectDialog({ projectId, onClose }: UnregisterProjectDialogProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const preview = useQuery({
    queryKey: ["project-removal-preview", projectId],
    queryFn: () => projectsRemovalPreview(projectId),
    staleTime: 0,
    gcTime: 0,
  });

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const previewError =
    preview.data?.error ?? (preview.isError ? "project_storage_unavailable" : null);
  const summary =
    preview.data !== undefined && preview.data.error === null
      ? removalSummary(preview.data.config_count, preview.data.workspace_count)
      : null;

  const confirm = async () => {
    if (busy) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await projectsUnregister(projectId);
      if (result.error !== null) {
        setError(formatProjectRemovalError(result.error));
        if (result.error === "project_not_found") {
          await queryClient.invalidateQueries({ queryKey: ["projects"] });
        }
        return;
      }
      queryClient.removeQueries({ queryKey: ["project", projectId] });
      await Promise.all(
        INVALIDATED_KEYS.map((key) => queryClient.invalidateQueries({ queryKey: [key] })),
      );
      void navigate("/projects");
    } catch {
      setError(formatProjectRemovalError("project_storage_unavailable"));
    } finally {
      setBusy(false);
    }
  };

  const shownError =
    error ?? (previewError !== null ? formatProjectRemovalError(previewError) : null);

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="unregister-project-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(30rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <h2 id="unregister-project-title" className="m-0 mb-4 text-xl font-semibold">
        Unregister project
      </h2>
      <p className="m-0 mb-2">
        Unregister this project? Its files and folders are not changed or deleted.
      </p>
      {preview.isLoading ? (
        <p className="m-0 mb-2 text-ink-muted">Checking what will be removed…</p>
      ) : null}
      {summary !== null ? <p className="m-0 mb-2">{summary}</p> : null}
      {shownError !== null ? (
        <p role="alert" className="m-0 mt-2 text-warning">
          {shownError}
        </p>
      ) : null}
      <div className="mt-6 flex justify-end gap-2">
        <button type="button" onClick={onClose} className={buttonSecondary}>
          Cancel
        </button>
        <button
          type="button"
          disabled={busy || preview.isLoading || previewError !== null}
          onClick={() => void confirm()}
          className={buttonPrimary}
        >
          Unregister
        </button>
      </div>
    </dialog>
  );
}
