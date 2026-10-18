import { useState } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { container, frame, sectionPad } from "../lib/classes";
import { shot } from "../lib/site";

const TABS = [
  {
    label: "Projects",
    body: "Register a folder and pin the projects you open every day.",
    image: "projects",
    alt: "DevShelf Projects screen listing registered projects",
  },
  {
    label: "Config catalog",
    body: "Track config files per project and environment, with Available, Missing or Inaccessible status.",
    image: "config-catalog",
    alt: "DevShelf Config Catalog screen",
  },
  {
    label: "Notes",
    body: "Private Markdown notes linked to a project. They autosave a second after you stop typing.",
    image: "notes",
    alt: "DevShelf Notes screen",
  },
];

export function Showcase() {
  const [active, setActive] = useState(0);
  const reduce = useReducedMotion();
  const tab = TABS[active];

  return (
    <section id="features" className={`${container} ${sectionPad}`}>
      <h2 className="text-3xl font-semibold tracking-tight md:text-4xl">
        From a folder to a restored workspace
      </h2>
      <div className="mt-12 grid gap-10 lg:grid-cols-12">
        <div
          role="tablist"
          aria-label="DevShelf screens"
          className="flex flex-col gap-1 lg:col-span-4"
        >
          {TABS.map((item, index) => {
            const selected = index === active;
            return (
              <button
                key={item.label}
                type="button"
                role="tab"
                id={`showcase-tab-${index}`}
                aria-selected={selected}
                aria-controls="showcase-panel"
                onClick={() => setActive(index)}
                className={`flex flex-col gap-1 border-l-2 px-5 py-4 text-left transition ${
                  selected
                    ? "border-accent bg-surface text-ink"
                    : "border-transparent text-ink-muted hover:text-ink"
                }`}
              >
                <span className="font-semibold">{item.label}</span>
                <span className="text-sm">{item.body}</span>
              </button>
            );
          })}
        </div>
        <div
          role="tabpanel"
          id="showcase-panel"
          aria-labelledby={`showcase-tab-${active}`}
          className={`${frame} bg-surface lg:col-span-8`}
        >
          <AnimatePresence mode="wait" initial={false}>
            <motion.img
              key={tab.image}
              src={shot(tab.image)}
              alt={tab.alt}
              width={1100}
              height={720}
              loading="lazy"
              className="block h-auto w-full"
              initial={reduce ? false : { opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={reduce ? undefined : { opacity: 0 }}
              transition={{ duration: 0.25 }}
            />
          </AnimatePresence>
        </div>
      </div>
    </section>
  );
}
