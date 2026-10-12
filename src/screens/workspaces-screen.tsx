import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link, useNavigate } from "react-router";
import { EmptyState } from "../components/empty-state";
import { NewWorkspaceDialog } from "../components/new-workspace-dialog";
import { listWorkspaces } from "../lib/commands";
import { formatWorkspaceError } from "../lib/workspace-form";

const primaryButtonClass =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink";

export function WorkspacesScreen() {
  const query = useQuery({ queryKey: ["workspaces"], queryFn: listWorkspaces });
  const navigate = useNavigate();
  const [dialogOpen, setDialogOpen] = useState(false);

  const workspaces = query.data?.workspaces ?? [];
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);
  const isEmpty = !query.isLoading && errorCode === null && workspaces.length === 0;

  const newButton = (
    <button type="button" onClick={() => setDialogOpen(true)} className={primaryButtonClass}>
      New workspace
    </button>
  );

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Workspaces</h1>
      <p className="m-0 mb-6 text-ink-muted">Named, ordered sets of files you reopen together.</p>

      {isEmpty ? null : <div className="mb-6">{newButton}</div>}

      {errorCode !== null ? (
        <p role="alert" className="m-0 mb-6 text-warning">
          {formatWorkspaceError(errorCode, null, "")}
        </p>
      ) : null}

      {query.isLoading ? (
        <p className="m-0 text-ink-muted">Loading workspaces...</p>
      ) : isEmpty ? (
        <>
          <EmptyState
            heading="No workspaces yet"
            message="A workspace saves an ordered set of files to reopen together."
          />
          <div className="mt-6 text-center">{newButton}</div>
        </>
      ) : (
        <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
          {workspaces.map((workspace) => (
            <li key={workspace.id} className="border-t border-hairline first:border-t-0">
              <Link
                to={`/workspaces/${workspace.id}`}
                className="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3 text-ink no-underline hover:bg-surface-raised focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent"
              >
                <span className="min-w-0 flex-1">
                  <span className="block font-semibold">{workspace.name}</span>
                  <span className="block text-sm text-ink-muted">
                    {workspace.project_names.length > 0
                      ? workspace.project_names.join(", ")
                      : "No registered projects"}
                  </span>
                </span>
                <span className="text-sm text-ink-muted">
                  {workspace.item_count} {workspace.item_count === 1 ? "item" : "items"}
                </span>
                <span className="text-sm text-ink-muted">
                  {workspace.last_restored_at === null
                    ? "Never restored"
                    : `Last restored ${new Date(workspace.last_restored_at).toLocaleString()}`}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}

      {dialogOpen ? (
        <NewWorkspaceDialog
          onClose={() => setDialogOpen(false)}
          onCreated={(workspace) => {
            setDialogOpen(false);
            void navigate(`/workspaces/${workspace.id}`);
          }}
        />
      ) : null}
    </div>
  );
}
