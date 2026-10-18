import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  appInfo,
  checkForUpdate,
  getUpdatePrefs,
  installUpdate,
  setUpdatePrefs,
} from "../lib/commands";
import { formatUpdateError } from "../lib/update-form";
import type { UpdateErrorCode } from "../types/commands";

const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

export function UpdatesSection() {
  const queryClient = useQueryClient();
  const info = useQuery({ queryKey: ["app-info"], queryFn: appInfo });
  const prefs = useQuery({ queryKey: ["update-prefs"], queryFn: getUpdatePrefs });
  const setPrefs = useMutation({
    mutationFn: setUpdatePrefs,
    onSuccess: (result) => queryClient.setQueryData(["update-prefs"], result),
  });
  const check = useMutation({
    mutationFn: checkForUpdate,
    onSuccess: (result) => {
      if (!result.error) {
        queryClient.setQueryData(["update-startup"], result.update);
      }
    },
  });
  const install = useMutation({ mutationFn: installUpdate });

  const prefsError: UpdateErrorCode | null = setPrefs.data?.error ?? prefs.data?.error ?? null;
  const checkError: UpdateErrorCode | null = check.isError
    ? "check_failed"
    : (check.data?.error ?? null);
  const found = check.data?.update ?? null;
  const upToDate = check.isSuccess && !found && !checkError;
  const installError: UpdateErrorCode | null =
    install.data?.error ?? (install.isError ? "install_failed" : null);

  return (
    <section className="mb-6 rounded-2xl border border-hairline bg-surface p-5">
      <h2 className="m-0 mb-3 text-xl font-semibold">Updates</h2>
      <p className="m-0 mb-4 text-ink-muted">Current version: {info.data?.version ?? "…"}</p>
      <label className="mb-4 flex cursor-pointer items-center gap-2">
        <input
          type="checkbox"
          checked={prefs.data?.auto_check ?? false}
          disabled={!prefs.data || setPrefs.isPending}
          onChange={(event) => setPrefs.mutate(event.target.checked)}
          className="accent-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        />
        Check for updates when DevShelf starts
      </label>
      <div className="flex flex-wrap items-center gap-3">
        <button
          type="button"
          onClick={() => check.mutate()}
          disabled={check.isPending}
          className={buttonSecondary}
        >
          {check.isPending ? "Checking…" : "Check for updates"}
        </button>
        {upToDate ? (
          <p role="status" className="m-0 text-ink-muted">
            You're up to date.
          </p>
        ) : null}
        {found ? (
          <>
            <p role="status" className="m-0 text-ink">
              DevShelf {found.version} is available.
            </p>
            <button
              type="button"
              onClick={() => install.mutate()}
              disabled={install.isPending}
              className={buttonPrimary}
            >
              {install.isPending ? "Installing…" : "Install and restart"}
            </button>
          </>
        ) : null}
      </div>
      {prefsError ? (
        <p role="alert" className="m-0 mt-3 text-warning">
          {formatUpdateError(prefsError)}
        </p>
      ) : null}
      {checkError ? (
        <p role="alert" className="m-0 mt-3 text-warning">
          {formatUpdateError(checkError)}
        </p>
      ) : null}
      {installError ? (
        <p role="alert" className="m-0 mt-3 text-warning">
          {formatUpdateError(installError)}
        </p>
      ) : null}
    </section>
  );
}
