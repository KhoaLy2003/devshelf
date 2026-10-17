import type { ProjectRemovalCode, VaultDeleteCode } from "../types/commands";

function plural(count: number, singular: string, pluralForm: string): string {
  return `${count} ${count === 1 ? singular : pluralForm}`;
}

// AC-A09: null when nothing else is removed; zero parts are left out of the sentence.
export function removalSummary(configCount: number, workspaceCount: number): string | null {
  const parts: string[] = [];
  if (configCount > 0) parts.push(plural(configCount, "config entry", "config entries"));
  if (workspaceCount > 0) parts.push(plural(workspaceCount, "workspace", "workspaces"));
  if (parts.length === 0) return null;
  return `This also removes ${parts.join(" and ")}.`;
}

// AC-B03 and the zero-secret edge case.
export function vaultDeleteMessage(secretCount: number): string {
  if (secretCount === 0) return "Delete the vault? No secrets are stored.";
  if (secretCount === 1) {
    return "Delete the vault and its 1 secret? The secret cannot be recovered. Notes are not affected.";
  }
  return `Delete the vault and all ${secretCount} secrets? The secrets cannot be recovered. Notes are not affected.`;
}

// AC-B03: the final button enables only for exactly DELETE.
export function canConfirmVaultDelete(typed: string): boolean {
  return typed === "DELETE";
}

export function formatProjectRemovalError(code: ProjectRemovalCode): string {
  switch (code) {
    case "project_not_found":
      return "This project is already removed.";
    case "project_storage_unavailable":
      return "DevShelf cannot reach its data folder. Restart DevShelf and try again.";
  }
}

export function formatVaultDeleteError(code: VaultDeleteCode): string {
  switch (code) {
    case "vault_password_required":
      return "Enter your master password.";
    case "vault_wrong_password":
      return "Wrong master password.";
    case "vault_not_set_up":
      return "The vault is not set up yet.";
    case "vault_storage_unavailable":
      return "Vault storage is unavailable. Try again.";
  }
}
