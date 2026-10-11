import { storageErrorMessage } from "../lib/storage-messages";
import type { StorageErrorCode } from "../types/commands";

interface StorageErrorScreenProps {
  code: StorageErrorCode;
  folder: string;
}

export function StorageErrorScreen({ code, folder }: StorageErrorScreenProps) {
  return (
    <main className="flex h-full items-center justify-center bg-canvas p-8 text-ink">
      <div className="max-w-[56ch] rounded-2xl border border-warning/45 bg-surface p-8">
        <h1 className="m-0 mb-3 text-2xl font-semibold">DevShelf can't start</h1>
        <p className="m-0 text-ink-muted">{storageErrorMessage(code, folder)}</p>
        <p className="m-0 mt-3 font-mono text-xs text-ink-muted">{code}</p>
      </div>
    </main>
  );
}
