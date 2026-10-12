import { useState } from "react";
import { pickWorkspaceItem, relinkWorkspaceItem } from "../lib/commands";
import { formatOpenError } from "../lib/editor-form";
import {
  formatRelinkError,
  groupOutcomes,
  outcomeReason,
  relinkedNotice,
  restoreSummaryLine,
} from "../lib/restore-form";
import { formatWorkspaceError } from "../lib/workspace-form";
import type {
  EditorProfileView,
  RestoreOutcomeView,
  WorkspaceItemView,
  WorkspaceView,
} from "../types/commands";

const inputClass =
  "w-full rounded-lg border bg-canvas px-3 py-2 font-mono text-sm text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-3 py-1.5 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

function itemName(path: string): string {
  const segments = path.split(/[\\/]/).filter((segment) => segment !== "");
  return segments[segments.length - 1] ?? path;
}

interface RestorePanelProps {
  outcomes: RestoreOutcomeView[];
  items: WorkspaceItemView[];
  profiles: EditorProfileView[];
  onClose: () => void;
  onRelinked: (workspace: WorkspaceView) => void;
}

interface RowProps {
  item: WorkspaceItemView;
  reason: string;
  canFix: boolean;
  relinked: boolean;
  onIgnore: () => void;
  onRelinked: (workspace: WorkspaceView) => void;
  markRelinked: () => void;
}

function RestoreRow({
  item,
  reason,
  canFix,
  relinked,
  onIgnore,
  onRelinked,
  markRelinked,
}: RowProps) {
  const [editing, setEditing] = useState(false);
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const name = itemName(item.path);
  const isFolder = item.kind === "folder";

  const startRelink = () => {
    setValue(item.path);
    setError(null);
    setEditing(true);
  };

  const browse = async () => {
    setBusy(true);
    try {
      const result = await pickWorkspaceItem(isFolder);
      if (result.path !== null) {
        setValue(result.path);
        setError(null);
      }
    } catch {
      setError(formatWorkspaceError("storage_unavailable", null, ""));
    } finally {
      setBusy(false);
    }
  };

  const apply = async () => {
    if (busy) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await relinkWorkspaceItem(item.id, value);
      if (result.error !== null || result.workspace === null) {
        const code = result.error ?? "storage_unavailable";
        setError(
          formatRelinkError(code, code === "item_kind_mismatch" ? item.kind : result.detail),
        );
        return;
      }
      onRelinked(result.workspace);
      markRelinked();
      setEditing(false);
    } catch {
      setError(formatWorkspaceError("storage_unavailable", null, ""));
    } finally {
      setBusy(false);
    }
  };

  const errorId = `relink-error-${item.id}`;

  return (
    <li className="border-t border-hairline px-4 py-3 first:border-t-0">
      <div className="flex flex-wrap items-start gap-x-4 gap-y-2">
        <div className="min-w-0 flex-1">
          <span className="block font-semibold">{name}</span>
          <span className="block break-all font-mono text-sm text-ink-muted">{item.path}</span>
          <span className="block text-sm text-ink-muted">{reason}</span>
          {relinked ? (
            <span role="status" className="block text-sm text-accent">
              {relinkedNotice()}
            </span>
          ) : null}
        </div>
        {canFix && !editing ? (
          <div className="flex gap-2">
            <button
              type="button"
              onClick={startRelink}
              aria-label={`Relink ${name}`}
              className={buttonSecondary}
            >
              Relink
            </button>
            <button
              type="button"
              onClick={onIgnore}
              aria-label={`Ignore ${name}`}
              className={buttonSecondary}
            >
              Ignore
            </button>
          </div>
        ) : null}
      </div>
      {editing ? (
        <div className="mt-3">
          <label htmlFor={`relink-path-${item.id}`} className="sr-only">
            New path for {name}
          </label>
          <input
            id={`relink-path-${item.id}`}
            autoFocus
            value={value}
            onChange={(event) => setValue(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                setEditing(false);
              }
            }}
            aria-invalid={error !== null}
            aria-describedby={error !== null ? errorId : undefined}
            className={`${inputClass} ${error !== null ? "border-warning" : "border-hairline"}`}
          />
          {error !== null ? (
            <p id={errorId} role="alert" className="m-0 mt-1 text-sm text-warning">
              {error}
            </p>
          ) : null}
          <div className="mt-2 flex flex-wrap gap-2">
            <button
              type="button"
              disabled={busy}
              onClick={() => void browse()}
              className={buttonSecondary}
            >
              {isFolder ? "Browse folder..." : "Browse file..."}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => void apply()}
              className={buttonPrimary}
            >
              Apply
            </button>
            <button type="button" onClick={() => setEditing(false)} className={buttonSecondary}>
              Cancel
            </button>
          </div>
        </div>
      ) : null}
    </li>
  );
}

