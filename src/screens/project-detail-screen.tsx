import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useParams } from "react-router";
import { useState } from "react";
import { getProject, setProjectPinned } from "../lib/commands";
import { UnregisterProjectDialog } from "../components/unregister-project-dialog";
import { formatProjectError } from "../lib/project-form";

const linkClass =
  "text-accent hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";

function formatRegistered(createdAt: string): string {
  const date = new Date(createdAt);
  return Number.isNaN(date.getTime()) ? createdAt : date.toLocaleDateString();
}

export function ProjectDetailScreen() {
  const params = useParams();
  const id = Number(params["id"]);
  const validId = Number.isInteger(id);
  const queryClient = useQueryClient();
  const [unregisterOpen, setUnregisterOpen] = useState(false);

  const query = useQuery({
    queryKey: ["project", id],
    queryFn: () => getProject(id),
    enabled: validId,
  });

  const pin = useMutation({
    mutationFn: (pinned: boolean) => setProjectPinned(id, pinned),
    onSettled: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["projects"] }),
        queryClient.invalidateQueries({ queryKey: ["home"] }),
        queryClient.invalidateQueries({ queryKey: ["project", id] }),
      ]);
    },
  });

  const back = (
    <Link to="/projects" className={linkClass}>
      Back to Projects
    </Link>
  );

  if (query.isLoading && validId) {
    return <p className="m-0 text-ink-muted">Loading project...</p>;
  }

  const project = query.data?.project ?? null;
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);

  if (!validId || project === null) {
    const message =
      !validId || errorCode === null || errorCode === "project_not_found"
        ? formatProjectError("project_not_found", null, "")
        : formatProjectError(errorCode, null, "");
    return (
      <div>
        <p role="alert" className="m-0 mb-4 text-lg">
          {message}
        </p>
        {back}
      </div>
    );
  }

  const pinError =
    pin.isError || (pin.data !== undefined && pin.data.error !== null)
      ? formatProjectError(pin.data?.error ?? "storage_unavailable", null, "")
      : null;

  return (
    <div>
      <div className="mb-4">{back}</div>
      <div className="mb-6 flex flex-wrap items-center gap-4">
        <h1 className="m-0 text-3xl font-semibold">{project.display_name}</h1>
        {project.pinned ? (
          <span className="rounded-full border border-hairline px-2 py-0.5 text-xs text-warning">
            Pinned
          </span>
        ) : null}
        <button
          type="button"
          disabled={pin.isPending}
          onClick={() => pin.mutate(!project.pinned)}
          className="rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50"
        >
          {project.pinned ? "Unpin" : "Pin"}
        </button>
        <button
          type="button"
          onClick={() => setUnregisterOpen(true)}
          className="rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-warning hover:border-warning focus-visible:outline-2 focus-visible:outline-accent"
        >
          Unregister
        </button>
      </div>
      {pinError !== null ? (
        <p role="alert" className="m-0 mb-4 text-warning">
          {pinError}
        </p>
      ) : null}

      <dl className="m-0 grid grid-cols-[10rem_1fr] gap-x-6 gap-y-3 rounded-2xl border border-hairline bg-surface p-6">
        <dt className="text-ink-muted">Type</dt>
        <dd className="m-0">{project.project_type}</dd>
        <dt className="text-ink-muted">Root path</dt>
        <dd className="m-0 break-all font-mono">{project.root_path}</dd>
        <dt className="text-ink-muted">Drive</dt>
        <dd className="m-0">{project.drive === "" ? "None" : project.drive}</dd>
        <dt className="text-ink-muted">Tags</dt>
        <dd className="m-0 flex flex-wrap gap-1.5">
          {project.tags.length === 0
            ? "No tags"
            : project.tags.map((tag) => (
                <span
                  key={tag}
                  className="rounded-full border border-hairline px-2 py-0.5 text-xs text-ink-muted"
                >
                  {tag}
                </span>
              ))}
        </dd>
        <dt className="text-ink-muted">Pinned</dt>
        <dd className="m-0">{project.pinned ? "Yes" : "No"}</dd>
        <dt className="text-ink-muted">Registered</dt>
        <dd className="m-0">{formatRegistered(project.created_at)}</dd>
      </dl>
      {unregisterOpen ? (
        <UnregisterProjectDialog projectId={id} onClose={() => setUnregisterOpen(false)} />
      ) : null}
    </div>
  );
}
