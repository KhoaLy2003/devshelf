import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { VaultGate } from "../components/vault-gate";
import { listProjects, secretsCreate, secretsList, secretsUpdate } from "../lib/commands";
import {
  SECRET_ENVIRONMENT_OPTIONS,
  SECRET_TYPE_OPTIONS,
  formatSecretError,
  secretErrorField,
  validateSecretForm,
  type SecretField,
} from "../lib/secret-form";
import { VAULT_STATUS_KEY } from "../lib/use-vault-shell";
import type {
  SecretCode,
  SecretEnvironment,
  SecretSummaryView,
  SecretType,
} from "../types/commands";

const linkClass =
  "text-accent hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";
const buttonPrimary =
  "rounded-lg bg-accent px-4 py-2 font-medium text-ink hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink disabled:opacity-50";
const buttonSecondary =
  "rounded-lg border border-hairline bg-surface-raised px-4 py-2 text-ink no-underline hover:border-ink-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50";
const fieldClass =
  "w-full max-w-md rounded-lg border border-hairline bg-canvas px-3 py-2 text-ink placeholder:text-ink-muted focus:outline-2 focus:outline-accent";

const REMOVED_PROJECT_VALUE = "removed";

interface EditorProps {
  existing: SecretSummaryView | null;
}

