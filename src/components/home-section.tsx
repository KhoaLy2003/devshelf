import type { ReactNode } from "react";
import { Link } from "react-router";

interface HomeSectionProps {
  heading: string;
  emptyText: string;
  isEmpty: boolean;
  allLabel: string;
  allTo: string;
  showAllLink: boolean;
  children: ReactNode;
}

export function HomeSection({
  heading,
  emptyText,
  isEmpty,
  allLabel,
  allTo,
  showAllLink,
  children,
}: HomeSectionProps) {
  return (
    <section className="min-w-0">
      <h2 className="m-0 mb-3 text-xs font-semibold uppercase tracking-widest text-ink-muted">
        {heading}
      </h2>
      {isEmpty ? (
        <p className="m-0 rounded-2xl border border-dashed border-hairline bg-surface px-4 py-6 text-ink-muted">
          {emptyText}
        </p>
      ) : (
        <ul className="m-0 list-none overflow-hidden rounded-2xl border border-hairline bg-surface p-0">
          {children}
        </ul>
      )}
      {showAllLink ? (
        <div className="mt-3">
          <Link
            to={allTo}
            className="text-accent hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ink"
          >
            {allLabel}
          </Link>
        </div>
      ) : null}
    </section>
  );
}
