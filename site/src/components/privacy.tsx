import { EyeSlash, LockKey, Timer, Warning, type Icon } from "@phosphor-icons/react";
import { Reveal } from "./reveal";
import { container, frame, sectionPad } from "../lib/classes";
import { shot } from "../lib/site";

const POINTS: { icon: Icon; title: string; body: string }[] = [
  {
    icon: LockKey,
    title: "Encrypted at rest",
    body: "Secrets and note bodies are encrypted. The note key lives in Windows Credential Manager.",
  },
  {
    icon: EyeSlash,
    title: "Masked by default",
    body: "A revealed value hides again after 30 seconds. Copy puts it on the clipboard without showing it.",
  },
  {
    icon: Timer,
    title: "Auto-lock",
    body: "Lock after 1, 5 or 15 minutes, or never.",
  },
  {
    icon: Warning,
    title: "No recovery",
    body: "The master password cannot be reset, so nobody else can open your vault either.",
  },
];

export function Privacy() {
  return (
    <section id="privacy" className={`${container} ${sectionPad}`}>
      <div className="grid items-center gap-12 lg:grid-cols-12">
        <Reveal className="lg:order-first lg:col-span-6">
          <div className={`${frame} bg-surface`}>
            <img
              src={shot("vault-secrets")}
              alt="DevShelf Secrets screen with masked values"
              width={1100}
              height={720}
              loading="lazy"
              className="block h-auto w-full"
            />
          </div>
        </Reveal>
        <Reveal className="lg:col-span-6">
          <h2 className="text-3xl font-semibold tracking-tight md:text-4xl">
            Your secrets stay on your disk
          </h2>
          <p className="mt-4 text-lg text-ink-muted">
            The vault is encrypted with a master password you choose. There is no account and no
            server to trust.
          </p>
          <div className="mt-10 grid gap-x-8 gap-y-6 sm:grid-cols-2">
            {POINTS.map(({ icon: Glyph, title, body }) => (
              <div key={title} className="flex flex-col gap-2">
                <Glyph size={20} weight="regular" aria-hidden="true" />
                <h3 className="font-semibold">{title}</h3>
                <p className="text-sm text-ink-muted">{body}</p>
              </div>
            ))}
          </div>
        </Reveal>
      </div>
    </section>
  );
}
