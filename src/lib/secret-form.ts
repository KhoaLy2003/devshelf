import type {
  SecretCode,
  SecretEnvironment,
  SecretSummaryView,
  SecretType,
} from "../types/commands";

export const SECRET_LABEL_MAX = 120;
export const SECRET_VALUE_MAX_BYTES = 4096;
export const REVEAL_IDLE_MS = 30000;
export const COPIED_VISIBLE_MS = 2000;

export const SECRET_TYPE_OPTIONS: ReadonlyArray<{ value: SecretType; label: string }> = [
  { value: "api_key", label: "API key" },
  { value: "password", label: "Password" },
  { value: "token", label: "Token" },
  { value: "connection_string", label: "Connection string" },
  { value: "other", label: "Other" },
];

export const SECRET_ENVIRONMENT_OPTIONS: ReadonlyArray<{
  value: SecretEnvironment;
  label: string;
}> = [
  { value: "local", label: "Local" },
  { value: "dev", label: "Dev" },
  { value: "staging", label: "Staging" },
  { value: "production", label: "Production" },
];

export interface SecretFormInput {
  label: string;
  secret_type: SecretType | "";
  environment: SecretEnvironment | "";
  value: string;
}

// First inline error for the editor (AC-03, AC-04, AC-05, AC-06); null when the form can be sent to Rust.
export function validateSecretForm(input: SecretFormInput, isEdit: boolean): SecretCode | null {
  const label = input.label.trim();
  if (label === "") return "secrets_label_required";
  if ([...label].length > SECRET_LABEL_MAX) return "secrets_label_too_long";
  if (input.secret_type === "") return "secrets_type_required";
  if (input.environment === "") return "secrets_environment_required";
  if (input.value === "") return isEdit ? null : "secrets_value_required";
  if (new TextEncoder().encode(input.value).length > SECRET_VALUE_MAX_BYTES) {
    return "secrets_value_too_long";
  }
  return null;
}

export function maskedValue(): string {
  return "••••••••";
}

// AC-13: a revealed row re-masks after 30 s without interaction.
export function revealTimerExpired(lastInteractionMs: number, nowMs: number): boolean {
  return nowMs - lastInteractionMs >= REVEAL_IDLE_MS;
}

export function formatSecretError(code: SecretCode): string {
  switch (code) {
    case "secrets_label_required":
      return "Enter a label.";
    case "secrets_value_required":
      return "Enter a value.";
    case "secrets_type_required":
      return "Choose a type.";
    case "secrets_environment_required":
      return "Choose an environment.";
    case "secrets_label_too_long":
      return "Use 120 characters or fewer.";
    case "secrets_value_too_long":
      return "Value is too long.";
    case "secrets_duplicate_label":
      return "A secret with this label already exists for this project.";
    case "secrets_not_found":
      return "This secret no longer exists.";
    case "secrets_unreadable":
      return "This secret can't be read. It may be damaged.";
    case "secrets_locked":
      return "The vault is locked. Unlock it to use secrets.";
    case "secrets_storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
    case "secrets_clipboard_unavailable":
      return "DevShelf can't reach the clipboard. Try again.";
  }
}

export type SecretField = "label" | "type" | "environment" | "value" | null;

// Which editor field an error belongs under; null means a form-level message.
export function secretErrorField(code: SecretCode): SecretField {
  switch (code) {
    case "secrets_label_required":
    case "secrets_label_too_long":
    case "secrets_duplicate_label":
      return "label";
    case "secrets_type_required":
      return "type";
    case "secrets_environment_required":
      return "environment";
    case "secrets_value_required":
    case "secrets_value_too_long":
      return "value";
    default:
      return null;
  }
}

export function secretTypeLabel(type: SecretType): string {
  return SECRET_TYPE_OPTIONS.find((o) => o.value === type)?.label ?? type;
}

export function secretEnvironmentLabel(environment: SecretEnvironment): string {
  return SECRET_ENVIRONMENT_OPTIONS.find((o) => o.value === environment)?.label ?? environment;
}

// AC-09, AC-12: project name, "No project", or "Project removed".
export function secretProjectLabel(
  item: Pick<SecretSummaryView, "project_id" | "project_missing">,
  projectNames: ReadonlyMap<number, string>,
): string {
  if (item.project_id === null) return "No project";
  if (item.project_missing) return "Project removed";
  return projectNames.get(item.project_id) ?? "Project removed";
}
