import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { checkProjectPath, pickProjectFolder, registerProject } from "../lib/commands";
import { formatProjectError } from "../lib/project-form";
import {
  PROJECT_TYPES,
  type ProjectErrorCode,
  type ProjectType,
  type ProjectView,
} from "../types/commands";

type Field = "path" | "name" | "type" | "tags" | "form";

function fieldOf(code: ProjectErrorCode): Field {
  switch (code) {
    case "path_empty":
    case "path_not_found":
    case "path_is_file":
    case "already_registered":
    case "path_too_long":
      return "path";
    case "name_empty":
    case "name_too_long":
      return "name";
    case "type_invalid":
      return "type";
    case "tag_invalid":
    case "too_many_tags":
      return "tags";
    case "storage_unavailable":
    case "project_not_found":
      return "form";
  }
}

interface RegisterProjectDialogProps {
  onClose: () => void;
  onRegistered: (project: ProjectView) => void;
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

export function RegisterProjectDialog({ onClose, onRegistered }: RegisterProjectDialogProps) {
  const queryClient = useQueryClient();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [type, setType] = useState<ProjectType>("Other");
  const [tags, setTags] = useState("");
  const [errors, setErrors] = useState<Partial<Record<Field, string>>>({});
  const [nestedIn, setNestedIn] = useState<string | null>(null);
  const pathRef = useRef("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
  }, []);

  const updatePath = (value: string) => {
    pathRef.current = value;
    setPath(value);
    setNestedIn(null);
  };

  const checkPath = async (value: string) => {
    if (value.trim() === "") {
      setNestedIn(null);
      return;
    }
    try {
      const result = await checkProjectPath(value);
      if (pathRef.current === value) {
        setNestedIn(result.nested_in);
      }
    } catch {
      if (pathRef.current === value) {
        setNestedIn(null);
      }
    }
  };

  const browse = async () => {
    try {
      const result = await pickProjectFolder();
      if (result.path !== null) {
        updatePath(result.path);
        setErrors((prev) => ({ ...prev, path: undefined }));
        await checkPath(result.path);
      }
    } catch {
      setErrors((prev) => ({
        ...prev,
        form: "Could not open the folder picker. Type the path instead.",
      }));
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    setBusy(true);
    setErrors({});
    try {
      const result = await registerProject({
        root_path: path,
        display_name: name,
        project_type: type,
        tags_raw: tags,
      });
      if (result.error !== null || result.project === null) {
        const code = result.error ?? "storage_unavailable";
        setErrors({ [fieldOf(code)]: formatProjectError(code, result.detail, path) });
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["projects"] });
      onRegistered(result.project);
    } catch {
      setErrors({ form: formatProjectError("storage_unavailable", null, path) });
    } finally {
      setBusy(false);
    }
  };

  const pathDescribedBy =
    [errors.path !== undefined ? "rp-path-error" : null, nestedIn !== null ? "rp-path-note" : null]
      .filter((id) => id !== null)
      .join(" ") || undefined;

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby="register-project-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      className="m-auto w-[min(36rem,calc(100vw-2rem))] rounded-2xl border border-hairline bg-surface p-6 text-ink backdrop:bg-black/70"
    >
      <form onSubmit={(event) => void submit(event)} noValidate>
        <h2 id="register-project-title" className="m-0 mb-4 text-xl font-semibold">
          Register project
        </h2>

        <div className="mb-4">
          <label htmlFor="rp-path" className="mb-1 block text-sm text-ink-muted">
            Root folder path
          </label>
          <div className="flex gap-2">
            <input
              id="rp-path"
              autoFocus
              value={path}
              onChange={(event) => updatePath(event.target.value)}
              onBlur={() => void checkPath(path)}
              placeholder="D:\work\my-project"
              aria-invalid={errors.path !== undefined}
              aria-describedby={pathDescribedBy}
              className={`${inputClass} font-mono ${errors.path ? "border-warning" : "border-hairline"}`}
            />
            <button type="button" onClick={() => void browse()} className={buttonSecondary}>
              Browse...
            </button>
          </div>
          <FieldError id="rp-path-error" message={errors.path} />
          {nestedIn !== null ? (
            <p id="rp-path-note" className="m-0 mt-1 text-sm text-ink-muted">
              This folder is inside {nestedIn}. Both projects are kept.
            </p>
          ) : null}
        </div>

        <div className="mb-4">
          <label htmlFor="rp-name" className="mb-1 block text-sm text-ink-muted">
            Display name
          </label>
          <input
            id="rp-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="notify-worker"
            aria-invalid={errors.name !== undefined}
            aria-describedby={errors.name !== undefined ? "rp-name-error" : undefined}
            className={`${inputClass} ${errors.name ? "border-warning" : "border-hairline"}`}
          />
          <FieldError id="rp-name-error" message={errors.name} />
        </div>

        <div className="mb-4">
          <label htmlFor="rp-type" className="mb-1 block text-sm text-ink-muted">
            Type
          </label>
          <select
            id="rp-type"
            value={type}
            onChange={(event) => setType(event.target.value as ProjectType)}
            aria-invalid={errors.type !== undefined}
            aria-describedby={errors.type !== undefined ? "rp-type-error" : undefined}
            className={`${inputClass} ${errors.type ? "border-warning" : "border-hairline"}`}
          >
            {PROJECT_TYPES.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
          <FieldError id="rp-type-error" message={errors.type} />
        </div>

        <div className="mb-4">
          <label htmlFor="rp-tags" className="mb-1 block text-sm text-ink-muted">
            Tags (comma separated)
          </label>
          <input
            id="rp-tags"
            value={tags}
            onChange={(event) => setTags(event.target.value)}
            placeholder="api, backend"
            aria-invalid={errors.tags !== undefined}
            aria-describedby={errors.tags !== undefined ? "rp-tags-error" : undefined}
            className={`${inputClass} ${errors.tags ? "border-warning" : "border-hairline"}`}
          />
          <FieldError id="rp-tags-error" message={errors.tags} />
        </div>

        <FieldError id="rp-form-error" message={errors.form} />

        <div className="mt-6 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={buttonSecondary}>
            Cancel
          </button>
          <button
            type="submit"
            disabled={busy}
            className="rounded-lg bg-accent px-4 py-2 font-medium text-on-accent hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50"
          >
            Register
          </button>
        </div>
      </form>
    </dialog>
  );
}
