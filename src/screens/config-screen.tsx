import { EmptyState } from "../components/empty-state";

export function ConfigScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Config Catalog</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Find the right environment's file. DevShelf stores paths and labels, never file contents.
      </p>
      <EmptyState
        heading="No config files registered"
        message="Registering a file stores its path only, never its contents."
      />
    </div>
  );
}
