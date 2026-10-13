import type { ReactNode } from "react";
import { VaultScreen } from "../screens/vault-screen";
import { useVaultStatusQuery } from "../lib/use-vault-shell";

// AC-30, AC-31: children render only while the vault is unlocked; otherwise the F08 lock screen shows.
// Children unmount on lock, which drops any revealed value held in their state (AC-15).
export function VaultGate({ children }: { children: ReactNode }) {
  const query = useVaultStatusQuery();
  if (query.isLoading) {
    return <p className="m-0 text-ink-muted">Loading vault…</p>;
  }
  if (query.data?.status?.state !== "unlocked") {
    return <VaultScreen />;
  }
  return <>{children}</>;
}