function SecretEditor({ existing }: EditorProps) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const isEdit = existing !== null;
  const [label, setLabel] = useState(existing?.label ?? "");
  const [secretType, setSecretType] = useState<SecretType | "">(existing?.secret_type ?? "");
  const [environment, setEnvironment] = useState<SecretEnvironment | "">(
    existing?.environment ?? "",
  );
  const [projectValue, setProjectValue] = useState(() => {
    if (existing === null || existing.project_id === null) return "";
    return existing.project_missing ? REMOVED_PROJECT_VALUE : String(existing.project_id);
  });
  const [value, setValue] = useState("");
  const [error, setError] = useState<SecretCode | null>(null);
  const [busy, setBusy] = useState(false);

  const projectsQuery = useQuery({ queryKey: ["projects"], queryFn: listProjects });
  const projects = projectsQuery.data?.projects ?? [];

  const errorField: SecretField = error === null ? null : secretErrorField(error);
  const errorText = error === null ? null : formatSecretError(error);
  const fieldError = (field: SecretField, id: string) =>
    errorField === field && errorText !== null ? (
      <p id={id} role="alert" className="m-0 mt-1 text-sm text-warning">
        {errorText}
      </p>
    ) : null;
  const invalid = (field: SecretField) => (errorField === field ? true : undefined);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const local = validateSecretForm(
      { label, secret_type: secretType, environment, value },
      isEdit,
    );
    if (local !== null || secretType === "" || environment === "") {
      setError(local ?? "secrets_type_required");
      return;
    }
    const projectId =
      projectValue === ""
        ? null
        : projectValue === REMOVED_PROJECT_VALUE
          ? (existing?.project_id ?? null)
          : Number(projectValue);
    setBusy(true);
    setError(null);
    try {
      const base = {
        label: label.trim(),
        secret_type: secretType,
        project_id: projectId,
        environment,
      };
      const result =
        existing === null
          ? await secretsCreate({ ...base, value })
          : await secretsUpdate({
              ...base,
              id: existing.id,
              value: value === "" ? null : value,
            });
      if (result.error !== null) {
        setError(result.error);
        if (result.error === "secrets_locked") {
          void queryClient.invalidateQueries({ queryKey: VAULT_STATUS_KEY });
        }
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["secrets"] });
      void navigate("/secrets");
    } catch {
      setError("secrets_storage_unavailable");
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={(event) => void submit(event)} noValidate className="flex flex-col gap-5">
      <div>
        <label htmlFor="secret-label" className="mb-1 block font-medium">
          Label
        </label>
        <input
          id="secret-label"
          className={fieldClass}
          value={label}
          onChange={(event) => setLabel(event.target.value)}
          aria-invalid={invalid("label")}
          aria-describedby={errorField === "label" ? "secret-label-error" : undefined}
          autoComplete="off"
        />
        {fieldError("label", "secret-label-error")}
      </div>
      <div>
        <label htmlFor="secret-type" className="mb-1 block font-medium">
          Type
        </label>
        <select
          id="secret-type"
          className={fieldClass}
          value={secretType}
          onChange={(event) => setSecretType(event.target.value as SecretType | "")}
          aria-invalid={invalid("type")}
          aria-describedby={errorField === "type" ? "secret-type-error" : undefined}
        >
          <option value="">Select a type</option>
          {SECRET_TYPE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {fieldError("type", "secret-type-error")}
      </div>
      <div>
        <label htmlFor="secret-project" className="mb-1 block font-medium">
          Project
        </label>
        <select
          id="secret-project"
          className={fieldClass}
          value={projectValue}
          onChange={(event) => setProjectValue(event.target.value)}
        >
          <option value="">No project</option>
          {existing !== null && existing.project_missing ? (
            <option value={REMOVED_PROJECT_VALUE}>Project removed</option>
          ) : null}
          {projects.map((project) => (
            <option key={project.id} value={String(project.id)}>
              {project.display_name}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label htmlFor="secret-environment" className="mb-1 block font-medium">
          Environment
        </label>
        <select
          id="secret-environment"
          className={fieldClass}
          value={environment}
          onChange={(event) => setEnvironment(event.target.value as SecretEnvironment | "")}
          aria-invalid={invalid("environment")}
          aria-describedby={errorField === "environment" ? "secret-environment-error" : undefined}
        >
          <option value="">Select an environment</option>
          {SECRET_ENVIRONMENT_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {fieldError("environment", "secret-environment-error")}
      </div>
      <div>
        <label htmlFor="secret-value" className="mb-1 block font-medium">
          Value
        </label>
        <input
          id="secret-value"
          type="password"
          className={fieldClass}
          value={value}
          onChange={(event) => setValue(event.target.value)}
          placeholder={isEdit ? "Leave blank to keep the current value." : undefined}
          aria-invalid={invalid("value")}
          aria-describedby={
            [
              errorField === "value" ? "secret-value-error" : null,
              isEdit ? "secret-value-hint" : null,
            ]
              .filter((part) => part !== null)
              .join(" ") || undefined
          }
          autoComplete="off"
          spellCheck={false}
        />
        {isEdit ? (
          <p id="secret-value-hint" className="m-0 mt-1 text-sm text-ink-muted">
            Leave blank to keep the current value.
          </p>
        ) : null}
        {fieldError("value", "secret-value-error")}
      </div>
      {error !== null && errorField === null ? (
        <p role="alert" className="m-0 text-warning">
          {formatSecretError(error)}
        </p>
      ) : null}
      <div className="flex items-center gap-3">
        <button type="submit" className={buttonPrimary} disabled={busy}>
          {isEdit ? "Save changes" : "Save secret"}
        </button>
        <Link to="/secrets" className={buttonSecondary}>
          Cancel
        </Link>
      </div>
    </form>
  );
}

function SecretEditorLoader({ id }: { id: number | null }) {
  const query = useQuery({
    queryKey: ["secrets", "editor", id],
    queryFn: () => secretsList({ project_id: null, filter_by_project: false, secret_type: null }),
    enabled: id !== null,
    staleTime: 0,
    refetchOnMount: "always",
    refetchOnWindowFocus: false,
  });

  if (id === null) return <SecretEditor key="new" existing={null} />;

  // Mount the form only with data fetched after mount, so fields never start from a stale cache.
  if (query.isLoading || (query.isFetching && !query.isFetchedAfterMount)) {
    return <p className="m-0 text-ink-muted">Loading secret...</p>;
  }
  const existing = query.data?.items.find((item) => item.id === id) ?? null;
  if (existing === null) {
    const code: SecretCode =
      query.data?.error ?? (query.isError ? "secrets_storage_unavailable" : "secrets_not_found");
    return (
      <p role="alert" className="m-0 text-warning">
        {formatSecretError(code)}
      </p>
    );
  }
  return <SecretEditor key={existing.id} existing={existing} />;
}

export function SecretEditorScreen() {
  const params = useParams();
  const rawId = params["id"];
  const id = rawId === undefined ? null : Number(rawId);
  const validId = id === null || Number.isInteger(id);

  return (
    <VaultGate>
      <div>
        <h1 className="m-0 mb-2 text-3xl font-semibold">
          {id === null ? "Add secret" : "Edit secret"}
        </h1>
        <p className="m-0 mb-6">
          <Link to="/secrets" className={linkClass}>
            Back to Secrets
          </Link>
        </p>
        {validId ? (
          <SecretEditorLoader id={id} />
        ) : (
          <p role="alert" className="m-0 text-warning">
            {formatSecretError("secrets_not_found")}
          </p>
        )}
      </div>
    </VaultGate>
  );
}
