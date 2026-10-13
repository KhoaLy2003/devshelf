import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { vaultStatus, vaultTouch } from "./commands";
import type { VaultStatusResult } from "../types/commands";

const TOUCH_THROTTLE_MS = 5000;

export const VAULT_STATUS_KEY = ["vault-status"] as const;

// AC-16: one status query shared by the shell and the Vault screen; polled every second while unlocked.
export function useVaultStatusQuery() {
  return useQuery({
    queryKey: VAULT_STATUS_KEY,
    queryFn: vaultStatus,
    staleTime: 0,
    refetchOnMount: "always",
    refetchInterval: (q) => (q.state.data?.status?.state === "unlocked" ? 1000 : false),
  });
}

// AC-15, AC-17: window input (any screen) while unlocked resets the Rust inactivity timer, at most once per 5 s.
export function useVaultInputTouch(active: boolean) {
  const queryClient = useQueryClient();
  const clientRef = useRef(queryClient);
  const activeRef = useRef(active);

  useEffect(() => {
    clientRef.current = queryClient;
    activeRef.current = active;
  }, [queryClient, active]);

  useEffect(() => {
    let last = 0;
    const touch = () => {
      if (!activeRef.current) return;
      const now = Date.now();
      if (now - last < TOUCH_THROTTLE_MS) return;
      last = now;
      vaultTouch()
        .then((result: VaultStatusResult) =>
          clientRef.current.setQueryData(VAULT_STATUS_KEY, result),
        )
        .catch(() => undefined);
    };
    window.addEventListener("pointerdown", touch);
    window.addEventListener("pointermove", touch);
    window.addEventListener("keydown", touch);
    return () => {
      window.removeEventListener("pointerdown", touch);
      window.removeEventListener("pointermove", touch);
      window.removeEventListener("keydown", touch);
    };
  }, []);
}

// App-shell hook: status polling, input listeners, and the countdown seconds (null = hide).
export function useVaultShell(): { secondsUntilLock: number | null } {
  const query = useVaultStatusQuery();
  const status = query.data?.status ?? null;
  const unlocked = status?.state === "unlocked";
  useVaultInputTouch(unlocked);
  return { secondsUntilLock: unlocked ? status.seconds_until_lock : null };
}
