import type { StorageErrorCode, StorageStatus } from "../types/commands";

export type GateDecision =
  | { kind: "loading" }
  | { kind: "error"; code: StorageErrorCode; folder: string }
  | { kind: "ready" };

export function resolveGate(query: {
  isPending: boolean;
  isError: boolean;
  data?: StorageStatus;
}): GateDecision {
  if (query.isPending) return { kind: "loading" };
  if (query.isError || !query.data) {
    return { kind: "error", code: "storage_unavailable", folder: "" };
  }
  if (!query.data.ok) {
    return {
      kind: "error",
      code: query.data.error_code ?? "storage_unavailable",
      folder: query.data.folder,
    };
  }
  return { kind: "ready" };
}
