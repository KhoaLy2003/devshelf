import { Reveal } from "./reveal";
import { container, sectionPad } from "../lib/classes";
import { shot } from "../lib/site";

const CHIPS = ["local", "dev", "staging", "production"];

const CELL = "flex h-full flex-col rounded-xl border border-hairline bg-surface p-6";

export function Bento() {
  return (
    <section className={`${container} ${sectionPad}`}>
      <h2 className="text-3xl font-semibold tracking-tight md:text-4xl">
        Details that save you the setup
      </h2>
      <div className="mt-12 grid grid-cols-1 gap-4 lg:grid-cols-4">
        <Reveal delay={0} className="lg:col-span-2 lg:row-span-2">
          <div className={`${CELL} overflow-hidden`}>
            <h3 className="text-lg font-semibold">Missing paths are reported, not hidden</h3>
            <p className="mt-2 text-sm text-ink-muted">
              Restore lists every missing item with Relink or Ignore, so a moved folder never fails
              silently.
            </p>
            <img
              src={shot("workspace-restore")}
              alt="DevShelf Workspace restore screen listing missing paths"
              width={1100}
              height={720}
              loading="lazy"
              className="mt-8 aspect-[11/7] w-full rounded-lg border border-hairline object-cover object-top"
            />
          </div>
        </Reveal>

        <Reveal delay={0.06} className="lg:col-span-2">
          <div
            className={CELL}
            style={{
              background:
                "linear-gradient(135deg, color-mix(in srgb, var(--color-accent) 22%, transparent), transparent 70%), var(--color-surface)",
            }}
          >
            <h3 className="text-lg font-semibold">One catalog, every environment</h3>
            <p className="mt-2 text-sm text-ink-muted">
              Group config files by local, dev, staging and production.
            </p>
            <div className="mt-6 flex flex-wrap gap-2">
              {CHIPS.map((chip) => (
                <span
                  key={chip}
                  className="rounded-lg border border-hairline bg-canvas/60 px-3 py-1 font-mono text-xs"
                >
                  {chip}
                </span>
              ))}
            </div>
          </div>
        </Reveal>

        <Reveal delay={0.12} className="lg:col-span-1">
          <div className={`${CELL} overflow-hidden`}>
            <h3 className="text-lg font-semibold">Dark and light</h3>
            <p className="mt-2 text-sm text-ink-muted">Dark by default. Switch in Settings.</p>
            <div className="mt-6 min-h-40 flex-1 overflow-hidden rounded-lg border border-hairline">
              <img
                src={shot("settings-light")}
                alt="DevShelf Settings screen in the light theme"
                width={1100}
                height={720}
                loading="lazy"
                className="h-full w-full object-cover object-top"
              />
            </div>
          </div>
        </Reveal>

        <Reveal delay={0.18} className="lg:col-span-1">
          <div
            className={CELL}
            style={{
              backgroundImage: "radial-gradient(var(--color-hairline) 1px, transparent 1px)",
              backgroundSize: "16px 16px",
            }}
          >
            <h3 className="text-lg font-semibold">Any editor</h3>
            <p className="mt-2 text-sm text-ink-muted">
              Add an editor by executable and arguments. Shell syntax is rejected.
            </p>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
