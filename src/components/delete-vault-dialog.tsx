import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { vaultDelete, vaultDeletePreview, vaultStatus } from "../lib/commands";
import {
  canConfirmVaultDelete,
  formatVaultDeleteError,
  vaultDeleteMessage,
} from "../lib/project-removal";
import { VAULT_STATUS_KEY } from "../lib/use-vault-shell";

interface DeleteVaultDialogProps {
  onClose: () => void;
}

const inputClass =
  "w-full rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonDanger =
  "rounded-lg bg-warning px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

export function DeleteVaultDialog({ onClose }: DeleteVaultDialogProps) {
  const queryClient = useQueryClient();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [password, setPassword] = useState("");
  const [typed, setTyped] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const preview = useQuery({
    queryKey: ["vault-delete-preview"],
    queryFn: vaultDeletePreview,
    staleTime: 0,
    gcTime: 0,
  });

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const previewError =
    preview.data?.error ?? (preview.isError ? "vault_storage_unavailable" : null);
  const ready = preview.data !== undefined && preview.data.error === null;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy || !ready || !canConfirmVaultDelete(typed)) {
      return;
    }
    if (password === "") {
      setError(formatVaultDeleteError("vault_password_required"));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await vaultDelete(password);
      if (result.error !== null) {
        setError(formatVaultDeleteError(result.error));
        return;
      }
      setPassword("");
      queryClient.setQueryData(VAULT_STATUS_KEY, await vaultStatus());
      await queryClient.invalidateQueries({ queryKey: ["secrets"] });
      onClose();
    } catch {
      setError(formatVaultDeleteError("vault_storage_unavailable"));
    } finally {
      setBusy(false);
    }
  };

  const shownError = error ?? (previewError !== null ? formatVaultDeleteError(previewError) : null);

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="delete-vault-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(30rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <form onSubmit={(event) => void submit(event)} noValidate>
        <h2 id="delete-vault-title" className="m-0 mb-4 text-xl font-semibold">
          Delete vault
        </h2>
        {preview.isLoading ? <p className="m-0 mb-4 text-ink-muted">Counting secrets…</p> : null}
        {ready ? <p className="m-0 mb-4">{vaultDeleteMessage(preview.data.secret_count)}</p> : null}
        <div className="mb-4">
          <label htmlFor="dv-password" className="mb-1 block text-sm text-ink-muted">
            Master password
          </label>
          <input
            id="dv-password"
            type="password"
            autoComplete="current-password"
            autoFocus
            value={password}
            disabled={busy}
            onChange={(event) => setPassword(event.target.value)}
            className={inputClass}
          />
        </div>
        <div className="mb-4">
          <label htmlFor="dv-typed" className="mb-1 block text-sm text-ink-muted">
            Type DELETE to confirm
          </label>
          <input
            id="dv-typed"
            autoComplete="off"
            value={typed}
            disabled={busy}
            onChange={(event) => setTyped(event.target.value)}
            className={inputClass}
          />
        </div>
        {shownError !== null ? (
          <p role="alert" className="m-0 mb-2 text-warning">
            {shownError}
          </p>
        ) : null}
        <div className="mt-6 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={buttonSecondary}>
            Cancel
          </button>
          <button
            type="submit"
            disabled={busy || !ready || !canConfirmVaultDelete(typed)}
            className={buttonDanger}
          >
            Delete vault
          </button>
        </div>
      </form>
    </dialog>
  );
}
