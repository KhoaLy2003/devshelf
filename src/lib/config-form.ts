import type {
  ConfigCode,
  ConfigEntryView,
  ConfigEnvironment,
  ConfigKind,
  ConfigStatus,
} from "../types/commands";

export interface ConfigFilters {
  project: string;
  environment: string;
  kind: string;
  status: string;
}

export interface ConfigChip {
  key: keyof ConfigFilters;
  label: string;
}

const INACCESSIBLE_COPY =
  "The file can't be reached. The drive may not be ready or access is denied.";

// AC-02: pre-select a kind from the file name; null means no pre-selection.
export function kindFromFileName(name: string): ConfigKind | null {
  const lower = name.trim().toLowerCase();
  if (/^docker-compose.*\.ya?ml$/.test(lower)) return "docker_compose";
  if (lower.endsWith(".yml") || lower.endsWith(".yaml")) return "yaml";
  if (lower.endsWith(".properties")) return "properties";
  if (lower.startsWith(".env")) return "dotenv";
  if (lower.endsWith(".json")) return "json";
  if (lower === "caddyfile" || lower.includes("nginx")) return "caddy_nginx";
  return null;
}

function isActive(value: string): boolean {
  return value !== "" && value !== "all";
}

// "missing" covers a folder at the path too (AC-10); the other statuses match exactly.
function matchesStatus(status: ConfigStatus, filter: string): boolean {
  if (!isActive(filter)) return true;
  if (filter === "missing") return status === "missing" || status === "folder_at_path";
  return status === filter;
}

export function applyFilters(
  entries: ConfigEntryView[],
  filters: ConfigFilters,
): ConfigEntryView[] {
  return entries.filter(
    (e) =>
      (!isActive(filters.project) || e.project_name === filters.project) &&
      (!isActive(filters.environment) || e.environment === filters.environment) &&
      (!isActive(filters.kind) || e.kind === filters.kind) &&
      matchesStatus(e.status, filters.status),
  );
}

function statusLabel(status: string): string {
  switch (status) {
    case "available":
      return "Available";
    case "missing":
    case "folder_at_path":
      return "Missing";
    case "inaccessible":
      return "Inaccessible";
    default:
      return status;
  }
}

export function activeChips(filters: ConfigFilters): ConfigChip[] {
  const chips: ConfigChip[] = [];
  if (isActive(filters.project)) {
    chips.push({ key: "project", label: `Project: ${filters.project}` });
  }
  if (isActive(filters.environment)) {
    chips.push({ key: "environment", label: `Environment: ${filters.environment}` });
  }
  if (isActive(filters.kind)) {
    chips.push({ key: "kind", label: `Kind: ${kindLabel(filters.kind)}` });
  }
  if (isActive(filters.status)) {
    chips.push({ key: "status", label: `Status: ${statusLabel(filters.status)}` });
  }
  return chips;
}

export function statusReason(status: ConfigStatus): string {
  switch (status) {
    case "available":
      return "The file is in place.";
    case "missing":
      return "The file was not found at this path.";
    case "folder_at_path":
      return "A folder is at this path, not a file.";
    case "inaccessible":
      return INACCESSIBLE_COPY;
  }
}

// `detail` carries the path for path codes, or "unreachable" for path_not_found when the
// metadata check failed. `name` is the fallback when no path detail is present.
export function formatConfigError(code: ConfigCode, detail: string | null, name: string): string {
  const path = detail ?? name;
  switch (code) {
    case "path_empty":
      return "Enter a file path.";
    case "path_not_found":
      return detail === "unreachable" ? INACCESSIBLE_COPY : `\`${path}\` was not found.`;
    case "path_is_folder":
      return "This is a folder. Pick a file.";
    case "path_relative":
      return "Enter an absolute path, for example `F:\\work\\application.yml`.";
    case "path_too_long":
      return "This path is too long for Windows.";
    case "duplicate_path":
      return `\`${path}\` is already in this project's config list.`;
    case "project_not_found":
      return "That project no longer exists. Choose another project.";
    case "environment_invalid":
      return "Pick an environment from the list.";
    case "kind_invalid":
      return "Pick a kind from the list.";
    case "config_not_found":
      return "This config file is no longer in the catalog.";
    case "storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

export function environmentLabel(env: ConfigEnvironment | string): string {
  switch (env) {
    case "local":
      return "Local";
    case "dev":
      return "Dev";
    case "staging":
      return "Staging";
    case "production":
      return "Production";
    default:
      return env;
  }
}

export function kindLabel(kind: ConfigKind | string): string {
  switch (kind) {
    case "yaml":
      return "YAML";
    case "properties":
      return "Properties";
    case "dotenv":
      return ".env";
    case "docker_compose":
      return "Docker Compose";
    case "caddy_nginx":
      return "Caddy/Nginx";
    case "json":
      return "JSON";
    default:
      return kind;
  }
}
