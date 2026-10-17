import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { AddConfigDialog } from "../components/add-config-dialog";
import { ConfigRow } from "../components/config-row";
import { EmptyState } from "../components/empty-state";
import { listConfigFiles, listEditorProfiles, listProjects } from "../lib/commands";
import {
  activeChips,
  applyFilters,
  environmentLabel,
  formatConfigError,
  kindLabel,
  type ConfigFilters,
} from "../lib/config-form";
import { CONFIG_ENVIRONMENTS, CONFIG_KINDS } from "../types/commands";

const NO_FILTERS: ConfigFilters = {
  project: "all",
  environment: "all",
  kind: "all",
  status: "all",
};

const primaryButton =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink";
const secondaryButton =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-sm text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent";
const selectClass =
  "rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";

export function ConfigScreen() {
  const query = useQuery({
    queryKey: ["config-files"],
    queryFn: listConfigFiles,
    staleTime: 0,
    refetchOnMount: "always",
  });
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const profilesQuery = useQuery({ queryKey: ["editor-profiles"], queryFn: listEditorProfiles });
  const [filters, setFilters] = useState<ConfigFilters>(NO_FILTERS);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [highlightId, setHighlightId] = useState<number | null>(null);

  const entries = query.data?.entries ?? [];
  const projects = projectsQuery.data?.projects ?? [];
  const profiles = profilesQuery.data?.profiles ?? [];
  const errorCode = query.data?.error ?? (query.isError ? "storage_unavailable" : null);
  const isEmpty = !query.isLoading && errorCode === null && entries.length === 0;
  const visible = applyFilters(entries, filters);
  const chips = activeChips(filters);

  const setFilter = (key: keyof ConfigFilters, value: string) =>
    setFilters((prev) => ({ ...prev, [key]: value }));
  const clearFilters = () => setFilters(NO_FILTERS);
  const openDialog = () => {
    setHighlightId(null);
    setDialogOpen(true);
  };

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Config Catalog</h1>
      <p className="m-0 mb-6 text-ink-muted">
        Find the right environment's file. DevShelf stores paths and labels, never file contents.
      </p>

      {errorCode !== null ? (
        <p role="alert" className="m-0 mb-6 text-warning">
          {formatConfigError(errorCode, query.data?.detail ?? null, "")}
        </p>
      ) : null}

      {query.isLoading ? (
        <p className="m-0 text-ink-muted">Loading config files...</p>
      ) : isEmpty ? (
        <>
          <EmptyState
            heading="No config files registered"
            message="Registering a file stores its path only, never its contents."
          />
          <div className="mt-4 text-center">
            <button type="button" onClick={openDialog} className={primaryButton}>
              Add config file
            </button>
          </div>
        </>
      ) : errorCode === null ? (
        <>
          <div className="mb-4 flex flex-wrap items-end gap-4">
            <label className="flex flex-col gap-1 text-sm text-ink-muted">
              Project
              <select
                value={filters.project}
                onChange={(event) => setFilter("project", event.target.value)}
                className={selectClass}
              >
                <option value="all">All</option>
                {projects.map((project) => (
                  <option key={project.id} value={project.display_name}>
                    {project.display_name}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm text-ink-muted">
              Environment
              <select
                value={filters.environment}
                onChange={(event) => setFilter("environment", event.target.value)}
                className={selectClass}
              >
                <option value="all">All</option>
                {CONFIG_ENVIRONMENTS.map((env) => (
                  <option key={env} value={env}>
                    {environmentLabel(env)}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm text-ink-muted">
              Kind
              <select
                value={filters.kind}
                onChange={(event) => setFilter("kind", event.target.value)}
                className={selectClass}
              >
                <option value="all">All</option>
                {CONFIG_KINDS.map((kind) => (
                  <option key={kind} value={kind}>
                    {kindLabel(kind)}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm text-ink-muted">
              Status
              <select
                value={filters.status}
                onChange={(event) => setFilter("status", event.target.value)}
                className={selectClass}
              >
                <option value="all">All</option>
                <option value="available">Available</option>
                <option value="missing">Missing</option>
                <option value="inaccessible">Inaccessible</option>
              </select>
            </label>
            <div className="ml-auto">
              <button type="button" onClick={openDialog} className={primaryButton}>
                Add config file
              </button>
            </div>
          </div>

          {chips.length > 0 ? (
            <div className="mb-4 flex flex-wrap items-center gap-2">
              {chips.map((chip) => (
                <span
                  key={chip.key}
                  className="inline-flex items-center gap-2 rounded-full border border-hairline bg-surface-raised px-3 py-1 text-sm"
                >
                  {chip.label}
                  <button
                    type="button"
                    aria-label={`Remove filter ${chip.label}`}
                    onClick={() => setFilter(chip.key, "all")}
                    className="text-ink-muted hover:text-ink focus-visible:outline-2 focus-visible:outline-accent"
                  >
                    ×
                  </button>
                </span>
              ))}
              <button type="button" onClick={clearFilters} className={secondaryButton}>
                Clear filters
              </button>
            </div>
          ) : null}

          {visible.length === 0 ? (
            <div className="rounded-2xl border border-hairline bg-surface p-8 text-center">
              <p className="m-0 mb-4 text-ink-muted">No config files match these filters.</p>
              <button type="button" onClick={clearFilters} className={secondaryButton}>
                Clear filters
              </button>
            </div>
          ) : (
            <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
              {visible.map((entry) => (
                <ConfigRow
                  key={entry.id}
                  entry={entry}
                  profiles={profiles}
                  highlighted={entry.id === highlightId}
                />
              ))}
            </ul>
          )}
        </>
      ) : null}

      {dialogOpen ? (
        <AddConfigDialog
          onClose={() => setDialogOpen(false)}
          onAdded={(entry) => {
            setHighlightId(entry.id);
            setDialogOpen(false);
          }}
        />
      ) : null}
    </div>
  );
}
