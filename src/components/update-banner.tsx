import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { checkForUpdate, getUpdatePrefs, installUpdate } from "../lib/commands";
import { formatUpdateError, startupUpdateCheck } from "../lib/update-form";
import type { UpdateErrorCode } from "../types/commands";

const buttonPrimary =
  "rounded-lg bg-accent px-3 py-1.5 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

export function UpdateBanner() {
  const [dismissed, setDismissed] = useState(false);
  const startup = useQuery({
    queryKey: ["update-startup"],
    queryFn: () => startupUpdateCheck({ getPrefs: getUpdatePrefs, check: checkForUpdate }),
  });
  const install = useMutation({ mutationFn: installUpdate });

  const update = startup.data;
  if (!update || dismissed) {
    return null;
  }

  const installError: UpdateErrorCode | null =
    install.data?.error ?? (install.isError ? "install_failed" : null);

  return (
    <div className="m-0 mb-4 flex flex-wrap items-center gap-3 rounded-lg border border-hairline bg-surface-raised px-3 py-2">
      <p role="status" className="m-0 text-ink">
        DevShelf {update.version} is available.
      </p>
      <button
        type="button"
        onClick={() => install.mutate()}
        disabled={install.isPending}
        className={buttonPrimary}
      >
        {install.isPending ? "Installing…" : "Install and restart"}
      </button>
      <button type="button" onClick={() => setDismissed(true)} className={buttonSecondary}>
        Later
      </button>
      {installError ? (
        <p role="alert" className="m-0 w-full text-warning">
          {formatUpdateError(installError)}
        </p>
      ) : null}
    </div>
  );
}
