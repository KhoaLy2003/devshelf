import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { listEditorProfiles, openInEditor, pickItem } from "../lib/commands";
import { fileNameOf, formatOpenError, openSuccessMessage } from "../lib/editor-form";

const inputClass =
  "w-full rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

interface Outcome {
  ok: boolean;
  message: string;
}

export function EditorOpenTest() {
  const list = useQuery({ queryKey: ["editor-profiles"], queryFn: listEditorProfiles });
  const profiles = list.data?.profiles ?? [];
  const defaultKind =
    profiles.find((profile) => profile.is_default)?.kind ?? profiles[0]?.kind ?? "";

  // The path is local state only and never persisted.
  const [path, setPath] = useState("");
  const [chosenKind, setChosenKind] = useState<string | null>(null);
  const [outcome, setOutcome] = useState<Outcome | null>(null);

  const kind = chosenKind ?? defaultKind;
  const profileName = profiles.find((profile) => profile.kind === kind)?.name ?? "";

  const open = useMutation({
    mutationFn: ({ itemPath, itemKind }: { itemPath: string; itemKind: string }) =>
      openInEditor(itemPath, itemKind),
    onSuccess: (result, variables) => {
      if (result.error !== null) {
        setOutcome({
          ok: false,
          message: formatOpenError(
            result.error,
            result.kind ?? variables.itemKind,
            profileName,
            variables.itemPath,
          ),
        });
        return;
      }
      setOutcome({
        ok: true,
        message: openSuccessMessage(
          fileNameOf(variables.itemPath),
          result.opened_name ?? profileName,
        ),
      });
    },
    onError: () => {
      setOutcome({
        ok: false,
        message: `${profileName} could not start. Check the path in Settings.`,
      });
    },
  });

  async function browse(folder: boolean) {
    const result = await pickItem(folder);
    if (result.path !== null) {
      setPath(result.path);
      setOutcome(null);
    }
  }

  return (
    <section
      className="mb-6 rounded-2xl border border-hairline bg-surface p-5"
      aria-labelledby="eo-heading"
    >
      <h2 id="eo-heading" className="m-0 mb-1 text-xl font-semibold">
        Open test
      </h2>
      <p className="m-0 mb-4 text-sm text-ink-muted">
        Pick a file or folder and open it with one of your editors. The path is not saved.
      </p>

      <label htmlFor="eo-path" className="mb-1 block text-sm text-ink-muted">
        File or folder path
      </label>
      <div className="flex flex-wrap gap-2">
        <input
          id="eo-path"
          value={path}
          onChange={(event) => {
            setPath(event.target.value);
            setOutcome(null);
          }}
          placeholder="D:\work\my-project\README.md"
          className={`${inputClass} min-w-64 flex-1 font-mono`}
        />
        <button type="button" onClick={() => void browse(false)} className={buttonSecondary}>
          Browse file...
        </button>
        <button type="button" onClick={() => void browse(true)} className={buttonSecondary}>
          Browse folder...
        </button>
      </div>

      <div className="mt-4 flex flex-wrap items-end gap-2">
        <div>
          <label htmlFor="eo-profile" className="mb-1 block text-sm text-ink-muted">
            Editor
          </label>
          <select
            id="eo-profile"
            value={kind}
            onChange={(event) => {
              setChosenKind(event.target.value);
              setOutcome(null);
            }}
            className={`${inputClass} w-56`}
          >
            {profiles.map((profile) => (
              <option key={profile.kind} value={profile.kind}>
                {profile.name}
              </option>
            ))}
          </select>
        </div>
        <button
          type="button"
          disabled={open.isPending || path.trim() === "" || kind === ""}
          onClick={() => {
            setOutcome(null);
            open.mutate({ itemPath: path.trim(), itemKind: kind });
          }}
          className={buttonPrimary}
        >
          Open
        </button>
      </div>

      {outcome !== null ? (
        <p
          role={outcome.ok ? "status" : "alert"}
          className={`m-0 mt-3 text-sm ${outcome.ok ? "text-ink-muted" : "text-warning"}`}
        >
          {outcome.message}
        </p>
      ) : null}
    </section>
  );
}
