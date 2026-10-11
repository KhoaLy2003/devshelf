interface EmptyStateProps {
  heading: string;
  message?: string;
}

export function EmptyState({ heading, message }: EmptyStateProps) {
  return (
    <section className="rounded-2xl border border-dashed border-hairline bg-surface px-6 py-12 text-center">
      <h2 className="m-0 mb-2 text-lg font-semibold text-ink">{heading}</h2>
      {message ? <p className="mx-auto m-0 max-w-[46ch] text-ink-muted">{message}</p> : null}
    </section>
  );
}
