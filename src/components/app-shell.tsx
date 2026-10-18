import { Outlet } from "react-router";
import { formatCountdown } from "../lib/vault-form";
import { useVaultShell } from "../lib/use-vault-shell";
import { UpdateBanner } from "./update-banner";
import { Sidebar } from "./sidebar";

export function AppShell() {
  const { secondsUntilLock } = useVaultShell();
  return (
    <div className="grid h-full grid-cols-[248px_1fr] bg-canvas text-ink">
      <Sidebar />
      <main className="min-w-0 overflow-y-auto px-10 py-8">
        <UpdateBanner />
        {secondsUntilLock !== null ? (
          <p
            role="status"
            data-testid="vault-countdown"
            className="m-0 mb-4 rounded-lg border border-hairline bg-surface-raised px-3 py-1.5 text-warning"
          >
            Locks in {formatCountdown(secondsUntilLock)}
          </p>
        ) : null}
        <Outlet />
      </main>
    </div>
  );
}
