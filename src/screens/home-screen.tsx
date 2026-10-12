import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router";
import { EmptyState } from "../components/empty-state";
import { HomeSection } from "../components/home-section";
import { getHomeOverview } from "../lib/commands";
import {
  capList,
  emptyPinnedText,
  emptyRecentText,
  lastUsedLabel,
  missingWarning,
  workspaceItemsLabel,
} from "../lib/home-form";
import { REGISTER_PROJECT_STATE } from "../lib/home-nav";
import { formatWorkspaceError } from "../lib/workspace-form";

const MAX_ROWS = 6;

const registerButtonClass =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink";
const rowLinkClass =
  "block px-4 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ink";
const rowClass = "border-b border-hairline last:border-b-0";

export function HomeScreen() {
  const navigate = useNavigate();
  const query = useQuery({
    queryKey: ["home"],
    queryFn: getHomeOverview,
    staleTime: 0,
    refetchOnMount: "always",
  });

  const pinned = query.data?.pinned_projects ?? [];
  const recent = query.data?.recent_workspaces ?? [];
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);
  const isEmpty =
    !query.isLoading &&
    errorCode === null &&
    query.data !== undefined &&
    query.data.project_count === 0 &&
    query.data.workspace_count === 0;

  const register = () => navigate("/projects", { state: REGISTER_PROJECT_STATE });

  const pinnedCap = capList(pinned, MAX_ROWS);
  const recentCap = capList(recent, MAX_ROWS);

  return (
    <div>
      <div className="mb-8 flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="m-0 mb-2 text-3xl font-semibold">Home</h1>
          <p className="m-0 text-ink-muted">Pick up where you left off.</p>
        </div>
        <button type="button" className={registerButtonClass} onClick={register}>
          Register project
        </button>
      </div>

      {errorCode !== null ? (
        <p role="alert" className="m-0 mb-6 text-warning">
          {formatWorkspaceError("storage_unavailable", null, "")}
        </p>
      ) : null}

      {query.isLoading ? (
        <p className="m-0 text-ink-muted">Loading home...</p>
      ) : isEmpty ? (
        <>
          <EmptyState
            heading="Nothing here yet"
            message="Register a project and save a workspace to see pinned projects and recent workspaces on this page."
          />
          <div className="mt-6 text-center">
            <button type="button" className={registerButtonClass} onClick={register}>
              Register your first project
            </button>
          </div>
        </>
      ) : errorCode === null ? (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <HomeSection
            heading="Pinned projects"
            emptyText={emptyPinnedText}
            isEmpty={pinned.length === 0}
            allLabel="All projects"
            allTo="/projects"
            showAllLink={pinnedCap.hiddenCount > 0}
          >
            {pinnedCap.shown.map((project) => (
              <li key={project.id} className={rowClass}>
                <Link to={`/projects/${project.id}`} className={rowLinkClass}>
                  <span className="block truncate font-medium" title={project.display_name}>
                    {project.display_name}
                  </span>
                  <span className="flex items-center gap-3 text-sm text-ink-muted">
                    <span className="min-w-0 truncate font-mono" title={project.root_path}>
                      {project.root_path}
                    </span>
                    <span className="shrink-0">{project.drive}</span>
                  </span>
                </Link>
              </li>
            ))}
          </HomeSection>

          <HomeSection
            heading="Recent workspaces"
            emptyText={emptyRecentText}
            isEmpty={recent.length === 0}
            allLabel="All workspaces"
            allTo="/workspaces"
            showAllLink={recentCap.hiddenCount > 0}
          >
            {recentCap.shown.map((workspace) => {
              const warning = missingWarning(workspace.last_restore_missing);
              const projectNames =
                workspace.project_names.length > 0
                  ? workspace.project_names.join(", ")
                  : "No registered projects";
              return (
                <li key={workspace.id} className={rowClass}>
                  <Link to={`/workspaces/${workspace.id}`} className={rowLinkClass}>
                    <span className="flex items-center justify-between gap-3">
                      <span className="min-w-0 truncate font-medium" title={workspace.name}>
                        {workspace.name}
                      </span>
                      <span className="shrink-0 text-sm text-ink-muted">
                        {workspaceItemsLabel(workspace.item_count)}
                      </span>
                    </span>
                    <span className="block truncate text-sm text-ink-muted" title={projectNames}>
                      {projectNames}
                    </span>
                    <span className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-ink-muted">
                      <span>{lastUsedLabel(workspace.last_restored_at)}</span>
                      {warning !== null ? (
                        <span className="rounded-full border border-warning px-2 py-0.5 text-xs text-warning">
                          {warning}
                        </span>
                      ) : null}
                    </span>
                  </Link>
                </li>
              );
            })}
          </HomeSection>
        </div>
      ) : null}
    </div>
  );
}
