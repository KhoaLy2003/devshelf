import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import {
  openConfigFile,
  pickConfigFile,
  relinkConfigFile,
  removeConfigFile,
} from "../lib/commands";
import { environmentLabel, formatConfigError, kindLabel, statusReason } from "../lib/config-form";
import { formatOpenError, openSuccessMessage } from "../lib/editor-form";
import type { ConfigEntryView, EditorProfileView } from "../types/commands";

interface ConfigRowProps {
  entry: ConfigEntryView;
  profiles: EditorProfileView[];
  highlighted: boolean;
}

const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-sm text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const inputClass =
  "rounded-lg border border-hairline bg-canvas px-3 py-1.5 text-sm text-ink focus:outline-2 focus:outline-accent";

function statusPill(entry: ConfigEntryView): { label: string; className: string } {
  switch (entry.status) {
    case "available":
      return { label: "Available", className: "border-accent text-accent" };
    case "inaccessible":
      return { label: "Inaccessible", className: "border-warning text-warning" };
    default:
      return { label: "Missing", className: "border-warning text-warning" };
  }
}

export function ConfigRow({ entry, profiles, highlighted }: ConfigRowProps) {
  const queryClient = useQueryClient();
  const [profileKind, setProfileKind] = useState("");
  const [openMessage, setOpenMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const [opening, setOpening] = useState(false);
  const [relinking, setRelinking] = useState(false);
  const [relinkPath, setRelinkPath] = useState("");
  const [relinkError, setRelinkError] = useState<string | null>(null);
  const [relinkBusy, setRelinkBusy] = useState(false);
  const [confirmingRemove, setConfirmingRemove] = useState(false);
  const [removeBusy, setRemoveBusy] = useState(false);
  const [removeError, setRemoveError] = useState<string | null>(null);

  const available = entry.status === "available";
  const pill = statusPill(entry);
  const reason = statusReason(entry.status);
  const selectedProfile = profiles.find((profile) => profile.kind === profileKind);

  useEffect(() => {
    if (entry.status === "available") {
      setOpenMessage(null);
    }
  }, [entry.status]);

  const open = async () => {
    if (opening) {
      return;
    }
    setOpening(true);
    setOpenMessage(null);
    const chosen = selectedProfile ?? null;
    const profileName = chosen?.name ?? "the default editor";
    try {
      const result = await openConfigFile(entry.id, chosen?.kind ?? null);
      if (result.error !== null) {
        if (result.error === "item_not_found") {
          void queryClient.invalidateQueries({ queryKey: ["config-files"] });
          setOpenMessage({
            text: statusReason(
              result.detail === "inaccessible"
                ? "inaccessible"
                : entry.status === "available"
                  ? "missing"
                  : entry.status,
            ),
            ok: false,
          });
        } else {
          setOpenMessage({
            text: formatOpenError(result.error, chosen?.kind ?? null, profileName, result.detail),
            ok: false,
          });
        }
        return;
      }
      const name =
        result.opened_name ??
        chosen?.name ??
        profiles.find((p) => p.is_default)?.name ??
        profileName;
      setOpenMessage({ text: openSuccessMessage(entry.name, name), ok: true });
    } catch {
      setOpenMessage({
        text: formatOpenError("storage_unavailable", null, profileName, null),
        ok: false,
      });
    } finally {
      setOpening(false);
    }
  };

  const browse = async () => {
    try {
      const result = await pickConfigFile();
      if (result.path !== null) {
        setRelinkPath(result.path);
        setRelinkError(null);
      }
    } catch {
      setRelinkError("Could not open the file picker. Type the path instead.");
    }
  };

  const cancelRelink = () => {
    setRelinking(false);
    setRelinkPath("");
    setRelinkError(null);
  };

  const applyRelink = async () => {
    if (relinkBusy) {
      return;
    }
    setRelinkBusy(true);
    setRelinkError(null);
    try {
      const result = await relinkConfigFile(entry.id, relinkPath);
      if (result.error !== null) {
        setRelinkError(formatConfigError(result.error, result.detail, relinkPath));
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["config-files"] });
      setOpenMessage(null);
      cancelRelink();
    } catch {
      setRelinkError(formatConfigError("storage_unavailable", null, relinkPath));
    } finally {
      setRelinkBusy(false);
    }
  };

  const confirmRemove = async () => {
    if (removeBusy) {
      return;
    }
    setRemoveBusy(true);
    setRemoveError(null);
    try {
      const result = await removeConfigFile(entry.id);
      if (result.error !== null) {
        setRemoveError(formatConfigError(result.error, result.detail, entry.name));
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["config-files"] });
    } catch {
      setRemoveError(formatConfigError("storage_unavailable", null, entry.name));
    } finally {
      setRemoveBusy(false);
    }
  };

  return (
    <li
      className={`border-b border-hairline p-4 last:border-b-0 ${highlighted ? "bg-surface-raised outline-2 -outline-offset-2 outline-accent" : ""}`}
    >
      <div className="grid grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_auto_auto_auto] items-start gap-4">
        <div className="min-w-0">
          <p className="m-0 truncate font-medium">{entry.name}</p>
          <p className="m-0 truncate font-mono text-sm text-ink-muted" title={entry.path}>
            {entry.path}
          </p>
        </div>
        <p className="m-0 truncate text-ink-muted">{entry.project_name ?? "No project"}</p>
        <p className="m-0 text-ink-muted">{environmentLabel(entry.environment)}</p>
        <p className="m-0 text-ink-muted">{kindLabel(entry.kind)}</p>
        <div>
          <span
            title={reason}
            className={`inline-block rounded-full border px-2 py-0.5 text-xs ${pill.className}`}
          >
            {pill.label}
          </span>
        </div>
      </div>
      {available ? null : <p className="m-0 mt-1 text-sm text-ink-muted">{reason}</p>}

      <div className="mt-3 flex flex-wrap items-center gap-2">
        {available ? (
          <>
            <button
              type="button"
              onClick={() => void open()}
              disabled={opening}
              aria-label={`Open ${entry.name}`}
              className="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50"
            >
              Open
            </button>
            <select
              aria-label={`Editor profile for ${entry.name}`}
              value={profileKind}
              onChange={(event) => setProfileKind(event.target.value)}
              className={inputClass}
            >
              <option value="">Default profile</option>
              {profiles.map((profile) => (
                <option key={profile.kind} value={profile.kind}>
                  {profile.name}
                </option>
              ))}
            </select>
          </>
        ) : (
          <button
            type="button"
            onClick={() => setRelinking(true)}
            disabled={relinking}
            className={buttonSecondary}
          >
            Relink
          </button>
        )}
        <button
          type="button"
          onClick={() => {
            setRemoveError(null);
            setConfirmingRemove(true);
          }}
          disabled={confirmingRemove}
          className={buttonSecondary}
        >
          Remove from catalog
        </button>
      </div>

      {openMessage !== null ? (
        <p
          role={openMessage.ok ? "status" : "alert"}
          className={`m-0 mt-2 text-sm ${openMessage.ok ? "text-ink-muted" : "text-warning"}`}
        >
          {openMessage.text}
        </p>
      ) : null}

      {relinking ? (
        <div className="mt-3">
          <div className="flex flex-wrap items-center gap-2">
            <input
              aria-label={`New path for ${entry.name}`}
              value={relinkPath}
              onChange={(event) => setRelinkPath(event.target.value)}
              placeholder="D:\work\my-project\application.yml"
              className={`${inputClass} min-w-64 flex-1 font-mono`}
            />
            <button type="button" onClick={() => void browse()} className={buttonSecondary}>
              Browse file...
            </button>
            <button
              type="button"
              onClick={() => void applyRelink()}
              disabled={relinkBusy}
              className={buttonSecondary}
            >
              Apply
            </button>
            <button type="button" onClick={cancelRelink} className={buttonSecondary}>
              Cancel
            </button>
          </div>
          {relinkError !== null ? (
            <p role="alert" className="m-0 mt-1 text-sm text-warning">
              {relinkError}
            </p>
          ) : null}
        </div>
      ) : null}

      {confirmingRemove ? (
        <div role="group" aria-label={`Remove ${entry.name}`} className="mt-3">
          <p className="m-0 mb-2 text-sm">
            Remove `{entry.name}` from the catalog? The file is not changed.
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setConfirmingRemove(false)}
              className={buttonSecondary}
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => void confirmRemove()}
              disabled={removeBusy}
              className={buttonSecondary}
            >
              Remove
            </button>
          </div>
          {removeError !== null ? (
            <p role="alert" className="m-0 mt-1 text-sm text-warning">
              {removeError}
            </p>
          ) : null}
        </div>
      ) : null}
    </li>
  );
}
