import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { AddItemDialog } from "../components/add-item-dialog";
import {
  deleteWorkspace,
  getWorkspace,
  listEditorProfiles,
  listProjects,
  moveWorkspaceItem,
  removeWorkspaceItem,
  renameWorkspace,
  restoreWorkspace,
  updateWorkspaceItem,
} from "../lib/commands";
import { RestorePanel } from "../components/restore-panel";
import { formatWorkspaceError, kindLabel } from "../lib/workspace-form";
import type {
  RestoreOutcomeView,
  WorkspaceItemView,
  WorkspaceResult,
  WorkspaceView,
} from "../types/commands";

const linkClass =
  "text-accent hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";
const inputClass =
  "rounded-lg border bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonDanger =
  "rounded-lg border border-warning bg-surface-raised px-3 py-1.5 text-warning hover:brightness-110 focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

function itemName(path: string): string {
  const segments = path.split(/[\\/]/).filter((segment) => segment !== "");
  return segments[segments.length - 1] ?? path;
}

export function WorkspaceDetailScreen() {
  const params = useParams();
  const id = Number(params["id"]);
  const validId = Number.isInteger(id);
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const query = useQuery({
    queryKey: ["workspace", id],
    queryFn: () => getWorkspace(id),
    enabled: validId,
  });
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const profilesQuery = useQuery({ queryKey: ["editor-profiles"], queryFn: listEditorProfiles });

  const [renaming, setRenaming] = useState(false);
  const [renameValue, setRenameValue] = useState("");
  const [renameError, setRenameError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmRemoveId, setConfirmRemoveId] = useState<number | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [outcomes, setOutcomes] = useState<RestoreOutcomeView[] | null>(null);
  const [runId, setRunId] = useState(0);

  const back = (
    <Link to="/workspaces" className={linkClass}>
      Back to Workspaces
    </Link>
  );

  if (query.isLoading && validId) {
    return <p className="m-0 text-ink-muted">Loading workspace...</p>;
  }

  const workspace = query.data?.workspace ?? null;
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);

  if (!validId || workspace === null) {
    return (
      <div>
        <h1 className="m-0 mb-4 text-3xl font-semibold">Workspace not found</h1>
        {errorCode !== null && errorCode !== "workspace_not_found" ? (
          <p role="alert" className="m-0 mb-4 text-warning">
            {formatWorkspaceError(errorCode, null, "")}
          </p>
        ) : (
          <p className="m-0 mb-4 text-ink-muted">
            {formatWorkspaceError("workspace_not_found", null, "")}
          </p>
        )}
        {back}
      </div>
    );
  }

  const projects = projectsQuery.data?.projects ?? [];
  const profiles = profilesQuery.data?.profiles ?? [];
  const items = [...workspace.items].sort((a, b) => a.position - b.position);

  const store = (result: WorkspaceResult): WorkspaceView | null => {
    if (result.error !== null || result.workspace === null) {
      return null;
    }
    queryClient.setQueryData(["workspace", id], {
      workspace: result.workspace,
      error: null,
      detail: null,
    });
    void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    void queryClient.invalidateQueries({ queryKey: ["home"] });
    return result.workspace;
  };

  const runItemMutation = async (action: () => Promise<WorkspaceResult>) => {
    setBusy(true);
    setNotice(null);
    try {
      const result = await action();
      if (store(result) === null) {
        setNotice(
          formatWorkspaceError(
            result.error ?? "storage_unavailable",
            result.detail,
            workspace.name,
          ),
        );
      }
    } catch {
      setNotice(formatWorkspaceError("storage_unavailable", null, workspace.name));
    } finally {
      setBusy(false);
    }
  };

  const startRename = () => {
    setRenameValue(workspace.name);
    setRenameError(null);
    setRenaming(true);
  };

  const submitRename = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    setBusy(true);
    setRenameError(null);
    try {
      const result = await renameWorkspace(id, renameValue);
      if (store(result) === null) {
        setRenameError(
          formatWorkspaceError(result.error ?? "storage_unavailable", result.detail, renameValue),
        );
        return;
      }
      setRenaming(false);
    } catch {
      setRenameError(formatWorkspaceError("storage_unavailable", null, renameValue));
    } finally {
      setBusy(false);
    }
  };

  const confirmDeleteNow = async () => {
    setBusy(true);
    setNotice(null);
    try {
      const result = await deleteWorkspace(id);
      if (result.error !== null) {
        setNotice(formatWorkspaceError(result.error, result.detail, workspace.name));
        setConfirmDelete(false);
        setBusy(false);
        return;
      }
      void navigate("/workspaces");
      void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      void queryClient.invalidateQueries({ queryKey: ["home"] });
      queryClient.removeQueries({ queryKey: ["workspace", id] });
      return;
    } catch {
      setNotice(formatWorkspaceError("storage_unavailable", null, workspace.name));
      setConfirmDelete(false);
    }
    setBusy(false);
  };

  const removeNow = async (item: WorkspaceItemView) => {
    setConfirmRemoveId(null);
    await runItemMutation(() => removeWorkspaceItem(item.id));
  };

  const runRestore = async () => {
    if (restoring) {
      return;
    }
    setRestoring(true);
    setNotice(null);
    try {
      const result = await restoreWorkspace(id);
      if (!result.completed && result.error !== null) {
        setOutcomes(null);
        setNotice(formatWorkspaceError(result.error, result.detail, workspace.name));
        return;
      }
      setRunId((current) => current + 1);
      setOutcomes(result.outcomes);
      if (result.error !== null) {
        setNotice(formatWorkspaceError(result.error, result.detail, workspace.name));
      }
      if (result.completed) {
        void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
        void queryClient.invalidateQueries({ queryKey: ["home"] });
        void queryClient.invalidateQueries({ queryKey: ["workspace", id] });
      }
    } catch {
      setOutcomes(null);
      setNotice(formatWorkspaceError("storage_unavailable", null, workspace.name));
    } finally {
      setRestoring(false);
    }
  };

  const changeItem = (item: WorkspaceItemView, projectId: number | null, editorKind: string) =>
    runItemMutation(() =>
      updateWorkspaceItem({ item_id: item.id, project_id: projectId, editor_kind: editorKind }),
    );

  return (
    <div>
      <p className="m-0 mb-4">{back}</p>

      <div className="mb-2 flex flex-wrap items-center gap-3">
        {renaming ? (
          <form
            onSubmit={(event) => void submitRename(event)}
            noValidate
            className="flex flex-wrap items-start gap-2"
          >
            <div>
              <label htmlFor="ws-rename" className="sr-only">
                Workspace name
              </label>
              <input
                id="ws-rename"
                autoFocus
                value={renameValue}
                onChange={(event) => setRenameValue(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    setRenaming(false);
                  }
                }}
                aria-invalid={renameError !== null}
                aria-describedby={renameError !== null ? "ws-rename-error" : undefined}
                className={`${inputClass} w-[min(24rem,80vw)] text-xl ${renameError !== null ? "border-warning" : "border-hairline"}`}
              />
              {renameError !== null ? (
                <p id="ws-rename-error" role="alert" className="m-0 mt-1 text-sm text-warning">
                  {renameError}
                </p>
              ) : null}
            </div>
            <button type="submit" disabled={busy} className={buttonPrimary}>
              Save
            </button>
            <button type="button" onClick={() => setRenaming(false)} className={buttonSecondary}>
              Cancel
            </button>
          </form>
        ) : (
          <>
            <h1 className="m-0 text-3xl font-semibold">{workspace.name}</h1>
            <button type="button" onClick={startRename} className={buttonSecondary}>
              Rename
            </button>
            <button
              type="button"
              onClick={() => setConfirmDelete(true)}
              disabled={busy}
              className={buttonDanger}
            >
              Delete
            </button>
          </>
        )}
      </div>

      {confirmDelete ? (
        <div
          role="alertdialog"
          aria-label="Confirm delete workspace"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setConfirmDelete(false);
            }
          }}
          className="mb-6 rounded-2xl border border-warning bg-surface p-4"
        >
          <p className="m-0 mb-3">
            Delete workspace `{workspace.name}`? Its files and folders are not changed.
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => void confirmDeleteNow()}
              disabled={busy}
              className={buttonDanger}
            >
              Delete workspace
            </button>
            <button
              type="button"
              autoFocus
              onClick={() => setConfirmDelete(false)}
              className={buttonSecondary}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}

      <div className="mb-2 mt-4 flex items-center justify-between gap-4">
        <p className="m-0 text-ink-muted">
          {items.length} {items.length === 1 ? "item" : "items"}, reopened in this order.
        </p>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => void runRestore()}
            disabled={items.length === 0 || restoring || busy}
            className={buttonPrimary}
          >
            {restoring ? "Restoring…" : "Restore"}
          </button>
          <button type="button" onClick={() => setAddOpen(true)} className={buttonSecondary}>
            Add item
          </button>
        </div>
      </div>
      <p className="m-0 mb-6 text-sm text-ink-muted">
        Restore reopens files and folders. Editor tab layout and unsaved changes are not restored.
      </p>

      {notice !== null ? (
        <p role="alert" className="m-0 mb-6 text-warning">
          {notice}
        </p>
      ) : null}

      {outcomes !== null ? (
        <RestorePanel
          key={runId}
          outcomes={outcomes}
          items={items}
          profiles={profiles}
          onClose={() => setOutcomes(null)}
          onRelinked={(updated) => {
            queryClient.setQueryData(["workspace", id], {
              workspace: updated,
              error: null,
              detail: null,
            });
            void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
            void queryClient.invalidateQueries({ queryKey: ["home"] });
          }}
        />
      ) : null}

      {items.length === 0 ? (
        <section className="rounded-2xl border border-dashed border-hairline bg-surface px-6 py-12 text-center">
          <p className="m-0 text-ink-muted">This workspace has no items.</p>
        </section>
      ) : (
        <div className="overflow-x-auto rounded-2xl border border-hairline bg-surface">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr className="text-sm text-ink-muted">
                <th scope="col" className="px-4 py-3 font-medium">
                  #
                </th>
                <th scope="col" className="px-4 py-3 font-medium">
                  Kind
                </th>
                <th scope="col" className="px-4 py-3 font-medium">
                  Name and path
                </th>
                <th scope="col" className="px-4 py-3 font-medium">
                  Project
                </th>
                <th scope="col" className="px-4 py-3 font-medium">
                  Editor profile
                </th>
                <th scope="col" className="px-4 py-3 font-medium">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {items.map((item, index) => {
                const name = itemName(item.path);
                const profileKnown = profiles.some((profile) => profile.kind === item.editor_kind);
                return (
                  <tr key={item.id} className="border-t border-hairline align-top">
                    <td className="px-4 py-3 text-ink-muted">{index + 1}</td>
                    <td className="px-4 py-3">{kindLabel(item.kind)}</td>
                    <td className="px-4 py-3">
                      <span className="block font-semibold">{name}</span>
                      <span className="block break-all font-mono text-sm text-ink-muted">
                        {item.path}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <select
                        aria-label={`Project for ${name}`}
                        value={item.project_id === null ? "" : String(item.project_id)}
                        disabled={busy}
                        onChange={(event) =>
                          void changeItem(
                            item,
                            event.target.value === "" ? null : Number(event.target.value),
                            item.editor_kind,
                          )
                        }
                        className={`${inputClass} border-hairline py-1.5`}
                      >
                        <option value="">No project</option>
                        {projects.map((project) => (
                          <option key={project.id} value={project.id}>
                            {project.display_name}
                          </option>
                        ))}
                      </select>
                    </td>
                    <td className="px-4 py-3">
                      <select
                        aria-label={`Editor profile for ${name}`}
                        value={item.editor_kind}
                        disabled={busy}
                        onChange={(event) =>
                          void changeItem(item, item.project_id, event.target.value)
                        }
                        className={`${inputClass} border-hairline py-1.5`}
                      >
                        {profileKnown ? null : (
                          <option value={item.editor_kind}>{item.editor_kind}</option>
                        )}
                        {profiles.map((profile) => (
                          <option key={profile.kind} value={profile.kind}>
                            {profile.name}
                          </option>
                        ))}
                      </select>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <button
                          type="button"
                          disabled={busy || index === 0}
                          onClick={() =>
                            void runItemMutation(() => moveWorkspaceItem(item.id, true))
                          }
                          aria-label={`Move ${name} up`}
                          className={buttonSecondary}
                        >
                          Move up
                        </button>
                        <button
                          type="button"
                          disabled={busy || index === items.length - 1}
                          onClick={() =>
                            void runItemMutation(() => moveWorkspaceItem(item.id, false))
                          }
                          aria-label={`Move ${name} down`}
                          className={buttonSecondary}
                        >
                          Move down
                        </button>
                        <button
                          type="button"
                          disabled={busy}
                          onClick={() => setConfirmRemoveId(item.id)}
                          aria-label={`Remove ${name}`}
                          className={buttonDanger}
                        >
                          Remove
                        </button>
                      </div>
                      {confirmRemoveId === item.id ? (
                        <div
                          role="alertdialog"
                          aria-label={`Confirm remove ${name}`}
                          onKeyDown={(event) => {
                            if (event.key === "Escape") {
                              setConfirmRemoveId(null);
                            }
                          }}
                          className="mt-3 rounded-lg border border-warning p-3"
                        >
                          <p className="m-0 mb-2 text-sm">
                            Remove `{name}` from this workspace? The file or folder is not changed.
                          </p>
                          <div className="flex gap-2">
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() => void removeNow(item)}
                              className={buttonDanger}
                            >
                              Remove item
                            </button>
                            <button
                              type="button"
                              autoFocus
                              onClick={() => setConfirmRemoveId(null)}
                              className={buttonSecondary}
                            >
                              Cancel
                            </button>
                          </div>
                        </div>
                      ) : null}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {addOpen ? (
        <AddItemDialog
          workspaceId={id}
          onClose={() => setAddOpen(false)}
          onAdded={(updated) => {
            queryClient.setQueryData(["workspace", id], {
              workspace: updated,
              error: null,
              detail: null,
            });
            void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
            void queryClient.invalidateQueries({ queryKey: ["home"] });
            setAddOpen(false);
          }}
        />
      ) : null}
    </div>
  );
}
