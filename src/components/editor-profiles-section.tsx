import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  listEditorProfiles,
  pickExecutable,
  saveEditorProfiles,
  setDefaultEditor,
} from "../lib/commands";
import { formatEditorError } from "../lib/editor-form";
import type { EditorKind, EditorProfileView, EditorProfilesResult } from "../types/commands";

const NAME_MAX = 40;

const inputClass =
  "w-full rounded-lg border bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";

interface Draft {
  name: string;
  exe_path: string;
}

type Drafts = Record<string, Draft>;

function toDrafts(profiles: EditorProfileView[]): Drafts {
  const drafts: Drafts = {};
  for (const profile of profiles) {
    drafts[profile.kind] = { name: profile.name, exe_path: profile.exe_path };
  }
  return drafts;
}

interface SaveFailure {
  kind: string | null;
  message: string;
}

export function EditorProfilesSection() {
  const queryClient = useQueryClient();
  const list = useQuery({ queryKey: ["editor-profiles"], queryFn: listEditorProfiles });
  const profiles = list.data?.profiles;

  const [drafts, setDrafts] = useState<Drafts | null>(null);
  const [saved, setSaved] = useState(false);
  const [failure, setFailure] = useState<SaveFailure | null>(null);
  const [defaultFailed, setDefaultFailed] = useState(false);
  const [defaultFailure, setDefaultFailure] = useState<SaveFailure | null>(null);

  // Seed drafts once from the first load; later refetches (e.g. default change) keep edits.
  useEffect(() => {
    if (drafts === null && profiles !== undefined && profiles.length > 0) {
      setDrafts(toDrafts(profiles));
    }
  }, [drafts, profiles]);

  function describe(result: EditorProfilesResult): SaveFailure | null {
    if (result.error === null) return null;
    const match = profiles?.find((profile) => profile.kind === result.kind);
    const name =
      (result.kind !== null ? drafts?.[result.kind]?.name : undefined) ?? match?.name ?? "";
    return {
      kind: result.kind,
      message: formatEditorError(result.error, result.kind, name, result.detail),
    };
  }

  const save = useMutation({
    mutationFn: saveEditorProfiles,
    onSuccess: (result) => {
      const problem = describe(result);
      setFailure(problem);
      setSaved(problem === null);
      if (problem === null) {
        setDefaultFailure(null);
        setDefaultFailed(false);
        queryClient.setQueryData(["editor-profiles"], result);
        setDrafts(toDrafts(result.profiles));
      }
    },
    onError: () => {
      setSaved(false);
      setFailure({
        kind: null,
        message: formatEditorError("storage_unavailable", null, "", null),
      });
    },
  });

  const makeDefault = useMutation({
    mutationFn: setDefaultEditor,
    onSuccess: async (result, kind) => {
      if (result.error !== null) {
        const name = drafts?.[kind]?.name ?? profiles?.find((p) => p.kind === kind)?.name ?? "";
        setDefaultFailure({
          kind,
          message: formatEditorError(result.error, kind, name, null),
        });
        setDefaultFailed(false);
      } else {
        setDefaultFailure(null);
        setDefaultFailed(false);
      }
      await queryClient.invalidateQueries({ queryKey: ["editor-profiles"] });
    },
    onError: () => setDefaultFailed(true),
  });

  function edit(kind: string, patch: Partial<Draft>) {
    setDrafts((current) => {
      if (current === null) return current;
      const existing = current[kind];
      if (existing === undefined) return current;
      return { ...current, [kind]: { ...existing, ...patch } };
    });
    setSaved(false);
  }

  async function browse(kind: string) {
    const result = await pickExecutable();
    if (result.path !== null) edit(kind, { exe_path: result.path });
  }

  function onSave() {
    if (drafts === null || profiles === undefined) return;
    setSaved(false);
    setFailure(null);
    save.mutate(
      profiles.map((profile) => {
        const draft = drafts[profile.kind] ?? { name: profile.name, exe_path: profile.exe_path };
        return { kind: profile.kind, name: draft.name, exe_path: draft.exe_path };
      }),
    );
  }

  function onReset() {
    if (profiles === undefined) return;
    setDrafts(toDrafts(profiles));
    setSaved(false);
    setFailure(null);
  }

  return (
    <section
      className="mb-6 rounded-2xl border border-hairline bg-surface p-5"
      aria-labelledby="ep-heading"
    >
      <h2 id="ep-heading" className="m-0 mb-1 text-xl font-semibold">
        Editors
      </h2>
      <p className="m-0 mb-4 text-sm text-ink-muted">
        Choose which editor opens your project files and folders.
      </p>

      {list.isError || (list.data !== undefined && list.data.error !== null) ? (
        <p role="alert" className="m-0 text-warning">
          {list.data?.error != null
            ? formatEditorError(list.data.error, list.data.kind, "", list.data.detail)
            : "DevShelf cannot reach its data folder. Restart DevShelf and try again."}
        </p>
      ) : null}

      {profiles !== undefined && drafts !== null ? (
        <>
          <ul className="m-0 flex list-none flex-col gap-4 p-0">
            {profiles.map((profile) => {
              const draft = drafts[profile.kind] ?? {
                name: profile.name,
                exe_path: profile.exe_path,
              };
              const isWindows = profile.kind === ("windows_default" satisfies EditorKind);
              const isCustom = profile.kind === ("custom" satisfies EditorKind);
              const error =
                failure !== null && failure.kind === profile.kind
                  ? failure.message
                  : defaultFailure !== null && defaultFailure.kind === profile.kind
                    ? defaultFailure.message
                    : undefined;
              const errorId = `ep-${profile.kind}-error`;
              return (
                <li
                  key={profile.kind}
                  className="rounded-xl border border-hairline bg-surface-raised p-4"
                >
                  <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                      {isCustom ? (
                        <>
                          <label htmlFor="ep-custom-name" className="sr-only">
                            Custom editor name
                          </label>
                          <input
                            id="ep-custom-name"
                            value={draft.name}
                            maxLength={NAME_MAX}
                            onChange={(event) => edit(profile.kind, { name: event.target.value })}
                            className={`${inputClass} w-56 border-hairline font-medium`}
                          />
                        </>
                      ) : (
                        <span className="font-medium">{profile.name}</span>
                      )}
                      {profile.is_default ? (
                        <span className="rounded-full border border-accent px-2 py-0.5 text-xs text-accent">
                          Default
                        </span>
                      ) : null}
                    </div>
                    {profile.is_default ? null : (
                      <button
                        type="button"
                        disabled={makeDefault.isPending}
                        onClick={() => {
                          setDefaultFailed(false);
                          setDefaultFailure(null);
                          makeDefault.mutate(profile.kind);
                        }}
                        className={buttonSecondary}
                      >
                        Set as default
                      </button>
                    )}
                  </div>
                  <label
                    htmlFor={`ep-${profile.kind}-path`}
                    className="mb-1 block text-sm text-ink-muted"
                  >
                    Executable path
                  </label>
                  <div className="flex gap-2">
                    <input
                      id={`ep-${profile.kind}-path`}
                      value={isWindows ? "" : draft.exe_path}
                      disabled={isWindows}
                      onChange={(event) => edit(profile.kind, { exe_path: event.target.value })}
                      aria-invalid={error !== undefined}
                      aria-describedby={
                        error !== undefined
                          ? errorId
                          : isWindows
                            ? `ep-${profile.kind}-note`
                            : undefined
                      }
                      placeholder={isWindows ? "" : "C:\\Program Files\\Editor\\editor.exe"}
                      className={`${inputClass} font-mono ${error !== undefined ? "border-warning" : "border-hairline"}`}
                    />
                    {isWindows ? null : (
                      <button
                        type="button"
                        onClick={() => void browse(profile.kind)}
                        className={buttonSecondary}
                      >
                        Browse...
                      </button>
                    )}
                  </div>
                  {isWindows ? (
                    <p id={`ep-${profile.kind}-note`} className="m-0 mt-1 text-sm text-ink-muted">
                      Opens with the Windows file association
                    </p>
                  ) : null}
                  {error !== undefined ? (
                    <p id={errorId} role="alert" className="m-0 mt-1 text-sm text-warning">
                      {error}
                    </p>
                  ) : null}
                </li>
              );
            })}
          </ul>

          {failure !== null && failure.kind === null ? (
            <p role="alert" className="m-0 mt-4 text-sm text-warning">
              {failure.message}
            </p>
          ) : null}
          {defaultFailed ? (
            <p role="alert" className="m-0 mt-4 text-sm text-warning">
              Could not change the default editor. Try again.
            </p>
          ) : null}

          <div className="mt-4 flex items-center gap-2">
            <button
              type="button"
              onClick={onSave}
              disabled={save.isPending}
              className={buttonPrimary}
            >
              Save
            </button>
            <button type="button" onClick={onReset} className={buttonSecondary}>
              Reset
            </button>
            {saved ? (
              <p role="status" className="m-0 text-sm text-ink-muted">
                Settings saved
              </p>
            ) : null}
          </div>
        </>
      ) : null}
    </section>
  );
}
