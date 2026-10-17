import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { vaultLock, vaultSetAutoLock, vaultSetup, vaultUnlock } from "../lib/commands";
import { VAULT_STATUS_KEY, useVaultStatusQuery } from "../lib/use-vault-shell";
import { formatVaultError, lockReasonText, validateSetup } from "../lib/vault-form";
import type { VaultCode, VaultStatusResult, VaultStatusView } from "../types/commands";

const AUTO_LOCK_OPTIONS = [
  { value: 0, label: "Never" },
  { value: 1, label: "1 minute" },
  { value: 5, label: "5 minutes" },
  { value: 15, label: "15 minutes" },
] as const;

const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const inputClass =
  "w-full max-w-sm rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink focus:outline-2 focus:outline-accent";

function SetupView({ onDone }: { onDone: (result: VaultStatusResult) => void }) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [error, setError] = useState<VaultCode | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) return;
    const local = validateSetup(password, confirmation, acknowledged);
    if (local !== null) {
      setError(local);
      return;
    }
    setBusy(true);
    try {
      const result = await vaultSetup({ password, confirmation, acknowledged });
      setPassword("");
      setConfirmation("");
      if (result.error !== null) {
        setError(result.error);
        return;
      }
      setError(null);
      onDone(result);
    } catch {
      setPassword("");
      setConfirmation("");
      setError("vault_storage_unavailable");
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-4">
      <p className="m-0 text-ink-muted">Choose a master password to protect your vault.</p>
      <label className="flex flex-col gap-1">
        <span>Master password</span>
        <input
          type="password"
          autoComplete="new-password"
          className={inputClass}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </label>
      <label className="flex flex-col gap-1">
        <span>Confirm master password</span>
        <input
          type="password"
          autoComplete="new-password"
          className={inputClass}
          value={confirmation}
          onChange={(e) => setConfirmation(e.target.value)}
        />
      </label>
      <p className="m-0 text-warning">
        If you forget this password, the vault cannot be opened. There is no reset.
      </p>
      <label className="flex items-center gap-2">
        <input
          type="checkbox"
          checked={acknowledged}
          onChange={(e) => setAcknowledged(e.target.checked)}
        />
        <span>I understand there is no recovery.</span>
      </label>
      {error !== null ? (
        <p role="alert" className="m-0 text-warning">
          {formatVaultError(error)}
        </p>
      ) : null}
      <div>
        <button type="submit" className={buttonPrimary} disabled={!acknowledged || busy}>
          {busy ? "Creating…" : "Create vault"}
        </button>
      </div>
    </form>
  );
}

function LockView({
  status,
  onDone,
}: {
  status: VaultStatusView;
  onDone: (result: VaultStatusResult) => void;
}) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<VaultCode | null>(null);
  const [busy, setBusy] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // AC-08: after a failed attempt the field is empty and focused.
  useEffect(() => {
    if (error !== null && !busy) inputRef.current?.focus();
  }, [error, busy]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) return;
    const attempt = password;
    if (attempt === "") {
      setError("vault_password_required");
      return;
    }
    setPassword("");
    setBusy(true);
    try {
      const result = await vaultUnlock(attempt);
      if (result.error !== null) {
        setError(result.error);
        return;
      }
      setError(null);
      onDone(result);
    } catch {
      setError("vault_storage_unavailable");
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-4">
      <p className="m-0 text-ink">{lockReasonText(status.lock_reason, status.auto_lock_minutes)}</p>
      <label className="flex flex-col gap-1">
        <span>Master password</span>
        <input
          ref={inputRef}
          type="password"
          autoComplete="current-password"
          className={inputClass}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </label>
      {error !== null ? (
        <p role="alert" className="m-0 text-warning">
          {formatVaultError(error)}
        </p>
      ) : null}
      <div>
        <button type="submit" className={buttonPrimary} disabled={busy}>
          {busy ? "Unlocking…" : "Unlock"}
        </button>
      </div>
    </form>
  );
}

function UnlockedView({
  status,
  onDone,
}: {
  status: VaultStatusView;
  onDone: (result: VaultStatusResult) => void;
}) {
  const [error, setError] = useState<VaultCode | null>(null);
  const [busy, setBusy] = useState(false);

  const run = async (action: () => Promise<VaultStatusResult>) => {
    setBusy(true);
    try {
      const result = await action();
      if (result.error !== null) {
        setError(result.error);
        return;
      }
      setError(null);
      onDone(result);
    } catch {
      setError("vault_storage_unavailable");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <p className="m-0 text-ink">The vault is unlocked.</p>
      <label className="flex flex-col gap-1">
        <span>Auto-lock after inactivity</span>
        <select
          className={inputClass}
          value={status.auto_lock_minutes}
          disabled={busy}
          onChange={(e) => void run(() => vaultSetAutoLock(Number(e.target.value)))}
        >
          {AUTO_LOCK_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
      {error !== null ? (
        <p role="alert" className="m-0 text-warning">
          {formatVaultError(error)}
        </p>
      ) : null}
      <div>
        <button
          type="button"
          className={buttonSecondary}
          disabled={busy}
          onClick={() => void run(vaultLock)}
        >
          Lock now
        </button>
      </div>
    </div>
  );
}

export function VaultScreen() {
  const queryClient = useQueryClient();
  const query = useVaultStatusQuery();

  const status = query.data?.status ?? null;
  const publish = (result: VaultStatusResult) => {
    queryClient.setQueryData(VAULT_STATUS_KEY, result);
  };

  const errorCode: VaultCode | null =
    query.data?.error ?? (query.isError ? "vault_storage_unavailable" : null);

  let content;
  if (query.isLoading) {
    content = <p className="m-0 text-ink-muted">Loading vault…</p>;
  } else if (status === null) {
    content = (
      <p role="alert" className="m-0 text-warning">
        {formatVaultError(errorCode ?? "vault_storage_unavailable")}
      </p>
    );
  } else if (status.state === "not_set_up") {
    content = <SetupView onDone={publish} />;
  } else if (status.state === "locked") {
    content = <LockView status={status} onDone={publish} />;
  } else {
    content = <UnlockedView status={status} onDone={publish} />;
  }

  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Vault</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Protected by a master password that stays on this device.
      </p>
      {content}
    </div>
  );
}
