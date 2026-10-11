import { EmptyState } from "../components/empty-state";

export function WorkspacesScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Workspaces</h1>
      <p className="m-0 mb-8 text-ink-muted">Named, ordered sets of files you reopen together.</p>
      <EmptyState
        heading="No workspaces yet"
        message="A workspace saves an ordered set of files to reopen together."
      />
    </div>
  );
}
