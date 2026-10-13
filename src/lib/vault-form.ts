import type { VaultCode, VaultLockReason } from "../types/commands";

export const MIN_PASSWORD_LENGTH = 12;

// AC-02, AC-03, AC-04: first failing inline error code, or null when the form is valid.
export function validateSetup(
  password: string,
  confirmation: string,
  acknowledged: boolean,
): VaultCode | null {
  if (password === "") return "vault_password_required";
  if (password !== confirmation) return "vault_password_mismatch";
  if ([...password].length < MIN_PASSWORD_LENGTH) return "vault_password_too_short";
  if (!acknowledged) return "vault_password_required";
  return null;
}

function minutesText(minutes: number): string {
  return minutes === 1 ? "1 minute" : `${minutes} minutes`;
}

// AC-21 names the reason; AC-18 states the configured duration for inactivity.
export function lockReasonText(reason: VaultLockReason | null, minutes: number): string {
  switch (reason) {
    case "manual":
      return "You locked the vault.";
    case "inactivity":
      return `Locked after ${minutesText(minutes)} of inactivity.`;
    case "launch":
    default:
      return "Locked because DevShelf just started.";
  }
}

// AC-16: m:ss countdown.
export function formatCountdown(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function formatVaultError(code: VaultCode): string {
  switch (code) {
    case "vault_password_required":
      return "Enter your master password.";
    case "vault_password_mismatch":
      return "Passwords don't match.";
    case "vault_password_too_short":
      return "Use at least 12 characters.";
    case "vault_wrong_password":
      return "Wrong master password.";
    case "vault_already_set_up":
      return "The vault is already set up.";
    case "vault_not_set_up":
      return "The vault is not set up yet.";
    case "vault_invalid_timeout":
      return "Choose Never, 1, 5 or 15 minutes.";
    case "vault_storage_unavailable":
      return "Vault storage is unavailable. Try again.";
  }
}
