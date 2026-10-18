import { CHANGELOG_URL, REPO_URL, RELEASES_URL, asset } from "../lib/site";
import { container } from "../lib/classes";

export function Footer() {
  return (
    <footer className="border-t border-hairline py-12">
      <div
        className={`${container} flex flex-col gap-8 md:flex-row md:items-start md:justify-between`}
      >
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2 font-semibold">
            <img src={asset("logo.png")} alt="" width={24} height={24} />
            DevShelf
          </div>
          <p className="text-sm text-ink-muted">A local-first project shelf for Windows.</p>
        </div>
        <nav aria-label="Footer" className="flex flex-wrap gap-6 text-sm text-ink-muted">
          <a href={REPO_URL} rel="noopener noreferrer" className="transition hover:text-ink">
            GitHub
          </a>
          <a href={RELEASES_URL} rel="noopener noreferrer" className="transition hover:text-ink">
            Releases
          </a>
          <a href={CHANGELOG_URL} rel="noopener noreferrer" className="transition hover:text-ink">
            Changelog
          </a>
        </nav>
      </div>
      <p className={`${container} mt-10 text-sm text-ink-muted`}>© 2026 DevShelf</p>
    </footer>
  );
}
