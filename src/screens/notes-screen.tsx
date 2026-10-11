import { EmptyState } from "../components/empty-state";

export function NotesScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Notes</h1>
      <p className="m-0 mb-8 text-ink-muted">
        Private Markdown notes, stored outside your repositories.
      </p>
      <EmptyState heading="No notes yet" />
    </div>
  );
}
