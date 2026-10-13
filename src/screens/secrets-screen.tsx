import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Link } from "react-router";
import { EmptyState } from "../components/empty-state";
import { VaultGate } from "../components/vault-gate";
import {
  listProjects,
  secretsCopy,
  secretsDelete,
  secretsList,
  secretsReveal,
} from "../lib/commands";
import {
  COPIED_VISIBLE_MS,
  SECRET_TYPE_OPTIONS,
  formatSecretError,
  maskedValue,
  revealTimerExpired,
  secretEnvironmentLabel,
  secretProjectLabel,
  secretTypeLabel,
} from "../lib/secret-form";
import { VAULT_STATUS_KEY } from "../lib/use-vault-shell";
import type { SecretCode, SecretSummaryView, SecretType } from "../types/commands";

const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink no-underline hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonDanger =
  "rounded-lg border border-warning bg-surface-raised px-3 py-1.5 text-warning hover:brightness-110 focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const selectClass =
  "rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";

const TIMER_POLL_MS = 1000;

interface Revealed {
  id: number;
  value: string;
  lastInteractionMs: number;
}

function SecretsList() {
  const queryClient = useQueryClient();
  const [projectFilter, setProjectFilter] = useState("all");
  const [typeFilter, setTypeFilter] = useState<SecretType | "all">("all");
  const [revealed, setRevealed] = useState<Revealed | null>(null);
  const [copiedId, setCopiedId] = useState<number | null>(null);
  const [confirmId, setConfirmId] = useState<number | null>(null);
  const [actionError, setActionError] = useState<SecretCode | null>(null);
  const copiedTimer = useRef<number | undefined>(undefined);

  const filterByProject = projectFilter !== "all";
  const filterProjectId =
    projectFilter === "none" || !filterByProject ? null : Number(projectFilter);
  const secretType = typeFilter === "all" ? null : typeFilter;
  const filtersActive = filterByProject || secretType !== null;

  const listQuery = useQuery({
    queryKey: ["secrets", "list", filterByProject, filterProjectId, secretType],
    queryFn: () =>
      secretsList({
        project_id: filterProjectId,
        filter_by_project: filterByProject,
        secret_type: secretType,
      }),
  });
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });

  const projects = projectsQuery.data?.projects ?? [];
  const projectNames = new Map(projects.map((p) => [p.id, p.display_name]));
  const items = listQuery.data?.items ?? [];
  const listError =
    listQuery.data?.error ?? (listQuery.isError ? "secrets_storage_unavailable" : null);

  // A locked code from any command means the vault locked underneath us: refresh status so the gate shows the lock screen.
  const reportError = (code: SecretCode | null) => {
    setActionError(code);
    if (code === "secrets_locked") {
      setRevealed(null);
      void queryClient.invalidateQueries({ queryKey: VAULT_STATUS_KEY });
    }
  };

  useEffect(() => {
    if (listError === "secrets_locked") {
      void queryClient.invalidateQueries({ queryKey: VAULT_STATUS_KEY });
    }
  }, [listError, queryClient]);

  // AC-13: re-mask after 30 s without interaction with the revealed row.
  const revealedId = revealed?.id ?? null;
  useEffect(() => {
    if (revealedId === null) return;
    const timer = setInterval(() => {
      setRevealed((current) =>
        current !== null && revealTimerExpired(current.lastInteractionMs, Date.now())
          ? null
          : current,
      );
    }, TIMER_POLL_MS);
    return () => clearInterval(timer);
  }, [revealedId]);

  useEffect(
    () => () => {
      window.clearTimeout(copiedTimer.current);
    },
    [],
  );

  const touchRow = (id: number) => {
    setRevealed((current) =>
      current !== null && current.id === id
        ? { ...current, lastInteractionMs: Date.now() }
        : current,
    );
  };

  const reveal = async (id: number) => {
    setActionError(null);
    try {
      const result = await secretsReveal(id);
      if (result.error !== null || result.value === null) {
        reportError(result.error ?? "secrets_storage_unavailable");
        return;
      }
      // AC-14: setting a new row replaces (re-masks) the previous one.
      setRevealed({ id, value: result.value, lastInteractionMs: Date.now() });
    } catch {
      reportError("secrets_storage_unavailable");
    }
  };

  const copy = async (id: number) => {
    setActionError(null);
    try {
      const result = await secretsCopy(id);
      if (result.error !== null) {
        reportError(result.error);
        return;
      }
      setCopiedId(id);
      window.clearTimeout(copiedTimer.current);
      copiedTimer.current = window.setTimeout(() => setCopiedId(null), COPIED_VISIBLE_MS);
    } catch {
      reportError("secrets_storage_unavailable");
    }
  };

  const remove = async (id: number) => {
    setActionError(null);
    try {
      const result = await secretsDelete(id);
      if (result.error !== null) {
        reportError(result.error);
        return;
      }
      setConfirmId(null);
      setRevealed((current) => (current?.id === id ? null : current));
      await queryClient.invalidateQueries({ queryKey: ["secrets"] });
    } catch {
      reportError("secrets_storage_unavailable");
    }
  };

  const clearFilters = () => {
    setProjectFilter("all");
    setTypeFilter("all");
  };

  const renderRow = (item: SecretSummaryView) => {
    const isRevealed = revealed?.id === item.id;
    const confirming = confirmId === item.id;
    return (
      <li
        key={item.id}
        className="border-b border-hairline px-4 py-3 last:border-b-0"
        onPointerMove={() => touchRow(item.id)}
        onPointerDown={() => touchRow(item.id)}
        onKeyDown={() => touchRow(item.id)}
        onFocus={() => touchRow(item.id)}
      >
        <div className="flex flex-wrap items-center gap-4">
          <div className="min-w-0 flex-1">
            <span className="block truncate font-semibold">{item.label}</span>
            <span className="block text-sm text-ink-muted">
              {secretTypeLabel(item.secret_type)} · {secretProjectLabel(item, projectNames)} ·{" "}
              {secretEnvironmentLabel(item.environment)}
            </span>
          </div>
          <code
            data-testid={`secret-value-${item.id}`}
            className="max-w-[28ch] break-all font-mono text-sm text-ink"
          >
            {isRevealed ? revealed.value : maskedValue()}
          </code>
          <div className="flex items-center gap-2">
            {isRevealed ? (
              <button type="button" className={buttonSecondary} onClick={() => setRevealed(null)}>
                Hide
              </button>
            ) : (
              <button
                type="button"
                className={buttonSecondary}
                onClick={() => void reveal(item.id)}
              >
                Reveal
              </button>
            )}
            <button type="button" className={buttonSecondary} onClick={() => void copy(item.id)}>
              Copy
            </button>
            <Link to={`/secrets/${item.id}`} className={buttonSecondary + " no-underline"}>
              Edit
            </Link>
            <button
              type="button"
              className={buttonDanger}
              onClick={() => setConfirmId(item.id)}
              aria-label={`Delete ${item.label}`}
            >
              Delete
            </button>
          </div>
        </div>
        {copiedId === item.id ? (
          <p role="status" className="m-0 mt-2 text-sm text-ink-muted">
            Copied.
          </p>
        ) : null}
        {confirming ? (
          <div
            role="alertdialog"
            aria-label="Confirm delete"
            className="mt-3 flex flex-wrap items-center gap-3 rounded-lg border border-warning px-3 py-2"
          >
            <span className="text-warning">Delete this secret? This cannot be undone.</span>
            <button type="button" className={buttonSecondary} onClick={() => setConfirmId(null)}>
              Cancel
            </button>
            <button type="button" className={buttonDanger} onClick={() => void remove(item.id)}>
              Delete
            </button>
          </div>
        ) : null}
      </li>
    );
  };

  let content;
  if (listQuery.isLoading) {
    content = <p className="m-0 text-ink-muted">Loading secrets...</p>;
  } else if (listError !== null) {
    content = (
      <p role="alert" className="m-0 text-warning">
        {formatSecretError(listError)}
      </p>
    );
  } else if (items.length === 0 && filtersActive) {
    content = <EmptyState heading="No secrets match these filters." />;
  } else if (items.length === 0) {
    content = (
      <EmptyState
        heading="No secrets yet"
        message="Add an API key, password, or token. Values are encrypted on this device."
      />
    );
  } else {
    content = (
      <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
        {items.map(renderRow)}
      </ul>
    );
  }

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Secrets</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Encrypted values, shown only when you reveal or copy them.
      </p>
      <div className="mb-6 flex flex-wrap items-center gap-3">
        <Link to="/secrets/new" className={buttonPrimary}>
          Add secret
        </Link>
        <div className="flex-1" />
        <select
          aria-label="Filter by project"
          className={selectClass}
          value={projectFilter}
          onChange={(event) => setProjectFilter(event.target.value)}
        >
          <option value="all">All projects</option>
          <option value="none">No project</option>
          {projects.map((project) => (
            <option key={project.id} value={String(project.id)}>
              {project.display_name}
            </option>
          ))}
        </select>
        <select
          aria-label="Filter by type"
          className={selectClass}
          value={typeFilter}
          onChange={(event) => setTypeFilter(event.target.value as SecretType | "all")}
        >
          <option value="all">All types</option>
          {SECRET_TYPE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {filtersActive ? (
          <button type="button" className={buttonSecondary} onClick={clearFilters}>
            Clear filters
          </button>
        ) : null}
      </div>
      {actionError !== null ? (
        <p role="alert" className="m-0 mb-4 text-warning">
          {formatSecretError(actionError)}
        </p>
      ) : null}
      {content}
    </div>
  );
}

export function SecretsScreen() {
  return (
    <VaultGate>
      <SecretsList />
    </VaultGate>
  );
}
