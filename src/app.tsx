import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { RouterProvider } from "react-router";
import { resolveGate } from "./lib/gate";
import { storageStatus } from "./lib/commands";
import { router } from "./router";
import { StorageErrorScreen } from "./screens/storage-error-screen";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false, staleTime: Infinity } },
});

function Gate() {
  const status = useQuery({ queryKey: ["storage-status"], queryFn: storageStatus });
  const gate = resolveGate(status);

  switch (gate.kind) {
    case "loading":
      return (
        <div className="flex h-full items-center justify-center bg-canvas text-ink-muted">
          Starting DevShelf…
        </div>
      );
    case "error":
      return <StorageErrorScreen code={gate.code} folder={gate.folder} />;
    case "ready":
      return <RouterProvider router={router} />;
  }
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Gate />
    </QueryClientProvider>
  );
}
