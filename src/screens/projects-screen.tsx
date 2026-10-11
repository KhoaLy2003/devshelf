import { EmptyState } from "../components/empty-state";

export function ProjectsScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Projects</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Registered project folders. Registration stores a reference only.
      </p>
      <EmptyState
        heading="No projects registered yet"
        message="A project is a reference to a folder. DevShelf never copies or changes your files."
      />
    </div>
  );
}
