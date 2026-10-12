import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router";
import { EmptyState } from "../components/empty-state";
import { RegisterProjectDialog } from "../components/register-project-dialog";
import { listProjects } from "../lib/commands";
import { formatProjectError } from "../lib/project-form";

const registerButtonClass =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink";

export function ProjectsScreen() {
  const query = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const location = useLocation();
  const navigate = useNavigate();
  const [dialogOpen, setDialogOpen] = useState(
    () => (location.state as { openRegister?: boolean } | null)?.openRegister === true,
  );
  useEffect(() => {
    if ((location.state as { openRegister?: boolean } | null)?.openRegister === true) {
      navigate(".", { replace: true, state: null });
    }
    // Read once on mount: the state is cleared right after it opens the dialog.
  }, []);
  const [highlightId, setHighlightId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const projects = query.data?.projects ?? [];
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);
  const isEmpty = !query.isLoading && errorCode === null && projects.length === 0;

  const openDialog = () => {
    setNotice(null);
    setDialogOpen(true);
  };

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Projects</h1>
      <p className="m-0 mb-6 text-ink-muted">
        Registered project folders. Registration stores a reference only.
      </p>

      {isEmpty ? null : (
        <div className="mb-6">
          <button type="button" className={registerButtonClass} onClick={openDialog}>
            Register project
          </button>
        </div>
      )}

      {notice !== null ? (
        <p
          role="status"
          className="m-0 mb-6 rounded-lg border border-hairline bg-surface px-4 py-3"
        >
          {notice}
        </p>
      ) : null}

      {errorCode !== null ? (
        <p role="alert" className="m-0 mb-6 text-warning">
          {formatProjectError(errorCode, null, "")}
        </p>
      ) : null}

      {query.isLoading ? (
        <p className="m-0 text-ink-muted">Loading projects...</p>
      ) : isEmpty ? (
        <>
          <EmptyState
            heading="No projects registered yet"
            message="A project is a reference to a folder. DevShelf never copies or changes your files."
          />
          <div className="mt-6 text-center">
            <button type="button" className={registerButtonClass} onClick={openDialog}>
              Register project
            </button>
          </div>
        </>
      ) : (
        <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
          {projects.map((project) => (
            <li
              key={project.id}
              className={`border-t border-hairline first:border-t-0 ${
                project.id === highlightId
                  ? "bg-surface-raised outline-2 -outline-offset-2 outline-accent"
                  : ""
              }`}
            >
              <Link
                to={`/projects/${project.id}`}
                className="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3 text-ink no-underline hover:bg-surface-raised focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent"
              >
                <span className="min-w-0 flex-1">
                  <span className="block font-semibold">
                    {project.pinned ? (
                      <span
                        role="img"
                        aria-label="Pinned"
                        title="Pinned"
                        className="mr-2 text-warning"
                      >
                        &#9733;
                      </span>
                    ) : null}
                    {project.display_name}
                  </span>
                  <span className="block break-all font-mono text-sm text-ink-muted">
                    {project.root_path}
                  </span>
                </span>
                <span className="flex flex-wrap gap-1.5">
                  {project.tags.map((tag) => (
                    <span
                      key={tag}
                      className="rounded-full border border-hairline px-2 py-0.5 text-xs text-ink-muted"
                    >
                      {tag}
                    </span>
                  ))}
                </span>
                <span className="min-w-[10rem] text-right text-sm text-ink-muted">
                  {project.project_type}
                  {project.drive !== "" ? ` on ${project.drive}` : ""}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}

      {dialogOpen ? (
        <RegisterProjectDialog
          onClose={() => setDialogOpen(false)}
          onRegistered={(project) => {
            setDialogOpen(false);
            setHighlightId(project.id);
            setNotice(
              `Project ${project.display_name} registered. No project files were copied or changed.`,
            );
          }}
        />
      ) : null}
    </div>
  );
}
