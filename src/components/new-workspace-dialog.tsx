import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { createWorkspace } from "../lib/commands";
import { formatWorkspaceError } from "../lib/workspace-form";
import type { WorkspaceView } from "../types/commands";

interface NewWorkspaceDialogProps {
  onClose: () => void;
  onCreated: (workspace: WorkspaceView) => void;
}

const inputClass =
  "w-full rounded-lg border bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

export function NewWorkspaceDialog({ onClose, onCreated }: NewWorkspaceDialogProps) {
  const queryClient = useQueryClient();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await createWorkspace(name);
      if (result.error !== null || result.workspace === null) {
        setError(formatWorkspaceError(result.error ?? "storage_unavailable", result.detail, name));
        return;
      }
      queryClient.setQueryData(["workspace", result.workspace.id], {
        workspace: result.workspace,
        error: null,
        detail: null,
      });
      await queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      onCreated(result.workspace);
    } catch {
      setError(formatWorkspaceError("storage_unavailable", null, name));
    } finally {
      setBusy(false);
    }
  };

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="new-workspace-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(30rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <form onSubmit={(event) => void submit(event)} noValidate>
        <h2 id="new-workspace-title" className="m-0 mb-4 text-xl font-semibold">
          New workspace
        </h2>
        <div className="mb-4">
          <label htmlFor="nw-name" className="mb-1 block text-sm text-ink-muted">
            Workspace name
          </label>
          <input
            id="nw-name"
            autoFocus
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Release checklist"
            aria-invalid={error !== null}
            aria-describedby={error !== null ? "nw-name-error" : undefined}
            className={`${inputClass} ${error !== null ? "border-warning" : "border-hairline"}`}
          />
          {error !== null ? (
            <p id="nw-name-error" role="alert" className="m-0 mt-1 text-sm text-warning">
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
            className="rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50"
          >
            Create
          </button>
        </div>
      </form>
    </dialog>
  );
}
