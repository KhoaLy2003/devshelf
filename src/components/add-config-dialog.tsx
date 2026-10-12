import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { addConfigFile, listProjects, pickConfigFile } from "../lib/commands";
import {
  environmentLabel,
  formatConfigError,
  kindFromFileName,
  kindLabel,
} from "../lib/config-form";
import {
  CONFIG_ENVIRONMENTS,
  CONFIG_KINDS,
  type ConfigCode,
  type ConfigEntryView,
} from "../types/commands";

type Field = "path" | "project" | "environment" | "kind" | "form";

function fieldOf(code: ConfigCode): Field {
  switch (code) {
    case "path_empty":
    case "path_not_found":
    case "path_is_folder":
    case "path_relative":
    case "path_too_long":
    case "duplicate_path":
      return "path";
    case "project_not_found":
      return "project";
    case "environment_invalid":
      return "environment";
    case "kind_invalid":
      return "kind";
    default:
      return "form";
  }
}

interface AddConfigDialogProps {
  onClose: () => void;
  onAdded: (entry: ConfigEntryView) => void;
}

const inputClass =
  "w-full rounded-lg border bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";

function FieldError({ id, message }: { id: string; message: string | undefined }) {
  if (message === undefined) {
    return null;
  }
  return (
    <p id={id} role="alert" className="m-0 mt-1 text-sm text-warning">
      {message}
    </p>
  );
}

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

export function AddConfigDialog({ onClose, onAdded }: AddConfigDialogProps) {
  const queryClient = useQueryClient();
  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [path, setPath] = useState("");
  const [projectId, setProjectId] = useState("");
  const [environment, setEnvironment] = useState("");
  const [kind, setKind] = useState("");
  const [errors, setErrors] = useState<Partial<Record<Field, string>>>({});
  const [busy, setBusy] = useState(false);

  const projects = projectsQuery.data?.projects ?? [];

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const browse = async () => {
    try {
      const result = await pickConfigFile();
      if (result.path !== null) {
        setPath(result.path);
        setErrors((prev) => ({ ...prev, path: undefined }));
        const detected = kindFromFileName(baseName(result.path));
        if (detected !== null) {
          setKind(detected);
          setErrors((prev) => ({ ...prev, kind: undefined }));
        }
      }
    } catch {
      setErrors((prev) => ({
        ...prev,
        form: "Could not open the file picker. Type the path instead.",
      }));
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    const local: Partial<Record<Field, string>> = {};
    if (path.trim() === "") {
      local.path = formatConfigError("path_empty", null, path);
    }
    if (projectId === "") {
      local.project = "Choose a project.";
    }
    if (environment === "") {
      local.environment = formatConfigError("environment_invalid", null, path);
    }
    if (kind === "") {
      local.kind = formatConfigError("kind_invalid", null, path);
    }
    if (Object.keys(local).length > 0) {
      setErrors(local);
      return;
    }
    setBusy(true);
    setErrors({});
    try {
      const result = await addConfigFile({
        project_id: Number(projectId),
        path,
        environment,
        kind,
      });
      if (result.error !== null || result.entry === null) {
        const code = result.error ?? "storage_unavailable";
        setErrors({ [fieldOf(code)]: formatConfigError(code, result.detail, path) });
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["config-files"] });
      onAdded(result.entry);
    } catch {
      setErrors({ form: formatConfigError("storage_unavailable", null, path) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="add-config-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(36rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <form onSubmit={(event) => void submit(event)} noValidate>
        <h2 id="add-config-title" className="m-0 mb-4 text-xl font-semibold">
          Add config file
        </h2>

        <div className="mb-4">
          <label htmlFor="ac-path" className="mb-1 block text-sm text-ink-muted">
            File path
          </label>
          <div className="flex gap-2">
            <input
              id="ac-path"
              autoFocus
              value={path}
              onChange={(event) => setPath(event.target.value)}
              placeholder="D:\work\my-project\application.yml"
              aria-invalid={errors.path !== undefined}
              aria-describedby={errors.path !== undefined ? "ac-path-error" : undefined}
              className={`${inputClass} font-mono ${errors.path ? "border-warning" : "border-hairline"}`}
            />
            <button
              type="button"
              onClick={() => void browse()}
              className={`${buttonSecondary} whitespace-nowrap`}
            >
              Browse file...
            </button>
          </div>
          <FieldError id="ac-path-error" message={errors.path} />
        </div>

        <div className="mb-4">
          <label htmlFor="ac-project" className="mb-1 block text-sm text-ink-muted">
            Project
          </label>
          <select
            id="ac-project"
            value={projectId}
            onChange={(event) => setProjectId(event.target.value)}
            aria-invalid={errors.project !== undefined}
            aria-describedby={errors.project !== undefined ? "ac-project-error" : undefined}
            className={`${inputClass} ${errors.project ? "border-warning" : "border-hairline"}`}
          >
            <option value="">Choose a project</option>
            {projects.map((project) => (
              <option key={project.id} value={String(project.id)}>
                {project.display_name}
              </option>
            ))}
          </select>
          <FieldError id="ac-project-error" message={errors.project} />
        </div>

        <div className="mb-4 grid grid-cols-2 gap-4">
          <div>
            <label htmlFor="ac-environment" className="mb-1 block text-sm text-ink-muted">
              Environment
            </label>
            <select
              id="ac-environment"
              value={environment}
              onChange={(event) => setEnvironment(event.target.value)}
              aria-invalid={errors.environment !== undefined}
              aria-describedby={
                errors.environment !== undefined ? "ac-environment-error" : undefined
              }
              className={`${inputClass} ${errors.environment ? "border-warning" : "border-hairline"}`}
            >
              <option value="">Choose an environment</option>
              {CONFIG_ENVIRONMENTS.map((option) => (
                <option key={option} value={option}>
                  {environmentLabel(option)}
                </option>
              ))}
            </select>
            <FieldError id="ac-environment-error" message={errors.environment} />
          </div>
          <div>
            <label htmlFor="ac-kind" className="mb-1 block text-sm text-ink-muted">
              Kind
            </label>
            <select
              id="ac-kind"
              value={kind}
              onChange={(event) => setKind(event.target.value)}
              aria-invalid={errors.kind !== undefined}
              aria-describedby={errors.kind !== undefined ? "ac-kind-error" : undefined}
              className={`${inputClass} ${errors.kind ? "border-warning" : "border-hairline"}`}
            >
              <option value="">Choose a kind</option>
              {CONFIG_KINDS.map((option) => (
                <option key={option} value={option}>
                  {kindLabel(option)}
                </option>
              ))}
            </select>
            <FieldError id="ac-kind-error" message={errors.kind} />
          </div>
        </div>

        <FieldError id="ac-form-error" message={errors.form} />

        <div className="mt-6 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={buttonSecondary}>
            Cancel
          </button>
          <button
            type="submit"
            disabled={busy}
            className="rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50"
          >
            Add config file
          </button>
        </div>
      </form>
    </dialog>
  );
}
