import { WindowsLogo } from "@phosphor-icons/react";
import { CopyButton } from "./copy-button";
import { Reveal } from "./reveal";
import { buttonPrimary, container, sectionPad } from "../lib/classes";
import { RELEASES_URL } from "../lib/site";

const SOURCE_COMMANDS = [
  "git clone https://github.com/KhoaLy2003/devshelf.git",
  "cd devshelf",
  "npm install",
  "npm run tauri dev",
];

export function Install() {
  return (
    <section id="download" className={`${container} ${sectionPad}`}>
      <Reveal>
        <div className="grid grid-cols-1 gap-10 rounded-xl border border-hairline bg-surface p-8 lg:grid-cols-2 lg:p-12">
          <div className="flex flex-col gap-6">
            <h2 className="text-3xl font-semibold tracking-tight md:text-4xl">
              Run it on Windows 10 or 11
            </h2>
            <p className="text-lg text-ink-muted">
              Download the installer and run it. It needs the WebView2 runtime, which is
              preinstalled on Windows 11 and current Windows 10.
            </p>
            <div className="flex flex-col items-start gap-3">
              <a href={RELEASES_URL} rel="noopener noreferrer" className={buttonPrimary}>
                <WindowsLogo size={20} weight="regular" aria-hidden="true" />
                Download for Windows
              </a>
              <p className="text-sm text-ink-muted">
                The installer is not code-signed, so SmartScreen may warn on first run. Choose More
                info, then Run anyway.
              </p>
            </div>
          </div>
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between gap-4">
              <h3 className="text-sm font-semibold">Or run it from source</h3>
              <CopyButton text={SOURCE_COMMANDS.join("\n")} />
            </div>
            <pre className="overflow-x-auto rounded-lg border border-hairline bg-canvas p-4 font-mono text-sm">
              {SOURCE_COMMANDS.join("\n")}
            </pre>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
