import { useState } from "react";
import { HELP_SECTIONS, getSection } from "../lib/help-content";

export function HelpScreen() {
  const [selectedId, setSelectedId] = useState(HELP_SECTIONS[0]?.id ?? "");
  const section = getSection(selectedId);

  return (
    <div>
      <h1 className="m-0 mb-8 text-3xl font-semibold">Help</h1>
      <div className="flex flex-col gap-6 md:flex-row md:items-start">
        <nav aria-label="Help sections" className="md:w-56 md:shrink-0">
          <ul className="m-0 flex list-none flex-col gap-1 p-0">
            {HELP_SECTIONS.map((s) => {
              const active = s.id === section.id;
              return (
                <li key={s.id}>
                  <button
                    type="button"
                    onClick={() => setSelectedId(s.id)}
                    aria-current={active ? "true" : undefined}
                    className={`block w-full cursor-pointer rounded-lg border-0 px-3 py-2 text-left text-sm font-medium focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
                      active
                        ? "bg-surface-raised text-ink"
                        : "bg-transparent text-ink-muted hover:bg-surface hover:text-ink"
                    }`}
                  >
                    {s.title}
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>
        <article
          aria-labelledby="help-section-title"
          className="min-w-0 flex-1 rounded-2xl border border-hairline bg-surface p-5"
        >
          <h2 id="help-section-title" className="m-0 mb-4 text-xl font-semibold">
            {section.title}
          </h2>
          {section.body.map((p) => (
            <p key={p} className="m-0 mb-3 max-w-[65ch] text-ink-muted">
              {p}
            </p>
          ))}
          {section.groups?.map((g) => (
            <div key={g.heading} className="mt-5">
              <h3 className="m-0 mb-2 text-xs font-semibold uppercase tracking-widest text-ink-muted">
                {g.heading}
              </h3>
              <ul className="m-0 list-disc pl-5 text-ink-muted">
                {g.items.map((item) => (
                  <li key={item} className="mb-1">
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </article>
      </div>
    </div>
  );
}
