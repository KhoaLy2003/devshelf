import { EmptyState } from "../components/empty-state";

export function HomeScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Home</h1>
      <p className="m-0 mb-8 text-ink-muted">Pick up where you left off.</p>
      <EmptyState
        heading="Nothing here yet"
        message="Register a project and save a workspace to see pinned projects and recent workspaces on this page."
      />
    </div>
  );
}