export function RestorePanel({
  outcomes,
  items,
  profiles,
  onClose,
  onRelinked,
}: RestorePanelProps) {
  const [ignored, setIgnored] = useState<ReadonlySet<number>>(new Set());
  const [relinkedIds, setRelinkedIds] = useState<ReadonlySet<number>>(new Set());

  const opened = outcomes.filter((outcome) => outcome.kind === "opened").length;
  const groups = groupOutcomes(outcomes);
  const missing = groups.missing.filter((outcome) => !ignored.has(outcome.item_id));
  const unreachable = groups.unreachable.filter((outcome) => !ignored.has(outcome.item_id));
  const couldNotOpen = groups.couldNotOpen;
  const summary = restoreSummaryLine(
    opened,
    missing.length,
    unreachable.length,
    couldNotOpen.length,
    ignored.size,
  );

  const itemById = new Map(items.map((item) => [item.id, item]));

  const ignore = (itemId: number) => {
    setIgnored((current) => new Set(current).add(itemId));
  };
  const markRelinked = (itemId: number) => {
    setRelinkedIds((current) => new Set(current).add(itemId));
  };

  const reasonFor = (outcome: RestoreOutcomeView, item: WorkspaceItemView): string => {
    if (outcome.kind === "could_not_open") {
      const profile = profiles.find((candidate) => candidate.kind === item.editor_kind);
      return formatOpenError(outcome.code, item.editor_kind, profile?.name ?? "The editor", null);
    }
    return outcomeReason(outcome);
  };

  const renderGroup = (heading: string, rows: RestoreOutcomeView[], canFix: boolean) => {
    const visible = rows.flatMap((outcome) => {
      const item = itemById.get(outcome.item_id);
      return item === undefined ? [] : [{ outcome, item }];
    });
    if (visible.length === 0) {
      return null;
    }
    return (
      <section aria-label={heading} className="border-t border-hairline">
        <h3 className="m-0 px-4 pt-3 text-sm font-semibold text-ink-muted">
          {heading} ({visible.length})
        </h3>
        <ul className="m-0 list-none p-0">
          {visible.map(({ outcome, item }) => (
            <RestoreRow
              key={item.id}
              item={item}
              reason={reasonFor(outcome, item)}
              canFix={canFix}
              relinked={relinkedIds.has(item.id)}
              onIgnore={() => ignore(item.id)}
              onRelinked={onRelinked}
              markRelinked={() => markRelinked(item.id)}
            />
          ))}
        </ul>
      </section>
    );
  };

  return (
    <section
      aria-label="Restore results"
      className="mb-6 rounded-2xl border border-hairline bg-surface"
    >
      <div className="flex items-center justify-between gap-4 px-4 py-3">
        <p role="status" className="m-0">
          {summary}
        </p>
        <button type="button" onClick={onClose} className={buttonSecondary}>
          Close
        </button>
      </div>
      {renderGroup("Missing", missing, true)}
      {renderGroup("Unreachable", unreachable, true)}
      {renderGroup("Could not open", couldNotOpen, false)}
    </section>
  );
}
