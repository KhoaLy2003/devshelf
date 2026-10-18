import { RELEASES_URL, asset } from "../lib/site";
import { buttonPrimary, container } from "../lib/classes";
import { ThemeToggle } from "./theme-toggle";

export function Nav() {
  return (
    <header className="sticky top-0 z-40 h-16 border-b border-hairline bg-canvas/80 backdrop-blur">
      <div className={`${container} flex h-full items-center justify-between gap-4`}>
        <a href="#" className="flex items-center gap-2 font-semibold">
          <img src={asset("logo.png")} alt="" width={24} height={24} />
          DevShelf
        </a>
        <nav
          aria-label="Primary"
          className="hidden items-center gap-8 text-sm text-ink-muted md:flex"
        >
          <a href="#features" className="transition hover:text-ink">
            Features
          </a>
          <a href="#privacy" className="transition hover:text-ink">
            Privacy
          </a>
          <a href="#faq" className="transition hover:text-ink">
            FAQ
          </a>
        </nav>
        <div className="flex items-center gap-3">
          <ThemeToggle />
          <a href={RELEASES_URL} rel="noopener noreferrer" className={buttonPrimary}>
            Download for Windows
          </a>
        </div>
      </div>
    </header>
  );
}
