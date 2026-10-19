import { useEffect, useRef, useState, type FormEvent } from "react";
import { exportData } from "../lib/commands";
import { canSubmitExport, exportSuccessText, formatExportError } from "../lib/export-form";
import { useVaultStatusQuery } from "../lib/use-vault-shell";
import type { DataExportResult, ExportErrorCode } from "../types/commands";

interface ExportDataDialogProps {
  onClose: () => void;
}

const inputClass =
  "w-full rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

/** Plaintext CSV export. The password is only asked for when a vault exists. */
export function ExportDataDialog({ onClose }: ExportDataDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const vaultStatus = useVaultStatusQuery();
  const vaultSetUp = vaultStatus.data?.status?.state !== "not_set_up";
  const [password, setPassword] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [error, setError] = useState<ExportErrorCode | null>(null);
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<DataExportResult | null>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy || !canSubmitExport(vaultSetUp, password, acknowledged)) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await exportData(password, acknowledged);
      if (result.cancelled) {
        return;
      }
      if (result.error !== null) {
        setError(result.error);
        return;
      }
      setDone(result);
    } catch {
      setError("export_storage_unavailable");
    } finally {
      setPassword("");
      setBusy(false);
    }
  };

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="export-data-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(36rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      {done !== null ? (
        <div>
          <h2 id="export-data-title" className="m-0 mb-4 text-xl font-semibold">
            Export complete
          </h2>
          <p className="m-0 mb-4 break-all">
            {exportSuccessText(done.folder ?? "", done.files.length)}
          </p>
          <ul className="m-0 mb-4 list-none p-0 text-sm text-ink-muted">
            {done.files.map((file) => (
              <li key={file.name}>
                {file.name} — {file.rows} rows
              </li>
            ))}
          </ul>
          <div className="flex justify-end">
            <button type="button" onClick={onClose} className={buttonSecondary}>
              Close
            </button>
          </div>
        </div>
      ) : (
        <form onSubmit={(event) => void submit(event)} noValidate>
          <h2 id="export-data-title" className="m-0 mb-4 text-xl font-semibold">
            Export data
          </h2>
          <p className="m-0 mb-4 text-warning">
            This export is not encrypted. Your notes and secret values are written as plain text to
            CSV files and Markdown note files that anyone who can open the folder can read,
            including backup and cloud-sync tools. Save them somewhere private, never inside a
            project repository, and delete them when you no longer need them.
          </p>
          <p className="m-0 mb-4 text-ink-muted">
            Includes projects, workspaces, config files, editor profiles and secrets as CSV, and
            each note as its own Markdown file in a notes folder.
          </p>
          {vaultSetUp ? (
            <div className="mb-4">
              <label htmlFor="ed-password" className="mb-1 block text-sm text-ink-muted">
                Master password
              </label>
              <input
                id="ed-password"
                type="password"
                autoComplete="current-password"
                className={inputClass}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </div>
          ) : null}
          <label className="mb-4 flex items-center gap-2">
            <input
              type="checkbox"
              checked={acknowledged}
              onChange={(event) => setAcknowledged(event.target.checked)}
            />
            <span>I understand the exported files contain my notes and secrets in plain text.</span>
          </label>
          {error !== null ? (
            <p role="alert" className="m-0 mb-4 text-warning">
              {formatExportError(error)}
            </p>
          ) : null}
          <div className="flex justify-end gap-2">
            <button type="button" onClick={onClose} className={buttonSecondary}>
              Cancel
            </button>
            <button
              type="submit"
              className={buttonPrimary}
              disabled={busy || !canSubmitExport(vaultSetUp, password, acknowledged)}
            >
              Choose folder and export
            </button>
          </div>
        </form>
      )}
    </dialog>
  );
}
